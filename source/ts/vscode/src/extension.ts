import * as vscode from 'vscode';
//import { activate as act, greet } from '../pkg/flams_vscode';
import { register_commands, register_server_commands, Settings } from './ts/commands';
import { setup } from './ts/setup';
import { Versions } from './ts/versions';
import * as language from 'vscode-languageclient/node';
import { FLAMSServer } from './ts/flams/server';
//import * as ws from 'ws';

export async function activate(context: vscode.ExtensionContext) {
  await local(context);
  //await remote(context);
}

async function local(context: vscode.ExtensionContext) {
	const ctx = new FLAMSPreContext(context);
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

export async function launch_local(context:FLAMSPreContext) {
  let versions = context.versions;
  let flams = await versions?.flamsVersion();
  let stex = await versions?.stexVersion();
  let flams_path = versions?.flams_path;
  let stex_path = versions?.stex_path;
  if (!(stex && flams && flams_path && stex_path)) {
    vscode.window.showErrorMessage(`𝖥𝖫∀𝖬∫: Error initializing`, { modal: true });
    return;
  }
	context.outputChannel.appendLine(`Using ${flams_path} (version ${flams.toString()})`);
	context.outputChannel.appendLine(`sTeX at ${stex_path} (version ${stex.toString()})`);
	context.outputChannel.appendLine("Initializing 𝖥𝖫∀𝖬∫ LSP Server");
  
  const toml = vscode.workspace.getConfiguration("flams").get<string>(Settings.SettingsToml);
  const args = toml ? ["--lsp","-c",toml] : ["--lsp"];
  const serverOptions: language.ServerOptions = {
		run: { command: flams_path, args: args },
		debug: { command: flams_path, args: args }
	};
  context.client = new language.LanguageClient("flams-server","𝖥𝖫∀𝖬∫ Language Server",serverOptions,{
		documentSelector: [{scheme:"file", language:"tex"},{scheme:"file", language:"latex"}],
		synchronize: {},
    //outputChannel: context.outputChannel,
    //traceOutputChannel: context.outputChannel,
    markdown: {
        isTrusted: true,
        supportHtml: true
    }
    /*trace: {
        server: {
            verbosity: language.Trace.Verbose,
            format: language.TraceFormat.Text
        }
    }*/
	});
	context.client.onNotification("flams/serverURL",(s:string) => {
		context.server = new FLAMSServer(s);
    const ctx = new FLAMSContext(context);
    register_server_commands(ctx);
	});
  context.client.start();
}

/*
async function remote(context: vscode.ExtensionContext) {
	const ctx = new FLAMSPreContext(context);
	register_commands(ctx);
  launch_remote(ctx);
}

export async function launch_remote(context: FLAMSPreContext) {
  // Initialize server first
  context.server = new FLAMSServer("http://localhost:3000");
  
  const wsock = new ws.WebSocket("http://localhost:3000/ws/lsp");
  const connection = ws.WebSocket.createWebSocketStream(wsock);

  connection.on("data",(chunk) => console.log(new TextDecoder().decode(chunk)));

  context.client = new language.LanguageClient(
    "flams-server",
    "𝖥𝖫∀𝖬∫ Language Server",
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
    const ctx = new FLAMSContext(context);
    ctx.remote_server = undefined;
    register_server_commands(ctx);
  });
}
*/

let _context : FLAMSContext | FLAMSPreContext | undefined = undefined;
export function get_pre_context() : FLAMSContext | FLAMSPreContext | undefined {
  return _context;
}

export function get_context() : FLAMSContext {
  if (!(_context instanceof FLAMSContext)) { throw new Error("context is undefined"); }
  return _context;
}


export class FLAMSPreContext {
  outputChannel: vscode.OutputChannel;
  vsc: vscode.ExtensionContext;
  versions: Versions = new Versions();
  client: language.LanguageClient | undefined;
  server: FLAMSServer | undefined;

  constructor(context: vscode.ExtensionContext) {
		this.vsc = context;
    this.outputChannel = vscode.window.createOutputChannel('FLAMS');
    _context = this;
	}

  register_command(name:string,callback: (...args: any[]) => any, thisArg?: any) {
    const disposable = vscode.commands.registerCommand(name, callback, thisArg);
    this.vsc.subscriptions.push(disposable);
  }
}

export class FLAMSContext {
  outputChannel: vscode.OutputChannel;
  vsc: vscode.ExtensionContext;
  versions: Versions;
  client: language.LanguageClient;
  server: FLAMSServer;
  remote_server: FLAMSServer | undefined;

  constructor(ctx:FLAMSPreContext) {
    if (!ctx.client || !ctx.server) { throw new Error("𝖥𝖫∀𝖬∫: Client/Server not initialized"); }
		this.vsc = ctx.vsc;
    this.outputChannel = ctx.outputChannel;
    this.versions = ctx.versions;
    this.client = ctx.client;
    this.server = ctx.server;
    this.remote_server = new FLAMSServer("https://mmt.beta.vollki.kwarc.info");
    _context = this;
	}

  register_command(name:string,callback: (...args: any[]) => any, thisArg?: any) {
    const disposable = vscode.commands.registerCommand(name, callback, thisArg);
    this.vsc.subscriptions.push(disposable);
  }
}