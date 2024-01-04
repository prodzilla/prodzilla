# Prodzilla 🦖

Prodzilla is a modern synthetic monitoring tool built in Rust. It's focused on surfacing whether existing behaviour in production is as expected in a human-readable format, so that even customers or stakeholders can contribute to system verification. 

A SaaS option will be available soon. More at [prodzilla.io](https://prodzilla.io/).

It's currently in active development.

## Usage

The application parses the `prodzilla.yml` file to generate a list of probes executed on a given schedule.

The yml file will represent a view of the intended behaviour of the services that any human / stakeholder can read and understand.

A full view of this is available in `prodzilla-future.yml`.

```yml
stories:
  - name: Create cardholder card
    steps:
      - name: Create cardholder
        as: CardholderUser
        url: https://api.cardwebsite.com/cardholders/create
        http_method: POST
        with: CreateCardholderRequest
        expect_back: ValidCreateCardholderResponse

      - name: Create card
        as: CardholderUser
        url: https://api.cardwebsite.com/cards/create
        http_method: POST
        with: CreateCardRequest
        expect_back: ValidCreateCardResponse

      - name: Get card in Admin panel
        as: CardholderUser
        url: https://api.cardwebsite.com/cards/create
        http_method: POST
        with: CreateCardRequest
        expect_back: ValidCreateCardResponse

    schedule: EveryMinute10sDelay
```

## Feature Roadmap

:white_check_mark: = Ready
:bricks: = In development

- Protocol Support
    - HTTP / HTTPS Calls
        - GET :white_check_mark:
        - POST :white_check_mark:
        - PUT :white_check_mark:
        - PATCH :white_check_mark:
    - Grpc
- Request Construction
    - Add headers :bricks:
    - Add body :white_check_mark:
    - Add custom timeouts
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
    - Webhooks
    - Email
    - Splunk / OpsGenie / PagerDuty / slack integrations?
- Complex Tests
    - Retries
    - Chained queries
    - Parameters in queries
    - Parametrized tests
- CI / CD Integration
    - Standalone easy-to-install image
    - Github Actions integration to trigger tests / use as smoke tests