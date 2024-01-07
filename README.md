# Prodzilla 🦖

Prodzilla is a modern synthetic monitoring tool built in Rust. It's focused on surfacing whether existing behaviour in production is as expected in a human-readable format, so that stakeholders, or even customers, can contribute to system verification. 

A SaaS option will be available soon. More at [prodzilla.io](https://prodzilla.io/).

It's in active development, but currently supports sending custom requests, verifying responses are as expected, and outputting alerts via webhooks.

## Table of Contents

- [Getting Started](#getting-started)
- [Feature Roadmap](#feature-roadmap)

## Getting Started

The application parses the `prodzilla.yml` file to generate a list of probes executed on a given schedule, and decide how to alert.

The bare minimum config required for a probe is: 

```yml
probes:
  - name: Your Probe Name
    url: https://github.com/prodzilla/prodzilla
    http_method: GET
    schedule:
      initial_delay: 5
      interval: 10
```

A full view of currently supported features can be inferred by checking out the [prodzilla.yml](/prodzilla.yml).


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
    - JSON output of results for all probes
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