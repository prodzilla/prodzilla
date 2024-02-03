# Prodzilla 🦖

Prodzilla is a modern synthetic monitoring tool built in Rust. It's focused on surfacing whether existing behaviour in production is as expected in a human-readable format, so that stakeholders, or even customers, can contribute to system verification.

It supports sending custom requests, verifying responses are as expected, and outputting alerts via webhooks, or viewing results in json from a web server. May add a UI in future.

It's also lightning fast, runs with ~8mb of ram, and free to host on Shuttle - all thanks to Rust.

To be part of the community, or for any questions, join our [Discord](https://discord.gg/ud55NhraUm) or get in touch at [prodzilla.io](https://prodzilla.io/).

## Table of Contents

- [Getting Started](#getting-started)
- [Configuring Synthetic Monitors](#configuring-synthetic-monitors)
    - [Probes](#probes)
    - [Stories](#stories)
    - [Variables](#variables)
    - [Expectations](#expectations)
- [Notifications for Failures](#notifications-for-failures)
- [Prodzilla Server Endpoints](#prodzilla-server-endpoints)
- [Deploying on Shuttle for free](#deploying-on-shuttle-for-free)
- [Feature Roadmap](#feature-roadmap)

## Getting Started

To get started probing your services, clone this repo, and in the root execute the command: 

```
cargo run
```

The application parses the [prodzilla.yml](/prodzilla.yml) file to generate a list of probes executed on a given schedule, and decide how to alert.

The bare minimum config required is: 

```yml
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

```yml
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

```yml
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

One unique aspect of Prodzilla is the ability to substitute in values from earlier steps, or generated values, as in the example above. Prodzilla currently supports the following variable substitutions.

Note that if a step name is used in a parameter but does not yet exist, Prodzilla will default to substituting an empty string.

| Substitute Value                             | Behaviour                                                                                                            |
|----------------------------------------------|----------------------------------------------------------------------------------------------------------------------|
| ${{steps.step-name.response.body}}           | Inserts the whole response body from the given step.                                                                 |
| ${{steps.step-name.response.body.fieldName}} | Inserts the value of a specific JSON field from a response body from a given step. Doesn't currently support arrays. |
| ${{generate.uuid}}                           | Inserts a brand new generated UUID.                                                                                  |

### Expectations

Expectations can be declared using the `expectations` block and supports an unlimited number of rules. Currently, the supported fields are `StatusCode` and `Body`, and the supported operations are `Equals`, `Contains`, and `IsOneOf` (which accepts a string value separated by the pipe symbol `|`).

Expectations can be put on Probes, or Steps within Stories.


## Notifications for Failures

If expectations aren't met for a Probe or Story, a webhook will be sent to any urls configured within `alerts`.

```yml
    - name: Probe or Story Name
      ...
      alerts:
        - url: https://webhook.site/54a9a526-c104-42a7-9b76-788e897390d8 

```

The webhook looks as such:
```yml
{
  "message": "Probe failed.",
  "probe_name": "Your Probe",
  "failure_timestamp": "2024-01-26T02:41:02.983025Z"
}

```

Slack, OpsGenie, and PagerDuty notification integrations are planned.

## Prodzilla Server Endpoints

You can visit `localhost:3000/probe_results` to get the latest 100 probe results for each probe you've initialised, which will look like this:

```json
{
    "Prodzilla Github": [
        {
            "probe_name": "Prodzilla Github",
            "timestamp_started": "2024-01-08T09:14:14.051667600Z",
            "success": true,
            "response": {
                "timestamp": "2024-01-08T09:14:15.259735200Z",
                "status_code": 200,
                "body": "<!DOCTYPE html>\n<html..."
            }
        },
        {
            "probe_name": "Prodzilla Github",
            "timestamp_started": "2024-01-08T09:14:24.053560100Z",
            "success": true,
            "response": {
                "timestamp": "2024-01-08T09:14:24.082027200Z",
                "status_code": 200,
                "body": "<!DOCTYPE html>\n<html..."
            }
        }
    ],
    "Webhook Receiver Probe": [
        {
            "probe_name": "Webhook Receiver Probe",
            "timestamp_started": "2024-01-08T09:14:11.052497Z",
            "success": true,
            "response": {
                "timestamp": "2024-01-08T09:14:12.621906Z",
                "status_code": 200,
                "body": "This URL has no default content configured. <a href=\"https://webhook.site/#!/54a9a526-c104-42a7-9b76-788e897390d8\">View in Webhook.site</a>."
            }
        }
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
    - Request bodies
    - Authenticated users
    - Validation
- Result storage
    - NativeDB?
- Output
    - JSON output of results for all probes :white_check_mark:
    - Prometheus Endpoint
    - UI output of results for all probes
- Forwarding alerts
    - Webhooks :white_check_mark:
    - Email
    - Splunk / OpsGenie / PagerDuty / slack integrations?
- Complex Tests
    - Retries
    - Chained queries :white_check_mark:
    - Parameters in queries :bricks:
    - Generation of fields e.g. UUIDs :bricks:
    - Parametrized tests
- Easy clone and deploy
    - On Shuttle :white_check_mark:
- CI / CD Integration
    - Standalone easy-to-install image
    - Github Actions integration to trigger tests / use as smoke tests
- Otel Support
    - TraceIds for every request