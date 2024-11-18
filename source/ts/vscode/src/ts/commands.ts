import * as vscode from 'vscode';
import { IMMTContext, IMMTPreContext } from '../extension';
import { CancellationToken } from 'vscode-languageclient';
import * as language from 'vscode-languageclient';
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

interface HtmlRequestParams {
  uri: language.URI
}

export function register_server_commands(context:IMMTContext) {
  vscode.commands.executeCommand('setContext', 'immt.loaded', true);
	vscode.window.registerWebviewViewProvider("immt-tools",
    webview(context,"stex-tools",msg => {
      const doc = vscode.window.activeTextEditor?.document;
      switch (msg.command) {
        case "dashboard":
          openIframe(context.server.url + "/dashboard","Dashboard");
          break;
        case "preview":
          if (doc) {
            context.client.sendRequest<string | undefined>("immt/htmlRequest",<HtmlRequestParams>{uri:doc.uri.toString()}).then(s => {
              if (s) {openIframe(context.server.url + "?uri=" + encodeURIComponent(s),s); }
              else {
                vscode.window.showInformationMessage("No preview available; building possibly failed");
              }
            });
          }
          break;
        case "browser":
          if (doc) {
            context.client.sendRequest<string | undefined>("immt/htmlRequest",<HtmlRequestParams>{uri:doc.uri.toString()}).then(s => {
              if (s) {
                const uri = vscode.Uri.parse(context.server.url).with({query:"uri=" + encodeURIComponent(s)});
                vscode.env.openExternal(uri);
              }
              else {
                vscode.window.showInformationMessage("No preview available; building possibly failed");
              }
            });
          }
          break;
      }
    })
  );
	vscode.window.registerTreeDataProvider("immt-mathhub",new MathHubTreeProvider(context));
  context.client.onNotification("immt/htmlResult",(s:string) => {
    openIframe(context.server.url + "?uri=" + encodeURIComponent(s),s);
	});
}

export function openIframe(url:string,title:string): vscode.WebviewPanel {
  const panel = vscode.window.createWebviewPanel('webviewPanel',title,vscode.ViewColumn.Beside,{
    enableScripts: true,
    enableForms:true     
  });
  panel.webview.html =  `
  <!DOCTYPE html>
  <html>
    <head></head>
    <body style="padding:0;width:100vw;height:100vh;overflow:hidden;">
      <iframe style="width:100vw;height:100vh;overflow:hidden;" src="${url}" title="${title}" style="background:white"></iframe>
    </body>
  </html>`;
  return panel;
}

export function webview(immtcontext:IMMTContext,html_file:string,onMessage?: (e:any) => any) : vscode.WebviewViewProvider {
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