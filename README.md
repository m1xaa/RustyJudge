# RustyJudge

RustyJudge is a Javascript linter written in Rust created as a project for the 7th semester course *Advanced Programming Techniques*.

The tool parses JavaScript source code using a Rust-based JS parser, builds an **Abstract Syntax Tree (AST)**, and then applies a set of simple linting rules that operate directly on the AST.

## Technology Stack

- Language: **Rust**
- Main dependencies:
  - Rust-based JS parser (e.g. **swc_ecma_parser**, **rslint_parser**)
  - CLI argument parser (e.g. **clap**)

## High-Level Design

- Pipeline:
  - Read JS source file
  - Parse the code into an AST using the JS parser
  - Build a context object (file name, AST root, ...)
  - Run a set of **lint rules** over the AST
  - Collect the diagnostics and print them via CLI

- Code structure:
  - **parser/** - wrapper around the JS parser
  - **rules/** - individual rule implementations
  - **linter/** - rule execution logic (core part of the project)
  - **cli/** - command line argument parser
  - **diagnostics/** - types of errors, warnings and locations
  - **test/**

## Lint Rules

All rules below work with the AST by implementing some kind of *visitor*.

### Rule 1: No `var` Declarations

- **Name:** `no-var`
- **Idea:** Warn whenever `var` is used. Modern JS prefers `let` or `const`.
- **AST target:** Variable declaration nodes with kind `var`.
- **Example warning:** "Avoid using `var`. Use `let` or `const` instead."

### Rule 2: Unused Variables (Basic)

- **Name:** `no-unused-vars`
- **Idea:** Detect variables that are declared but never used.
- **How (simplified):**
  - Collect all declared identifiers (from variable declarations and function parameters).
  - Collect all identifier usages.
  - Report those that are declared but never referenced.
- **Limitations:** No full scoping analysis, no shadowing handling in the first version.

### Rule 3: No `==` / `!=` (Prefer Strict Equality)

- **Name:** `strict-equality`
- **Idea:** Warn when `==` or `!=` is used instead of `===` / `!==`.
- **AST target:** Binary expression nodes with operators `==` or `!=`.
- **Example warning:** "Use `===` instead of `==` for strict equality."

### Rule 4: Disallow `console.log` in Code

- **Name:** `no-console-log`
- **Idea:** Warn when `console.log()` is called (e.g. leftover debug code).
- **AST target:** Call expressions where the callee is `console.log`.
- **Example warning:** "Unexpected `console.log` call. Remove debug logging before production."

### Rule 5: Maximum Line Length

- **Name:** `max-line-length`
- **Idea:** Enforce a line length limit (e.g. 100 characters) to improve readability.
- **Implementation detail:** This rule can be run directly on the raw file content before/after parsing, not on the AST.
- **Configuration:** Default max length, overridable via CLI flag.

### Rule 6: Duplicate Function Parameters

- **Name:** `no-duplicate-params`
- **Idea:** Warn when a function has the same parameter name more than once.
- **AST target:** Function declaration/expression parameter lists.
- **Example:** `function f(a, b, a) {}` should be flagged.

### Rule 7: Empty Block Statements

- **Name:** `no-empty-block`
- **Idea:** Report block statements `{}` that have no statements inside.
- **AST target:** Block nodes where the statement list is empty.
- **Possible exception (later):** Allow if there is a comment inside the block (not mandatory in the first version).

## Usage

The basic usage of this tool would be through the CLI.

Below are listed possible usage options:

- `rustyjudge lint <files...>` - main command to run linting (must specify at least one file)
- `rustyjudge lint [dir]/*` - lint all JS files inside the *[dir]*
- `rustyjudge lint <files...> --max-line-length 120` - overrides the *max-line-length* rule

---

# RustyJudge - Thesis Extensions

## Control Flow Graph (CFG) Analysis

The thesis version of RustyJudge introduces **Control Flow Graph** construction and analysis, enabling more complex linting rules that require understanding of program execution flow.

**CFG Implementation:**

- `cfg/` - Control flow graph builder and data structures
- `cfg_analyzer/` - Analysis algorithms on top of CFG
- Graph construction from AST
- Basic block identification
- Edge labeling (conditional, unconditional jumps)

### Rule 8: Unreachable Code Detection

- **Name:** `no-unreachable-code`
- **Description:** Detects code that can never be executed
- **Analysis method:**
  - Builds CFG from AST
  - Performs reachability analysis from function entry point
  - Reports statements in unreachable blocks
- **Example:**

```javascript
function test() {
  return 42;
  console.log("This is unreachable"); // Warning
}
```

### Rule 9: Missing Return Paths

- **Name:** `consistent-return`
- **Description:** Ensures all code paths in a function return a value (or none)
- **Analysis method:**
  - Traverses all paths through the CFG
  - Checks if all paths lead to a return statement
- **Example:**

```javascript
function getValue(x) {
  if (x > 0) {
    return x;
  }
  // Warning: missing return in else branch
}
```

### Rule 10: Infinite Loop Detection

- **Name:** `no-infinite-loops`
- **Description:** Warns about potential infinite loops with no exit condition
- **Analysis method:**
  - Identifies loops in CFG
  - Checks for break statements or conditional exits
  - Reports loops with no observable exit path

### Rule 11: Dead Store Detection

- **Name:** `no-dead-stores`
- **Description:** Detects variable assignments that are never read before being overwritten
- **Analysis method:**
  - Performs dataflow analysis on CFG
  - Tracks variable writes and reads
  - Reports assignments that are overwritten before use

---

## VSCode Extension

To improve developer experience, RustyJudge includes a **VSCode extension** that provides real-time linting feedback directly in the editor.

**Features:**

- Real-time diagnostics as you type
- Inline error and warning messages displayed in the editor
- Integration with VSCode's problems panel

**How it works:**

- Uses Language Server Protocol (LSP) for communication between VSCode and RustyJudge
- Runs RustyJudge linter in the background on file changes
- Sends diagnostics back to the editor for display

**Extension Structure:**

- `vscode-extension/` - VSCode extension source code
- Language Server implementation in Rust
- Communication bridge between VSCode and RustyJudge CLI

---

## Performance Benchmarking

A comprehensive benchmark suite compares RustyJudge's performance against existing JavaScript linters:

**Compared Tools:**

1. **Rust-based linter:** RSLint or similar Rust implementation
2. **Non-Rust linter:** ESLint (Node.js/JavaScript implementation)

**Benchmark Metrics:**

- Parsing time
- Rule execution time
- Total linting time
- Memory usage
- Scalability with file size and project size

**Benchmark Structure:**

- `benchmarks/` - Benchmark suite
- Test corpus of various JS files (small, medium, large)
- Automated benchmark runner
- Performance visualization and reporting

**Running Benchmarks:**

```bash
cargo bench
```

---

## Project Structure (Extensions)

```text
rustyjudge/
+-- src/
|   +-- cfg/            # Control flow graph
|   +-- cfg_rules/      # CFG-based rules
|   +-- ...
+-- vscode-extension/   # VSCode extension
+-- benchmarks/         # Performance benchmarks
+-- ...
```

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
