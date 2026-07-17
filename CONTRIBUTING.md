# Contributing to Halo

## Code of Conduct

This project adheres to the [Contributor Covenant](https://www.contributor-covenant.org/). By participating, you are expected to uphold this code.

## Getting Started

See [docs/guides/development.md](docs/guides/development.md) for local setup.

## How to Contribute

### Reporting Bugs

1. Check if the bug already exists in [Issues](https://github.com/Gakuus/Halo/issues)
2. If not, open a new issue with:
   - Steps to reproduce
   - Expected vs actual behavior
   - Environment (OS, Rust version, browser if frontend)
   - Logs or error output

### Suggesting Features

1. Open an issue with the `enhancement` label
2. Describe the problem you're solving (not just the solution)
3. If possible, link to the relevant ADR or use case

### Pull Requests

1. Fork the repo and create a branch from `develop`
   - Branch naming: `feature/XX-description` or `fix/XX-description`
2. Make your changes
3. Run checks locally:
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo test --lib
   ```
4. Open a PR against `develop`
5. Ensure CI passes (fmt → clippy → build → test)

#### PR Checklist

- [ ] Branch from `develop`, target `develop`
- [ ] No merge conflicts
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -D warnings` passes (no new warnings)
- [ ] `cargo test --lib` passes (new tests for new code)
- [ ] If adding a dependency: `cargo deny check` passes
- [ ] If changing API: update `docs/api/rest-api.md`
- [ ] If changing domain: update `docs/domain/domain-model.md`
- [ ] ADR updated if architectural change

## Coding Conventions

See [docs/guides/conventions.md](docs/guides/conventions.md).

## Testing

See [docs/guides/testing-strategy.md](docs/guides/testing-strategy.md).

## Security

If you find a security vulnerability, **do not** open a public issue. Email `security@halo.app` or follow the disclosure policy in [docs/planning/security-plan.md](docs/planning/security-plan.md).

## License

By contributing, you agree that your contributions will be licensed under the project's license (see [LICENSE](LICENSE)).
