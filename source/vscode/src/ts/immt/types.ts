
/*
export interface Settings {
  mathhubs: string[],
  debug:boolean,
  server: {
    port:number,
    ip:string
    database?:string
  },
  log_dir:string,
  buildqueue: {
    num_threads:number
  }
}
*/

export interface BuildState {
  new:number,
  stale:number,
  deleted:number,
  up_to_date:number,
  last_built:number,
  last_changed:number
}

export interface ArchiveGroup {
  id: string,
  summary?: BuildState
}

export interface Archive {
  id: string,
  summary?: BuildState
}

export interface Directory {
  rel_path: string,
  summary?: BuildState
}

export interface File {
  rel_path: string,
  format: string
}

export interface Inputref {
  uri: string,
  id: string,
  children?: TOCElem[]
}

export interface Section {
  title?: string,
  uri: string,
  id: string,
  children?: TOCElem[]
}

export type TOCElem = {Inputref:Inputref} | {Section:Section};

export type LoginState = "Loading" | "Admin" | {User:string} | "None" | "NoAccounts";

export enum Language {
  English = "en",
  German = "de",
  French = "fr",
  Romanian = "ro",
  Arabic = "ar",
  Bulgarian = "bg",
  Russian = "ru",
  Finnish = "fi",
  Turkish = "tr",
  Slovenian = "sl",
}


export class DocumentURI {
  uri:string;
  constructor(uri:string) {
    this.uri = uri;
  }
}

export type CSS = { Link: string } | { Inline: string };

export type DocumentURIParams = DocumentURI | 
  { a: string, rp: string } | 
  { a:string, p?:string, d:string, l?:Language }
;


export type URIParams = DocumentURI | 
  { a:string} | // ArchiveURI
  { a: string, rp: string } | // DocumentURI 
  { a:string, p?:string, d:string, l?:Language } | // DocumentURI
  { a:string, p?:string, d:string, l?:Language, e:string } | // DocumentElementURI
  { a:string, p?:string, m:string, l?:Language } | // ModuleURI
  { a:string, p?:string, m:string, l?:Language, s:string } // SymbolURI
;
