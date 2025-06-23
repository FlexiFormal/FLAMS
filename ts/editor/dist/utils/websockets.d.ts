import { ConnectionConfig } from "monaco-editor-wrapper";
import { IWebSocket, WebSocketMessageReader, WebSocketMessageWriter } from 'vscode-ws-jsonrpc';
import { Message } from 'vscode-jsonrpc';
export declare function getWS(url: string): ConnectionConfig;
export declare class NoContentLengthReader extends WebSocketMessageReader {
    constructor(socket: IWebSocket);
    readMessage(message: any): void;
}
export declare class ContentLengthWriter extends WebSocketMessageWriter {
    constructor(socket: IWebSocket);
    write(msg: Message): Promise<void>;
}
