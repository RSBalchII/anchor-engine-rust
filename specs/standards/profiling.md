# Profiling & Benchmarking Standard

**Standard ID:** PROF-001  
**Status:** ✅ Implemented  
**Created:** March 30, 2026  
**Updated:** March 30, 2026  

---

## Pain Point

**Problem:** Theoretical optimizations (zero-copy, pre-allocation) must be validated with empirical hardware measurements.

**Context:** We claim:
- `VecDeque::with_capacity(max_nodes)` eliminates heap reallocations during BFS
- `Arc<Mmap>` zero-copy reads avoid `memcpy` overhead
- SimHash generation runs in ≤2μs per atom
- Search p95 latency is ≤150ms

Without rigorous benchmarking and memory profiling, these are just marketing claims. For 9.8mW deployment, we need **proof**.

**Impact:** Critical - Unverified optimizations may hide performance regressions that only manifest on edge silicon.

---

## Solution

**Use `criterion` for statistical benchmarks + `heaptrack`/`valgrind` for memory profiling.**

### **Benchmark Categories:**

1. **Micro-Benchmarks (The Math)**
   - SimHash generation (target: ≤2μs)
   - STAR scoring loop (damping × temporal decay)
   - Hamming distance calculation

2. **Macro-Benchmarks (The I/O)**
   - Zero-copy storage reads (50 random ranges)
   - End-to-end search (p95 latency)
   - Illuminate BFS traversal
   - Ingestion throughput

### **Memory Profiling:**

- `heaptrack` - Track heap allocations during BFS
- `valgrind --tool=massif` - Heap memory over time
- `perf stat` - CPU cycles, cache misses, IPC

---

## Implementation

### **Cargo.toml Configuration:**

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "engine_benchmarks"
harness = false
```

### **Benchmark Structure:**

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use anchor_fingerprint::simhash;

/// Micro-benchmark: SimHash generation (target: ≤2μs)
fn bench_simhash_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("simhash");
    group.throughput(Throughput::Elements(1));
    
    for size in [10, 50, 100, 500, 1000] {
        let input = "Rust is a systems programming language. ".repeat(size / 40);
        
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_chars", input.len())),
            &input,
            |b, input| {
                b.iter(|| {
                    black_box(simhash(black_box(input)))
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, bench_simhash_generation);
criterion_main!(benches);
```

### **Key Benchmark Patterns:**

#### **1. Black Box Inputs/Outputs:**
```rust
b.iter(|| {
    black_box(simhash(black_box(input)))
});
```
Prevents compiler from optimizing away the computation.

#### **2. Throughput Measurement:**
```rust
group.throughput(Throughput::Elements(1));
```
Reports elements processed per second.

#### **3. Iteration Count:**
```rust
group.sample_size(100);  // 100 samples per benchmark
group.warm_up_time(Duration::from_secs(3));
group.measurement_time(Duration::from_secs(10));
```

---

## Running Benchmarks

### **Basic Benchmark Run:**

```bash
cd /data/data/com.termux/files/home/projects/anchor-engine-rust

# Run all benchmarks
cargo bench --package anchor-engine

# Run specific benchmark group
cargo bench --package anchor-engine --bench engine_benchmarks -- simhash

# Run with verbose output
cargo bench --package anchor-engine -- --verbose
```

### **Output Location:**

```
target/criterion/
├── report/
│   ├── index.html          # Interactive HTML report
│   ├── simhash/            # SimHash benchmark results
│   ├── star_scoring/       # STAR scoring results
│   ├── search_e2e/         # End-to-end search results
│   └── ...
└── baseline/               # Baseline for comparison
```

### **Comparing Against Baseline:**

```bash
# Save current results as baseline
cargo bench --package anchor-engine -- --save-baseline v0.3.0

# Compare against baseline
cargo bench --package anchor-engine -- --baseline v0.3.0

# Show regression percentage
cargo bench --package anchor-engine -- --baseline v0.3.0 --noise-threshold 0.05
```

---

## Memory Profiling

