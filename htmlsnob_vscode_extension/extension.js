const os = require('os');
const path = require('path');const vscode = require('vscode');
const clientNode = require("vscode-languageclient/node");
const path = require('path');

function getBinaryPath(context) {
    const platform = os.platform();
    let binaryName = '';

    switch (platform) {
        case 'win32':
            binaryName = 'htmlsnob_lsp-windows-x86_64.exe';
            break;
        case 'darwin':
            binaryName = 'htmlsnob_lsp-macos-x86_64';
            break;
        case 'linux':
            binaryName = 'htmlsnob_lsp-linux';
            break;
        default:
            throw new Error(`Unsupported platform: ${platform}`);
    }

    return path.join(context.extensionPath, 'bin', binaryName);
}

function activate(context) {
  const serverPath = getBinaryPath(context);

  const serverOptions = {
    run: { command: serverPath },
    debug: { command: serverPath }
  };

  const clientOptions = {
    documentSelector: [
        { scheme: 'file', language: 'html' },
        { scheme: 'file', language: 'handlebars' },
        { scheme: 'file', language: 'blade' },
        { scheme: 'file', language: 'ejs' },
        { scheme: 'file', language: 'eex' },
        { scheme: 'file', language: 'erb' },
        { scheme: 'file', language: 'go' },
        { scheme: 'file', language: 'jinja2' },
        { scheme: 'file', language: 'liquid' },
        { scheme: 'file', language: 'mustache' },
        { scheme: 'file', language: 'twig' },
    ],
  };

  // Create and start the client
  client = new clientNode.LanguageClient('htmlsnobLanguageServer', 'HTMLSnob Language Server', serverOptions, clientOptions);
  client.start();
}

function deactivate() {}

module.exports = {
  activate,
  deactivate
};
