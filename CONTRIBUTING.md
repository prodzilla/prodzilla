# Contributing to Prodzilla

Thank you for your interest in contributing to Prodzilla! We welcome contributions from the community.

## Quick Links

- **[AGENTS.md](AGENTS.md)**: Comprehensive guide for developers and AI agents with detailed architecture, coding conventions, build instructions, and development workflows
- **[README.md](README.md)**: User-facing documentation and feature overview
- **[Discord Community](https://discord.gg/ud55NhraUm)**: Join our community for questions and discussions

## Getting Started

### Prerequisites

1. **Rust Toolchain**: Install via [rustup](https://rustup.rs/)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup update stable
   ```

2. **Clone the Repository**:
   ```bash
   git clone https://github.com/prodzilla/prodzilla.git
   cd prodzilla
   ```

3. **Build and Run**:
   ```bash
   cargo build
   cargo run -- --file prodzilla.yml
   ```

### Development Workflow

1. **Create a Branch**: Use descriptive branch names
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/your-bug-fix
   ```

2. **Make Changes**: Follow the coding conventions in [AGENTS.md](AGENTS.md)

3. **Format and Lint**: Always run before committing
   ```bash
   cargo fmt --all
   cargo clippy --all-targets --all-features
   ```

4. **Test**: Ensure all tests pass
   ```bash
   cargo test
   ```

5. **Commit**: Use clear, descriptive commit messages
   ```bash
   git commit -m "Add feature: description of what was added"
   # or
   git commit -m "Fix: description of what was fixed"
   ```

6. **Push and Create PR**:
   ```bash
   git push origin your-branch-name
   ```
   Then open a Pull Request on GitHub.

## Coding Standards

**For detailed coding conventions, architecture patterns, and best practices, see [AGENTS.md](AGENTS.md).**

### Key Principles

- **Rust Edition**: Use Rust 2021
- **Formatting**: Always run `cargo fmt --all`
- **Linting**: Resolve all `cargo clippy` warnings (CI enforces `-D warnings`)
- **Testing**: Write tests for new features using `#[tokio::test]` and `wiremock`
- **Error Handling**: Avoid `.unwrap()` in production code; prefer `?` and proper error types
- **Async**: Never block the Tokio runtime (use `tokio::time::sleep`, not `std::thread::sleep`)
- **Observability**: Add tracing and metrics to new I/O operations

### Module Organization

- **Probes & Stories**: `src/probe/`
- **OpenTelemetry**: `src/otel/`
- **Web API**: `src/web_server/`
- **Alerts**: `src/alerts/`
- **Configuration**: `src/config.rs`

## Pull Request Guidelines

### Before Submitting

- [ ] All tests pass: `cargo test`
- [ ] Code is formatted: `cargo fmt --all`
- [ ] No clippy warnings: `cargo clippy --all-targets --all-features`
- [ ] New features have tests
- [ ] Documentation updated if behavior changes
- [ ] Commit messages are clear and descriptive

### PR Description

Please include:
- **What**: Description of the changes
- **Why**: Motivation or issue reference
- **How**: Brief explanation of the approach
- **Testing**: How you tested the changes

### Review Process

1. CI checks must pass (build, test, lint, format)
2. At least one maintainer review
3. Address review feedback
4. Once approved, a maintainer will merge

## Types of Contributions

### Bug Fixes

- Check existing issues or create a new one
- Reference the issue in your PR
- Include a test that reproduces the bug (if applicable)

### New Features

- Discuss in an issue or Discord first for larger features
- Follow existing patterns and conventions
- Add tests and documentation
- Update README.md if user-facing

### Documentation

- Improvements to README.md, AGENTS.md, or code comments
- Ensure all links work
- Keep technical accuracy

### Tests

- Add missing test coverage
- Improve existing tests
- Use `wiremock` for HTTP mocking

## Building and Testing

### Build
```bash
# Debug build
cargo build

# Release build
cargo build --release

# Check without building
cargo check
```

### Test
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Run Locally
```bash
cargo run -- --file prodzilla.yml
```

The server will start at `http://localhost:3000`.

### Docker
```bash
# Build image
docker build -t prodzilla .

# Run container
docker run -v $(pwd)/prodzilla.yml:/prodzilla.yml prodzilla
```

## CI/CD

All pull requests trigger:
- **Build**: `cargo build --verbose`
- **Test**: `cargo test --verbose`
- **Clippy**: `cargo clippy --all-targets --all-features`
- **Format Check**: `cargo fmt --all -- --check`

See [AGENTS.md](AGENTS.md#cicd-overview) for details on reproducing CI failures locally.

## Debugging Tips

### Enable Trace Logs
```bash
RUST_LOG=trace cargo run -- --file prodzilla.yml
```

### View OpenTelemetry Traces
```bash
export OTEL_TRACES_EXPORTER=stdout
cargo run -- --file prodzilla.yml
```

### Enable Prometheus Metrics
```bash
export OTEL_METRICS_EXPORTER=prometheus
cargo run -- --file prodzilla.yml
# Metrics at http://localhost:9464/metrics
```

## Common Pitfalls

See [AGENTS.md - Common Pitfalls and Troubleshooting](AGENTS.md#common-pitfalls-and-troubleshooting) for detailed guidance on:
- Avoiding locks across `.await` points
- Reusing HTTP clients (don't create new ones)
- Proper error handling
- Async best practices

## Getting Help

- **Discord**: [Join our community](https://discord.gg/ud55NhraUm)
- **Issues**: [GitHub Issues](https://github.com/prodzilla/prodzilla/issues)
- **Discussions**: [GitHub Discussions](https://github.com/prodzilla/prodzilla/discussions)
- **Documentation**: [AGENTS.md](AGENTS.md) for technical details

## Code of Conduct

We expect all contributors to:
- Be respectful and inclusive
- Provide constructive feedback
- Focus on what is best for the community
- Show empathy towards other community members

## License

By contributing to Prodzilla, you agree that your contributions will be licensed under the [Apache 2.0 License](LICENSE).

---

**Thank you for contributing to Prodzilla!** 🦖
