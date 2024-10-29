import * as vscode from 'vscode';
//import { activate as act, greet } from '../pkg/immt_vscode';
import { register_commands, register_server_commands, Settings } from './ts/commands';
import { setup } from './ts/setup';
import { Versions } from './ts/versions';
import * as language from 'vscode-languageclient/node';
import { IMMTServer } from './ts/immt/server';
//import * as ws from 'ws';

export async function activate(context: vscode.ExtensionContext) {
  await local(context);
  //await remote(context);
}

async function local(context: vscode.ExtensionContext) {
	const ctx = new IMMTPreContext(context);
	register_commands(ctx);
	if (await ctx.versions.isValid()) {
		launch_local(ctx);
	} else {
		setup(ctx);
	}
}


export function deactivate() {
  const context = get_pre_context();
	if (context?.client) {
		context.client.stop();
	}
}

export async function launch_local(context:IMMTPreContext) {
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
    const ctx = new IMMTContext(context);
    register_server_commands(ctx);
	});
  context.client.start();
}

/*
async function remote(context: vscode.ExtensionContext) {
	const ctx = new IMMTPreContext(context);
	register_commands(ctx);
  launch_remote(ctx);
}

export async function launch_remote(context: IMMTPreContext) {
  // Initialize server first
  context.server = new IMMTServer("http://localhost:3000");
  
  const wsock = new ws.WebSocket("http://localhost:3000/ws/lsp");
  const connection = ws.WebSocket.createWebSocketStream(wsock);

  connection.on("data",(chunk) => console.log(new TextDecoder().decode(chunk)));

  context.client = new language.LanguageClient(
    "immt-server",
    "iMMT Language Server",
    () => Promise.resolve({
      reader: connection,
      writer: connection,
    }),
    {
        documentSelector: [
            {scheme: "file", language: "tex"},
            {scheme: "file", language: "latex"}
        ],
        synchronize: {}
    }
  );
  await context.client.start().then(() => {
    const ctx = new IMMTContext(context);
    ctx.remote_server = undefined;
    register_server_commands(ctx);
  });
}
*/

let _context : IMMTContext | IMMTPreContext | undefined = undefined;
export function get_pre_context() : IMMTContext | IMMTPreContext | undefined {
  return _context;
}

export function get_context() : IMMTContext {
  if (!(_context instanceof IMMTContext)) { throw new Error("context is undefined"); }
  return _context;
}


export class IMMTPreContext {
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

export class IMMTContext {
  outputChannel: vscode.OutputChannel;
  vsc: vscode.ExtensionContext;
  versions: Versions;
  client: language.LanguageClient;
  server: IMMTServer;
  remote_server: IMMTServer | undefined;

  constructor(ctx:IMMTPreContext) {
    if (!ctx.client || !ctx.server) { throw new Error("iMMT: Client/Server not initialized"); }
		this.vsc = ctx.vsc;
    this.outputChannel = ctx.outputChannel;
    this.versions = ctx.versions;
    this.client = ctx.client;
    this.server = ctx.server;
    this.remote_server = new IMMTServer("https://mmt.beta.vollki.kwarc.info");
    _context = this;
	}

  register_command(name:string,callback: (...args: any[]) => any, thisArg?: any) {
    const disposable = vscode.commands.registerCommand(name, callback, thisArg);
    this.vsc.subscriptions.push(disposable);
  }
}