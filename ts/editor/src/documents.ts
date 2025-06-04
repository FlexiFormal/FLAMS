import * as vscode from 'vscode';

export class MonacoDocument {
  _text:string="";
  _uri:vscode.Uri;
  _language:string;
  set text(value:string) {
    this._text = value;
  }
  get text():string {
    return this._text;
  }
  get uri():vscode.Uri {
    return this._uri;
  }
  get language():string {
    return this._language;
  }

  constructor(uri:string,language:string) {
    this._uri = vscode.Uri.file(uri);
    this._language = language;
  }
}