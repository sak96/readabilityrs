use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use readabilityrs::Readability;
use std::fs;
use std::path::Path;

/// Load test case HTML from the test-pages directory
fn load_test_case(name: &str) -> String {
    let path = Path::new("tests/test-pages").join(name).join("source.html");
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to load test case: {}", name))
}

/// Benchmark parsing a single document
fn bench_parse_single(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_single");
    let test_cases = [
        ("001", "Small (~5KB)"),
        ("medium-1", "Medium (~50KB)"),
        ("nytimes-1", "Large (~150KB)"),
        ("guardian-1", "Very Large (~1MB)"),
    ];

    for (case_name, label) in test_cases.iter() {
        let html = match load_test_case(case_name) {
            html if !html.is_empty() => html,
            _ => continue,
        };

        let size = html.len();
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(BenchmarkId::new(*label, size), &html, |b, html| {
            b.iter(|| {
                let readability = Readability::new(black_box(html), None, None).unwrap();
                black_box(readability.parse())
            });
        });
    }

    group.finish();
}

/// Benchmark with URL resolution enabled
fn bench_with_url(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_with_url");

    let html = load_test_case("base-url");
    let size = html.len();
    group.throughput(Throughput::Bytes(size as u64));

    group.bench_function("with_url_resolution", |b| {
        b.iter(|| {
            let readability =
                Readability::new(black_box(&html), Some("https://example.com/article"), None)
                    .unwrap();
            black_box(readability.parse())
        });
    });

    group.finish();
}

/// Benchmark batch processing of multiple documents
fn bench_batch_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_processing");

    let test_names = [
        "001",
        "002",
        "aclu",
        "ars-1",
        "bbc-1",
        "buzzfeed-1",
        "cnet",
        "cnn",
        "ehow-1",
        "herald-sun-1",
    ];

    let documents: Vec<String> = test_names
        .iter()
        .filter_map(|name| {
            let path = Path::new("tests/test-pages").join(name).join("source.html");
            fs::read_to_string(&path).ok()
        })
        .collect();

    if documents.is_empty() {
        return;
    }

    let total_size: usize = documents.iter().map(|d| d.len()).sum();
    group.throughput(Throughput::Bytes(total_size as u64));

    group.bench_function("10_documents", |b| {
        b.iter(|| {
            for html in &documents {
                let readability = Readability::new(black_box(html), None, None).unwrap();
                black_box(readability.parse());
            }
        });
    });

    group.finish();
}

fn bench_large_documents(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_documents");
    group.sample_size(20);

    let large_cases = [
        ("yahoo-2", "Yahoo (~1.6MB)"),
        ("wikipedia-2", "Wikipedia (~1MB)"),
        ("guardian-1", "Guardian (~1.1MB)"),
    ];

    for (case_name, label) in large_cases.iter() {
        let html = match load_test_case(case_name) {
            html if !html.is_empty() => html,
            _ => continue,
        };

        let size = html.len();
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(BenchmarkId::new(*label, size), &html, |b, html| {
            b.iter(|| {
                let readability = Readability::new(black_box(html), None, None).unwrap();
                black_box(readability.parse())
            });
        });
    }

    group.finish();
}

fn bench_readerable_check(c: &mut Criterion) {
    use readabilityrs::is_probably_readerable;

    let mut group = c.benchmark_group("readerable_check");

    let test_cases = [
        ("001", "Small"),
        ("nytimes-1", "Medium"),
        ("guardian-1", "Large"),
    ];

    for (case_name, label) in test_cases.iter() {
        let html = match load_test_case(case_name) {
            html if !html.is_empty() => html,
            _ => continue,
        };

        let size = html.len();
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(BenchmarkId::new(*label, size), &html, |b, html| {
            b.iter(|| black_box(is_probably_readerable(black_box(html), None)));
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_parse_single,
    bench_with_url,
    bench_batch_processing,
    bench_large_documents,
    bench_readerable_check,
);

criterion_main!(benches);
