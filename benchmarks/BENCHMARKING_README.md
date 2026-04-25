# RustyJudge Benchmarking

This document describes the benchmarking setup for **RustyJudge**, a JavaScript linter implemented in Rust.

The goal of benchmarking is to measure the performance of the linter on JavaScript files of different sizes and to evaluate how the tool scales as the input grows.

## Benchmarking goals

The benchmark suite is designed to measure:

- parsing time
- semantic model and CFG construction time
- rule execution time
- total linting time
- memory usage
- scalability with file size and project size

The benchmarking setup is split into two parts:

1. **Criterion benchmarks** for precise time measurements.
2. **Memory benchmark runner** for process-level memory usage.

## Folder structure

Recommended project structure:

```text
RustyJudge/
├── benches/
│   └── rustyjudge_benchmark.rs
├── benchmarks/
│   ├── corpus/
│   │   ├── small.js
│   │   ├── medium.js
│   │   └── large.js
│   └── README.md
├── tools/
│   └── generate_benchmark_corpus.py
├── src/
│   └── bin/
│       └── benchmark_memory.rs
└── Cargo.toml
```

## Benchmark corpus

The benchmark corpus is stored in:

```text
benchmarks/corpus/
```

The corpus contains JavaScript files of different sizes:

```text
small.js
medium.js
large.js
```

Each file contains JavaScript code with functions, variable declarations, loops, conditional branches, and linting issues.

The purpose of using several input sizes is to test how RustyJudge behaves as the number of lines, functions, symbols, and CFG blocks increases.

## Generating benchmark files

The benchmark corpus can be generated using:

```bash
python tools/generate_benchmark_corpus.py
```

This script creates or overwrites:

```text
benchmarks/corpus/small.js
benchmarks/corpus/medium.js
benchmarks/corpus/large.js
```

The generated files contain repeated JavaScript function patterns that exercise both AST-based and CFG-based lint rules.

## Criterion benchmarks

RustyJudge uses Criterion for measuring execution time.

The Criterion benchmark file is located at:

```text
benches/rustyjudge_benchmark.rs
```

This file measures four main phases:

### 1. Parsing time

Measures only JavaScript parsing:

```text
source code -> AST
```

This shows how much time is spent converting JavaScript source code into the parser's syntax tree representation.

### 2. Semantic model and CFG construction time

Measures:

```text
AST -> SemanticModel -> CFGs -> liveness data
```

This includes:

- symbol resolution
- scope handling
- script-level CFG construction
- function-level CFG construction
- liveness analysis

### 3. Rule execution time

Measures only rule execution over an already prepared context.

This includes both:

- AST-based lint rules
- CFG-based lint rules

The parsing and semantic model construction are performed before the measured section, so this benchmark focuses on how expensive the rule checks are.

### 4. Total linting time

Measures the complete linting pipeline:

```text
parse -> build semantic model/CFG -> run rules -> collect diagnostics
```

This is the most important metric from the user's perspective because it represents the full cost of linting a file.

## Cargo configuration

Criterion benchmarks require a benchmark target in `Cargo.toml`:

```toml
[[bench]]
name = "rustyjudge_benchmark"
harness = false
```

The benchmark file must be placed at:

```text
benches/rustyjudge_benchmark.rs
```

`harness = false` is required because Criterion provides its own benchmark runner instead of using Rust's default test/benchmark harness.

## Running Criterion benchmarks

From the root of the RustyJudge project, run:

```bash
cargo bench --bench rustyjudge_benchmark
```

Criterion will execute all benchmark groups and generate a report.

The HTML report is usually available at:

```text
target/criterion/report/index.html
```

Open this file in a browser to inspect detailed benchmark results, charts, and statistical information.

## Benchmark groups

The Criterion benchmark defines these groups:

```text
rustyjudge_parse
rustyjudge_semantic_cfg
rustyjudge_rules_only
rustyjudge_total_lint
```

Each group is executed for:

```text
small.js
medium.js
large.js
```

This allows comparison across file sizes.

## Memory benchmark

Criterion is used for timing measurements, but memory usage is measured separately.

The memory benchmark runner is located at:

```text
src/bin/benchmark_memory.rs
```

It starts the RustyJudge CLI as a separate process and periodically samples the process memory usage.

The output is printed as CSV:

```text
dataset,elapsed_ms,peak_memory_kib,exit_code
small,12,8460,0
medium,38,9120,0
large,180,14120,0
```

## Running the memory benchmark

First build the RustyJudge CLI in release mode:

```bash
cargo build --release --bin rusty_judge
```

Then run:

```bash
cargo run --release --bin benchmark_memory
```

If the CLI binary has a different path, set the `RUSTY_JUDGE_BIN` environment variable.

### PowerShell

```powershell
$env:RUSTY_JUDGE_BIN="target/release/rusty_judge.exe"
cargo run --release --bin benchmark_memory
```

### Bash

```bash
RUSTY_JUDGE_BIN=target/release/rusty_judge cargo run --release --bin benchmark_memory
```

## Interpreting memory results

The memory benchmark measures peak memory at the process level.

This means it does not separate memory usage per internal phase, such as parsing or rule execution. Instead, it reports the maximum observed memory consumption of the whole RustyJudge process during one linting run.

This is sufficient for comparing how memory usage changes across small, medium, and large input files.


## Scalability analysis

Scalability is evaluated by comparing the benchmark results across different input sizes.

The main question is:

```text
How does RustyJudge performance change as the JavaScript input becomes larger?
```

Important observations include:

- whether parsing time grows linearly with input size
- whether CFG construction becomes more expensive for larger functions
- whether rule execution time increases with the number of statements and symbols
- whether memory usage grows predictably with file size

## Comparison with external linters

This setup currently measures RustyJudge only.

For comparison with external tools such as ESLint or RSLint, the most reliable common metric is:

```text
total linting time
```

RustyJudge can expose internal measurements such as parsing time and rule execution time because it is part of this project.

External tools may not expose the same internal phases through their CLI interfaces. Therefore, when comparing with ESLint or RSLint, it is acceptable to measure total time and memory usage as process-level metrics.

If ESLint statistics are enabled through its API or CLI options, additional metrics such as parsing time or rule execution time can be included separately.

## Recommended benchmark workflow

Run the complete benchmark workflow in this order:

```bash
python tools/generate_benchmark_corpus.py
cargo bench --bench rustyjudge_benchmark
cargo build --release --bin rusty_judge
cargo run --release --bin benchmark_memory
```

Then collect results from:

```text
target/criterion/report/index.html
benchmarks/results/
```

and summarize them in the project report.

## Notes

Benchmark results should be collected on the same machine and under similar system conditions.

For more stable results:

- close unnecessary applications
- run benchmarks multiple times
- use release builds
- avoid measuring debug builds
- compare tools on the same input corpus
