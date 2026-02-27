# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Prodzilla is a lightweight synthetic monitoring tool written in Rust that tests user flows in production. It supports single-step monitors (traditional health checks) and multi-step monitors (chained requests with variable passing between steps). Runs with <15MB RAM. Fully integrated with OpenTelemetry for tracing and metrics.

## Build & Development Commands

```bash
cargo run                        # Run with default prodzilla.yml config
cargo run -- -f custom.yml       # Run with custom config file
cargo build --locked --release   # Release build
cargo test --verbose             # Run all tests
cargo clippy --all-targets --all-features  # Lint
cargo fmt --all -- --check       # Format check
```

The web server listens on port 3000. Prometheus metrics (if enabled) default to port 9464.

## Architecture

Single binary, async Rust application using Axum (web) and Tokio (runtime).

**Core flow:** `main.rs` → loads YAML config → initializes OTel → spawns per-monitor async scheduling tasks → starts Axum web server.

**Key modules:**

- `src/config.rs` — YAML config loading with validation (unique monitor names, mutual exclusivity of single-step vs multi-step fields)
- `src/app_state.rs` — Shared state: `RwLock<HashMap<String, Vec<MonitorResult>>>` storing last 100 results per monitor
- `src/monitor/` — Core monitoring logic:
  - `model.rs` — Monitor, Step, Expectation, MonitorResult, StepResult types
  - `schedule.rs` — Scheduling loop, spawns tokio tasks per monitor with initial_delay + interval
  - `monitor_logic.rs` — `Monitorable` trait, step execution, variable substitution orchestration
  - `http.rs` — HTTP calls via lazy-static reqwest::Client, OTel trace context propagation
  - `expectations.rs` — Response validation (Equals, NotEquals, Contains, Matches regex, IsOneOf with `|` separator)
  - `variables.rs` — `${{ steps.name.response.body.field }}`, `${{ generate.uuid }}`, `${{ env.VAR }}` substitution via regex
- `src/web_server/` — Axum routes: `GET /monitors`, `GET /monitors/{name}/results`, `GET /monitors/{name}/trigger`, `GET /metrics`
- `src/alerts/outbound_webhook.rs` — Webhook alerting with auto-detected Slack formatting, body truncation to 500 chars
- `src/otel/` — OpenTelemetry setup: metrics (OTLP/stdout/Prometheus) and tracing (OTLP/stdout)
- `src/errors.rs` — Custom error types
- `src/test_utils.rs` — Builder functions for test monitor construction

**Terminology:** "Monitors" is the unified term (replaces older "probes"/"stories" naming). Each monitor has one or more "steps."

## Testing Patterns

- Unit tests are co-located in modules (`#[cfg(test)]` blocks)
- `wiremock` for HTTP server mocking in tests
- `test_utils.rs` provides builder helpers: `get_simple_monitor()`, `get_default_schedule()`, etc.
- Integration tests in `src/web_server/tests.rs`

## Configuration

Config file is YAML (`prodzilla.yml` by default). Key structure:
- Monitors define `url` + `http_method` (single-step) OR `steps` array (multi-step) — never both
- Variable syntax: `${{ steps.step-name.response.body.fieldName }}` for chaining step outputs
- OTel configured via standard env vars: `OTEL_EXPORTER_OTLP_ENDPOINT`, `OTEL_METRICS_EXPORTER`, `OTEL_TRACES_EXPORTER`, `RUST_LOG`

## CI

- **test.yaml** — `cargo build && cargo test` on push to main and PRs
- **lint.yaml** — clippy + fmt on all pushes
- **release.yaml** — GitHub releases on `v[0-9]+.*` tags
- **docker.yaml** — Multi-platform Docker images (amd64/arm64) to GHCR on tag push
