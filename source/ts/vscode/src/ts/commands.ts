import * as vscode from 'vscode';
import { IMMTContext, IMMTPreContext } from '../extension';
import { CancellationToken } from 'vscode-languageclient';
import { MathHubTreeProvider } from './mathhub';

export enum Commands {
  HelloWorld = "immt.helloWorld",
  ImmtMissing = "immt.immt_missing",
}

export enum Settings {
  PreviewOn = "preview",
  SettingsToml = "settings_toml",
  ImmtPath = "immt_path"
}

export function register_commands(context:IMMTPreContext) {
  //import { greet } from '../../pkg/immt_vscode';
  /*context.register_command(Commands.HelloWorld, () => {
    vscode.window.showInformationMessage(greet("Dude"));
  });*/
}

export function register_server_commands(context:IMMTContext) {
  vscode.commands.executeCommand('setContext', 'immt.loaded', true);
	vscode.window.registerWebviewViewProvider("immt-tools",
    webview(context,"stex-tools")
  );
	vscode.window.registerTreeDataProvider("immt-mathhub",new MathHubTreeProvider(context));
}

export function webview(immtcontext:IMMTContext,html_file:string,onMessage?: vscode.Event<any>) : vscode.WebviewViewProvider {
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