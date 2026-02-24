# Test Suite Reference

## OVERVIEW

Integration tests covering biological systems, neural networks, social dynamics, and performance validation.

## STRUCTURE

```
tests/
├── common/           # Shared test utilities (WorldBuilder, EntityBuilder, macros)
├── *_v2.rs           # Versioned test suites (ecology_v2, civilization_v2)
├── *_pbt.rs          # Property-based tests (proptest)
├── *_edge_cases.rs   # Boundary condition tests
├── perf_gate.rs      # Performance regression gate
└── stress_test.rs    # High-load scenarios
```

## WHERE TO LOOK

- **Lifecycle & Genetics**: `lifecycle.rs`, `genetic_flow.rs`, `genetic_bottlenecks.rs`
- **Neural Networks**: `brain_pbt.rs`, `brain_properties.rs`
- **Ecology**: `ecology.rs`, `ecology_v2.rs`, `environmental_succession.rs`
- **Social Systems**: `social_v2.rs`, `social_hierarchy.rs`, `territory_war.rs`
- **Civilization**: `civilization.rs`, `civilization_v2.rs`, `macro_evolution.rs`
- **Performance**: `perf_gate.rs`, `stress_test.rs`, `performance_benchmark.rs`
- **Determinism**: `determinism.rs`, `determinism_suite.rs`

## CONVENTIONS

- **Async Tests**: All tests use `#[tokio::test]` attribute
- **Deterministic Mode**: Set `config.world.deterministic = true` with explicit seeds for reproducibility
- **Test Builders**: Use `WorldBuilder` and `EntityBuilder` from `common/mod.rs` for setup
- **Assertion Macros**: `assert_energy_above!`, `assert_entity_dead!`, `assert_population!` from `common/macros.rs`
- **Property Testing**: Use `proptest!` macro with `prop_compose!` strategies for generative tests
- **Performance Gates**: `perf_gate.rs` enforces tick time thresholds (scales with entity count)
- **Naming**: `test_<feature>_<scenario>` pattern, versioned tests append `_v2`

## RUNNING TESTS

```bash
# All tests
cargo test

# Specific test file
cargo test --test lifecycle

# Specific test function
cargo test --test lifecycle test_reproduction_and_genetics

# Property-based tests (proptest)
cargo test --test brain_pbt

# Performance gate
cargo test --test perf_gate

# Release mode for performance tests
cargo test --release --test perf_gate
```

## ANTI-PATTERNS

- **Don't** use random seeds without `deterministic = true` - tests become flaky
- **Don't** skip warmup ticks in performance tests - first ticks have initialization overhead
- **Don't** hardcode entity counts without using `cfg!(debug_assertions)` scaling
- **Don't** mutate world state directly in parallel tests - use `WorldBuilder` modifiers
- **Don't** ignore `proptest` regression files - commit `.proptest-regressions` to catch failures
