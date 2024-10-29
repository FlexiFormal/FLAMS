import { LogLevel } from 'vscode/services';
import getKeybindingsServiceOverride from '@codingame/monaco-vscode-keybindings-service-override';
import { MonacoEditorLanguageClientWrapper, WrapperConfig } from 'monaco-editor-wrapper';
import { configureMonacoWorkers } from './utils/utils';
import { IWebSocket, toSocket, WebSocketMessageReader, WebSocketMessageWriter } from 'vscode-ws-jsonrpc';
import * as vscode from 'vscode';
import { DataCallback, Disposable, Message, MessageWriter } from 'vscode-jsonrpc';

const text = `\\documentclass{article}
\\usepackage{stex}
\\usemodule[sTeX/Algebra/General]{mod?Group}
\\begin{document}
\\vardef{vG}[name=G]{G}

Let $\\vG$ a \\sn{group}
\\end{document}
`;

const text2 = `\\documentclass{article}
\\usepackage{stex}
\\usemodule[sTeX/Algebra/General]{mod?Group}
\\begin{document}
\\vardef{vG}[name=G]{G}

Let $\\vG$ a \\sn{group}
\\end{document}
`;

class NoContentLengthReader extends WebSocketMessageReader {
  constructor(socket: IWebSocket) {
    super(socket);
  }
  readMessage(message: any): void {
    console.log("Read: ",message);
    if (message.toString().startsWith('Content-Length:')) {
        this.state = 'listening';
        return;
    }
    if (this.state === 'initial') {
        this.events.splice(0, 0, { message });
    } else if (this.state === 'listening') {
        try {
            const data = JSON.parse(message);
            this.callback!(data);
        } catch (err) {
            const error: Error = {
                name: '' + 400,
                // eslint-disable-next-line @typescript-eslint/no-explicit-any
                message: `Error during message parsing, reason = ${typeof err === 'object' ? (err as any).message : 'unknown'}`
            };
            this.fireError(error);
        }
    }
  }
}


class ContentLengthWriter extends WebSocketMessageWriter {
  constructor(socket: IWebSocket) {
    super(socket);
  }
  async write(msg: Message): Promise<void> {
    try {
        const content = JSON.stringify(msg);
        console.log("Write: ",content);
        const len = content.length;
        this.socket.send(`Content-Length: ${len}\r\n\r\n${content}`);
    } catch (e) {
        this.errorCount++;
        this.fireError(e, msg, this.errorCount);
    }
  }
}

export function mountEditor(element:HTMLElement): WrapperConfig {
  const socket = new WebSocket("http://localhost:3000/ws/lsp");
  const iWebSocket = toSocket(socket);
  const reader = new NoContentLengthReader(iWebSocket);
  const writer = new ContentLengthWriter(iWebSocket);
  return {
    logLevel: LogLevel.Debug,
    vscodeApiConfig: {
        userServices: {
            ...getKeybindingsServiceOverride(),
        },
        userConfiguration: {
            json: JSON.stringify({
                'workbench.colorTheme': 'Default Dark Modern',
                'editor.guides.bracketPairsHorizontal': 'active',
                'editor.lightbulb.enabled': 'On',
                'editor.wordBasedSuggestions': 'off',
                'editor.experimental.asyncTokenization': true
            })
        }
    },
    editorAppConfig: {
        $type: 'extended',
        codeResources: {
            main: {
                text,
                fileExt: 'tex',
                enforceLanguageId: "latex"
            }
        },
        useDiffEditor: false,
        monacoWorkerFactory: configureMonacoWorkers,
        htmlContainer: element
    },
    languageClientConfigs: {
        latex: {
            languageId: 'latex',
            connection: {
                options: {
                    $type: 'WebSocketDirect',
                    webSocket: socket,
                    startOptions: {
                        onCall: () => {
                            console.log('Connected to socket.');
                        },
                        reportStatus: true
                    },
                    stopOptions: {
                        onCall: () => {
                            console.log('Disconnected from socket.');
                        },
                        reportStatus: true
                    }
                },
                messageTransports: { reader:reader, writer:writer }
            },
            clientOptions: {
                documentSelector: ['latex'],
                workspaceFolder: {
                    index: 0,
                    name: 'workspace',
                    uri: vscode.Uri.parse("/workspace")
                },
            }
        }
    }
};
}

async function test() {
  const wrapper = new MonacoEditorLanguageClientWrapper();

  const config = mountEditor(document.getElementById('monaco-editor-root')!);
  await wrapper.initAndStart(config);
  // wait wrapper.dispose();
};


test();