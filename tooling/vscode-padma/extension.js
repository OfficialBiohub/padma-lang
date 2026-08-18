const vscode = require('vscode');
const { execFile } = require('child_process');
const { LanguageClient, TransportKind } = require('vscode-languageclient/node');

let languageClient;

function activePadmaDocument() {
  const document = vscode.window.activeTextEditor?.document;
  if (!document || document.languageId !== 'padma') {
    vscode.window.showWarningMessage('Open a Padma (.pd) file first.');
    return undefined;
  }
  return document;
}

function padmaCommand() {
  return vscode.workspace.getConfiguration('padma').get('command', 'padma');
}

function languageServerCommand() {
  return vscode.workspace.getConfiguration('padma').get('languageServer.command', 'padma-lsp');
}

function runInTerminal(argumentsList, label) {
  const terminal = vscode.window.createTerminal({ name: label });
  terminal.show(true);
  const quoted = argumentsList.map(quoteForPosixShell).join(' ');
  terminal.sendText(`${quoteForPosixShell(padmaCommand())} ${quoted}`);
}

function quoteForPosixShell(value) {
  return `'${String(value).replace(/'/g, "'\\''")}'`;
}

function diagnosticRange(range) {
  const start = range?.start ?? { line: 1, column: 1 };
  const end = range?.end ?? start;
  return new vscode.Range(
    Math.max(0, start.line - 1),
    Math.max(0, start.column - 1),
    Math.max(0, end.line - 1),
    Math.max(0, end.column - 1),
  );
}

function checkDocument(document, collection) {
  return new Promise((resolve) => {
    execFile(padmaCommand(), ['check', '--json', document.fileName], { maxBuffer: 1024 * 1024 }, (error, stdout, stderr) => {
      let result;
      try {
        result = JSON.parse(stdout);
      } catch {
        const detail = stderr || error?.message || 'Padma did not return a JSON diagnostic report.';
        vscode.window.showErrorMessage(`Padma check failed: ${detail}`);
        resolve();
        return;
      }

      const diagnostics = (result.diagnostics ?? []).map((item) => {
        const diagnostic = new vscode.Diagnostic(
          diagnosticRange(item.range),
          `[${item.code}] ${item.message}`,
          vscode.DiagnosticSeverity.Error,
        );
        diagnostic.source = 'Padma';
        diagnostic.code = item.code;
        if (item.hint) {
          diagnostic.relatedInformation = [
            new vscode.DiagnosticRelatedInformation(
              new vscode.Location(document.uri, diagnostic.range),
              item.hint,
            ),
          ];
        }
        return diagnostic;
      });

      collection.set(document.uri, diagnostics);
      if (diagnostics.length === 0) {
        vscode.window.showInformationMessage('Padma check found no diagnostics.');
      }
      resolve();
    });
  });
}

async function startLanguageServer(context) {
  if (languageClient) {
    vscode.window.showInformationMessage('Padma language server is already running.');
    return;
  }
  const command = languageServerCommand();
  const serverOptions = {
    run: { command, transport: TransportKind.stdio },
    debug: { command, transport: TransportKind.stdio },
  };
  const clientOptions = {
    documentSelector: [{ scheme: 'file', language: 'padma' }],
  };
  languageClient = new LanguageClient('padmaLanguageServer', 'Padma Language Server', serverOptions, clientOptions);
  context.subscriptions.push(languageClient.start());
  vscode.window.showInformationMessage('Starting Padma language server.');
}

async function stopLanguageServer() {
  if (!languageClient) {
    vscode.window.showInformationMessage('Padma language server is not running.');
    return;
  }
  const client = languageClient;
  languageClient = undefined;
  await client.stop();
  vscode.window.showInformationMessage('Padma language server stopped.');
}

function activate(context) {
  const diagnostics = vscode.languages.createDiagnosticCollection('padma');
  context.subscriptions.push(diagnostics);

  context.subscriptions.push(vscode.commands.registerCommand('padma.runFile', () => {
    const document = activePadmaDocument();
    if (document) runInTerminal([document.fileName], 'Padma: Run');
  }));

  context.subscriptions.push(vscode.commands.registerCommand('padma.checkFile', async () => {
    const document = activePadmaDocument();
    if (document) await checkDocument(document, diagnostics);
  }));

  context.subscriptions.push(vscode.commands.registerCommand('padma.formatFile', () => {
    const document = activePadmaDocument();
    if (document) runInTerminal(['fmt', document.fileName], 'Padma: Format');
  }));

  context.subscriptions.push(vscode.commands.registerCommand('padma.lintFile', () => {
    const document = activePadmaDocument();
    if (document) runInTerminal(['lint', document.fileName], 'Padma: Lint');
  }));

  context.subscriptions.push(vscode.commands.registerCommand('padma.startLanguageServer', () => startLanguageServer(context)));
  context.subscriptions.push(vscode.commands.registerCommand('padma.stopLanguageServer', stopLanguageServer));
}

function deactivate() {
  return languageClient?.stop();
}

module.exports = { activate, deactivate };