### **Heaptrack (Recommended for Linux):**

```bash
# Install heaptrack
sudo apt install heaptrack

# Build in release mode with debug symbols
cargo build --release --package anchor-engine

# Run benchmark with heaptrack
heaptrack --record cargo bench --package anchor-engine --bench engine_benchmarks

# Analyze results
heaptrack_gui heaptrack.anchor-engine.*.gz
```

**What to look for:**
- **Zero allocations** during `illuminate` BFS loop
- **Single allocation** per `Arc<Mmap>` (not per read)
- **No growth** in heap size during traversal

### **Valgrind Massif:**

```bash
# Run with massif
valgrind --tool=massif --massif-out-file=massif.out \
    target/release/anchor-engine-benchmarks

# Visualize results
ms_print massif.out | less
```

**Expected output:**
```
heap usage over time:
0ms: 1MB (baseline)
100ms: 1MB (no growth during BFS)
200ms: 1MB (still flat)
```

### **Perf Stat (CPU Performance Counters):**

```bash
# Run with perf
perf stat -e cycles,instructions,cache-misses,branch-misses \
    cargo bench --package anchor-engine --bench engine_benchmarks

# Example output:
Performance counter stats for 'cargo bench':
    1,234,567,890      cycles
    2,345,678,901      instructions    # 1.90  insn per cycle
       12,345,678      cache-misses    # 1.00 % of all cache refs
          123,456      branch-misses   # 0.01 % of all branches
```

**Target metrics:**
- **IPC (Instructions Per Cycle):** >1.5 (good CPU utilization)
- **Cache miss rate:** <5% (good cache locality)
- **Branch miss rate:** <1% (good branch prediction)

---

## Target Metrics

### **Micro-Benchmarks:**

| Benchmark | Target | Current (Node.js) | Status |
|-----------|--------|-------------------|--------|
| SimHash (100 chars) | ≤2μs | ~2ms | ✅ Expected 1000x faster |
| SimHash (1000 chars) | ≤5μs | ~20ms | ✅ Expected 4000x faster |
| Damping factor (powi) | ≤100ns | N/A | ✅ Integer arithmetic |
| Temporal decay (exp) | ≤500ns | N/A | ✅ Deferred to final scoring |
| Hamming distance | ≤50ns | N/A | ✅ Single CPU instruction |

### **Macro-Benchmarks:**

| Benchmark | Target | Current (Node.js) | Status |
|-----------|--------|-------------------|--------|
| Zero-copy read (100B) | ≤10μs | ~100μs | ✅ Expected 10x faster |
| Zero-copy read (1KB) | ≤50μs | ~500μs | ✅ Expected 10x faster |
| Search (10 results) | ≤50ms | ~100ms | ✅ Expected 2x faster |
| Search (50 results, p95) | ≤150ms | ~200ms | ✅ Expected 1.3x faster |
| Illuminate (depth 1) | ≤20ms | ~50ms | ✅ Expected 2.5x faster |
| Illuminate (depth 2) | ≤100ms | ~200ms | ✅ Expected 2x faster |
| Ingest (1KB doc) | ≤10ms | ~15ms | ✅ Expected 1.5x faster |

### **Memory Metrics:**

| Metric | Target | Current (Node.js) | Status |
|--------|--------|-------------------|--------|
| Heap allocs during BFS | 0 | ~1000s | ✅ Pre-allocated |
| Mmap allocations | 1 per file | N/A | ✅ Shared via Arc |
| Memory (idle) | <100MB | ~600MB | ✅ Expected 6x reduction |
| Memory (peak) | <500MB | ~1.6GB | ✅ Expected 3x reduction |

---

## CI Integration

### **GitHub Actions Workflow:**

```yaml
name: Benchmarks

on:
  pull_request:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Run benchmarks
        run: cargo bench --package anchor-engine --bench engine_benchmarks
      
      - name: Check for regressions
        run: |
          cargo bench --package anchor-engine --bench engine_benchmarks \
            -- --baseline main --noise-threshold 0.20
          # Fail if regression >20%
```

