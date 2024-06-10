![- Production Monitoring (1)](https://github.com/prodzilla/prodzilla/assets/22592280/2a36189e-bce7-4aed-a1a1-9b9706c415d0)

Prodzilla is a modern synthetic monitoring tool built in Rust. It's focused on testing complex user flows in production, whilst maintaining human readability.

Prodzilla supports chained requests to endpoints, passing of values from one response to another request, verifying responses are as expected, and outputting alerts via webhooks on failures. It also exposes an API that allow viewing results in json and manual triggering of probes. It's integrated with OpenTelemetry, so includes a trace_id for every request made to your system. May add a UI in future.

It's also lightning fast, runs with < 15mb of ram, and is free to host on [Shuttle](https://shuttle.rs/).

The long-term goals of Prodzilla are:
- Reduce divergence and duplication of code between blackbox, end-to-end testing and production observability
- Avoid situations where documented system behaviour is out of date, or system behaviour in specific situations is totally unknown
- Make testing in production easier

To be part of the community, or for any questions, join our [Discord](https://discord.gg/ud55NhraUm) or get in touch at [prodzilla.io](https://prodzilla.io/).

## Table of Contents

1. [Table of Contents](#table-of-contents)
2. [Getting Started](#getting-started)
3. [Configuring Synthetic Monitors](#configuring-synthetic-monitors)
   1. [Probes](#probes)
   2. [Stories](#stories)
   3. [Variables](#variables)
   4. [Expectations](#expectations)
4. [Notifications for Failures](#notifications-for-failures)
5. [Prodzilla Server Endpoints](#prodzilla-server-endpoints)
   1. [Get Probes and Stories](#get-probes-and-stories)
   2. [Get Probe and Story Results](#get-probe-and-story-results)
   3. [Trigger Probe or Story (In Development)](#trigger-probe-or-story-in-development)
6. [Deploying on Shuttle for Free](#deploying-on-shuttle-for-free)
7. [Feature Roadmap](#feature-roadmap)

## Getting Started

To get started probing your services, clone this repo, and in the root execute the command: 

```
cargo run
```

The application parses the [prodzilla.yml](/prodzilla.yml) file to generate a list of probes executed on a given schedule, and decide how to alert.

The bare minimum config required is: 

```yaml
probes:
  - name: Your Probe Name
    url: https://yoururl.com/some/path
    http_method: GET
    schedule:
      initial_delay: 5
      interval: 60
```

## Configuring Synthetic Monitors

Prodzilla offers two ways to check live endpoints, Probes and Stories.

### Probes
Probes define a single endpoint to be called with given parameters, and assert the response is as expected. This is a traditional synthetic monitor.

A complete Probe config looks as follows:

```yaml
  - name: Your Post Url
    url: https://your.site/some/path
    http_method: POST
    with:
      headers:
        x-client-id: ClientId
      body: '"{"test": true}"'
    expectations:
      - field: StatusCode
        operation: Equals 
        value: "200"
    schedule:
      initial_delay: 2
      interval: 60
    alerts:
      - url: https://notify.me/some/path
```

### Stories

Stories define a chain of calls to different endpoints, to emulate the flow a real user would go through. Values from the response of earlier calls can be input to the request of another using the ${{}} syntax.

```yaml
stories:
  - name: Get IP Address Info User Flow
    steps:
      - name: get-ip
        url: https://api.ipify.org/?format=json
        http_method: GET
        expectations:
          - field: StatusCode
            operation: Equals 
            value: "200"
      - name: get-location
        url: https://ipinfo.io/${{steps.get-ip.response.body.ip}}/geo
        http_method: GET
        expectations:
          - field: StatusCode
            operation: Equals 
            value: "200"
    schedule:
      initial_delay: 5
      interval: 10
    alerts:
      - url: https://webhook.site/54a9a526-c104-42a7-9b76-788e897390d8 

```

### Variables

One unique aspect of Prodzilla is the ability to substitute in values from earlier steps, environment variables, or generated values, as in the example above. Prodzilla currently supports the following variable substitutions.

| Substitute Value                             | Behaviour                                                                                                            |
| -------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| ${{steps.step-name.response.body}}           | Inserts the whole response body from the given step.                                                                 |
| ${{steps.step-name.response.body.fieldName}} | Inserts the value of a specific JSON field from a response body from a given step. Doesn't currently support arrays. |
| ${{generate.uuid}}                           | Inserts a generated UUID.                                                                                            |
| ${{env.VAR_NAME}}                            | Insert the environment variable VAR_NAME                                                                             |

Note that if a step name is used in a parameter but does not yet exist, Prodzilla will default to substituting an empty string.
If a requested environment variable is not set, Prodzilla will panic in order to avoid silent errors.

### Expectations

Expectations can be declared using the `expectations` block and supports an unlimited number of rules. Currently, the supported fields are `StatusCode` and `Body`, and the supported operations are `Equals`, `Contains`, `Matches` which accepts a regular expression, and `IsOneOf` (which accepts a string value separated by the pipe symbol `|`).

Expectations can be put on Probes, or Steps within Stories.


## Notifications for Failures

If expectations aren't met for a Probe or Story, a webhook will be sent to any urls configured within `alerts`.

```yaml
    - name: Probe or Story Name
      ...
      alerts:
        - url: https://webhook.site/54a9a526-c104-42a7-9b76-788e897390d8 
        - slack_webhook: https://hooks.slack.com/services/T000/B000/XXXX

```

The webhook looks as such:
```yaml
{
  "message": "Probe failed.",
  "probe_name": "Your Probe",
  "failure_timestamp": "2024-01-26T02:41:02.983025Z",
  "trace_id": "123456789abcdef"
}

```

Slack webhooks can also be used through the `slack_webhook` parameter, which are formatted like

> Probe Your Probe failed at 2024-06-10 08:16:33.935659994 UTC. Trace ID: 123456789abcdef

Slack, OpsGenie, and PagerDuty notification integrations are planned.

## Prodzilla Server Endpoints

Prodzilla also exposes a web server, which you can use to retrieve details about probes and stories, or trigger them. When running locally, these will exist at `localhost:3000`, e.g. `localhost:3000/stories`.


### Get Probes and Stories

These endpoints output the running probes and stories, as well as their current status.

Paths:
- /probes
- /stories

Example Response:

```json
[
    {
        "name": "get-ip-user-flow",
        "status": "OK", //  or "FAILING"
        "last_probed": "2024-02-05T10:01:10.665835200Z"
    }
    ...
]
```

### Get Probe and Story Results

These endpoints output all of the results for a probe or story.

Paths:
- /probes/{name}/results
- /stories/{name}/results

Query Parameters:
- show_response: bool - This determines whether the response, including the body, is output. Defaults to false.

Example Response (for stories, probes will look slightly different):
```json
[
    {
        "story_name": "get-ip-user-flow",
        "timestamp_started": "2024-02-05T10:02:40.670211600Z",
        "success": true,
        "step_results": [
            {
                "step_name": "get-ip",
                "timestamp_started": "2024-02-05T10:02:40.670318700Z",
                "success": true,
                "trace_id": "4df1663f21766a4f498eb4ba09180e93"
            },
            {
                "step_name": "get-location",
                "timestamp_started": "2024-02-05T10:02:40.931422100Z",
                "success": true,
                "trace_id": "28118007da1860cc5dd76c9128b14dee"
            }
        ]
    }
    ...
]
```

### Trigger Probe or Story (In Development)

These endpoints will trigger a probe or story immediately, store the result alongside the scheduled results, and return the result.

Paths:
- /probes/{name}/trigger
- /stories{name}/trigger

Example Response (for stories, probes will look slightly different):
```json
{
    "story_name": "get-ip-user-flow",
    "timestamp_started": "2024-02-10T00:36:05.768730400Z",
    "success": true,
    "step_results": [
        ...
    ]
}
```

## Deploying on Shuttle for Free

[Shuttle.rs](https://shuttle.rs) allows hosting of Rust apps for free. Check out [How I'm Getting Free Synthetic Monitoring](https://codingupastorm.dev/2023/11/07/prodzilla-and-shuttle/) for a tutorial on how to deploy Prodzilla to Shuttle for free.


## Feature Roadmap

The intention is to develop a base set of synthetic monitoring features, before focusing on longer-term goals such as:
- Supporting complex user flows typically not tested in production
- Increasing visibility of existing production behaviour from current and past probes
- Automatically generating probes based on OpenAPI schemas, and on deployment
- Other tools specifically to help test in production, such as flagging, managing and routing test requests and users
- Automatic doc generation - both for customers and internal use - based on observed behaviour

Progress on the base set of synthetic monitoring features is loosely tracked below:

:white_check_mark: = Ready
:bricks: = In development

- Protocol Support
    - HTTP / HTTPS Calls :white_check_mark:
    - gRPC
- Request Construction
    - Add headers :white_check_mark:
    - Add body :white_check_mark:
    - Custom timeouts
- Response Validation
    - Status code :white_check_mark:
    - Response body :white_check_mark:
    - Specific fields
    - Regex
- Yaml Objects / Reusable parameters / Human Readability
    - Reusable Request bodies
    - Reusable Authenticated users
    - Reusable Validation
- Result storage
    - In Memory :white_check_mark:
    - In a Database
- Output
    - JSON output of results for all probes :white_check_mark:
    - Prometheus Endpoint
    - UI output of results for all probes
- Forwarding alerts
    - Webhooks :white_check_mark:
    - Slack :bricks:
    - Email
    - Splunk / OpsGenie / PagerDuty / slack integrations?
- Complex Tests
    - Retries
    - Chained queries :white_check_mark:
    - Parameters in queries :white_check_mark:
    - Triggering probes manually :white_check_mark:
    - Generation of fields e.g. UUIDs :white_check_mark:
    - Parametrized tests
- Easy clone and deploy
    - On Shuttle :white_check_mark:
- CI / CD Integration
    - Standalone easy-to-install image :bricks:
    - Github Actions integration to trigger tests / use as smoke tests :bricks:
- Otel Support
    - TraceIds for every request :white_check_mark:

