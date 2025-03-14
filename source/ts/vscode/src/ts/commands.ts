import * as vscode from 'vscode';
import { FLAMSContext, FLAMSPreContext } from '../extension';
import { CancellationToken } from 'vscode-languageclient';
import * as language from 'vscode-languageclient';
import { MathHubTreeProvider } from './mathhub';

export enum Commands {
  HelloWorld = "flams.helloWorld",
  FlamsMissing = "flams.flams_missing",
}

export enum Settings {
  PreviewOn = "preview",
  SettingsToml = "settings_toml",
  FlamsPath = "flams_path"
}

export function register_commands(context:FLAMSPreContext) {
  //import { greet } from '../../pkg/flams_vscode';
  /*context.register_command(Commands.HelloWorld, () => {
    vscode.window.showInformationMessage(greet("Dude"));
  });*/
}

interface HtmlRequestParams {
  uri: language.URI
}

interface ReloadParams {}

export function register_server_commands(context:FLAMSContext) {
  vscode.commands.executeCommand('setContext', 'flams.loaded', true);
	vscode.window.registerWebviewViewProvider("flams-tools",
    webview(context,"stex-tools",msg => {
      const doc = vscode.window.activeTextEditor?.document;
      switch (msg.command) {
        case "dashboard":
          openIframe(context.server.url + "/dashboard","Dashboard");
          break;
        case "preview":
          if (doc) {
            context.client.sendRequest<string | undefined>("flams/htmlRequest",<HtmlRequestParams>{uri:doc.uri.toString()}).then(s => {
              if (s) {openIframe(context.server.url + "?uri=" + encodeURIComponent(s),doc.fileName); }
              else {
                vscode.window.showInformationMessage("No preview available; building possibly failed");
              }
            });
          }
          break;
        case "reload":
          context.client.sendNotification("flams/reload",<ReloadParams>{});
          break;
        case "browser":
          if (doc) {
            context.client.sendRequest<string | undefined>("flams/htmlRequest",<HtmlRequestParams>{uri:doc.uri.toString()}).then(s => {
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

  const mathhub = new MathHubTreeProvider(context);
	vscode.window.registerTreeDataProvider("flams-mathhub",mathhub);
  
	context.vsc.subscriptions.push(vscode.commands.registerCommand("flams.mathhub.install", mathhub.install));

  context.client.onNotification("flams/htmlResult",(s:string) => {
    openIframe(context.server.url + "?uri=" + encodeURIComponent(s),s.split("&d=")[1]);
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

export function webview(flamscontext:FLAMSContext,html_file:string,onMessage?: (e:any) => any) : vscode.WebviewViewProvider {
  return <vscode.WebviewViewProvider> {
    resolveWebviewView(webviewView: vscode.WebviewView, context: vscode.WebviewViewResolveContext, token: CancellationToken): Thenable<void> | void {
      webviewView.webview.options = {
        enableScripts: true,
        enableForms:true     
      };
      const tkuri = webviewView.webview.asWebviewUri(vscode.Uri.joinPath(
        flamscontext.vsc.extensionUri,
          "resources","bundled.js"
      ));
      const cssuri = webviewView.webview.asWebviewUri(vscode.Uri.joinPath(
        flamscontext.vsc.extensionUri,
          "resources","codicon.css"
      ));
      if (onMessage) {
        webviewView.webview.onDidReceiveMessage(onMessage);
      }
      const file = vscode.Uri.joinPath(flamscontext.vsc.extensionUri,"resources",html_file + ".html");
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