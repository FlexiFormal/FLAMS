import * as vscode from 'vscode';
export declare class MonacoDocument {
    _text: string;
    _uri: vscode.Uri;
    _language: string;
    set text(value: string);
    get text(): string;
    get uri(): vscode.Uri;
    get language(): string;
    constructor(uri: string, language: string);
}
