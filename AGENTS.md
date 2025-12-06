# Project overview
- Prodzilla is a Rust 2021 synthetic monitoring service using `axum` for HTTP, `tokio` runtime, `reqwest` for outbound calls, `tracing` for logs, and OpenTelemetry for traces/metrics. It exposes a JSON API and optionally a Prometheus endpoint.

## Language, toolchain, formatting
- Use Rust edition 2021. Prefer stable toolchain.
- Always run `cargo fmt` and `cargo clippy -D warnings` on edits.
- Keep code readable with descriptive names; avoid single-letter or abbreviated identifiers.

## Dependencies and architecture
- Web server: `axum` 0.7; return `axum::Json<T>` for JSON responses; inject shared state via `Extension<Arc<AppState>>`.
- Async runtime: `tokio` 1.x; never block the runtime (no std::thread::sleep).
- HTTP client: `reqwest` 0.11 with a single reused client via `lazy_static!`. Reuse the existing client(s) instead of creating new ones.
- Telemetry: OpenTelemetry via `opentelemetry`, `opentelemetry-otlp`, `opentelemetry-prometheus`, `tracing`, `tracing-subscriber`.

## Error handling
- Functions that cross async/task boundaries should return `Result<T, Box<dyn std::error::Error + Send>>` (or `Box<dyn Error + Send>` for errors) to preserve sendability.
- Prefer converting third-party errors with `MapToSendError` (see `errors.rs`) rather than `.unwrap()` or `.expect()`.
- Only use `.unwrap()` in tests or truly infallible contexts; otherwise bubble errors up.
- When implementing errors, implement `std::fmt::Display` and `std::error::Error`.

## Logging and tracing
- Use `tracing` macros (`trace!`, `debug!`, `info!`, `warn!`, `error!`), not `println!`.
- Instrument work with OpenTelemetry spans. For HTTP calls: propagate context using `opentelemetry_http::HeaderInjector`; attach attributes:
  - HTTP spans: `http.method`, `http.url`, `http.status_code`
  - Step/probe/story spans: `name`, `type` (probe|story|step), and `story_name` on step spans
- On errors or expectation failures: record error on the active span and set span status to error.
- Respect sensitive data: if an operation is marked `sensitive`, do not log or attach response body; use “Redacted”.

## Metrics
- Use the existing `Metrics` in `src/otel/metrics.rs`:
  - `runs` (Counter<u64>)
  - `duration` (Histogram<u64>, milliseconds)
  - `errors` (Counter<u64>)
  - `status` (Gauge<u64>, 0=OK, 1=Error)
  - `http_status_code` (Gauge<u64>, 0 if HTTP call failed)
- Always include attributes `name` and `type` (probe|story|step). Steps also include `story_name`.
- If you add new monitors or flows, ensure metrics update paths mirror existing patterns.

## OpenTelemetry exporters and env
- Follow existing env-based configuration:
  - `OTEL_EXPORTER_OTLP_ENDPOINT` (default `http://localhost:4317`)
  - `OTEL_EXPORTER_OTLP_PROTOCOL` in {`grpc`, `http/protobuf`, `http/json`}
  - `OTEL_EXPORTER_OTLP_TIMEOUT` seconds (default 10)
  - `OTEL_METRICS_EXPORTER` in {`otlp`, `stdout`, `prometheus`} (unset = disabled)
  - `OTEL_TRACES_EXPORTER` in {`otlp`, `stdout`} (unset = disabled)
  - `OTEL_RESOURCE_ATTRIBUTES` as standard
- Prometheus:
  - `OTEL_METRICS_EXPORTER=prometheus`
  - `OTEL_EXPORTER_PROMETHEUS_HOST` (default `localhost`)
  - `OTEL_EXPORTER_PROMETHEUS_PORT` (default `9464`)

## HTTP clients and timeouts
- Use the module-level `reqwest::Client` singletons (via `lazy_static!`) with user-agent:
  - Probes: `Prodzilla Probe/1.0`
  - Alerts: `Prodzilla Alert/1.0`
- Apply request timeouts (default 10s for probes; alerts use 10s); make timeouts configurable via parameters where relevant.
- Propagate trace headers on outbound requests.

## State and concurrency
- Shared state is in `AppState` guarded by `RwLock`s. Do not hold locks across `.await` points.
- Clone `Arc<AppState>` when spawning tasks; ensure spawned tasks are `Send`.
- Scheduling:
  - Use `tokio::spawn` with the provided `probing_loop` pattern.
  - Never block the loop; sleep using `tokio::time`.

## Web API conventions
- Routes live under `src/web_server`. Follow existing route structure and response types.
- Prefer returning `Json<T>` with serializable DTOs from `src/web_server/model.rs`.
- Avoid panics in handlers. If you touch these, replace `.unwrap()` with graceful error responses and proper status codes.
- Honor `show_response` query param: if false, strip bodies before returning.

