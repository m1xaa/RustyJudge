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
```
rustyjudge/
├── src/
│   ├── cfg/            # Control flow graph
│   ├── cfg_rules/      # CFG-based rules
│   └── ...
├── vscode-extension/   # VSCode extension
├── benchmarks/         # Performance benchmarks
└── ...
```
