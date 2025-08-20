# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Development Commands
- `cargo run` - Run the application with default config (prodzilla.yml)
- `cargo run -- -f <config_file>` - Run with custom config file
- `cargo run -- --help` - Show all command line options
- `cargo build` - Build the project
- `cargo build --release` - Build optimized release binary
- `cargo test` - Run all tests
- `cargo check` - Check code without building

### Docker Commands
- `docker build -t prodzilla .` - Build Docker image
- `docker run -v $(pwd)/prodzilla.yml:/prodzilla.yml prodzilla/prodzilla:latest` - Run with Docker

## Architecture

Prodzilla is a Rust-based synthetic monitoring tool that executes HTTP probes and multi-step stories on scheduled intervals.

### Core Components

**Main Application Flow:**
- `main.rs` - Entry point, parses CLI args, initializes OpenTelemetry, starts monitoring and web server
- `config.rs` - Loads and parses YAML configuration with environment variable substitution
- `app_state.rs` - Shared application state containing configuration and results

**Monitoring Engine:**
- `probe/` - Core monitoring functionality
  - `model.rs` - Data structures for Probes and Stories
  - `schedule.rs` - Tokio-based scheduling for probes and stories
  - `probe_logic.rs` - Execution logic for individual probes
  - `http_probe.rs` - HTTP client implementation
  - `expectations.rs` - Response validation logic
  - `variables.rs` - Variable substitution (${{}} syntax)

**Web Server:**
- `web_server/` - Axum-based HTTP API
  - `probes.rs` - Endpoints for probe status and results
  - `stories.rs` - Endpoints for story status and results
  - `prometheus_metrics.rs` - Prometheus metrics export

**Alerting:**
- `alerts/` - Webhook notifications for failures
  - `outbound_webhook.rs` - HTTP webhook implementation with Slack formatting

**Observability:**
- `otel/` - OpenTelemetry integration
  - `tracing.rs` - Distributed tracing setup
  - `metrics.rs` - Metrics collection and export

### Key Patterns

**Configuration Structure:**
- YAML-based configuration in `prodzilla.yml`
- Environment variable substitution with `${{ env.VAR_NAME }}`
- Probes for single endpoint monitoring
- Stories for multi-step user flows

**Variable Substitution:**
- `${{ steps.step-name.response.body }}` - Full response body from previous step
- `${{ steps.step-name.response.body.field }}` - JSON field from response
- `${{ generate.uuid }}` - Generate UUID
- `${{ env.VAR_NAME }}` - Environment variable

**Scheduling:**
- Uses Tokio intervals for probe execution
- Each probe/story runs independently
- Results stored in memory (AppState)

**Error Handling:**
- Custom error types in `errors.rs`
- Expectation failures trigger alerts
- OpenTelemetry spans marked with error status on failures

## Testing

Tests are located within each module using `#[cfg(test)]`.
Key test file: `src/config.rs` contains configuration loading tests.
Use `test_utils.rs` for shared testing utilities.

## API Endpoints

The web server (default port 3000) exposes these endpoints:

### Get Probes and Stories
- `GET /probes` - List all probes with status and last execution time
- `GET /stories` - List all stories with status and last execution time

### Get Results
- `GET /probes/{name}/results` - Get all results for a specific probe
- `GET /stories/{name}/results` - Get all results for a specific story
- Query parameter: `show_response=true` - Include response body in results

### Trigger Execution (In Development)
- `POST /probes/{name}/trigger` - Manually trigger a probe
- `POST /stories/{name}/trigger` - Manually trigger a story

### Metrics
- `GET /metrics` - Prometheus metrics (when `OTEL_METRICS_EXPORTER=prometheus`)

## Configuration

Default config file: `prodzilla.yml`
Server runs on port 3000 by default.
Prometheus metrics on port 9464 when enabled.

Environment variables for OpenTelemetry:
- `OTEL_EXPORTER_OTLP_ENDPOINT`
- `OTEL_METRICS_EXPORTER` (otlp/stdout/prometheus)
- `OTEL_TRACES_EXPORTER` (otlp/stdout)
- `RUST_LOG` for logging level