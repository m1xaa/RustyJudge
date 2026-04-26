# RustyJudge Benchmarking

This document describes the benchmarking setup for **RustyJudge**, a JavaScript linter implemented in Rust.

The benchmark suite is organized inside the `benchmarks/` folder. It measures RustyJudge and compares it with external JavaScript linters on the same generated JavaScript corpus.

## Benchmarking goals

The benchmark suite is designed to measure:

- parsing time
- semantic model and CFG construction time
- rule execution time
- total linting time
- scalability with file size

For RustyJudge, internal phases can be measured directly because the benchmark calls RustyJudge functions from Rust code.

For ESLint, parsing time and rule execution time are collected through ESLint statistics.

For RSLint, only total wall-clock time is measured through the CLI, because the CLI does not expose internal parsing and rule execution timings in the same way.

## Current folder structure

The benchmark-related files are organized as follows:

```text
RustyJudge/
├── benchmarks/
│   ├── corpus/
│   │   ├── small.js
│   │   ├── medium.js
│   │   └── large.js
│   ├── eslint/
│   │   ├── eslint.config.mjs
│   │   ├── package.json
│   │   ├── package-lock.json
│   │   └── run_eslint_benchmark.mjs
│   ├── results/
│   │   ├── eslint.csv
│   │   └── rslint.csv
│   ├── rslint/
│   │   └── run-rslint-benchmark.mjs
│   ├── rusty_judge/
│   │   └── run_rusty_judge_benchmark.rs
│   ├── tools/
│   │   └── generate_benchmark_corpus.py
│   └── README.md
└── Cargo.toml
```

## Benchmark corpus

The benchmark corpus is stored in:

```text
benchmarks/corpus/
```

It contains JavaScript files of different sizes:

```text
small.js
medium.js
large.js
```

Each file contains generated JavaScript code with functions, variable declarations, loops, conditional branches, and linting issues.

The goal of using several file sizes is to evaluate how each linter behaves as the amount of JavaScript code increases.

## Generating benchmark files

The corpus generator is located at:

```text
benchmarks/tools/generate_benchmark_corpus.py
```

Run it from the project root:

```bash
python benchmarks/tools/generate_benchmark_corpus.py
```

This creates or overwrites:

```text
benchmarks/corpus/small.js
benchmarks/corpus/medium.js
benchmarks/corpus/large.js
```

The generated files use repeated JavaScript function patterns that exercise both AST-based and CFG-based RustyJudge rules.

## RustyJudge benchmark

RustyJudge uses Criterion for precise execution-time benchmarks.

The RustyJudge benchmark file is located at:

```text
benchmarks/rusty_judge/run_rusty_judge_benchmark.rs
```

Because this file is not in Cargo's default `benches/` directory, it must be registered explicitly in `Cargo.toml`.

### Cargo configuration

Add or keep this section in `Cargo.toml`:

```toml
[[bench]]
name = "rusty_judge_benchmark"
path = "benchmarks/rusty_judge/run_rusty_judge_benchmark.rs"
harness = false
```

The `name` value is the benchmark target name used in the `cargo bench` command.

The `path` value tells Cargo where the benchmark file is located.

The `harness = false` setting is required because Criterion provides its own benchmark runner instead of using Rust's default test/benchmark harness.

### RustyJudge measured phases

The RustyJudge Criterion benchmark measures four phases:

### 1. Parsing time

Measures only JavaScript parsing:

```text
source code -> AST
```

This shows how much time RustyJudge spends converting JavaScript source code into the parser's syntax tree representation.

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

Measures only lint rule execution over an already prepared `RuleContext`.

This includes both:

- AST-based lint rules
- CFG-based lint rules

Parsing and semantic model construction are done before the measured section, so this benchmark focuses on the cost of the rule checks.

### 4. Total linting time

Measures the complete RustyJudge pipeline:

```text
parse -> build semantic model/CFG -> run rules -> collect diagnostics
```

This is the most important user-facing metric because it represents the full cost of linting a file.

### Building RustyJudge before running the benchmark

Before running the RustyJudge benchmark, build the project in release mode from the project root:

```bash
cargo build --release
```

This confirms that the project compiles successfully with release optimizations before benchmark results are collected.

Criterion benchmarks are compiled and executed through `cargo bench`, but running a release build first is useful as a sanity check and ensures that the project has no release-build compilation issues.

If the benchmark setup also depends on the RustyJudge CLI binary, the release build places the compiled binary under:

```text
target/release/
```

### Running the RustyJudge benchmark

From the project root, run:

```bash
cargo bench --bench rusty_judge_benchmark
```

Criterion will execute all benchmark groups and generate a report.

The HTML report is usually available at:

```text
target/criterion/report/index.html
```

Open this file in a browser to inspect detailed benchmark results, charts, and statistical information.

### RustyJudge benchmark groups

The benchmark defines these groups:

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

This allows comparison across different file sizes.

## ESLint benchmark

The ESLint benchmark is located in:

```text
benchmarks/eslint/
```

