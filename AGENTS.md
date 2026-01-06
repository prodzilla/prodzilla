# AGENTS.md - AI Agent Guide for Prodzilla

> **Purpose**: This document provides comprehensive context and actionable information for AI agents (GitHub Copilot, Cursor, Claude Code, etc.) to effectively assist with the Prodzilla codebase.

## Table of Contents
- [Project Overview](#project-overview)
- [Tech Stack and Supported Runtimes](#tech-stack-and-supported-runtimes)
- [Repository Layout](#repository-layout)
- [Development Environment Setup](#development-environment-setup)
- [Build Steps](#build-steps)
- [Execution Instructions](#execution-instructions)
- [Test Strategy](#test-strategy)
- [Linting, Formatting, and Code Quality](#linting-formatting-and-code-quality)
- [CI/CD Overview](#cicd-overview)
- [Branching, Commit, and Release Conventions](#branching-commit-and-release-conventions)
- [Code, Design, and Architectural Conventions](#code-design-and-architectural-conventions)
- [Security and Performance Guidelines](#security-and-performance-guidelines)
- [Common Pitfalls and Troubleshooting](#common-pitfalls-and-troubleshooting)
- [AI Agent Prompts/Recipes](#ai-agent-promptsrecipes)
- [Documentation Links](#documentation-links)

---

## Project Overview

**Prodzilla** is a modern synthetic monitoring tool built in Rust, focused on testing complex user flows in production while maintaining human readability.

### What is Prodzilla?
- A Rust 2021 synthetic monitoring service using `axum` for HTTP, `tokio` runtime, `reqwest` for outbound calls, `tracing` for logs, and OpenTelemetry for traces/metrics
- Exposes a JSON API and optionally a Prometheus endpoint
- Supports chained requests (called "stories") to test user flows
- Enables variable substitution between steps
- Provides alerting via webhooks (with special Slack formatting)
- Lightweight: runs with < 15MB of RAM
- Free to host on [Shuttle.rs](https://shuttle.rs)

### Key Features
- **Probes**: Single endpoint monitors with configurable expectations
- **Stories**: Multi-step user flows with variable passing between steps
- **Variable Substitution**: Support for step outputs, environment variables, and UUID generation
- **Expectations**: Validate status codes, response bodies with regex, equality, and containment checks
- **Observability**: Full OpenTelemetry integration for traces and metrics
- **Alerts**: Webhook notifications on failures with special Slack message formatting

### Goals
- Reduce divergence between blackbox testing and production observability
- Avoid outdated documentation by testing actual system behavior
- Make testing in production easier and more accessible

---

## Tech Stack and Supported Runtimes

### Language and Edition
- **Rust**: Edition 2021
- **Toolchain**: Stable (tested with 1.92.0+)
- **Minimum Version**: Rust 1.70+ recommended

### Core Dependencies
| Dependency | Version | Purpose |
|------------|---------|---------|
| `axum` | 0.7.2 | HTTP web framework |
| `tokio` | 1.x | Async runtime with "full" features |
| `reqwest` | 0.11 | HTTP client for outbound requests |
| `serde` | 1.0 | Serialization/deserialization |
| `serde_yaml` | 0.9 | YAML config parsing |
| `tracing` | 0.1 | Structured logging |
| `tracing-subscriber` | 0.3 | Log subscriber with env-filter |
| `opentelemetry` | 0.23.0 | Telemetry framework |
| `opentelemetry-otlp` | 0.16.0 | OTLP exporter |
| `opentelemetry-prometheus` | 0.16.0 | Prometheus exporter |
| `lazy_static` | 1.4.0 | Global static clients |
| `wiremock` | 0.5.22 | HTTP mocking for tests |
| `chrono` | 0.4.31 | Datetime handling |
| `regex` | 1.10.3 | Regular expression support |
| `uuid` | 1 | UUID generation |
| `clap` | 4.5.4 | CLI argument parsing |

### Runtime Requirements
- **Operating System**: Linux, macOS, Windows (tested on Ubuntu in CI)
- **Memory**: < 15MB typical usage
- **Network**: Outbound HTTP/HTTPS access for probes
- **Optional**: Docker for containerized deployment

---

## Repository Layout

```
prodzilla/
├── .github/
│   └── workflows/           # CI/CD workflow definitions
│       ├── test.yaml        # Build and test on push/PR
│       ├── lint.yaml        # Clippy and fmt checks
│       ├── docker.yaml      # Docker image builds on tags
│       └── release.yaml     # GitHub release creation
├── src/
│   ├── main.rs             # Application entry point
│   ├── config.rs           # YAML config parsing and variable substitution
│   ├── app_state.rs        # Shared application state
│   ├── errors.rs           # Error types and MapToSendError trait
│   ├── test_utils.rs       # Shared test utilities
│   ├── probe/              # Probe execution logic
│   │   ├── mod.rs          # Probe types and execution
│   │   ├── http_probe.rs   # HTTP client and probe execution
│   │   ├── expectations.rs # Expectation validation
│   │   └── story.rs        # Multi-step story execution
│   ├── otel/               # OpenTelemetry integration
│   │   ├── mod.rs          # OTEL initialization
│   │   ├── metrics.rs      # Metrics definitions
│   │   └── traces.rs       # Trace setup and propagation
│   ├── alerts/             # Alert dispatching
│   │   ├── mod.rs          # Alert types
│   │   └── outbound_webhook.rs # Webhook delivery
│   └── web_server/         # HTTP API server
│       ├── mod.rs          # Route setup
│       ├── handlers.rs     # Request handlers
│       └── model.rs        # API response models
├── AGENTS.md               # This file - AI agent guide
├── README.md               # User-facing documentation
├── CONTRIBUTING.md         # Contribution guidelines
├── Cargo.toml              # Rust project manifest
├── Cargo.lock              # Dependency lock file
├── Dockerfile              # Container image definition
├── prodzilla.yml           # Example configuration file
└── LICENSE                 # Apache 2.0 license

```

### Key Files to Know
- **`src/main.rs`**: Entry point, CLI parsing, server setup
- **`src/config.rs`**: YAML deserialization and `${{}}` variable substitution
- **`src/app_state.rs`**: `AppState` with `RwLock`-guarded probe/story results
- **`src/probe/http_probe.rs`**: Core HTTP execution logic with singleton client
- **`src/probe/expectations.rs`**: Pure functions for validating expectations
- **`src/otel/`**: OpenTelemetry setup, metric registration, trace propagation
- **`src/web_server/`**: Axum routes for `/probes`, `/stories`, `/metrics`
- **`prodzilla.yml`**: Sample config showing probe and story syntax

---

## Development Environment Setup

### Prerequisites
1. **Install Rust**: Use [rustup](https://rustup.rs/)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup update stable
   ```

2. **Verify Installation**:
   ```bash
   rustc --version  # Should be 1.70+
   cargo --version
   ```

3. **Optional Tools**:
   - `cargo-watch` for live reloading: `cargo install cargo-watch`
   - Docker for containerized testing

### Clone the Repository
```bash
git clone https://github.com/prodzilla/prodzilla.git
cd prodzilla
```

### Environment Variables
No environment variables are strictly required for local development. Optional OpenTelemetry configuration:

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level: `trace`, `debug`, `info`, `warn`, `error` |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | `http://localhost:4317` | OTLP collector endpoint |
| `OTEL_EXPORTER_OTLP_PROTOCOL` | `grpc` | Protocol: `grpc`, `http/protobuf`, `http/json` |
| `OTEL_EXPORTER_OTLP_TIMEOUT` | `10` | Exporter timeout in seconds |
| `OTEL_METRICS_EXPORTER` | (unset) | Metrics exporter: `otlp`, `stdout`, `prometheus` |
| `OTEL_TRACES_EXPORTER` | (unset) | Trace exporter: `otlp`, `stdout` |
| `OTEL_EXPORTER_PROMETHEUS_HOST` | `localhost` | Prometheus exporter host |
| `OTEL_EXPORTER_PROMETHEUS_PORT` | `9464` | Prometheus exporter port |
| `OTEL_RESOURCE_ATTRIBUTES` | - | Resource attributes (e.g., `service.name=prodzilla`) |

### Configuration File
Edit `prodzilla.yml` or create a custom config file. Minimal config:
```yaml
probes:
  - name: example-probe
    url: https://httpbin.org/status/200
    http_method: GET
    schedule:
      initial_delay: 5
      interval: 60
```

---

## Build Steps

### Standard Build
```bash
cargo build
```
This compiles the project in debug mode to `target/debug/prodzilla`.

### Release Build
```bash
cargo build --release
```
Produces an optimized binary in `target/release/prodzilla`.

### Build with Verbose Output
```bash
cargo build --verbose
```

### Clean Build Artifacts
```bash
cargo clean
```

### Check Without Building
```bash
cargo check
```
Faster than full build; useful for quick validation.

---

## Execution Instructions

### Run Locally (Debug)
```bash
cargo run -- --file prodzilla.yml
```

### Run Release Binary
```bash
cargo build --release
./target/release/prodzilla --file prodzilla.yml
```

### Command-Line Options
```bash
prodzilla --help
```
- `--file <PATH>` or `-f <PATH>`: Path to config file (default: `prodzilla.yml`)

### Using Docker
```bash
# Pull from GitHub Container Registry
docker pull ghcr.io/prodzilla/prodzilla:latest

# Run with local config
docker run -v $(pwd)/prodzilla.yml:/prodzilla.yml ghcr.io/prodzilla/prodzilla:latest
```

### Local Development with Live Reload
```bash
cargo install cargo-watch
cargo watch -x 'run -- --file prodzilla.yml'
```

### Accessing the API
Once running, the server binds to `localhost:3000`:
- `http://localhost:3000/` - Health check
- `http://localhost:3000/probes` - List all probes
- `http://localhost:3000/stories` - List all stories
- `http://localhost:3000/probes/{name}/results` - Get probe results
- `http://localhost:3000/stories/{name}/results` - Get story results
- `http://localhost:3000/metrics` - Prometheus metrics (if enabled)

---

## Test Strategy

### Test Framework
- **Framework**: `#[tokio::test]` for async tests
- **HTTP Mocking**: `wiremock` for deterministic HTTP tests
- **Location**: Tests live in the same files as implementation (inline `#[cfg(test)]` modules)

### Running Tests
```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests in a specific module
cargo test probe::

# Run with verbose output
cargo test --verbose
```

### Test Conventions
- Use `#[tokio::test]` for async test functions
- Mock HTTP with `wiremock::MockServer`
- Avoid real network calls
- Keep tests deterministic and fast
- Include tracing setup in tests validating header propagation
- Use `.unwrap()` freely in tests (not in production code)

### Coverage Expectations
- All new probe/story execution logic should have tests
- All expectation operations should have unit tests
- HTTP client behavior should be tested with mocks
- Variable substitution should have comprehensive test cases

### Example Test Pattern
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};

    #[tokio::test]
    async fn test_probe_execution() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;
        
        // Test logic here
    }
}
```

---

## Linting, Formatting, and Code Quality

### Formatting with `rustfmt`
```bash
# Format all code
cargo fmt --all

# Check formatting without modifying files
cargo fmt --all -- --check
```

**Convention**: Always run `cargo fmt --all` before committing.

### Linting with `clippy`
```bash
# Run clippy
cargo clippy --all-targets --all-features

# Treat warnings as errors (CI mode)
cargo clippy -- -D warnings
```

**Convention**: All clippy warnings must be resolved. CI enforces `-D warnings`.

### Code Quality Rules
- **Naming**: Use descriptive names; avoid single-letter or abbreviated identifiers
- **Comments**: Only add comments if they match existing style or explain complex logic
- **Functions**: Keep small with early returns; avoid deep nesting
- **Clones**: Minimize; only clone cheap types or use references
- **Explicit Types**: Prefer in public APIs; constrain generics appropriately

---

## CI/CD Overview

### Workflows
All workflows are in `.github/workflows/`:

#### 1. **Test Workflow** (`test.yaml`)
- **Trigger**: Push to `main`, all pull requests
- **Jobs**:
  - Checkout code
  - Install stable Rust toolchain
  - `cargo build --verbose`
  - `cargo test --verbose`
- **Purpose**: Ensure code builds and tests pass

#### 2. **Lint Workflow** (`lint.yaml`)
- **Trigger**: All pushes
- **Jobs**:
  - **Clippy**: `cargo clippy --all-targets --all-features`
  - **Fmt**: `cargo fmt --all -- --check`
- **Purpose**: Enforce code quality and formatting

#### 3. **Docker Workflow** (`docker.yaml`)
- **Trigger**: Git tags matching `v*` (e.g., `v1.0.0`)
- **Jobs**:
  - Build multi-platform Docker images (linux/amd64, linux/arm64)
  - Push to GitHub Container Registry (`ghcr.io/prodzilla/prodzilla`)
  - Generate artifact attestations
- **Purpose**: Publish Docker images on releases

#### 4. **Release Workflow** (`release.yaml`)
- **Trigger**: Git tags matching `v[0-9]+.*`
- **Jobs**:
  - Create GitHub release using `taiki-e/create-gh-release-action`
- **Purpose**: Automate GitHub release creation

### Reproducing CI Failures Locally

#### Build Failures
```bash
cargo build --verbose
```

#### Test Failures
```bash
cargo test --verbose
```

#### Clippy Failures
```bash
cargo clippy --all-targets --all-features
```

#### Format Failures
```bash
cargo fmt --all -- --check
```

### CI Environment
- **OS**: `ubuntu-latest`
- **Rust**: Stable toolchain (auto-updated in CI)
- **Caching**: Not currently enabled (could be added for faster builds)

---

## Branching, Commit, and Release Conventions

### Branching Strategy
- **Main Branch**: `main` - production-ready code
- **Feature Branches**: `feature/description` or `your-name/description`
- **Bug Fixes**: `fix/description`
- **No force pushes to `main`**

### Commit Message Guidelines
- Use clear, descriptive commit messages
- Start with a verb in imperative mood (e.g., "Add", "Fix", "Update", "Refactor")
- Keep first line under 72 characters
- Add detailed explanation in body if needed

**Examples**:
- ✅ `Add support for regex expectations in probes`
- ✅ `Fix race condition in story execution`
- ✅ `Update OpenTelemetry to version 0.23.0`
- ❌ `fix bug` (too vague)
- ❌ `WIP` (not descriptive)

### Release Process
1. **Versioning**: Follow [Semantic Versioning](https://semver.org/)
   - MAJOR: Breaking changes
   - MINOR: New features, backward compatible
   - PATCH: Bug fixes, backward compatible

2. **Creating a Release**:
   ```bash
   git tag -a v1.2.3 -m "Release version 1.2.3"
   git push origin v1.2.3
   ```

3. **Automated Actions**:
   - Docker image builds and pushes to GHCR
   - GitHub release created automatically
   - No manual intervention needed

### Pull Request Guidelines
- Ensure all CI checks pass (tests, lint, fmt)
- Add tests for new features
- Update documentation if behavior changes
- Keep PRs focused and reasonably sized
- Link to related issues

---

## Code, Design, and Architectural Conventions

### Module Structure
- **Domain logic**: `src/probe/` (probes, stories, expectations)
- **Telemetry**: `src/otel/` (OpenTelemetry setup, metrics, traces)
- **Web API**: `src/web_server/` (routes, handlers, models)
- **Alerts**: `src/alerts/` (webhook dispatching)
- **Configuration**: `src/config.rs` (YAML parsing, variable substitution)
- **Shared state**: `src/app_state.rs` (RwLock-guarded state)

### Architectural Patterns

#### 1. **Shared State with RwLocks**
```rust
pub struct AppState {
    pub probe_results: Arc<RwLock<HashMap<String, Vec<ProbeResult>>>>,
    pub story_results: Arc<RwLock<HashMap<String, Vec<StoryResult>>>>,
}
```
- **Rule**: Do not hold locks across `.await` points
- **Pattern**: Read/write quickly, release lock immediately

#### 2. **Singleton HTTP Clients**
```rust
lazy_static! {
    static ref HTTP_CLIENT: Client = Client::builder()
        .user_agent("Prodzilla Probe/1.0")
        .build()
        .unwrap();
}
```
- **Rule**: Never create new HTTP clients; reuse singletons
- **Locations**:
  - Probes: `src/probe/http_probe.rs`
  - Alerts: `src/alerts/outbound_webhook.rs`

#### 3. **Pure Expectation Evaluation**
- Keep expectation evaluation pure and testable
- Located in `src/probe/expectations.rs`
- No side effects, easy to unit test

#### 4. **Async Task Spawning**
```rust
let state = Arc::clone(&app_state);
tokio::spawn(async move {
    probing_loop(probe, state).await;
});
```
- Use `tokio::spawn` for concurrent probe/story execution
- Clone `Arc<AppState>` before moving into task
- Never block the runtime (no `std::thread::sleep`)

### Design Principles
- **Minimal allocations**: Prefer references over clones
- **Early returns**: Reduce nesting
- **Explicit error handling**: Bubble errors up, avoid `.unwrap()` in production
- **Observability first**: Add tracing and metrics to all I/O operations
- **Configuration over code**: Use YAML for probe/story definitions

---

## Security and Performance Guidelines

### Security Considerations

#### 1. **Sensitive Data Handling**
- Mark probes/steps as `sensitive: true` to redact response bodies
- **Never log secrets or credentials**
- Use environment variables for sensitive config (via `${{env.VAR_NAME}}`)
- Alerts for sensitive probes show "Redacted" instead of response body

#### 2. **Input Validation**
- YAML config is validated at startup
- Regular expressions in expectations are compiled once
- Variable substitution prevents injection by replacing within controlled contexts

#### 3. **HTTP Security**
- User-Agent headers identify Prodzilla (`Prodzilla Probe/1.0`)
- Respect timeouts (default 10s for probes)
- No automatic following of redirects (configurable if needed)

#### 4. **Secrets Management**
- **Never commit secrets to `prodzilla.yml`**
- Use `${{env.SECRET_NAME}}` for webhook URLs, API keys, etc.
- Docker deployments: pass secrets via environment variables

### Performance Guidelines

#### 1. **Concurrency**
- Each probe/story runs in its own `tokio` task
- No global locks held during I/O
- RwLock access is short-lived

#### 2. **Memory Usage**
- Target: < 15MB RAM for typical workloads
- Results stored in-memory (bounded by probe schedule)
- Consider memory limits for high-frequency probes

#### 3. **HTTP Client Reuse**
- Single `reqwest::Client` per module (via `lazy_static!`)
- Connection pooling handled by `reqwest`

#### 4. **Timeouts**
- Default probe timeout: 10s (configurable via `with.timeout_seconds`)
- Default alert timeout: 10s
- Prevents hung requests from blocking scheduler

#### 5. **Observability Overhead**
- OpenTelemetry adds minimal overhead
- Use `stdout` exporter for debugging, `otlp` or `prometheus` for production
- Span events only record truncated bodies (≤500 chars)

---

## Common Pitfalls and Troubleshooting

### Common Pitfalls

#### 1. **Holding Locks Across `.await`**
❌ **Wrong**:
```rust
let results = state.probe_results.write().unwrap();
let response = reqwest::get("https://example.com").await?; // Deadlock risk!
results.insert(name, result);
```

✅ **Correct**:
```rust
let response = reqwest::get("https://example.com").await?;
let mut results = state.probe_results.write().unwrap();
results.insert(name, result);
drop(results); // Release lock immediately
```

#### 2. **Creating New HTTP Clients**
❌ **Wrong**:
```rust
let client = reqwest::Client::new(); // Don't create new clients!
```

✅ **Correct**:
```rust
use lazy_static::lazy_static;
lazy_static! {
    static ref HTTP_CLIENT: Client = /* singleton */;
}
HTTP_CLIENT.get(url).send().await?
```

#### 3. **Using `.unwrap()` in Production Code**
❌ **Wrong**:
```rust
let config: Config = serde_yaml::from_str(&contents).unwrap(); // Panics!
```

✅ **Correct**:
```rust
let config: Config = serde_yaml::from_str(&contents)
    .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;
```

#### 4. **Blocking the Tokio Runtime**
❌ **Wrong**:
```rust
std::thread::sleep(Duration::from_secs(10)); // Blocks executor!
```

✅ **Correct**:
```rust
tokio::time::sleep(Duration::from_secs(10)).await;
```

### Troubleshooting Guide

#### Problem: Build fails with "could not compile"
**Solution**:
```bash
cargo clean
rustup update stable
cargo build
```

#### Problem: Tests fail with "connection refused"
**Cause**: Tests trying to reach real network endpoints
**Solution**: Ensure all HTTP calls use `wiremock::MockServer`

#### Problem: Clippy warnings in CI
**Solution**:
```bash
cargo clippy --all-targets --all-features
# Fix all warnings, then:
cargo clippy -- -D warnings
```

#### Problem: Format check fails in CI
**Solution**:
```bash
cargo fmt --all
git add .
git commit -m "Apply rustfmt"
```

#### Problem: Metrics not appearing
**Cause**: `OTEL_METRICS_EXPORTER` not set
**Solution**:
```bash
export OTEL_METRICS_EXPORTER=prometheus
cargo run -- --file prodzilla.yml
# Access metrics at http://localhost:9464/metrics
```

#### Problem: Traces not exporting
**Cause**: `OTEL_TRACES_EXPORTER` not set
**Solution**:
```bash
export OTEL_TRACES_EXPORTER=stdout  # Or 'otlp' for collector
cargo run -- --file prodzilla.yml
```

#### Problem: Variable substitution not working
**Cause**: Step name mismatch or wrong syntax
**Solution**: Check step name matches exactly:
```yaml
# Step definition
- name: get-ip
  url: https://api.ipify.org/?format=json
  
# Reference (note: exact name match)
- name: use-ip
  url: https://example.com/${{steps.get-ip.response.body.ip}}
```

---

## AI Agent Prompts/Recipes

### Example Tasks and Preferred Outputs

#### Task 1: "Add a new expectation operator"
**Preferred Approach**:
1. Add enum variant to `ExpectationOperation` in `src/probe/expectations.rs`
2. Implement evaluation logic in `evaluate_expectation()`
3. Add unit tests in the same file
4. Update AGENTS.md and README.md with the new operator
5. Run `cargo fmt` and `cargo clippy`

**Example**:
```rust
// In src/probe/expectations.rs
pub enum ExpectationOperation {
    // ... existing variants
    StartsWith,  // New operator
}

// In evaluate_expectation()
ExpectationOperation::StartsWith => {
    actual_value.starts_with(expected_value)
}

// Add test
#[test]
fn test_starts_with_expectation() {
    let result = evaluate_expectation(/* ... */);
    assert!(result.is_ok());
}
```

#### Task 2: "Add a new metric"
**Preferred Approach**:
1. Define the metric in `src/otel/metrics.rs` as part of the `Metrics` struct
2. Register it in the `new()` method
3. Add recording calls where the metric should be updated
4. Ensure attributes include `name` and `type` (and `story_name` for steps)
5. Document in README.md under "Tracked metrics"

#### Task 3: "Fix a bug in variable substitution"
**Preferred Approach**:
1. Add a failing test in `src/config.rs` demonstrating the bug
2. Fix the logic in the substitution functions
3. Verify the test passes
4. Run full test suite: `cargo test`
5. Check for related edge cases

#### Task 4: "Add a new web API endpoint"
**Preferred Approach**:
1. Define handler in `src/web_server/handlers.rs`
2. Add response model in `src/web_server/model.rs` if needed
3. Register route in `src/web_server/mod.rs`
4. Use `Extension<Arc<AppState>>` for state access
5. Return `Json<T>` for JSON responses
6. Document endpoint in README.md

#### Task 5: "Investigate CI failure"
**Preferred Approach**:
1. Check workflow logs in GitHub Actions
2. Reproduce locally:
   - Build: `cargo build --verbose`
   - Test: `cargo test --verbose`
   - Lint: `cargo clippy --all-targets --all-features`
   - Format: `cargo fmt --all -- --check`
3. Fix the issue
4. Verify locally before pushing

### General AI Agent Guidelines
- **Always run `cargo fmt` and `cargo clippy`** after code changes
- **Preserve existing patterns**: Don't introduce new HTTP clients, web frameworks, or async runtimes
- **Add observability**: New I/O operations should have traces and metrics
- **Write tests**: Use `wiremock` for HTTP, `#[tokio::test]` for async
- **Update docs**: If behavior changes, update README.md and AGENTS.md
- **Respect conventions**: Follow error handling, concurrency, and security guidelines

---

## Documentation Links

### Internal Documentation
- **[README.md](README.md)**: User-facing documentation, getting started guide, feature overview
- **[CONTRIBUTING.md](CONTRIBUTING.md)**: Contribution guidelines and development workflow
- **[LICENSE](LICENSE)**: Apache 2.0 license
- **[prodzilla.yml](prodzilla.yml)**: Example configuration file

### Code Documentation
- **Main entry point**: [`src/main.rs`](src/main.rs)
- **Config parsing**: [`src/config.rs`](src/config.rs)
- **Probe execution**: [`src/probe/http_probe.rs`](src/probe/http_probe.rs)
- **Expectations**: [`src/probe/expectations.rs`](src/probe/expectations.rs)
- **OpenTelemetry**: [`src/otel/`](src/otel/)
- **Web API**: [`src/web_server/`](src/web_server/)

### External Resources
- **Website**: [prodzilla.io](https://prodzilla.io/)
- **Discord Community**: [Join Discord](https://discord.gg/ud55NhraUm)
- **GitHub Repository**: [github.com/prodzilla/prodzilla](https://github.com/prodzilla/prodzilla)
- **Docker Images**: [ghcr.io/prodzilla/prodzilla](https://ghcr.io/prodzilla/prodzilla)

### Rust Ecosystem Documentation
- **Axum**: [docs.rs/axum](https://docs.rs/axum)
- **Tokio**: [tokio.rs](https://tokio.rs)
- **Reqwest**: [docs.rs/reqwest](https://docs.rs/reqwest)
- **Tracing**: [docs.rs/tracing](https://docs.rs/tracing)
- **OpenTelemetry**: [opentelemetry.io](https://opentelemetry.io)
- **Serde**: [serde.rs](https://serde.rs)

### Deployment
- **Shuttle.rs**: [shuttle.rs](https://shuttle.rs) - Free Rust app hosting
- **Tutorial**: [How I'm Getting Free Synthetic Monitoring](https://codingupastorm.dev/2023/11/07/prodzilla-and-shuttle/)

---

## Appendix: Detailed Coding Conventions

> These conventions should be followed for all code changes.



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

