# Performance Profiling Guide for Developers

This guide covers techniques for profiling and optimizing Primordium's performance.

## Quick Start: Profiling a Full Tick

Using Cargo's built-in flamegraph support:

1. Build with profiling instrumentation:
```bash
cargo build --release --bin primordium
```

2. Run with flamegraph sampling:
```bash
cargo flamegraph --bin primordium --output flamegraph.svg
```

This will:
- Launch the simulation
- Sample CPU stack traces periodically (default: ~10 Hz)
- Generate an interactive SVG flamegraph when you exit with `Q`

## Running Specific Benchmarks

Run micro-benchmarks for hot paths:

```bash
# Benchmark spatial hash queries
cargo test --test performance_benchmark benchmark_spatial_hash_query -- --bench

# Benchmark brain forward pass
cargo test --test performance_benchmark benchmark_brain_forward_pass -- --bench

# Full tick scaling benchmark  
cargo test --test performance_benchmark benchmark_full_tick_scaling -- --bench
```

Note: These tests are marked `#[ignore]` by default; use `-- --ignored` flag.

## Cargo Profiler

For more detailed profiling data:

```bash
# Run simulation with instrumentation
cargo run --release --bin primordium &

# Generate flamegraph (run after some ticks)
cargo flamegraph --bin primordium --output profile.svg

# Generate inferno graph (interactive viewer)
cargo install inferno
cargo flamegraph --bin primordium --output profile.svg --format inferno
```

## Identifying Hot Paths

1. **Visual Inspect Flamegraph**:
   - Wide = hot path (spends most CPU time)
   - Narrow = cold path
   - The stack width represents CPU time; wider = hotter

2. **Check Common Hot Paths**:

| Component | Expected % of Tick Time | Notes |
|----------|-----------------------|-------|
| Perception (`perceive_and_decide_internal`) | ~40-60% | Rayon parallelized per entity |
| Spatial Hashing (`query_callback`) | ~10-20% | Already optimized, focus on caller side |
| Action Execution (`execute_interactions`) | ~10-15% | ECS queries, entity despawns |
| Social/Stats systems | ~10-15% | Batch processing |

## Profiling Specific Functions

For more granular profiling, you can instrument specific code sections:

```rust
#[inline(never)]
fn profiled_function() {
    // Code to profile
}
```

Then use:
```bash
cargo flamegraph -- --bin primordium
```

## Memory Profiling

To identify memory issues:

```bash
# Run with memory profiling
cargo run --release --bin primordium

# In another terminal, sample memory:
pmap -p $(pgrep primordium) > memory_profile.txt

# Or use valgrind (Linux only):
valgrind --tool=cachegrind ./target/release/primordium
```

## Tools Summary

| Tool | Use Case | Command |
|------|----------|---------|
| `cargo flamegraph` | CPU flamegraph visualization | `cargo flamegraph --bin primordordium` |
| `inferno` | Interactive flamegraph viewer | `cargo install inferno` |
| `perf` | Linux system-wide profiling | `perf record -g ./target/release/primordordium` |
| `valgrind/cachegrind` | Memory profiling | `valgrind --tool=cachegrind ./target/release/primordium` |
| `cargo bench` | Run benchmark suite | `cargo bench` |

## Common Performance Bottlenecks

1. **Too many entities in one location** → SpatialHash overhead increases
2. **Large neural network** → Brain metabolic cost increases
3. **Frequent reallocations** → Garbage collection pressure
4. **Serial contention** → Rayon not effective, reduce shared mutable state

## Continuous Performance Monitoring

Run the performance regression gate before pushing changes:

```bash
cargo test --test perf_gate
```

Expected: Average tick < 1500ms.

## Tips for Effective Profiling

- Run real workloads with hundreds of entities for meaningful data
- Profile release builds, not debug builds (optimizations change behavior)
- Use deterministic mode for consistent measurements
- Profile for at least 100 ticks to capture warm-up effects
- Compare before/after changes for actual improvements, not just absolute numbers
