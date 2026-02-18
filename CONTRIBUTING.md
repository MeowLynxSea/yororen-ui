# Contributing to yororen-ui

Thank you for your interest in yororen-ui! We welcome all forms of contributions, including bug fixes, new features, documentation improvements, and more.

## Development Setup

### Prerequisites

- Rust 1.75+
- Cargo

### Local Development

```bash
# Clone the repository
git clone https://github.com/MeowLynxSea/yororen-ui.git
cd yororen-ui

# Build the project
cargo build

# Run tests
cargo test

# Run clippy checks
cargo clippy
```

## Code Standards

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo clippy` to check code style
- Ensure code is formatted with `cargo fmt`
- All public APIs must have documentation comments

## Commit Standards

We use [Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code formatting (no functional changes)
- `refactor`: Code refactoring
- `perf`: Performance improvement
- `test`: Test related
- `chore`: Build process or auxiliary tool changes

### Examples

```bash
git commit -m "feat(button): add loading state support"
git commit -m "fix(toast): fix memory leak in notification queue"
git commit -m "docs(readme): add installation instructions"
```

## Pull Request Process

1. Fork the repository and create a branch
2. Make your changes and commit
3. Ensure all tests and checks pass
4. Submit a Pull Request
5. Wait for code review

### PR Checklist

- [ ] `cargo build` succeeds
- [ ] All `cargo test` pass
- [ ] `cargo clippy` has no warnings
- [ ] Code is formatted (`cargo fmt`)
- [ ] Documentation is added/updated

## Code of Conduct

Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## Reporting Issues

- Use GitHub Issues to report bugs
- Use GitHub Issues for feature requests
- When submitting issues, please provide reproduction steps and environment information
