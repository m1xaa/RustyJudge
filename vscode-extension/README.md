# RustyJudge VS Code Extension

This folder contains the Visual Studio Code extension for **RustyJudge**, a JavaScript linter implemented in Rust.

The extension uses the **Language Server Protocol (LSP)** to communicate with the RustyJudge language server. The language server analyzes JavaScript files, runs the linting rules, and sends diagnostics back to VS Code.

## Project purpose

The goal of this extension is to provide real-time linting feedback inside Visual Studio Code.

When a JavaScript file is opened or edited, the extension starts the RustyJudge language server and displays diagnostics directly in the editor and in the **Problems** panel.

## Main features

RustyJudge currently supports both AST-based and CFG-based linting rules, including:

- no `var` declarations
- unused variables
- strict equality checks
- `console.log` detection
- duplicate function parameters
- empty block detection
- unreachable code detection
- consistent return analysis
- dead store detection
- potential infinite loop detection

## Folder structure

```text
vscode-extension/
├── src/
│   └── extension.ts              # VS Code extension client
├── dist/
│   └── extension.js              # Bundled JavaScript extension output
├── server/
│   └── win32-x64/
│       └── rusty_judge_lsp.exe   # RustyJudge LSP binary, built manually
├── package.json                  # VS Code extension manifest
├── tsconfig.json                 # TypeScript configuration
├── esbuild.js                    # Bundling script
└── README.md
```

## Important note about the bundled server

The extension expects a RustyJudge language server binary inside the `server/` folder.

For Windows x64, the expected path is:

```text
vscode-extension/server/win32-x64/rusty_judge_lsp.exe
```

The binary may be ignored by Git because it is a generated build artifact.

If the binary is not present, you must build it manually and copy it into the expected folder before running or packaging the extension.

## Building the Rust language server

From the root of the RustyJudge project, run:

```bash
cargo build --release --bin rusty_judge_lsp
```

On Windows, this creates:

```text
target/release/rusty_judge_lsp.exe
```

Copy that file into:

```text
vscode-extension/server/win32-x64/rusty_judge_lsp.exe
```

If the folder does not exist, create it manually:

```text
vscode-extension/server/win32-x64/
```

## Installing extension dependencies

From the `vscode-extension` folder, run:

```bash
npm install
```

This installs the VS Code extension dependencies, including the LSP client package.

## Compiling the extension

From the `vscode-extension` folder, run:

```bash
npm run compile
```

This checks the TypeScript code and bundles the extension into:

```text
dist/extension.js
```

## Running the extension in development mode

1. Open the `vscode-extension` folder in VS Code.
2. Make sure the Rust LSP binary exists at:

   ```text
   server/win32-x64/rusty_judge_lsp.exe
   ```

3. Press `F5`.

This opens a new **Extension Development Host** VS Code window.

In that new window, open a `.js` file. RustyJudge should start automatically and diagnostics should appear in the editor and in the **Problems** panel.

## Test JavaScript file

You can use this simple JavaScript file to check whether the extension works:

```javascript
function getValue(x) {
  if (x > 0) {
    return x;
  }
}

let a = 1;
a = 2;

var oldStyle = 10;

console.log(a);
```

Expected behavior:

- `consistent-return` should warn about the missing return path.
- `no-dead-stores` may warn about overwritten assignments.
- `no-var` should warn about `var oldStyle`.
- Other enabled rules may also produce diagnostics depending on the current implementation.

## Packaging the extension as VSIX

Before packaging, make sure:

1. The Rust language server is built in release mode.
2. The binary is copied into:

   ```text
   server/win32-x64/rusty_judge_lsp.exe
   ```

3. The extension has been compiled:

   ```bash
   npm run compile
   ```

Then run:

```bash
npx vsce package --target win32-x64
```

This creates a `.vsix` file, for example:

```text
rustyjudge-win32-x64-0.0.1.vsix
```

## Installing the packaged extension

You can install the generated `.vsix` file manually:

1. Open VS Code.
2. Open the Extensions view.
3. Click the `...` menu.
4. Select **Install from VSIX...**.
5. Choose the generated `.vsix` file.

Or install it from the terminal:

```bash
code --install-extension rustyjudge-win32-x64-0.0.1.vsix
```

After installation, restart VS Code and open a `.js` file.

## Git and generated files

Recommended Git behavior:

### Usually committed

```text
vscode-extension/src/
vscode-extension/dist/
vscode-extension/package.json
vscode-extension/package-lock.json
vscode-extension/tsconfig.json
vscode-extension/esbuild.js
vscode-extension/README.md
```

### Usually ignored

```text
vscode-extension/node_modules/
*.vsix
```

The generated `.vsix` file should usually not be committed because it can be recreated with:

```bash
npx vsce package --target win32-x64
```

### Language server binary

If the bundled LSP binary is ignored by Git, every developer must build it manually:

```bash
cargo build --release --bin rusty_judge_lsp
```

and then copy it to:

```text
vscode-extension/server/win32-x64/rusty_judge_lsp.exe
```

If the binary is committed, the extension can be packaged without rebuilding the server.

If the binary is ignored, the build step above is required.

## Troubleshooting

### The extension does not show diagnostics

Check the following:

1. The opened file is recognized by VS Code as JavaScript.
2. The LSP binary exists at:

   ```text
   server/win32-x64/rusty_judge_lsp.exe
   ```

3. The extension was compiled:

   ```bash
   npm run compile
   ```

4. The Rust server can run on your machine.

### The server binary is missing

Build it again:

```bash
cargo build --release --bin rusty_judge_lsp
```

Then copy it into:

```text
vscode-extension/server/win32-x64/rusty_judge_lsp.exe
```

### Windows blocks the binary

If Windows security policies block the generated `.exe`, the extension may fail to start the language server.

In that case, check:

- Windows Security
- App Control / Application Control policies
- antivirus quarantine or protection logs
- Event Viewer logs related to Code Integrity or AppLocker

## Known limitations

RustyJudge is a student project and does not aim to fully replace production JavaScript linters such as ESLint.

Current limitations may include:

- incomplete support for all JavaScript syntax forms
- limited handling of advanced destructuring
- limited type inference
- simplified infinite loop detection
- partial support for complex nested scopes and runtime-dependent behavior

## Development notes

The extension client is implemented in TypeScript.

The Rust language server handles:

- parsing JavaScript source code
- building the AST
- building the semantic model
- constructing CFGs
- running lint rules
- sending diagnostics to VS Code

The VS Code extension is responsible for:

- starting the Rust language server
- connecting to it through LSP
- forwarding document events
- displaying diagnostics returned by the server
