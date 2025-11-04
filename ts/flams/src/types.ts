import * as FTML from "./ftml-viewer-base";

export type DocumentUri = FTML.DocumentUri;
export type SymbolUri = FTML.SymbolUri;
export type DocumentElementUri = FTML.DocumentElementUri;
export type Name = FTML.Name;

export type ProblemResponse = FTML.ProblemResponse;
export type ProblemResponseType = FTML.ProblemResponseType;
export type ProblemFeedback = FTML.ProblemFeedback;
export type ProblemSolutions = FTML.Solutions;

export type ParagraphKind = FTML.ParagraphKind;
export type SectionLevel = FTML.SectionLevel;
export type CSS = FTML.CSS;
export type TOCElem = FTML.TOCElem;
export type Institution = FTML.Institution;
export type ArchiveIndex = FTML.ArchiveIndex;
export type Instance = FTML.Instance;
export type Language = FTML.Language;
export type CognitiveDimension = FTML.CognitiveDimension;
export type LOKind = FTML.LOKind;
export type ArchiveGroup = FTML.ArchiveGroupData;
export type Archive = FTML.ArchiveData;
export type Directory = FTML.DirectoryData;
export type File = FTML.FileData;
export type SearchResult = FTML.SearchResult;
export type QueryFilter = FTML.QueryFilter;
export type Quiz = FTML.Quiz;
export type SlideElement = FTML.SlideElement;
export type ArchiveId = FTML.ArchiveId;
export type SolutionData = FTML.SolutionData;
export type ProblemFeedbackJson = FTML.ProblemFeedbackJson;
export type OMDoc = FTML.OMDoc;
export type URI = FTML.URI;

export type DocumentUriParams =
  | { uri: DocumentUri }
  | { a: string; rp: string }
  | { a: string; p?: string; d: string; l: Language };

export type SymbolUriParams =
  | { uri: SymbolUri }
  | { a: string; p?: string; m: string; s: string };

export type DocumentElementUriParams =
  | { uri: DocumentElementUri }
  | { a: string; p?: string; d: string; l: Language; e: string };

export type URIParams =
  | { uri: URI }
  | { a: string } // ArchiveUri
  | { a: string; rp: string } // DocumentUri
  | { a: string; p?: string; d: string; l?: Language } // DocumentUri
  | { a: string; p?: string; d: string; l?: Language; e: string } // DocumentElementUri
  | { a: string; p?: string; m: string; l?: Language } // ModuleUri
  | { a: string; p?: string; m: string; l?: Language; s: string }; // SymbolUri
