import * as vscode from 'vscode';
import { get_context, IMMTContext } from '../extension';
import { CancellationToken } from 'vscode-languageclient';
import { MathHubTreeProvider } from './mathhub';
import { IMMTServer } from './immt/server';

export enum Commands {
  HelloWorld = "immt.helloWorld",
  ImmtMissing = "immt.immt_missing",
}

export enum Settings {
  PreviewOn = "preview",
  SettingsToml = "settings_toml",
  ImmtPath = "immt_path"
}

export function register_commands() {
  let context = get_context();
  if (!context) { throw new Error("context is undefined"); }

  //import { greet } from '../../pkg/immt_vscode';
  /*context.register_command(Commands.HelloWorld, () => {
    vscode.window.showInformationMessage(greet("Dude"));
  });*/
}

export function register_server_commands() {
  let context = get_context();
  if (!context) { throw new Error("context is undefined"); }
  if (!context.server || !context.client) {
    vscode.window.showErrorMessage("iMMT: Server/Client not running");
    return;
  }

	vscode.window.registerWebviewViewProvider("immt-tools",
    webview("stex-tools")
  );
	vscode.window.registerTreeDataProvider("immt-mathhub",new MathHubTreeProvider());
}

export function webview(html_file:string,onMessage?: vscode.Event<any>) : vscode.WebviewViewProvider {
  let immtcontext = get_context();
  if (!immtcontext) { throw new Error("context is undefined"); }
  if (!immtcontext.server || !immtcontext.client) {
    throw new Error("iMMT: Server/Client not running");
  }
  return <vscode.WebviewViewProvider> {
    resolveWebviewView(webviewView: vscode.WebviewView, context: vscode.WebviewViewResolveContext, token: CancellationToken): Thenable<void> | void {
      webviewView.webview.options = {
        enableScripts: true,
        enableForms:true     
      };
      const tkuri = webviewView.webview.asWebviewUri(vscode.Uri.joinPath(
        immtcontext.vsc.extensionUri,
          "resources","toolkit.min.js"
      ));
      const cssuri = webviewView.webview.asWebviewUri(vscode.Uri.joinPath(
        immtcontext.vsc.extensionUri,
          "resources","codicon.css"
      ));
      if (onMessage) {
        webviewView.webview.onDidReceiveMessage(onMessage);
      }
      const file = vscode.Uri.joinPath(immtcontext.vsc.extensionUri,"resources",html_file + ".html");
      vscode.workspace.fs.readFile(file).then((c) => {
        webviewView.webview.html = Buffer.from(c).toString().replace("%%HEAD%%",
          `<link href="${cssuri}" rel="stylesheet"/>
          <script type="module" src="${tkuri}"></script>
          <script>const vscode = acquireVsCodeApi();</script>
          `);
      });
    }
  };
}