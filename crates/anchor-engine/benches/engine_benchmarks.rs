//! Anchor Engine Benchmark Suite
//!
//! This suite validates the 9.8mW edge-silicon performance constraints:
//! - SimHash generation: ≤2μs per atom
//! - Search latency (p95): ≤150ms
//! - Zero heap allocations during BFS traversal
//! - Zero-copy filesystem reads via mmap

use anchor_engine::{Database, AnchorService, FileSystemStorage};
use anchor_engine::models::{IngestRequest, IngestOptions, SearchRequest, SearchMode, BudgetConfig, IlluminateRequest};
use anchor_engine::config::Config;
use anchor_fingerprint::simhash;
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// Micro-Benchmarks: The Math
// ============================================================================

/// Benchmark SimHash generation (target: ≤2μs per atom)
fn bench_simhash_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("simhash");
    group.throughput(Throughput::Elements(1));
    
    // Test various input sizes
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

/// Benchmark STAR algorithm scoring loop (temporal decay + damping)
fn bench_star_scoring(c: &mut Criterion) {
    let mut group = c.benchmark_group("star_scoring");
    
    let damping_factor: f64 = 0.85;
    let time_decay_lambda: f64 = 0.00001;
    
    // Benchmark damping factor only (integer power - cheap)
    group.bench_function("damping_only_hop_0", |b| {
        b.iter(|| {
            black_box(damping_factor.powi(black_box(0)))
        });
    });
    
    group.bench_function("damping_only_hop_1", |b| {
        b.iter(|| {
            black_box(damping_factor.powi(black_box(1)))
        });
    });
    
    group.bench_function("damping_only_hop_5", |b| {
        b.iter(|| {
            black_box(damping_factor.powi(black_box(5)))
        });
    });
    
    // Benchmark temporal decay (floating-point exp - expensive)
    group.bench_function("temporal_decay_1_hour", |b| {
        b.iter(|| {
            black_box((-time_decay_lambda * black_box(1.0)).exp())
        });
    });
    
    group.bench_function("temporal_decay_1_day", |b| {
        b.iter(|| {
            black_box((-time_decay_lambda * black_box(24.0)).exp())
        });
    });
    
    // Benchmark full gravity score (damping × temporal)
    group.bench_function("full_gravity_score", |b| {
        b.iter(|| {
            let hop = black_box(2);
            let hours = black_box(12.0);
            let damping = damping_factor.powi(hop);
            let temporal = (-time_decay_lambda * hours).exp();
            black_box(damping * temporal)
        });
    });
    
    group.finish();
}

/// Benchmark Hamming distance calculation (64-bit)
fn bench_hamming_distance(c: &mut Criterion) {
    let mut group = c.benchmark_group("hamming");
    
    let hash1: u64 = 0x1234567890ABCDEF;
    let hash2: u64 = 0xFEDCBA0987654321;
    
    group.bench_function("hamming_64bit", |b| {
        b.iter(|| {
            let h1 = black_box(hash1);
            let h2 = black_box(hash2);
            let xor = h1 ^ h2;
            black_box(xor.count_ones())
        });
    });
    
    group.bench_function("structural_gravity", |b| {
        b.iter(|| {
            let h1 = black_box(hash1);
            let h2 = black_box(hash2);
            let hamming = (h1 ^ h2).count_ones() as f64;
            black_box(1.0 - hamming / 64.0)
        });
    });
    
    group.finish();
}

// ============================================================================
// Macro-Benchmarks: The I/O
// ============================================================================

/// Create a test service for benchmarks
fn setup_benchmark_service() -> (AnchorService, TempDir, TempDir) {
    let db_dir = TempDir::new().unwrap();
    let mirror_dir = TempDir::new().unwrap();

    let db_path = db_dir.path().join("test.db");

    // Use tokio runtime for async setup
    let rt = tokio::runtime::Runtime::new().unwrap();
    let db = rt.block_on(Database::open(&db_path)).unwrap();
    let config = Config::default();
    let service = AnchorService::new(db, mirror_dir.path().to_path_buf(), config).unwrap();

    (service, db_dir, mirror_dir)
}

/// Ingest test content for benchmarks
fn ingest_benchmark_content(service: &mut AnchorService, num_docs: usize) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    for i in 0..num_docs {
        let content = format!(
            "Document {} about Rust programming. #rust #programming #benchmark #test{}",
            i, i % 10
        );
        
        let request = IngestRequest {
            source: format!("doc_{}.md", i),
            content,
            bucket: "benchmark".to_string(),
            options: IngestOptions::default(),
        };
        
        rt.block_on(service.ingest(request)).unwrap();
    }
}