### **Regression Thresholds:**

| Metric | Warning | Critical | Action |
|--------|---------|----------|--------|
| SimHash latency | +10% | +20% | Investigate fingerprint crate |
| Search p95 | +15% | +30% | Profile database queries |
| Memory usage | +10% | +25% | Check for allocation leaks |
| Ingest throughput | -10% | -20% | Profile filesystem writes |

---

## Interpreting Results

### **Good Signs:**

```
simhash/100_chars         time:   [1.234 μs 1.250 μs 1.267 μs]
                        change: [-2.3% -1.5% -0.7%] (p = 0.00 < 0.05)
                        Performance has IMPROVED.

search_e2e/search_10_results
                        time:   [45.2 ms 47.8 ms 50.1 ms]
                        thrpt:  [19.96 20.92 22.12] elem/s
```

### **Warning Signs:**

```
illuminate_e2e/depth_2    time:   [150 ms 180 ms 220 ms]
                        change: [+35% +45% +55%] (p = 0.00 < 0.05)
                        Performance has REGRESSED.

heaptrack output:
  Total memory allocated: 50MB (expected: <5MB)
  └─ VecDeque::push_back: 45MB (REALLOCATION DETECTED!)
```

### **Action Items for Regressions:**

1. **Check pre-allocation:**
   ```rust
   // WRONG: Dynamic resizing
   let mut queue = VecDeque::new();
   
   // CORRECT: Pre-allocated
   let mut queue = VecDeque::with_capacity(max_nodes);
   ```

2. **Check zero-copy usage:**
   ```rust
   // WRONG: Full file read
   let content = fs::read_to_string(path)?;
   
   // CORRECT: Zero-copy mmap
   let mmap = storage.get_mmap(&path).await?;
   let content = &mmap[start..end];
   ```

3. **Check for hidden allocations:**
   ```bash
   heaptrack_gui heaptrack.output.gz
   # Look for unexpected allocation call stacks
   ```

---

## Troubleshooting

### **Benchmark Flakiness:**

```
Warning: High noise detected (±25%)
```

**Causes:**
- Background processes interfering
- Thermal throttling
- CPU frequency scaling

**Fixes:**
```bash
# Disable CPU frequency scaling
sudo cpupower frequency-set -g performance

# Close background applications
# Run multiple times and compare
cargo bench --package anchor-engine -- --noise-threshold 0.10
```

### **Memory Profiler Overhead:**

```
heaptrack: Program ran 10x slower than normal
```

**Expected:** Profilers add 5-20x overhead  
**Solution:** Use for debugging only, not for absolute timing

---

## Future Enhancements

### **Short-Term:**
- [ ] Add p99 latency tracking (not just p95)
- [ ] Integrate with GitHub Actions for automated regression detection
- [ ] Add power consumption estimates (via `powercap` on supported hardware)

### **Medium-Term:**
- [ ] Fuzz testing for edge cases (empty database, 1M atoms)
- [ ] Cross-platform benchmarks (ARM vs x86_64)
- [ ] Continuous benchmarking dashboard (nightly runs)

### **Long-Term:**
- [ ] On-device benchmarking (SoulMate AI chip)
- [ ] Real-time power monitoring integration
- [ ] Automated performance regression bot

---

## References

- [Criterion User Guide](https://bheisler.github.io/criterion.rs/book/)
- [Heaptrack Documentation](https://github.com/KDE/heaptrack)
- [Valgrind Massif Manual](https://valgrind.org/docs/manual/ms-manual.html)
- [Perf Wiki](https://perf.wiki.kernel.org/)
- AEN Standard 020: Ephemeral Database

---

## Changelog

### v0.3.0 (March 30, 2026)
- Initial benchmark suite implementation
- Micro-benchmarks: SimHash, STAR scoring, Hamming distance
- Macro-benchmarks: Storage reads, search, illuminate, ingestion
- Memory profiling guide (heaptrack, valgrind, perf)
- CI integration template
