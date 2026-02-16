import * as vscode from 'vscode';
import { exec } from 'child_process';
import * as path from 'path';

let diagnosticCollection: vscode.DiagnosticCollection;

export function activate(context: vscode.ExtensionContext) {

    diagnosticCollection = vscode.languages.createDiagnosticCollection('rustyjudge');
    context.subscriptions.push(diagnosticCollection);

    const disposable = vscode.commands.registerCommand('rustyjudge.run', () => {

        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showErrorMessage("No active file");
            return;
        }

        const filePath = editor.document.fileName;

        const binaryPath = path.join(
            context.extensionPath,
            '..',
            'target',
            'debug',
            'rusty_judge.exe'
        );

        exec(`"${binaryPath}" "${filePath}"`, (error, stdout, stderr) => {

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
                    const diagnostics: vscode.Diagnostic[] = [];

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
