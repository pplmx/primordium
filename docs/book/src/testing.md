# Testing Strategy

Primordium uses a comprehensive testing strategy combining unit tests, integration tests, property-based testing, and long-haul stability verification.

## Test Organization

```
tests/
├── common/               # Shared test utilities
│   ├── mod.rs           # Test builders and macros
│   └── macros.rs        # Assertion helpers
├── brain_pbt.rs          # Property-based brain tests (proptest)
├── physics_pbt.rs        # Property-based physics tests (proptest)
├── determinism.rs        # Determinism verification
├── determinism_suite.rs  # Long-term determinism checks
├── stability_long_haul.rs
├── long_haul_suite.rs    # Extended stability suite
├── ecosystem_stability.rs
├── evolution_validation.rs
├── social_hierarchy.rs   # Hierarchy and caste tests
└── ...                   # Additional integration tests
```

## Running Tests

### All Tests
```bash
cargo test --workspace --all-features
```

### Specific Test File
```bash
cargo test --test brain_pbt
```

### Property-Based Tests
```bash
cargo test --test brain_pbt -- --test-threads=1
```

### Long-Haul Tests
```bash
# Run stability_long_haul (~30s)
cargo test --test stability_long_haul stability_long_haul -- --ignored

# Run extended suite (~2 hours)
cargo test --test long_haul_suite extended_stability_suite -- --ignored
```

## Test Categories

### 1. Unit Tests
- Location: `src/*/tests` modules
- Purpose: Test individual functions and modules
- Count: 100+ unit tests

### 2. Integration Tests
- Location: `tests/` directory
- Purpose: Test system-level behavior and interactions
- Count: 40+ integration tests

### 3. Property-Based Tests
- Framework: `proptest` (already in dependencies)
- Purpose: Fuzz testing with millions of inputs
- Files:
  - `brain_pbt.rs`: Neural network forward pass, HexDNA roundtrip
  - `physics_pbt.rs`: Spatial hashing, entity behavior, physics constraints

### 4. Determinism Tests
- Purpose: Ensure reproducible simulation with same seed
- Files:
  - `determinism.rs`: Basic world determinism (100 ticks)
  - `determinism_suite.rs`: Long-term determinism (200+ ticks with parallelism check)

### 5. Long-Haul Tests
- Purpose: Numerical stability, memory leaks, drift detection
- Files:
  - `stability_long_haul.rs`: 2000 ticks, 500 entities (~30s)
  - `long_haul_suite.rs`: Extended test suite (~2 hours)
    - Quick check: 100 ticks
    - Standard check: 1000 ticks
    - Extended run: 5000 ticks with population dynamics
    - Disaster resilience test
    - Parallel determinism stress
    - Boundary condition stress

## Quality Gates

Every commit must pass:

```bash
# 1. Code formatting
cargo fmt --all

# 2. Linting (no warnings allowed)
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 3. All tests
cargo test --workspace --all-features
```

### Pre-commit Hooks
These quality gates are enforced via git hooks:
- `pre-commit`: Runs fmt, clippy, and tests
- `pre-push`: Runs full test suite

## Property-Based Testing

### Philosophy
Property-based testing complements example-based tests by:
- Discovering edge cases you wouldn't think of
- Testing thousands of inputs in a single test
- Capturing minimal counterexamples

### Example: Brain Forward Pass
```rust
proptest! {
    #[test]
    fn test_brain_forward_no_nan(
        brain in arb_brain(50),
        inputs in any::<[f32; 29]>()
    ) {
        let (outputs, next_hidden) = brain.forward(inputs);
        for &o in &outputs {
            prop_assert!(o.is_finite());
        }
    }
}
```

## Determinism Guarantees

### When is Simulation Deterministic?
Simulation is deterministic when:
1. `config.world.deterministic = true`
2. `config.world.seed` is set to a specific value
3. Same initial conditions (entities, environment)

### What's Not Deterministic?
- Random entity UUIDs (use `Uuid::from_u128()` for tests)
- System timer for performance metrics
- Hardware-dependent RNG (use mock in deterministic mode)

### Regression Testing
The `determinism_suite.rs` maintains a regression test that:
- Creates two worlds with identical seed
- Runs 200+ updates
- Compares hashes at each tick
- Fails if any divergence is detected

## Long-Haul Testing

### Why Run Long-Haul Tests?
1. **Numerical Stability**: Detect floating-point drift
2. **Memory Leaks**: Verify no unbounded allocation
3. **System Stress**: Test high-entity-count scenarios
4. **Regression Detection**: Catch subtle bugs over time

### Running Long-Haul Tests Manually
```bash
# Extended stability (~30s)
cargo test --test stability_long_haul -- --ignored

# Full suite (~2 hours) - run before major releases
cargo test --test long_haul_suite -- --ignored
```

### CI Integration
Long-haul tests are `#[ignore]` by default to keep CI fast. Run them:
- Before releases
- After major world/system changes
- When memory usage issues are suspected

## Test Maintenance

### Adding New Tests
1. Place unit tests in `src/*/tests` modules
2. Place integration tests in `tests/`
3. Use `#[tokio::test]` for async tests
4. Set `config.world.deterministic = true` for flaky tests
5. Use existing builders from `tests/common/` when possible

### Debugging Failing Tests

1. **Run single test**:
   ```bash
   cargo test --test brain_pbt test_brain_forward_no_nan
   ```

2. **Enable backtrace**:
   ```bash
   RUST_BACKTRACE=1 cargo test --test failing_test
   ```

3. **Property test failure**: 
   - Check `.proptest-regressions/` for minimized counterexample
   - Run with `--nocapture` to see debug output

4. **Determinism failure**:
   - Verify `deterministic = true` and seed is set
   - Check for `Uuid::new_v4()` usage
   - Ensure no system time dependencies

## Performance Testing

### Performance Gate
The `perf_gate.rs` test enforces tick time thresholds:
- 100 entities, 200 ticks, max 1500ms average tick time
- Scales with entity count
- Run with: `cargo test --test perf_gate`

### Benchmarking
```bash
# Release mode for performance tests
cargo test --release --test perf_gate

# Measure memory usage
cargo test --release --test stability_long_haul -- --ignored
```

## Best Practices

1. **Determinism First**: Always test deterministic mode before random mode
2. **Property Tests**: Add proptest for functions with complex input handling
3. **Descriptive Names**: Use `test_<feature>_<scenario>` pattern
4. **Cleanup**: Remove test logs and temporary files in `setup()`/`teardown()`
5. **Fast Feedback**: Keep quick tests in CI, defer long-haul to manual runs
6. **Documentation**: Add doc comments explaining test intent

---

*Last updated: 2026-02-26*
