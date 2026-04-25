import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

function getPlatformFolder(): string | undefined {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === 'win32' && arch === 'x64') {
    return 'win32-x64';
  }

  if (platform === 'win32' && arch === 'arm64') {
    return 'win32-arm64';
  }

  if (platform === 'linux' && arch === 'x64') {
    return 'linux-x64';
  }

  if (platform === 'linux' && arch === 'arm64') {
    return 'linux-arm64';
  }

  if (platform === 'darwin' && arch === 'x64') {
    return 'darwin-x64';
  }

  if (platform === 'darwin' && arch === 'arm64') {
    return 'darwin-arm64';
  }

  return undefined;
}

function getServerPath(context: vscode.ExtensionContext): string {
  const platformFolder = getPlatformFolder();

  if (!platformFolder) {
    throw new Error(
      `Unsupported platform: ${process.platform}-${process.arch}`
    );
  }

  const exeName =
    process.platform === 'win32'
      ? 'rusty_judge_lsp.exe'
      : 'rusty_judge_lsp';

  return context.asAbsolutePath(
    path.join('server', platformFolder, exeName)
  );
}

export async function activate(context: vscode.ExtensionContext) {
  let serverPath: string;

  try {
    serverPath = getServerPath(context);
  } catch (error) {
    vscode.window.showErrorMessage(String(error));
    return;
  }

  if (!fs.existsSync(serverPath)) {
    vscode.window.showErrorMessage(
      `RustyJudge LSP binary was not found at: ${serverPath}`
    );
    return;
  }

  if (process.platform !== 'win32') {
    try {
      fs.chmodSync(serverPath, 0o755);
    } catch {
      // If chmod fails, let VS Code try to start it and show the real error.
    }
  }

  const serverOptions: ServerOptions = {
    command: serverPath,
    transport: TransportKind.stdio,
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      { scheme: 'file', language: 'javascript' },
      { scheme: 'file', language: 'javascriptreact' },
    ],
  };

  client = new LanguageClient(
    'rustyjudge',
    'RustyJudge LSP',
    serverOptions,
    clientOptions
  );

  context.subscriptions.push(client);

  await client.start();
}

export async function deactivate() {
  if (client) {
    await client.stop();
  }
}