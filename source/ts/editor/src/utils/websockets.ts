import { ConnectionConfig } from "monaco-editor-wrapper";
import { IWebSocket, toSocket, WebSocketMessageReader, WebSocketMessageWriter } from 'vscode-ws-jsonrpc';
import { Message } from 'vscode-jsonrpc';

export function getWS(url:string):ConnectionConfig {
  const socket = new WebSocket(url);
  const iWebSocket = toSocket(socket);
  const reader = new NoContentLengthReader(iWebSocket);
  const writer = new ContentLengthWriter(iWebSocket);
  return {
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
}
}


export class NoContentLengthReader extends WebSocketMessageReader {
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


export class ContentLengthWriter extends WebSocketMessageWriter {
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