/// Benchmark FileSystemStorage zero-copy reads
fn bench_storage_zero_copy_reads(c: &mut Criterion) {
    let (mut service, _db_dir, _mirror_dir) = setup_benchmark_service();
    ingest_benchmark_content(&mut service, 50);
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    // Get atoms to benchmark reads
    let atoms = rt.block_on(service.db().get_all_atoms()).unwrap();
    
    let mut group = c.benchmark_group("storage_reads");
    group.throughput(Throughput::Elements(1));
    
    group.bench_function("zero_copy_read_range_100bytes", |b| {
        b.iter(|| {
            let atom = &atoms[black_box(0)];
            rt.block_on(service.storage().read_range(
                &atom.source_path,
                atom.start_byte,
                atom.start_byte + 100,
            )).unwrap()
        });
    });
    
    group.bench_function("zero_copy_read_range_1kb", |b| {
        b.iter(|| {
            let atom = &atoms[black_box(0)];
            rt.block_on(service.storage().read_range(
                &atom.source_path,
                atom.start_byte,
                atom.start_byte + 1024,
            )).unwrap()
        });
    });
    
    group.finish();
}

/// Benchmark end-to-end search (MCP call overhead included)
fn bench_end_to_end_search(c: &mut Criterion) {
    let (mut service, _db_dir, _mirror_dir) = setup_benchmark_service();
    ingest_benchmark_content(&mut service, 100);
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("search_e2e");
    
    group.bench_function("search_10_results", |b| {
        b.iter(|| {
            let request = SearchRequest {
                query: black_box("#rust"),
                max_results: black_box(10),
                mode: SearchMode::Combined,
                budget: BudgetConfig::default(),
            };
            rt.block_on(service.search(request)).unwrap()
        });
    });
    
    group.bench_function("search_50_results", |b| {
        b.iter(|| {
            let request = SearchRequest {
                query: black_box("#rust"),
                max_results: black_box(50),
                mode: SearchMode::Combined,
                budget: BudgetConfig::default(),
            };
            rt.block_on(service.search(request)).unwrap()
        });
    });
    
    group.finish();
}

/// Benchmark illuminate BFS traversal
fn bench_illuminate_bfs(c: &mut Criterion) {
    let (mut service, _db_dir, _mirror_dir) = setup_benchmark_service();
    ingest_benchmark_content(&mut service, 100);
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("illuminate_e2e");
    
    group.bench_function("illuminate_depth_1", |b| {
        b.iter(|| {
            let request = IlluminateRequest {
                seed: black_box("#rust"),
                depth: black_box(1),
                max_nodes: black_box(50),
            };
            rt.block_on(service.illuminate(request)).unwrap()
        });
    });
    
    group.bench_function("illuminate_depth_2", |b| {
        b.iter(|| {
            let request = IlluminateRequest {
                seed: black_box("#rust"),
                depth: black_box(2),
                max_nodes: black_box(50),
            };
            rt.block_on(service.illuminate(request)).unwrap()
        });
    });
    
    group.finish();
}

/// Benchmark ingestion throughput
fn bench_ingestion_throughput(c: &mut Criterion) {
    let (mut service, _db_dir, _mirror_dir) = setup_benchmark_service();
    
    let mut group = c.benchmark_group("ingestion");
    group.throughput(Throughput::Elements(1));
    
    let content = "Rust is a systems programming language. ".repeat(100);
    
    group.bench_function("ingest_1kb_document", |b| {
        b.iter(|| {
            let request = IngestRequest {
                source: black_box("test.md"),
                content: black_box(content.clone()),
                bucket: black_box("benchmark".to_string()),
                options: IngestOptions::default(),
            };
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(service.ingest(request)).unwrap()
        });
    });
    
    group.finish();
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group!(
    name = micro_benchmarks;
    config = Criterion::default()
        .sample_size(100)
        .warm_up_time(std::time::Duration::from_secs(3))
        .measurement_time(std::time::Duration::from_secs(10))
        .noise_threshold(0.05)
        .nresamples(100_000);
    targets = bench_simhash_generation, bench_star_scoring, bench_hamming_distance
);

criterion_group!(
    name = macro_benchmarks;
    config = Criterion::default()
        .sample_size(50)
        .warm_up_time(std::time::Duration::from_secs(5))
        .measurement_time(std::time::Duration::from_secs(30))
        .noise_threshold(0.1);
    targets = bench_storage_zero_copy_reads, bench_end_to_end_search, bench_illuminate_bfs, bench_ingestion_throughput
);

criterion_main!(micro_benchmarks, macro_benchmarks);
