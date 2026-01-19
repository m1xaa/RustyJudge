# RustyJudge

RustyJudge is a Javascript linter written in Rust created as a project for the 7th semester course *Advanced Programming Techniques*.  

The tool parses JavaScript source code using a Rust-based JS parser, builds an **Abstract Syntax Tree (AST)**,  
and then applies a set of simple linting rules that operate directly on the AST.  


## Technology Stack  
- Language: **Rust**  
- Main dependencies: 
  - Rust-based JS parser(e.g. **swc_ecma_parser**, **rslint_parser**)
  - CLI argument parser(e.g. **clap**)


## High-Level Design

- Pipeline:
    - Read JS source file
    - Parse the code into an AST using the JS parser
    - Build a context object(file name, AST root, ...)
    - Run a set of **lint rules** over the AST
    - Collect the diagnostics and print them via CLI

- Code structure
    - **parser/** - wrapper around the JS parser
    - **rules/** - individual rule implementations
    - **linter/** - rule execution logic(core part of the project)
    - **cli/** - command line argument parser
    - **diagnostics/** - types of errors, warnings and locations
    - **test/**
 
## Lint Rules 

All rules below work with the AST by implementing some kind of *visitor*.

### Rule 1: No `var` Declarations

* **Name:** `no-var`
* **Idea:** Warn whenever `var` is used. Modern JS prefers `let` or `const`.
* **AST target:** Variable declaration nodes with kind `var`.
* **Example warning:**
   * "Avoid using `var`. Use `let` or `const` instead."

### Rule 2: Unused Variables (Basic)

* **Name:** `no-unused-vars`
* **Idea:** Detect variables that are declared but never used.
* **How (simplified):**
   * Collect all declared identifiers (from variable declarations and function parameters).
   * Collect all identifier usages.
   * Report those that are declared but never referenced.
* **Limitations:** No full scoping analysis, no shadowing handling in the first version.

### Rule 3: No `==` / `!=` (Prefer Strict Equality)

* **Name:** `strict-equality`
* **Idea:** Warn when `==` or `!=` is used instead of `===` / `!==`.
* **AST target:** Binary expression nodes with operators `==` or `!=`.
* **Example warning:**
   * "Use `===` instead of `==` for strict equality."

### Rule 4: Disallow `console.log` in Code

* **Name:** `no-console-log`
* **Idea:** Warn when `console.log()` is called (e.g. leftover debug code).
* **AST target:** Call expressions where the callee is `console.log`.
* **Example warning:**
   * "Unexpected `console.log` call. Remove debug logging before production."

### Rule 5: Maximum Line Length

* **Name:** `max-line-length`
* **Idea:** Enforce a line length limit (e.g. 100 characters) to improve readability.
* **Implementation detail:** This rule can be run directly on the raw file content before/after parsing, not on the AST.
* **Configuration:** Default max length, overridable via CLI flag.

### Rule 6: Duplicate Function Parameters

* **Name:** `no-duplicate-params`
* **Idea:** Warn when a function has the same parameter name more than once.
* **AST target:** Function declaration/expression parameter lists.
* **Example:** `function f(a, b, a) {}` should be flagged.

### Rule 7: Empty Block Statements

* **Name:** `no-empty-block`
* **Idea:** Report block statements `{}` that have no statements inside.
* **AST target:** Block nodes where the statement list is empty.
* **Possible exception (later):** Allow if there is a comment inside the block (not mandatory in the first version).

## Usage

The basic usage of this tool would be through the CLI.  
Below are listed possible usage options:
- `rusty_judge <files...>` - main command to run linting (must specify at least one file)
- `rusty_judge [dir]/*` - lint all JS files inside the *[dir]*
- `rusty_judge <files...> --max-line-length 120` - overrides the *max-line-length* rule
 
