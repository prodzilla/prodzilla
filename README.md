# Prodzilla 🦖

Prodzilla is a modern synthetic monitoring tool built in Rust. It's focused on surfacing whether existing behaviour in production is as expected in a human-readable format, so that stakeholders, or even customers, can contribute to system verification.

It supports sending custom requests, verifying responses are as expected, and outputting alerts via webhooks.

It's also lightning fast, runs with ~8mb of ram, and free to host on Shuttle - all thanks to Rust.

To be part of the community, or for any questions, join our [Discord](https://discord.gg/ud55NhraUm) or get in touch at [prodzilla.io](https://prodzilla.io/).

## Table of Contents

- [Getting Started](#getting-started)
- [Deploying on Shuttle for free](#deploying-on-shuttle-for-free)
- [Notifications for Probe Results](#notifications-for-probe-results)
- [Feature Roadmap](#feature-roadmap)

## Getting Started

To get started probing your services, all you need to do is clone this repo, and in the root execute the command: 

```
cargo run
```

The application parses the `prodzilla.yml` file to generate a list of probes executed on a given schedule, and decide how to alert.

The bare minimum config required for a probe is: 

```yml
probes:
  - name: Your Probe Name
    url: https://github.com/prodzilla/prodzilla
    http_method: GET
    schedule:
      initial_delay: 5
      interval: 60
```

A full view of currently supported features can be inferred by checking out the [prodzilla.yml](/prodzilla.yml).

## Deploying on Shuttle for Free

[Shuttle.rs](https://shuttle.rs) allows hosting of Rust apps for free. Check out [How I'm Getting Free Synthetic Monitoring](https://codingupastorm.dev/2023/11/07/prodzilla-and-shuttle/) for a tutorial on how to deploy Prodzilla to Shuttle for free.


## Notifications for Probe Results

Prodzilla will send through a webhook when one of your probes fails due to expectations not being met. Expectations can be declared using the `expectations` block and supports an unlimited number of rules. Currently, the supported fields are `StatusCode` and `Body`, and the supported operations are `Equals`, `Contains`, and `IsOneOf` (which accepts a string value separated by the pipe symbol `|`). 

If expectations aren't met, a copy of the result will be sent as a webhook to any urls configured within `alerts`.

```yml
    expectations:
      - field: StatusCode
        operation: Equals 
        value: "200"
      - field: Body
        operation: Contains 
        value: "prodzilla"
    alerts:
      - url: https://webhook.site/54a9a526-c104-42a7-9b76-788e897390d8 
```

You can also visit `/probe_results` to get the latest 100 probe results for each probe you've initialised, which will look like this:

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
- Authentication
    - Bearer Tokens
    - Requests
- Yaml Objects - Reusable parameters
    - Request bodies
    - Authenticated users
    - Validation
- Result storage
    - NativeDB?
- UI / Output
    - JSON output of results for all probes :white_check_mark:
    - UI output of results for all probes
- Forwarding alerts
    - Webhooks :white_check_mark:
    - Email
    - Splunk / OpsGenie / PagerDuty / slack integrations?
- Complex Tests
    - Retries
    - Chained queries
    - Parameters in queries
    - Parametrized tests
- Easy clone and deploy
    - On Shuttle :bricks:
- CI / CD Integration
    - Standalone easy-to-install image
    - Github Actions integration to trigger tests / use as smoke tests
- Otel Support
    - TraceIds for every request