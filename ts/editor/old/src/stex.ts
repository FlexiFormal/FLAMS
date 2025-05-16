import { /*initServices,*/ MonacoLanguageClient } from 'monaco-languageclient';
import { ProtocolNotificationType, ProtocolRequestType0 ,CloseAction, ErrorAction, MessageTransports } from 'vscode-languageclient';
import { WebSocketMessageReader, WebSocketMessageWriter, toSocket } from 'vscode-ws-jsonrpc';

console.log("Here: stex.ts")

export const createLanguageClient = (transports: MessageTransports): MonacoLanguageClient => {
  return new MonacoLanguageClient({
      name: 'sTeX Language Client',
      clientOptions: {
          // use a language id as a document selector
          documentSelector: ['latex','tex'],
          // disable the default error handler
          errorHandler: {
              error: () => ({ action: ErrorAction.Continue }),
              closed: () => ({ action: CloseAction.DoNotRestart })
          },
          synchronize: {}
      },
      // create a language client connection from the sTeX RPC connection on demand
      connectionProvider: {
          get: () => {
              return Promise.resolve(transports);
          }
      }
  });
};


export const createUrl = (hostname: string, port: number, path: string, searchParams: Record<string, any> = {}, secure: boolean = location.protocol === 'https:'): string => {
  const protocol = secure ? 'wss' : 'ws';
  const url = new URL(`${protocol}://${hostname}:${port}${path}`);

  for (let [key, value] of Object.entries(searchParams)) {
      if (value instanceof Array) {
          value = value.join(',');
      }
      if (value) {
          url.searchParams.set(key, value);
      }
  }

  return url.toString();
};

interface MathHubMessage {
  mathhub:string,
  remote:string
}
export interface Repository {
  id: string,
  isLocal:boolean,
  localPath:string
}
export interface RepoGroup {
  id: string,
  isLocal:boolean,
  children: MHEntry[]
}
export type MHEntry = Repository | RepoGroup;

export const createWebSocketAndStartClient = async (url: string): Promise<WebSocket> => {
  const webSocket = new WebSocket(url);
  webSocket.onopen = () => {
      const socket = toSocket(webSocket);
      const reader = new WebSocketMessageReader(socket);
      const writer = new WebSocketMessageWriter(socket);
      const languageClient = createLanguageClient({
          reader,
          writer
      });
      languageClient.start();
      reader.onClose(() => languageClient.stop());


	  languageClient.sendNotification(new ProtocolNotificationType<MathHubMessage,void>("sTeX/setMathHub"),{
		  mathhub:"/home/jazzpirate/work/MathHub",
		  remote:"https://stexmmt.mathhub.info/:sTeX"
	  }).then(()=> {
      languageClient.sendRequest(new ProtocolRequestType0<MHEntry[], any, any, any>("sTeX/getMathHubContent"))
    });

  };
  return webSocket;
};


import { ExtensionHostKind, IExtensionManifest, registerExtension } from 'vscode/extensions'
let manifest = {
	"name": "stexide",
	"displayName": "sTeX",
	"publisher": "kwarc",
	"version": "1.1.3",
  "engines": {
    "vscode": "*"
  },
  "activationEvents": ["*"],
  "editor.semanticHighlighting.enabled": true,
	"contributes": {
		"semanticTokenTypes": [
			{
				"id": "stex-module",
				"superType": "namespace",
				"description": "An sTeX Module"
			},
			{
				"id": "stex-symdecl",
				"superType": "keyword",
				"description": "An sTeX symbol or notation declaration"
			},
			{
				"id": "stex-constant",
				"superType": "string",
				"description": "An sTeX Symbol"
			},
			{
				"id": "stex-variable",
				"superType": "variable",
				"description": "An sTeX Variable"
			},
			{
				"id": "stex-file",
				"superType": "file",
				"description": "An sTeX File reference"
			}
		],
		"semanticTokenModifiers": [
			{
				"id": "stex-deprecatedmodule",
				"description": "This module is deprecated"
			}
		],
		"configurationDefaults": {
				"editor.semanticTokenColorCustomizations": {
						"rules": {
							"stex-module":{
								"fontStyle": "italic bold underline"
							},
							"stex-constant":{
								"fontStyle": "underline"
							},
							"stex-symdecl":{
								"fontStyle": "italic bold underline"
							},
							"stex-variable":{
								"foreground":"#858282",
								"fontStyle": "italic underline"
							},
							"stex-file":{
								"fontStyle": "italic underline"
							}
						}
				}
		}
  }
}
await registerExtension(manifest as IExtensionManifest, ExtensionHostKind.LocalWebWorker)

const url = createUrl('localhost', 5008, '');
createWebSocketAndStartClient(url);