## Config and YAML
- Deserialize config with `serde_yaml`; top-level shape is `Config { probes, stories }`.
- Preserve variable substitution semantics (leading and trailing whitespace is optional and trimmed):
  - `${{steps.<step-name>.response.body}}` → entire body
  - `${{steps.<step-name>.response.body.<field>}}` → JSON field
  - `${{generate.uuid}}` → new UUID
  - `${{ env.VAR_NAME }}` → environment variable (logs a warning if missing; substitutes empty string)
- Keep `#[serde(default)]` for optional vectors/fields and `#[serde(skip_serializing_if = "Option::is_none")]` for optional outputs.

## Expectations
- Supported fields: `StatusCode`, `Body`
- Supported ops: `Equals`, `NotEquals`, `Contains`, `NotContains`, `Matches` (regex), `IsOneOf` (pipe-separated)
- Maintain existing evaluation flow; add new ops in `probe::expectations` while keeping pure, testable functions.

## Testing
- Use `#[tokio::test]` with `wiremock` for HTTP mocking. Avoid real network calls.
- Keep tests deterministic and fast; prefer short delays in mocks where necessary.
- Include tracing setup in tests that validate header propagation.

## Security and privacy
- Respect `sensitive: bool` on probes/steps:
  - Do not log or include raw response bodies in alerts/metrics when sensitive.
  - Use truncated bodies (<=500 chars) only for non-sensitive responses.
- Never include secrets in logs; prefer environment variables for secret material.

## Style and structure
- Follow module structure: domain logic under `src/probe`, telemetry under `src/otel`, web under `src/web_server`, alerts under `src/alerts`.
- Keep functions small with early returns; avoid deep nesting.
- Prefer explicit types in public APIs; keep generics constrained.
- Minimize clones; where needed, clone only cheap types or use references.

## When making changes
- Do not introduce new global clients; reuse existing singletons and patterns.
- Add observability (tracing + metrics) to new flows that perform external IO or meaningful work.
- Update README and config examples only if you change the public behavior or configuration surface.
- If adding metrics or attributes, ensure they are consistently attached for probes, stories, and steps.

## Developer quickstart
- Build/run:
  - `cargo run -- --file prodzilla.yml`
- Format/lint:
  - `cargo fmt --all`
  - `cargo clippy -D warnings`
- Tests:
  - `cargo test`

## Local observability
- Traces:
  - Set `OTEL_TRACES_EXPORTER=stdout` to print spans to stdout locally.
- Metrics (Prometheus):
  - Set `OTEL_METRICS_EXPORTER=prometheus`.
  - Server binds using `OTEL_EXPORTER_PROMETHEUS_HOST` (default `localhost`) and `OTEL_EXPORTER_PROMETHEUS_PORT` (default `9464`).
  - Scrape path is `/metrics`.

## HTTP clients (reuse only)
- Probes HTTP client (singleton): `src/probe/http_probe.rs` (user-agent `Prodzilla Probe/1.0`).
- Alerts HTTP client (singleton): `src/alerts/outbound_webhook.rs` (user-agent `Prodzilla Alert/1.0`).
- These are created via `lazy_static!`; do not introduce new clients—reuse these.

## Timeouts
- Probes:
  - Default request timeout: 10s (`DEFAULT_REQUEST_TIMEOUT_SECS` in `src/probe/http_probe.rs`).
  - Override per-call with `with.timeout_seconds` (`ProbeInputParameters.timeout_seconds`).
- Alerts:
  - Webhook timeout: 10s (`REQUEST_TIMEOUT_SECS` in `src/alerts/outbound_webhook.rs`).

## Web API routes (for reference)
- `/`
- `/probes`
- `/probes/:name/results`
- `/probes/:name/trigger`
- `/stories`
- `/stories/:name/results`
- `/stories/:name/trigger`
- `/metrics` (only when Prometheus metrics are enabled)

## Config entry points
- Default config file is `prodzilla.yml`. Override via CLI: `--file <path>`.
- YAML loading and variable substitution live in `src/config.rs`.

## Telemetry for outbound HTTP
- Create/enter a span and propagate context headers using `opentelemetry_http::HeaderInjector`.
- Set attributes for each call: `http.method`, `http.url`, and `http.status_code`.
- For `sensitive: true`, do not attach response bodies to spans; otherwise, truncate bodies to <= 500 chars.

## Testing tips
- Use `wiremock` for HTTP; avoid real network calls.
- Keep tests deterministic with short, bounded delays only where necessary.

## Non-goals
- Do not introduce a new web framework, DI container, or async runtime.
- Do not add database persistence without explicit instruction.

