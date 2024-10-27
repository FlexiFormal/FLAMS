import * as vscode from 'vscode';
//import { activate as act, greet } from '../pkg/immt_vscode';
import { register_commands, register_server_commands, Settings } from './ts/commands';
import { setup } from './ts/setup';
import { Versions } from './ts/versions';
import * as language from 'vscode-languageclient/node';
import { IMMTServer } from './ts/immt/server';

export async function activate(context: vscode.ExtensionContext) {
	//console.log('Congratulations, your extension "immt" is now active!');
	//act(context);
	const ctx = new IMMTContext(context);
	register_commands();
	if (await ctx.versions.isValid()) {
		launch(ctx);
	} else {
		setup(ctx);
	}
}

export function deactivate() {
  const context = get_context();
	if (context?.client) {
		context.client.stop();
	}
}

export async function launch(context:IMMTContext) {
  let versions = context.versions;
  let immt = await versions?.immtversion();
  let stex = await versions?.stexversion();
  let immt_path = versions?.immt_path;
  let stex_path = versions?.stex_path;
  if (!(stex && immt && immt_path && stex_path)) {
    vscode.window.showErrorMessage(`iMMT: Error initializing`, { modal: true });
    return;
  }
	context.outputChannel.appendLine(`Using ${immt_path} (version ${immt.toString()})`);
	context.outputChannel.appendLine(`sTeX at ${stex_path} (version ${stex.toString()})`);
	context.outputChannel.appendLine("Initializing iMMT LSP Server");
  
  const toml = vscode.workspace.getConfiguration("immt").get<string>(Settings.SettingsToml);
  const args = toml ? ["--lsp","-c",toml] : ["--lsp"];
  const serverOptions: language.ServerOptions = {
		run: { command: immt_path, args: args },
		debug: { command: immt_path, args: args }
	};
  context.client = new language.LanguageClient("immt-server","iMMT Language Server",serverOptions,{
		documentSelector: [{scheme:"file", language:"tex"},{scheme:"file", language:"latex"}],
		synchronize: {}
	});
	context.client.onNotification("immt/serverURL",(s:string) => {
		context.server = new IMMTServer(s);
    register_server_commands();
	});
  context.client.start();


}


let _context : IMMTContext | undefined = undefined;
export function get_context() : IMMTContext | undefined {
  return _context;
}


export class IMMTContext {
  outputChannel: vscode.OutputChannel;
  vsc: vscode.ExtensionContext;
  versions: Versions = new Versions();
  client: language.LanguageClient | undefined;
  server: IMMTServer | undefined;

  constructor(context: vscode.ExtensionContext) {
		this.vsc = context;
    this.outputChannel = vscode.window.createOutputChannel('iMMT');
    _context = this;
	}

  register_command(name:string,callback: (...args: any[]) => any, thisArg?: any) {
    const disposable = vscode.commands.registerCommand(name, callback, thisArg);
    this.vsc.subscriptions.push(disposable);
  }
}