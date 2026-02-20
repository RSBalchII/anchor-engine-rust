//! Benchmarks for anchor-fingerprint.
//!
//! Run with: `cargo bench`

use anchor_fingerprint::{hamming_distance, simhash, similarity};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

/// Sample short text for benchmarking (~50 chars)
const SHORT_TEXT: &str = "The quick brown fox jumps over the lazy dog";

/// Sample medium text for benchmarking (~500 chars)
const MEDIUM_TEXT: &str = "
    Rust is a systems programming language that runs blazingly fast, prevents 
    segfaults, and guarantees thread safety. It provides memory safety without 
    garbage collection, concurrency without data races, and abstraction without 
    overhead. Rust is perfect for writing high-performance, reliable software.
    
    The language was designed for safety, concurrency, and performance. It 
    achieves these goals through its ownership system, which tracks resource 
    management at compile time. This eliminates entire classes of bugs that 
    plague other languages.
    
    Rust has a rich type system with pattern matching, generics, and trait-based 
    polymorphism. The package manager Cargo makes it easy to build, test, and 
    document Rust code. The community is vibrant and welcoming.
";

/// Sample long text for benchmarking (~2000 chars)
const LONG_TEXT: &str = "
    Rust is a multi-paradigm, general-purpose programming language. It emphasizes 
    performance, type safety, and concurrency. Rust enforces memory safety 
    through its ownership system, which is checked at compile time. This means 
    that many bugs are caught early in the development process, rather than 
    appearing in production.
    
    The language was first designed in 2006 by Graydon Hoare at Mozilla Research. 
    Since then, it has been refined by a large community of contributors. Rust 
    has been voted the most loved programming language in the Stack Overflow 
    Developer Survey for several years running.
    
    Key features of Rust include:
    
    - Zero-cost abstractions: High-level constructs compile to efficient machine code
    - Move semantics: Efficient transfer of ownership without copying
    - Guaranteed memory safety: No null pointers, dangling pointers, or buffer overflows
    - Threads without data races: The type system prevents concurrent programming bugs
    - Trait-based generics: Write flexible, reusable code without runtime overhead
    - Pattern matching: Powerful control flow for complex data structures
    - Type inference: Write less boilerplate while maintaining type safety
    - Minimal runtime: No garbage collector, no hidden control flow
    - Efficient C bindings: Interoperate with C code without overhead
    - Procedural macros: Generate code at compile time
    
    Rust's package manager, Cargo, handles dependency management, building, 
    testing, and documentation. The ecosystem is growing rapidly, with crates 
    available for web development, embedded systems, networking, cryptography, 
    and more.
    
    The compiler provides excellent error messages, often suggesting fixes. 
    This makes learning Rust easier than many other systems languages. The 
    community is known for being helpful and inclusive.
    
    Use cases for Rust include:
    
    - Operating systems: Write kernels, drivers, and system utilities
    - Web assembly: Compile Rust to WASM for browser-based applications
    - Command-line tools: Build fast, reliable CLI utilities
    - Network services: Create high-performance servers and microservices
    - Embedded systems: Program microcontrollers with limited resources
    - Game engines: Develop performance-critical game infrastructure
    - Cryptography: Implement secure algorithms with confidence
    - Blockchain: Build distributed ledgers with safety guarantees
    
    The borrow checker is Rust's most distinctive feature. It tracks how 
    references to data are used, ensuring that there are no dangling pointers 
    or data races. While it has a learning curve, mastering the borrow checker 
    leads to more robust code.
    
    Rust's type system is expressive without being complex. Algebraic data 
    types, pattern matching, and trait objects provide powerful abstraction 
    mechanisms. The language supports both functional and object-oriented 
    programming styles.
    
    Performance-wise, Rust competes with C and C++. It provides fine-grained 
    control over memory layout, allocation strategies, and optimization hints. 
    The compiler leverages LLVM's optimization passes to generate efficient code.
";

fn benchmark_simhash_short(c: &mut Criterion) {
    c.bench_function("simhash_short_50_chars", |b| {
        b.iter(|| simhash(black_box(SHORT_TEXT)))
    });
}

fn benchmark_simhash_medium(c: &mut Criterion) {
    c.bench_function("simhash_medium_500_chars", |b| {
        b.iter(|| simhash(black_box(MEDIUM_TEXT)))
    });
}

fn benchmark_simhash_long(c: &mut Criterion) {
    c.bench_function("simhash_long_2000_chars", |b| {
        b.iter(|| simhash(black_box(LONG_TEXT)))
    });
}

fn benchmark_hamming_distance(c: &mut Criterion) {
    let a = 0x1234567890ABCDEFu64;
    let b = 0xFEDCBA0987654321u64;

    c.bench_function("hamming_distance", |b| {
        b.iter(|| hamming_distance(black_box(a), black_box(b)))
    });
}

fn benchmark_similarity(c: &mut Criterion) {
    let a = 0x1234567890ABCDEFu64;
    let b = 0xFEDCBA0987654321u64;

    c.bench_function("similarity", |b| {
        b.iter(|| similarity(black_box(a), black_box(b)))
    });
}

fn benchmark_tokenize(c: &mut Criterion) {
    c.bench_function("tokenize_medium", |b| {
        b.iter(|| anchor_fingerprint::tokenize(black_box(MEDIUM_TEXT)))
    });
}

fn benchmark_full_pipeline(c: &mut Criterion) {
    c.bench_function("full_pipeline_simhash", |b| {
        b.iter(|| {
            let tokens = anchor_fingerprint::tokenize(black_box(MEDIUM_TEXT));
            let hash = simhash_with_tokens(&tokens);
            hash
        })
    });
}

fn benchmark_simhash_with_tokens(c: &mut Criterion) {
    let tokens = anchor_fingerprint::tokenize(MEDIUM_TEXT);

    c.bench_function("simhash_with_tokens", |b| {
        b.iter(|| anchor_fingerprint::simhash_with_tokens(black_box(&tokens)))
    });
}

criterion_group!(
    benches,
    benchmark_simhash_short,
    benchmark_simhash_medium,
    benchmark_simhash_long,
    benchmark_hamming_distance,
    benchmark_similarity,
    benchmark_tokenize,
    benchmark_full_pipeline,
    benchmark_simhash_with_tokens,
);

criterion_main!(benches);
