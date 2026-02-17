import * as vscode from 'vscode';
import * as path from 'path';
import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind } from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext) {
  const out = vscode.window.createOutputChannel('RustyJudge');
  out.appendLine('activate() called');

  const serverPath = path.join(
    context.extensionPath,
    '..',
    'target',
    'debug',
    'rusty_judge_lsp.exe'
  );

  out.appendLine('serverPath = ' + serverPath);

  const serverOptions: ServerOptions = {
    command: serverPath,
    transport: TransportKind.stdio,
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      { scheme: 'file', language: 'javascript' },
    ],
    outputChannel: out,
    traceOutputChannel: out,
  };

  client = new LanguageClient('rustyjudge', 'RustyJudge LSP', serverOptions, clientOptions);

  context.subscriptions.push(client);
  client.start().then(() => out.appendLine('client.start() resolved'))
    .catch(err => {
      out.appendLine('client.start() FAILED: ' + String(err));
      console.error(err);
    });
}

export async function deactivate() {
  await client?.stop();
}
