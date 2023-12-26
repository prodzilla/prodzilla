# Prodzilla

A synthetic monitoring tool built in Rust.

Planned Feature Roadmap:

- Protocol Support
    - HTTP / HTTPS Calls
        - GET
        - POST
        - PUT
        - PATCH
    - Grpc
- Request Construction:
    - Add headers
    - Add body
- Response Validation
    - Status code
    - Response body
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
    - Chained queries or calls
- CI / CD Integration
    - Standalone easy-to-install image
    - Github Actions integration to trigger tests / use as smoke tests