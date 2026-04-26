use std::fs;
use std::hint::black_box;
use std::path::Path;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use rusty_judge::{
    build_semantic_model_from_root,
    default_rules,
    parse_source,
    RuleContext,
};

const MAX_LINE_LENGTH: usize = 120;

fn corpus_files() -> Vec<(&'static str, &'static str)> {
    vec![
        ("small", "benchmarks/corpus/small.js"),
        ("medium", "benchmarks/corpus/medium.js"),
        ("large", "benchmarks/corpus/large.js"),
    ]
}

fn read_source(path: &str) -> String {
    fs::read_to_string(Path::new(path))
        .unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

fn line_count(source: &str) -> u64 {
    source.lines().count() as u64
}

fn run_rules(ctx: &RuleContext) -> usize {
    let rules = default_rules();

    let mut diagnostics = Vec::new();

    for rule in &rules {
        diagnostics.extend(rule.check(ctx));
    }

    diagnostics.len()
}

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("rustyjudge_parse");

    for (name, path) in corpus_files() {
        let source = read_source(path);

        group.throughput(Throughput::Elements(line_count(&source)));

        group.bench_with_input(
            BenchmarkId::new("parse", name),
            &source,
            |b, source| {
                b.iter(|| {
                    let root = parse_source(black_box(source))
                        .expect("parse failed");

                    black_box(root);
                });
            },
        );
    }

    group.finish();
}

fn bench_semantic_cfg(c: &mut Criterion) {
    let mut group = c.benchmark_group("rustyjudge_semantic_cfg");

    for (name, path) in corpus_files() {
        let source = read_source(path);

        group.throughput(Throughput::Elements(line_count(&source)));

        group.bench_with_input(
            BenchmarkId::new("semantic_cfg", name),
            &source,
            |b, source| {
                b.iter(|| {
                    let root = parse_source(black_box(source))
                        .expect("parse failed");

                    let semantic = build_semantic_model_from_root(root)
                        .expect("semantic model build failed");

                    black_box(semantic);
                });
            },
        );
    }

    group.finish();
}

fn bench_rules_only(c: &mut Criterion) {
    let mut group = c.benchmark_group("rustyjudge_rules_only");

    for (name, path) in corpus_files() {
        let source = read_source(path);
        let root = parse_source(&source).expect("parse failed");
        let semantic = build_semantic_model_from_root(root.clone())
            .expect("semantic model build failed");

        group.throughput(Throughput::Elements(line_count(&source)));

        group.bench_with_input(
            BenchmarkId::new("rules", name),
            &source,
            |b, source| {
                b.iter(|| {
                    let ctx = RuleContext::new(
                        "benchmark.js".to_string(),
                        root.clone(),
                        black_box(source.clone()),
                        MAX_LINE_LENGTH,
                        Some(semantic.clone()),
                    );

                    let diagnostics_count = run_rules(&ctx);

                    black_box(diagnostics_count);
                });
            },
        );
    }

    group.finish();
}

fn bench_total_lint(c: &mut Criterion) {
    let mut group = c.benchmark_group("rustyjudge_total_lint");

    for (name, path) in corpus_files() {
        let source = read_source(path);

        group.throughput(Throughput::Elements(line_count(&source)));

        group.bench_with_input(
            BenchmarkId::new("total", name),
            &source,
            |b, source| {
                b.iter(|| {
                    let root = parse_source(black_box(source))
                        .expect("parse failed");

                    let semantic = build_semantic_model_from_root(root.clone())
                        .expect("semantic model build failed");

                    let ctx = RuleContext::new(
                        "benchmark.js".to_string(),
                        root,
                        black_box(source.clone()),
                        MAX_LINE_LENGTH,
                        Some(semantic),
                    );

                    let diagnostics_count = run_rules(&ctx);

                    black_box(diagnostics_count);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_parse,
    bench_semantic_cfg,
    bench_rules_only,
    bench_total_lint
);

criterion_main!(benches);