import * as vscode from 'vscode';
import * as path from 'path';
import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind } from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext) {
  const serverPath = path.join(
    context.extensionPath,
    '..',
    'target',
    'debug',
    'rusty_judge_lsp.exe'
  );

  const serverOptions: ServerOptions = {
    command: serverPath,
    transport: TransportKind.stdio,
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      { scheme: 'file', language: 'javascript' },
    ],
  };

  client = new LanguageClient(
    'rustyjudge',
    'RustyJudge LSP',
    serverOptions,
    clientOptions
  );

  context.subscriptions.push(client);
  client.start();
}

export async function deactivate() {
  if (client) {
    await client.stop();
  }
}
