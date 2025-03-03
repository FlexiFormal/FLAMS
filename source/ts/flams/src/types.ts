import * as FTML from "./ftml-viewer-base";

export type DocumentURI = FTML.DocumentURI;
export type SymbolURI = string;
export type DocumentElementURI = FTML.DocumentElementURI;
export type Name = FTML.Name;

export type ExerciseResponse = FTML.ExerciseResponse;
export type ExerciseResponseType = FTML.ExerciseResponseType;
export type ExerciseFeedback = FTML.ExerciseFeedback;
export type ExerciseSolutions = FTML.Solutions;

export type ParagraphKind = FTML.ParagraphKind;
export type SectionLevel = FTML.SectionLevel;
export type CSS = FTML.CSS;
export type TOCElem = FTML.TOCElem;
export type TOC = FTML.TOC;
export type Institution = FTML.Institution;
export type ArchiveIndex = FTML.ArchiveIndex;
export type Instance = FTML.Instance;
export type Language = FTML.Language;
export type CognitiveDimension = FTML.CognitiveDimension;
export type LOKind = FTML.LOKind;

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

export type DocumentURIParams = {uri:DocumentURI} | 
  { a: string, rp: string } | 
  { a:string, p?:string, d:string, l:Language }
;

export type SymbolURIParams = {uri:SymbolURI} |
  { a:string, p?:string, m:string, s:string };

export type DocumentElementURIParams = {uri:DocumentElementURI} |
  {a:string, p?:string, d:string, l:Language, e:string};


export type URIParams = {uri:DocumentURI} | 
  { a:string} | // ArchiveURI
  { a: string, rp: string } | // DocumentURI 
  { a:string, p?:string, d:string, l?:Language } | // DocumentURI
  { a:string, p?:string, d:string, l?:Language, e:string } | // DocumentElementURI
  { a:string, p?:string, m:string, l?:Language } | // ModuleURI
  { a:string, p?:string, m:string, l?:Language, s:string } // SymbolURI
;