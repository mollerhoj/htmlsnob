const vscode = require('vscode');
const clientNode = require("vscode-languageclient/node");
const path = require('path');


function activate(context) {
  const serverPath = context.asAbsolutePath(path.join('..', 'target', 'debug', 'htmlsnob_lsp'));

  

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
