"use strict";
var __create = Object.create;
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getProtoOf = Object.getPrototypeOf;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toESM = (mod, isNodeMode, target) => (target = mod != null ? __create(__getProtoOf(mod)) : {}, __copyProps(
  // If the importer is in node compatibility mode or this is not an ESM
  // file that has been converted to a CommonJS file using a Babel-
  // compatible transform (i.e. "__esModule" has not been set), then set
  // "default" to the CommonJS "module.exports" for node compatibility.
  isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", { value: mod, enumerable: true }) : target,
  mod
));
var __toCommonJS = (mod) => __copyProps(__defProp({}, "__esModule", { value: true }), mod);

// src/extension.ts
var extension_exports = {};
__export(extension_exports, {
  activate: () => activate
});
module.exports = __toCommonJS(extension_exports);
var vscode = __toESM(require("vscode"));
var import_child_process = require("child_process");
var path = __toESM(require("path"));
var diagnosticCollection;
function activate(context) {
  diagnosticCollection = vscode.languages.createDiagnosticCollection("rustyjudge");
  context.subscriptions.push(diagnosticCollection);
  const disposable = vscode.commands.registerCommand("rustyjudge.run", () => {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showErrorMessage("No active file");
      return;
    }
    const filePath = editor.document.fileName;
    const binaryPath = path.join(
      context.extensionPath,
      "..",
      "target",
      "debug",
      "rusty_judge.exe"
    );
    (0, import_child_process.exec)(`"${binaryPath}" "${filePath}"`, (error, stdout, stderr) => {
      if (error) {
        vscode.window.showErrorMessage("RustyJudge failed to execute");
        console.error(stderr);
        return;
      }
      try {
        const reports = JSON.parse(stdout);
        diagnosticCollection.clear();
        for (const report of reports) {
          const uri = vscode.Uri.file(report.file);
          const diagnostics = [];
          for (const d of report.diagnostics) {
            const range = new vscode.Range(
              new vscode.Position(d.row - 1, d.col - 1),
              new vscode.Position(d.row - 1, d.col)
            );
            const diagnostic = new vscode.Diagnostic(
              range,
              d.message,
              vscode.DiagnosticSeverity.Warning
            );
            diagnostics.push(diagnostic);
          }
          diagnosticCollection.set(uri, diagnostics);
        }
      } catch (e) {
        vscode.window.showErrorMessage("Invalid JSON from RustyJudge");
        console.error(e);
      }
    });
  });
  context.subscriptions.push(disposable);
}
// Annotate the CommonJS export names for ESM import in node:
0 && (module.exports = {
  activate
});
//# sourceMappingURL=extension.js.map
