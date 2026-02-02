# Contributing to Primordium

Thank you for your interest in contributing to Primordium! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Contributing Guidelines](#contributing-guidelines)
- [Commit Message Convention](#commit-message-convention)
- [Testing](#testing)
- [Documentation](#documentation)
- [Pull Request Process](#pull-request-process)

## Code of Conduct

This project adheres to a code of conduct that expects all participants to:

- Be respectful and inclusive
- Welcome newcomers and help them learn
- Focus on constructive feedback
- Accept responsibility and apologize when mistakes happen

## Getting Started

### Prerequisites

- **Rust** (latest stable version)
- **Git**
- **Node.js** (for WebAssembly development)
- **wasm-pack** (for WASM builds)

### Fork and Clone

```bash
# Fork the repository on GitHub, then clone your fork
git clone https://github.com/YOUR_USERNAME/primordium.git
cd primordium

# Add upstream remote
git remote add upstream https://github.com/pplmx/primordium.git
```

## Development Setup

### Build the Project

```bash
# Build the native TUI version
cargo build --release

# Run the simulation
cargo run --release

# Build for WebAssembly
cd www && wasm-pack build --target web
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_name

# Run integration tests only
cargo test --test '*'
```

### Code Quality Checks

```bash
# Format code
cargo fmt

# Run Clippy lints
cargo clippy -- -D warnings

# Check for security vulnerabilities
cargo audit
```

## Contributing Guidelines

### What to Contribute

We welcome contributions in the following areas:

- **Bug fixes**: Fix issues reported in the codebase
- **Feature implementations**: Work on roadmap items (see ROADMAP.md)
- **Documentation**: Improve README, wiki, or inline comments
- **Tests**: Add missing test coverage
- **Performance optimizations**: Improve simulation performance
- **Refactoring**: Improve code structure and maintainability

### Code Style

- Follow **Rust naming conventions**
- Use **descriptive variable names**
- Keep functions focused and small
- Add comments explaining "why", not "what"
- Avoid `unwrap()` and `expect()` in production code
- Use `?` operator for error propagation

### Example: Proper Error Handling

```rust
// ❌ Bad - can panic
let data = fs::read_to_string("config.toml").unwrap();

// ✅ Good - graceful error handling
let data = fs::read_to_string("config.toml")
    .map_err(|e| {
        tracing::error!("Failed to read config: {}", e);
        e
    })?;
```

### Architecture Guidelines

The project follows a **layered architecture**:

```
src/
├── model/          # Core simulation engine (no IO)
│   ├── state/      # Data structures
│   ├── systems/    # Logic/behavior
│   └── infra/      # External protocols
├── app/            # Application lifecycle
├── ui/             # Rendering
└── client/         # WASM client
```

- **Model layer** must be pure logic (no disk/network IO)
- **Systems** should be stateless functions
- **State** should only contain data (no methods that mutate)

## Commit Message Convention

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style (formatting, semicolons, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or fixing tests
- `chore`: Build process or auxiliary tool changes

### Scopes

- `core`: Core simulation engine
- `brain`: Neural network logic
- `ui`: User interface
- `net`: Networking/P2P
- `docs`: Documentation
- `test`: Tests

### Examples

```bash
feat(brain): add support for dynamic topology mutations

fix(net): handle websocket disconnect gracefully

docs(readme): update installation instructions

refactor(model): split World into smaller systems
```

## Testing

### Test Requirements

- All new features must include tests
- Bug fixes must include regression tests
- Maintain or improve code coverage

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_behavior() {
        // Arrange
        let input = create_test_data();
        
        // Act
        let result = process(&input);
        
        // Assert
        assert_eq!(result.expected_value, actual_value);
    }

    #[test]
    fn test_edge_case() {
        // Test boundary conditions
    }
}
```

### Integration Tests

Integration tests are located in the `tests/` directory:

```bash
tests/
├── lifecycle.rs           # Life cycle tests
├── ecology.rs             # Ecosystem tests
├── social_dynamics.rs     # Social behavior tests
├── migration_network.rs   # P2P networking tests
└── ...
```

## Documentation

### Code Documentation

- All public APIs must have doc comments
- Use examples in documentation
- Explain the "why" behind complex logic

```rust
/// Calculates the social rank of an entity based on multiple factors.
///
/// The rank is computed as a weighted sum:
/// - Energy: 30%
/// - Age: 30%
/// - Offspring count: 10%
/// - Reputation: 30%
///
/// # Arguments
///
/// * `entity` - The entity to evaluate
/// * `context` - Additional contextual information
///
/// # Returns
///
/// A value between 0.0 and 1.0 representing the entity's social rank
///
/// # Examples
///
/// ```
/// let rank = calculate_social_rank(&entity, &context);
/// assert!(rank >= 0.0 && rank <= 1.0);
/// ```
pub fn calculate_social_rank(entity: &Entity, context: &Context) -> f32 {
    // Implementation
}
```

### Documentation Updates

When adding features, update:

1. **CHANGELOG.md**: Add entry under appropriate phase
2. **README.md**: If user-facing feature
3. **Wiki docs** (`docs/wiki/`): If technical feature
4. **AGENTS.md**: If architecture changes

## Pull Request Process

### Before Submitting

1. **Sync with upstream**:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run quality checks**:
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo test
   ```

3. **Update documentation** as needed

4. **Write clear commit messages** following our convention

### Submitting PR

1. Push to your fork:
   ```bash
   git push origin feature/my-feature
   ```

2. Create Pull Request on GitHub

3. Fill out the PR template:
   - Describe what changed and why
   - Reference related issues
   - Include screenshots for UI changes

### PR Review Process

- All PRs require at least one review
- Address review feedback promptly
- Keep PRs focused and reasonably sized
- Be open to feedback and discussion

### After Merge

- Delete your feature branch
- Update your local main branch
- Continue with a new branch for next contribution

## Questions?

If you have questions or need help:

1. Check existing [documentation](./docs)
2. Review [ARCHITECTURE.md](./ARCHITECTURE.md)
3. Open an issue for discussion

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

**Thank you for contributing to Primordium!** 🧬✨
