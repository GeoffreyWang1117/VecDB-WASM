# Contributing to VecDB-WASM

Thank you for your interest in contributing to VecDB-WASM! 🎉

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Documentation](#documentation)

## Code of Conduct

This project adheres to a code of conduct. By participating, you are expected to uphold this code:

- Be respectful and inclusive
- Welcome newcomers and help them learn
- Focus on what is best for the community
- Show empathy towards other community members

## Getting Started

### Prerequisites

- Rust 1.70+ and Cargo
- Node.js 14+
- wasm-pack
- A modern web browser

### Setup Development Environment

```bash
# Clone the repository
git clone https://github.com/GeoffreyWang1117/VecDB-WASM.git
cd VecDB-WASM

# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm-pack
cargo install wasm-pack

# Build the project
npm run build:dev

# Run tests
npm test

# Serve examples
npm run serve:examples
```

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/bug-description
```

Branch naming conventions:
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `test/` - Test improvements
- `perf/` - Performance improvements

### 2. Make Changes

```bash
# Format code
npm run fmt

# Run linter
npm run clippy

# Build
npm run build:dev

# Test
npm test
```

### 3. Commit Changes

Follow conventional commit messages:

```bash
# Format
<type>(<scope>): <subject>

# Examples
feat(search): add radius search functionality
fix(hnsw): resolve memory leak in graph construction
docs(api): update search method documentation
test(distance): add SIMD correctness tests
perf(index): optimize batch insert performance
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `perf`: Performance improvements
- `chore`: Build process or auxiliary tool changes

### 4. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub.

## Pull Request Process

### Before Submitting

- [ ] Code follows the project's coding standards
- [ ] All tests pass (`cargo test`)
- [ ] New tests added for new functionality
- [ ] Documentation updated (if applicable)
- [ ] Code formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] CHANGELOG.md updated (for significant changes)

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
How has this been tested?

## Checklist
- [ ] Tests pass
- [ ] Documentation updated
- [ ] CHANGELOG updated
```

### Review Process

1. At least one maintainer will review your PR
2. Address review comments
3. Once approved, a maintainer will merge

## Coding Standards

### Rust Code

```rust
// Use descriptive variable names
let vector_dimension = 128;  // ✅ Good
let dim = 128;               // ❌ Avoid

// Add documentation comments
/// Calculates the cosine similarity between two vectors.
///
/// # Arguments
/// * `a` - First vector
/// * `b` - Second vector
///
/// # Returns
/// Similarity score between 0 and 1
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    // Implementation
}

// Use Result for error handling
pub fn insert(&mut self, id: u64, vector: Vec<f32>) -> Result<(), String> {
    if vector.len() != self.dimension {
        return Err(format!("Dimension mismatch: expected {}, got {}",
                          self.dimension, vector.len()));
    }
    // ...
}

// Keep functions focused and small
// ✅ Good: Single responsibility
fn validate_dimension(vector: &[f32], expected: usize) -> Result<(), String> {
    if vector.len() != expected {
        return Err(format!("Invalid dimension"));
    }
    Ok(())
}

// Use meaningful type names
struct SearchResult {  // ✅ Clear
    id: u64,
    score: f32,
}
```

### JavaScript/WASM Bindings

```javascript
// Use async/await for promises
async function loadDatabase(name) {
    const data = await VectorDB.load_from_indexeddb(name);
    return VectorDB.import_snapshot(data);
}

// Error handling
try {
    db.insert(id, vector);
} catch (error) {
    console.error('Failed to insert vector:', error);
}

// Use const/let, not var
const db = new VectorDB(128, Metric.Cosine, IndexType.HNSW);
let results = db.search(query, 10);
```

### File Organization

```
src/
├── lib.rs              # Public API exports
├── bindings.rs         # WASM bindings
├── storage.rs          # Vector storage
├── distance/           # Distance metrics
│   ├── mod.rs
│   └── simd.rs
├── index/              # Index implementations
│   ├── mod.rs
│   ├── flat.rs
│   └── hnsw.rs
└── persistence/        # Persistence layer
    ├── mod.rs
    ├── snapshot.rs
    └── indexeddb.rs
```

## Testing Guidelines

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    #[should_panic(expected = "dimension mismatch")]
    fn test_invalid_dimension() {
        // Test error cases
    }
}
```

### WASM Tests

```rust
#[cfg(test)]
mod wasm_tests {
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_create_db() {
        let db = VectorDB::new(128, Metric::Cosine, IndexType::Flat);
        assert!(db.is_ok());
    }
}
```

### Integration Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_cosine_similarity

# Run with output
cargo test -- --nocapture
```

### Test Coverage

Aim for:
- 80%+ code coverage
- All public APIs tested
- Edge cases covered
- Error paths tested

## Documentation

### Code Documentation

```rust
/// Calculate the Euclidean distance between two vectors.
///
/// # Arguments
///
/// * `a` - First vector as a slice of f32
/// * `b` - Second vector as a slice of f32
///
/// # Returns
///
/// The Euclidean distance as f32
///
/// # Panics
///
/// Panics if vectors have different dimensions
///
/// # Examples
///
/// ```
/// let a = vec![1.0, 2.0, 3.0];
/// let b = vec![4.0, 5.0, 6.0];
/// let dist = euclidean_distance(&a, &b);
/// assert_eq!(dist, 5.196);
/// ```
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    // Implementation
}
```

### API Documentation

Update docs/API.md when adding new public APIs.

### Examples

Add examples to `examples/` directory demonstrating new features.

### Changelog

Update CHANGELOG.md for notable changes:

```markdown
## [Unreleased]

### Added
- New radius search functionality
- Performance monitoring API

### Changed
- Improved HNSW build performance by 20%

### Fixed
- Memory leak in graph construction
```

## Performance Considerations

### Benchmarking

```bash
# Run benchmarks
cargo bench

# Profile with perf
cargo build --release
perf record -g target/release/vecdb_wasm
perf report
```

### Optimization Checklist

- [ ] Algorithm complexity optimal
- [ ] No unnecessary allocations
- [ ] SIMD used where applicable
- [ ] Benchmarks show improvement
- [ ] No performance regressions

## Areas for Contribution

### High Priority

- [ ] Web Workers integration
- [ ] TypeScript type definitions
- [ ] Additional distance metrics
- [ ] Performance optimizations
- [ ] Documentation improvements

### Features

- [ ] IVF index implementation
- [ ] Product Quantization
- [ ] Range search
- [ ] Batch search optimization
- [ ] Vector normalization utilities

### Testing

- [ ] Increase test coverage
- [ ] Browser compatibility tests
- [ ] Performance benchmarks
- [ ] Stress tests

### Documentation

- [ ] More usage examples
- [ ] Video tutorials
- [ ] Blog posts
- [ ] Translation to other languages

## Communication

- **Issues**: Use GitHub Issues for bug reports and feature requests
- **Discussions**: Use GitHub Discussions for questions and ideas
- **Pull Requests**: For code contributions

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Questions?

Feel free to ask questions in:
- GitHub Discussions
- GitHub Issues (for bug-related questions)
- Email: support@vecdb-wasm.dev

Thank you for contributing! 🚀