Files:

```text
benchmarks/eslint/eslint.config.mjs
benchmarks/eslint/package.json
benchmarks/eslint/package-lock.json
benchmarks/eslint/run_eslint_benchmark.mjs
```

The ESLint benchmark measures:

- parse time
- rule execution time
- ESLint internal total time
- total wall-clock time
- number of diagnostics

The script uses ESLint's Node API and writes results to CSV.

### Installing ESLint benchmark dependencies

From the `benchmarks/eslint/` folder, run:

```bash
npm install
```

This installs the ESLint dependency used by the benchmark script.

### Running the ESLint benchmark

From the project root, run:

```bash
node benchmarks/eslint/run_eslint_benchmark.mjs
```

The script uses:

```text
benchmarks/eslint/eslint.config.mjs
```

as its ESLint configuration file.

### ESLint result file

The output is written to:

```text
benchmarks/results/eslint.csv
```

The CSV columns are:

```text
dataset,file,lines,parse_ms,rule_ms,eslint_internal_total_ms,total_wall_ms,diagnostics
```

## RSLint benchmark

The RSLint benchmark is located in:

```text
benchmarks/rslint/
```

File:

```text
benchmarks/rslint/run-rslint-benchmark.mjs
```

The RSLint benchmark measures:

- total wall-clock time
- exit status

It does not measure parsing time or rule execution time separately, because the RSLint CLI does not expose those internal phases in the same benchmark interface.

### Installing RSLint

Make sure the `rslint` command is available in the terminal:

```bash
rslint --help
```

If it is not available, install it through Cargo if supported by the local environment:

```bash
cargo install rslint
```

If the binary is not in `PATH`, provide it through the `RSLINT_BIN` environment variable.

### Running the RSLint benchmark

From the project root, run:

```bash
node benchmarks/rslint/run-rslint-benchmark.mjs
```

If `rslint` is not in `PATH`, run it like this on PowerShell:

```powershell
$env:RSLINT_BIN="C:\path\to\rslint.exe"
node benchmarks/rslint/run-rslint-benchmark.mjs
```

### RSLint result file

The output is written to:

```text
benchmarks/results/rslint.csv
```

The CSV columns are:

```text
dataset,file,lines,total_wall_ms,last_status
```

## Results folder

All generated benchmark result files are stored in:

```text
benchmarks/results/
```

Currently used files:

```text
benchmarks/results/eslint.csv
benchmarks/results/rslint.csv
```

RustyJudge Criterion results are generated under:

```text
target/criterion/
```

The main Criterion HTML report is usually:

```text
target/criterion/report/index.html
```

## Recommended benchmark workflow

Run the full benchmark workflow from the project root:

```bash
python benchmarks/tools/generate_benchmark_corpus.py
cargo build --release
cargo bench --bench rusty_judge_benchmark
node benchmarks/eslint/run_eslint_benchmark.mjs
node benchmarks/rslint/run-rslint-benchmark.mjs
```

The first command regenerates the JavaScript corpus.

The second command builds RustyJudge in release mode.

The third command runs the RustyJudge Criterion benchmark.

The fourth command runs the ESLint benchmark and writes:

```text
benchmarks/results/eslint.csv
```

The fifth command runs the RSLint benchmark and writes:

```text
benchmarks/results/rslint.csv
```

## Metrics summary

| Tool | Parse time | Rule time | Total time | Notes |
|---|---:|---:|---:|---|
| RustyJudge | yes | yes | yes | measured internally through Criterion |
| ESLint | yes | yes | yes | measured through ESLint stats and Node wall-clock time |
| RSLint | no | no | yes | measured through CLI wall-clock time |

## Interpreting the results

### RustyJudge

Use Criterion output for detailed phase measurements.

The most important RustyJudge groups are:

- `rustyjudge_parse`
- `rustyjudge_semantic_cfg`
- `rustyjudge_rules_only`
- `rustyjudge_total_lint`

### ESLint

Use `benchmarks/results/eslint.csv`.

Important columns:

- `parse_ms`
- `rule_ms`
- `total_wall_ms`

### RSLint

Use `benchmarks/results/rslint.csv`.

Important column:

- `total_wall_ms`

For RSLint, parse time and rule time should be marked as `N/A` in comparison tables.

## Notes

Benchmark results should be collected on the same machine and under similar system conditions.

For more stable results:

- close unnecessary applications
- run benchmarks multiple times
- use release builds where applicable
- avoid comparing debug builds with release builds
- use the same generated corpus for all tools
- keep ESLint and RSLint versions documented in the final report

## Known limitations

The benchmark does not guarantee a perfectly identical rule set between RustyJudge, ESLint, and RSLint.

RustyJudge rules include project-specific CFG-based checks.

ESLint uses a configured set of comparable built-in rules.

RSLint is measured through the CLI, so only total wall-clock time is used.

Because of these differences, the comparison should be interpreted as an indicative performance comparison, not as a fully controlled scientific benchmark of identical rule implementations.
