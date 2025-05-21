/* tslint:disable */
/* eslint-disable */
declare function injectCss$1(css: CSS): void;
/**
 * activates debug logging
 */
declare function set_debug_log(): void;
/**
 * sets up a leptos context for rendering FTML documents or fragments.
 * If a context already exists, does nothing, so is cheap to call
 * [render_document] and [render_fragment] also inject a context
 * iff none already exists, so this is optional in every case.
 */
declare function ftml_setup(to: HTMLElement, children: LeptosContinuation, allow_hovers?: boolean | null, on_section_title?: (uri: DocumentElementURI,lvl:SectionLevel) => (LeptosContinuation | undefined) | null, on_fragment?: (uri: DocumentElementURI,kind:FragmentKind) => (LeptosContinuation | undefined) | null, on_inputref?: (uri: DocumentURI) => (LeptosContinuation | undefined) | null, on_problem?: (r:ProblemResponse) => void | null, problem_states?: ProblemStates | null): FTMLMountHandle;
/**
 * render an FTML document to the provided element
 * #### Errors
 */
declare function render_document(to: HTMLElement, document: DocumentOptions, context?: LeptosContext | null, allow_hovers?: boolean | null, on_section_title?: (uri: DocumentElementURI,lvl:SectionLevel) => (LeptosContinuation | undefined) | null, on_fragment?: (uri: DocumentElementURI,kind:FragmentKind) => (LeptosContinuation | undefined) | null, on_inputref?: (uri: DocumentURI) => (LeptosContinuation | undefined) | null, on_problem?: (r:ProblemResponse) => void | null, problem_states?: ProblemStates | null): FTMLMountHandle;
/**
 * render an FTML document fragment to the provided element
 * #### Errors
 */
declare function render_fragment(to: HTMLElement, fragment: FragmentOptions, context?: LeptosContext | null, allow_hovers?: boolean | null, on_section_title?: (uri: DocumentElementURI,lvl:SectionLevel) => (LeptosContinuation | undefined) | null, on_fragment?: (uri: DocumentElementURI,kind:FragmentKind) => (LeptosContinuation | undefined) | null, on_inputref?: (uri: DocumentURI) => (LeptosContinuation | undefined) | null, on_problem?: (r:ProblemResponse) => void | null, problem_states?: ProblemStates | null): FTMLMountHandle;
/**
 * sets the server url used to the provided one; by default `https://mathhub.info`.
 */
declare function set_server_url(server_url: string): void;
/**
 * gets the current server url
 */
declare function get_server_url(): string;
/**
 * The `ReadableStreamType` enum.
 *
 * *This API requires the following crate features to be activated: `ReadableStreamType`*
 */
type ReadableStreamType = "bytes";
/**
 * State of a particular problem
 */
type ProblemState = { type: "Interactive"; current_response?: ProblemResponse | undefined; solution?: SolutionData[] | undefined } | { type: "Finished"; current_response?: ProblemResponse | undefined } | { type: "Graded"; feedback: ProblemFeedbackJson };

type ProblemStates = Map<DocumentElementURI, ProblemState>;

/**
 * Options for rendering an FTML document
 * - `FromBackend`: calls the backend for the document
 *     uri: the URI of the document (as string)
 *     toc: if defined, will render a table of contents for the document
 * - `HtmlString`: render the provided HTML String
 *     html: the HTML String
 *     toc: if defined, will render a table of contents for the document
 */
type DocumentOptions = { type: "FromBackend"; uri: DocumentURI; gottos?: Gotto[] | undefined; toc: TOCOptions | undefined } | { type: "HtmlString"; html: string; gottos?: Gotto[] | undefined; toc: TOCElem[] | undefined };

/**
 * Options for rendering an FTML document fragment
 * - `FromBackend`: calls the backend for the document fragment
 *     uri: the URI of the document fragment (as string)
 * - `HtmlString`: render the provided HTML String
 *     html: the HTML String
 */
type FragmentOptions = { type: "FromBackend"; uri: DocumentElementURI } | { type: "HtmlString"; html: string; uri?: DocumentElementURI | undefined };

/**
 * Options for rendering a table of contents
 * `GET` will retrieve it from the remote backend
 * `TOCElem[]` will render the provided TOC
 */
type TOCOptions = "GET" | { Predefined: TOCElem[] };

interface OMDocSymbol {
    uri: SymbolURI;
    df: Term | undefined;
    tp: Term | undefined;
    arity: ArgSpec;
    macro_name: string | undefined;
}

type OMDocDeclaration = ({ type: "Symbol" } & OMDocSymbol) | ({ type: "NestedModule" } & OMDocModule<OMDocDeclaration>) | ({ type: "Structure" } & OMDocStructure<OMDocDeclaration>) | ({ type: "Morphism" } & OMDocMorphism<OMDocDeclaration>) | ({ type: "Extension" } & OMDocExtension<OMDocDeclaration>);

interface OMDocExtension<E> {
    uri: SymbolURI;
    target: SymbolURI;
    uses: ModuleURI[];
    children: E[];
}

interface OMDocStructure<E> {
    uri: SymbolURI;
    macro_name: string | undefined;
    uses: ModuleURI[];
    extends: ModuleURI[];
    children: E[];
    extensions: [SymbolURI, OMDocSymbol[]][];
}

interface OMDocMorphism<E> {
    uri: SymbolURI;
    total: boolean;
    target: ModuleURI | undefined;
    uses: ModuleURI[];
    children: E[];
}

interface OMDocModule<E> {
    uri: ModuleURI;
    imports: ModuleURI[];
    uses: ModuleURI[];
    metatheory: ModuleURI | undefined;
    signature: Language | undefined;
    children: E[];
}

/**
 * An entry in a table of contents. Either:
 * 1. a section; the title is assumed to be an HTML string, or
 * 2. an inputref to some other document; the URI is the one for the
 *    inputref itself; not the referenced Document. For the TOC,
 *    which document is inputrefed is actually irrelevant.
 */
type TOCElem = { type: "Section"; title: string | undefined; uri: DocumentElementURI; id: string; children: TOCElem[] } | { type: "SkippedSection"; children: TOCElem[] } | { type: "Inputref"; uri: DocumentURI; title: string | undefined; id: string; children: TOCElem[] } | { type: "Paragraph"; styles: Name[]; kind: ParagraphKind } | { type: "Slide" };

/**
 * A section that has been \"covered\" at the specified timestamp; will be marked accordingly
 * in the TOC.
 */
interface Gotto {
    uri: DocumentElementURI;
    timestamp?: Timestamp | undefined;
}

type OMDocDocumentElement = ({ type: "Slide" } & OMDocSlide) | ({ type: "Section" } & OMDocSection) | ({ type: "Module" } & OMDocModule<OMDocDocumentElement>) | ({ type: "Morphism" } & OMDocMorphism<OMDocDocumentElement>) | ({ type: "Structure" } & OMDocStructure<OMDocDocumentElement>) | ({ type: "Extension" } & OMDocExtension<OMDocDocumentElement>) | { type: "DocumentReference"; uri: DocumentURI; title: string | undefined } | ({ type: "Variable" } & OMDocVariable) | ({ type: "Paragraph" } & OMDocParagraph) | ({ type: "Problem" } & OMDocProblem) | { type: "TopTerm"; uri: DocumentElementURI; term: Term } | ({ type: "SymbolDeclaration" } & SymbolURI|OMDocSymbol);

interface OMDocProblem {
    uri: DocumentElementURI;
    sub_problem: boolean;
    autogradable: boolean;
    points: number | undefined;
    title: string | undefined;
    preconditions: [CognitiveDimension, SymbolURI][];
    objectives: [CognitiveDimension, SymbolURI][];
    uses: ModuleURI[];
    children: OMDocDocumentElement[];
}

interface OMDocParagraph {
    uri: DocumentElementURI;
    kind: ParagraphKind;
    formatting: ParagraphFormatting;
    uses: ModuleURI[];
    fors: ModuleURI[];
    title: string | undefined;
    children: OMDocDocumentElement[];
    definition_like: boolean;
}

interface OMDocVariable {
    uri: DocumentElementURI;
    arity: ArgSpec;
    macro_name: string | undefined;
    tp: Term | undefined;
    df: Term | undefined;
    is_seq: boolean;
}

interface OMDocSlide {
    uri: DocumentElementURI;
    uses: ModuleURI[];
    children: OMDocDocumentElement[];
}

interface OMDocSection {
    title: string | undefined;
    uri: DocumentElementURI;
    uses: ModuleURI[];
    children: OMDocDocumentElement[];
}

interface OMDocDocument {
    uri: DocumentURI;
    title: string | undefined;
    uses: ModuleURI[];
    children: OMDocDocumentElement[];
}

type OMDoc = ({ type: "Slide" } & OMDocSlide) | ({ type: "Document" } & OMDocDocument) | ({ type: "Section" } & OMDocSection) | ({ type: "DocModule" } & OMDocModule<OMDocDocumentElement>) | ({ type: "Module" } & OMDocModule<OMDocDeclaration>) | ({ type: "DocMorphism" } & OMDocMorphism<OMDocDocumentElement>) | ({ type: "Morphism" } & OMDocMorphism<OMDocDeclaration>) | ({ type: "DocStructure" } & OMDocStructure<OMDocDocumentElement>) | ({ type: "Structure" } & OMDocStructure<OMDocDeclaration>) | ({ type: "DocExtension" } & OMDocExtension<OMDocDocumentElement>) | ({ type: "Extension" } & OMDocExtension<OMDocDeclaration>) | ({ type: "SymbolDeclaration" } & OMDocSymbol) | ({ type: "Variable" } & OMDocVariable) | ({ type: "Paragraph" } & OMDocParagraph) | ({ type: "Problem" } & OMDocProblem) | { type: "Term"; uri: DocumentElementURI; term: Term } | { type: "DocReference"; uri: DocumentURI; title: string | undefined } | ({ type: "Other" } & string);

type FragmentKind = ({ type: "Section" } & SectionLevel) | ({ type: "Paragraph" } & ParagraphKind) | { type: "Slide" } | { type: "Problem"; is_sub_problem: boolean; is_autogradable: boolean };

type LeptosContinuation = (e:HTMLDivElement,o:LeptosContext) => void;

type SolutionData = { Solution: { html: string; answer_class: string | undefined } } | { ChoiceBlock: ChoiceBlock } | { FillInSol: FillInSol };

interface ChoiceBlock {
    multiple: boolean;
    inline: boolean;
    range: DocumentRange;
    styles: string[];
    choices: Choice[];
}

interface Choice {
    correct: boolean;
    verdict: string;
    feedback: string;
}

interface FillInSol {
    width: number | undefined;
    opts: FillInSolOption[];
}

type FillInSolOption = { Exact: { value: string; verdict: boolean; feedback: string } } | { NumericalRange: { from: number | undefined; to: number | undefined; verdict: boolean; feedback: string } } | { Regex: { regex: Regex; verdict: boolean; feedback: string } };

interface ProblemFeedbackJson {
    correct: boolean;
    solutions: string[];
    data: CheckedResult[];
    score_fraction: number;
}

interface BlockFeedback {
    is_correct: boolean;
    verdict_str: string;
    feedback: string;
}

interface FillinFeedback {
    is_correct: boolean;
    feedback: string;
    kind: FillinFeedbackKind;
}

type FillinFeedbackKind = { Exact: string } | { NumRange: { from: number | undefined; to: number | undefined } } | { Regex: string };

type CheckedResult = { type: "SingleChoice"; selected: number | undefined; choices: BlockFeedback[] } | { type: "MultipleChoice"; selected: boolean[]; choices: BlockFeedback[] } | { type: "FillinSol"; matching: number | undefined; text: string; options: FillinFeedback[] };

interface ProblemResponse {
    uri: DocumentElementURI;
    responses: ProblemResponseType[];
}

/**
 * Either a list of booleans (multiple choice), a single integer (single choice),
 * or a string (fill-in-the-gaps)
 */
type ProblemResponseType = { type: "MultipleChoice"; value: boolean[] } | { type: "SingleChoice"; value: number | undefined } | { type: "Fillinsol"; value: string };

interface AnswerClass {
    id: string;
    feedback: string;
    kind: AnswerKind;
}

type AnswerKind = { Class: number } | { Trait: number };

type CognitiveDimension = "Remember" | "Understand" | "Apply" | "Analyze" | "Evaluate" | "Create";

interface Quiz {
    css: CSS[];
    title: string | undefined;
    elements: QuizElement[];
    solutions: Map<DocumentElementURI, string>;
    answer_classes: Map<DocumentElementURI, AnswerClass[]>;
}

type QuizElement = { Section: { title: string; elements: QuizElement[] } } | { Problem: QuizProblem } | { Paragraph: { html: string } };

interface QuizProblem {
    html: string;
    title_html: string | undefined;
    uri: DocumentElementURI;
    total_points: number | undefined;
    preconditions: [CognitiveDimension, SymbolURI][];
    objectives: [CognitiveDimension, SymbolURI][];
}

interface FileStateSummary {
    new: number;
    stale: number;
    deleted: number;
    up_to_date: number;
    last_built: Timestamp;
    last_changed: Timestamp;
}

type Informal = { Term: number } | { Node: { tag: string; attributes: [string, string][]; children: Informal[] } } | { Text: string };

type Var = { Name: Name } | { Ref: { declaration: DocumentElementURI; is_sequence: boolean | undefined } };

type ArgMode = "Normal" | "Sequence" | "Binding" | "BindingSequence";

interface Arg {
    term: Term;
    mode: ArgMode;
}

type Term = { OMID: ContentURI } | { OMV: Var } | { OMA: { head: Term; args: Arg[] } } | { Field: { record: Term; key: Name; owner: Term | undefined } } | { OML: { name: Name; df: Term | undefined; tp: Term | undefined } } | { Informal: { tag: string; attributes: [string, string][]; children: Informal[]; terms: Term[] } };

type SlideElement = { type: "Slide"; html: string; uri: DocumentElementURI } | { type: "Paragraph"; html: string; uri: DocumentElementURI } | { type: "Inputref"; uri: DocumentURI } | { type: "Section"; uri: DocumentElementURI; title: string | undefined; children: SlideElement[] };

interface DocumentRange {
    start: number;
    end: number;
}

interface FileData {
    rel_path: string;
    format: string;
}

interface DirectoryData {
    rel_path: string;
    summary?: FileStateSummary | undefined;
}

interface ArchiveGroupData {
    id: ArchiveId;
    summary?: FileStateSummary | undefined;
}

interface ArchiveData {
    id: ArchiveId;
    git?: string | undefined;
    summary?: FileStateSummary | undefined;
}

interface Instance {
    semester: string;
    instructors?: string[] | undefined;
    TAs?: string[] | undefined;
    leadTAs?: string[] | undefined;
}

type ArchiveIndex = { type: "library"; archive: ArchiveId; title: string; teaser?: string | undefined; thumbnail?: string | undefined } | { type: "book"; title: string; authors: string[]; file: DocumentURI; teaser?: string | undefined; thumbnail?: string | undefined } | { type: "paper"; title: string; authors: string[]; file: DocumentURI; thumbnail?: string | undefined; teaser?: string | undefined; venue?: string | undefined; venue_url?: string | undefined } | { type: "course"; title: string; landing: DocumentURI; acronym: string | undefined; instructors: string[]; institution: string; instances: Instance[]; notes: DocumentURI; slides?: DocumentURI | undefined; thumbnail?: string | undefined; quizzes?: boolean; homeworks?: boolean; teaser?: string | undefined } | { type: "self-study"; title: string; landing: DocumentURI; notes: DocumentURI; acronym?: string | undefined; slides?: DocumentURI | undefined; thumbnail?: string | undefined; teaser?: string | undefined };

type Institution = { type: "university"; title: string; place: string; country: string; url: string; acronym: string; logo: string } | { type: "school"; title: string; place: string; country: string; url: string; acronym: string; logo: string };

type ParagraphKind = "Definition" | "Assertion" | "Paragraph" | "Proof" | "SubProof" | "Example";

type ParagraphFormatting = "Block" | "Inline" | "Collapsed";

type ArchiveId = string;

type SearchResultKind = "Document" | "Paragraph" | "Definition" | "Example" | "Assertion" | "Problem";

type SearchResult = { Document: DocumentURI } | { Paragraph: { uri: DocumentElementURI; fors: SymbolURI[]; def_like: boolean; kind: SearchResultKind } };

interface QueryFilter {
    allow_documents?: boolean;
    allow_paragraphs?: boolean;
    allow_definitions?: boolean;
    allow_examples?: boolean;
    allow_assertions?: boolean;
    allow_problems?: boolean;
    definition_like_only?: boolean;
}

type SectionLevel = "Part" | "Chapter" | "Section" | "Subsection" | "Subsubsection" | "Paragraph" | "Subparagraph";

type Name = string;

type LOKind = { type: "Definition" } | { type: "Example" } | ({ type: "Problem" } & CognitiveDimension) | ({ type: "SubProblem" } & CognitiveDimension);

type Language = "en" | "de" | "fr" | "ro" | "ar" | "bg" | "ru" | "fi" | "tr" | "sl";

type ModuleURI = string;

type SymbolURI = string;

type ContentURI = string;

type DocumentElementURI = string;

type DocumentURI = string;

type URI = string;

type ArgSpec = ArgMode[];

type CSS = { Link: string } | { Inline: string } | { Class: { name: string; css: string } };

type Timestamp = number;

type Regex = string;

declare class FTMLMountHandle {
  private constructor();
  free(): void;
  /**
   * unmounts the view and cleans up the reactive system.
   * Not calling this is a memory leak
   */
  unmount(): void;
}
declare class IntoUnderlyingByteSource {
  private constructor();
  free(): void;
  start(controller: ReadableByteStreamController): void;
  pull(controller: ReadableByteStreamController): Promise<any>;
  cancel(): void;
  readonly type: ReadableStreamType;
  readonly autoAllocateChunkSize: number;
}
declare class IntoUnderlyingSink {
  private constructor();
  free(): void;
  write(chunk: any): Promise<any>;
  close(): Promise<any>;
  abort(reason: any): Promise<any>;
}
declare class IntoUnderlyingSource {
  private constructor();
  free(): void;
  pull(controller: ReadableStreamDefaultController): Promise<any>;
  cancel(): void;
}
declare class LeptosContext {
  private constructor();
  free(): void;
  /**
   * Cleans up the reactive system.
   * Not calling this is a memory leak
   */
  cleanup(): void;
  wasm_clone(): LeptosContext;
}
declare class ProblemFeedback {
  private constructor();
  free(): void;
  static from_jstring(s: string): ProblemFeedback | undefined;
  to_jstring(): string | undefined;
  static from_json(arg0: ProblemFeedbackJson): ProblemFeedback;
  to_json(): ProblemFeedbackJson;
  correct: boolean;
  score_fraction: number;
}
declare class Solutions {
  private constructor();
  free(): void;
  static from_jstring(s: string): Solutions | undefined;
  to_jstring(): string | undefined;
  static from_solutions(solutions: SolutionData[]): Solutions;
  to_solutions(): SolutionData[];
  check_response(response: ProblemResponse): ProblemFeedback | undefined;
  default_feedback(): ProblemFeedback;
}

type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly injectCss: (a: number) => void;
  readonly set_debug_log: () => void;
  readonly ftml_setup: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => number;
  readonly render_document: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => void;
  readonly render_fragment: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => void;
  readonly __wbg_ftmlmounthandle_free: (a: number, b: number) => void;
  readonly ftmlmounthandle_unmount: (a: number, b: number) => void;
  readonly set_server_url: (a: number, b: number) => void;
  readonly get_server_url: (a: number) => void;
  readonly __wbg_leptoscontext_free: (a: number, b: number) => void;
  readonly leptoscontext_cleanup: (a: number, b: number) => void;
  readonly leptoscontext_wasm_clone: (a: number) => number;
  readonly __wbg_intounderlyingbytesource_free: (a: number, b: number) => void;
  readonly intounderlyingbytesource_type: (a: number) => number;
  readonly intounderlyingbytesource_autoAllocateChunkSize: (a: number) => number;
  readonly intounderlyingbytesource_start: (a: number, b: number) => void;
  readonly intounderlyingbytesource_pull: (a: number, b: number) => number;
  readonly intounderlyingbytesource_cancel: (a: number) => void;
  readonly __wbg_intounderlyingsink_free: (a: number, b: number) => void;
  readonly intounderlyingsink_write: (a: number, b: number) => number;
  readonly intounderlyingsink_close: (a: number) => number;
  readonly intounderlyingsink_abort: (a: number, b: number) => number;
  readonly __wbg_intounderlyingsource_free: (a: number, b: number) => void;
  readonly intounderlyingsource_pull: (a: number, b: number) => number;
  readonly intounderlyingsource_cancel: (a: number) => void;
  readonly __wbg_solutions_free: (a: number, b: number) => void;
  readonly solutions_from_jstring: (a: number, b: number) => number;
  readonly solutions_to_jstring: (a: number, b: number) => void;
  readonly solutions_from_solutions: (a: number, b: number) => number;
  readonly solutions_to_solutions: (a: number, b: number) => void;
  readonly solutions_check_response: (a: number, b: number) => number;
  readonly solutions_default_feedback: (a: number) => number;
  readonly __wbg_problemfeedback_free: (a: number, b: number) => void;
  readonly __wbg_get_problemfeedback_correct: (a: number) => number;
  readonly __wbg_set_problemfeedback_correct: (a: number, b: number) => void;
  readonly __wbg_get_problemfeedback_score_fraction: (a: number) => number;
  readonly __wbg_set_problemfeedback_score_fraction: (a: number, b: number) => void;
  readonly problemfeedback_from_jstring: (a: number, b: number) => number;
  readonly problemfeedback_to_jstring: (a: number, b: number) => void;
  readonly problemfeedback_from_json: (a: number) => number;
  readonly problemfeedback_to_json: (a: number) => number;
  readonly __wbindgen_export_0: (a: number, b: number) => number;
  readonly __wbindgen_export_1: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_2: (a: number) => void;
  readonly __wbindgen_export_3: (a: number, b: number, c: number) => void;
  readonly __wbindgen_export_4: WebAssembly.Table;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_export_5: (a: number, b: number, c: number) => void;
  readonly __wbindgen_export_6: (a: number, b: number) => void;
  readonly __wbindgen_export_7: (a: number, b: number, c: number) => void;
  readonly __wbindgen_export_8: (a: number, b: number, c: number) => number;
  readonly __wbindgen_export_9: (a: number, b: number, c: number) => void;
  readonly __wbindgen_export_10: (a: number, b: number) => void;
  readonly __wbindgen_export_11: (a: number, b: number, c: number) => void;
  readonly __wbindgen_export_12: (a: number, b: number, c: number) => void;
  readonly __wbindgen_export_13: (a: number, b: number, c: number, d: number) => void;
}

type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
declare function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
declare function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;

declare function init(): Promise<void>;

type ftmlViewerBase_d_AnswerClass = AnswerClass;
type ftmlViewerBase_d_AnswerKind = AnswerKind;
type ftmlViewerBase_d_ArchiveData = ArchiveData;
type ftmlViewerBase_d_ArchiveGroupData = ArchiveGroupData;
type ftmlViewerBase_d_ArchiveId = ArchiveId;
type ftmlViewerBase_d_ArchiveIndex = ArchiveIndex;
type ftmlViewerBase_d_Arg = Arg;
type ftmlViewerBase_d_ArgMode = ArgMode;
type ftmlViewerBase_d_ArgSpec = ArgSpec;
type ftmlViewerBase_d_BlockFeedback = BlockFeedback;
type ftmlViewerBase_d_CSS = CSS;
type ftmlViewerBase_d_CheckedResult = CheckedResult;
type ftmlViewerBase_d_Choice = Choice;
type ftmlViewerBase_d_ChoiceBlock = ChoiceBlock;
type ftmlViewerBase_d_CognitiveDimension = CognitiveDimension;
type ftmlViewerBase_d_ContentURI = ContentURI;
type ftmlViewerBase_d_DirectoryData = DirectoryData;
type ftmlViewerBase_d_DocumentElementURI = DocumentElementURI;
type ftmlViewerBase_d_DocumentOptions = DocumentOptions;
type ftmlViewerBase_d_DocumentRange = DocumentRange;
type ftmlViewerBase_d_DocumentURI = DocumentURI;
type ftmlViewerBase_d_FTMLMountHandle = FTMLMountHandle;
declare const ftmlViewerBase_d_FTMLMountHandle: typeof FTMLMountHandle;
type ftmlViewerBase_d_FileData = FileData;
type ftmlViewerBase_d_FileStateSummary = FileStateSummary;
type ftmlViewerBase_d_FillInSol = FillInSol;
type ftmlViewerBase_d_FillInSolOption = FillInSolOption;
type ftmlViewerBase_d_FillinFeedback = FillinFeedback;
type ftmlViewerBase_d_FillinFeedbackKind = FillinFeedbackKind;
type ftmlViewerBase_d_FragmentKind = FragmentKind;
type ftmlViewerBase_d_FragmentOptions = FragmentOptions;
type ftmlViewerBase_d_Gotto = Gotto;
type ftmlViewerBase_d_Informal = Informal;
type ftmlViewerBase_d_InitInput = InitInput;
type ftmlViewerBase_d_InitOutput = InitOutput;
type ftmlViewerBase_d_Instance = Instance;
type ftmlViewerBase_d_Institution = Institution;
type ftmlViewerBase_d_IntoUnderlyingByteSource = IntoUnderlyingByteSource;
declare const ftmlViewerBase_d_IntoUnderlyingByteSource: typeof IntoUnderlyingByteSource;
type ftmlViewerBase_d_IntoUnderlyingSink = IntoUnderlyingSink;
declare const ftmlViewerBase_d_IntoUnderlyingSink: typeof IntoUnderlyingSink;
type ftmlViewerBase_d_IntoUnderlyingSource = IntoUnderlyingSource;
declare const ftmlViewerBase_d_IntoUnderlyingSource: typeof IntoUnderlyingSource;
type ftmlViewerBase_d_LOKind = LOKind;
type ftmlViewerBase_d_Language = Language;
type ftmlViewerBase_d_LeptosContext = LeptosContext;
declare const ftmlViewerBase_d_LeptosContext: typeof LeptosContext;
type ftmlViewerBase_d_LeptosContinuation = LeptosContinuation;
type ftmlViewerBase_d_ModuleURI = ModuleURI;
type ftmlViewerBase_d_Name = Name;
type ftmlViewerBase_d_OMDoc = OMDoc;
type ftmlViewerBase_d_OMDocDeclaration = OMDocDeclaration;
type ftmlViewerBase_d_OMDocDocument = OMDocDocument;
type ftmlViewerBase_d_OMDocDocumentElement = OMDocDocumentElement;
type ftmlViewerBase_d_OMDocExtension<E> = OMDocExtension<E>;
type ftmlViewerBase_d_OMDocModule<E> = OMDocModule<E>;
type ftmlViewerBase_d_OMDocMorphism<E> = OMDocMorphism<E>;
type ftmlViewerBase_d_OMDocParagraph = OMDocParagraph;
type ftmlViewerBase_d_OMDocProblem = OMDocProblem;
type ftmlViewerBase_d_OMDocSection = OMDocSection;
type ftmlViewerBase_d_OMDocSlide = OMDocSlide;
type ftmlViewerBase_d_OMDocStructure<E> = OMDocStructure<E>;
type ftmlViewerBase_d_OMDocSymbol = OMDocSymbol;
type ftmlViewerBase_d_OMDocVariable = OMDocVariable;
type ftmlViewerBase_d_ParagraphFormatting = ParagraphFormatting;
type ftmlViewerBase_d_ParagraphKind = ParagraphKind;
type ftmlViewerBase_d_ProblemFeedback = ProblemFeedback;
declare const ftmlViewerBase_d_ProblemFeedback: typeof ProblemFeedback;
type ftmlViewerBase_d_ProblemFeedbackJson = ProblemFeedbackJson;
type ftmlViewerBase_d_ProblemResponse = ProblemResponse;
type ftmlViewerBase_d_ProblemResponseType = ProblemResponseType;
type ftmlViewerBase_d_ProblemState = ProblemState;
type ftmlViewerBase_d_ProblemStates = ProblemStates;
type ftmlViewerBase_d_QueryFilter = QueryFilter;
type ftmlViewerBase_d_Quiz = Quiz;
type ftmlViewerBase_d_QuizElement = QuizElement;
type ftmlViewerBase_d_QuizProblem = QuizProblem;
type ftmlViewerBase_d_Regex = Regex;
type ftmlViewerBase_d_SearchResult = SearchResult;
type ftmlViewerBase_d_SearchResultKind = SearchResultKind;
type ftmlViewerBase_d_SectionLevel = SectionLevel;
type ftmlViewerBase_d_SlideElement = SlideElement;
type ftmlViewerBase_d_SolutionData = SolutionData;
type ftmlViewerBase_d_Solutions = Solutions;
declare const ftmlViewerBase_d_Solutions: typeof Solutions;
type ftmlViewerBase_d_SymbolURI = SymbolURI;
type ftmlViewerBase_d_SyncInitInput = SyncInitInput;
type ftmlViewerBase_d_TOCElem = TOCElem;
type ftmlViewerBase_d_TOCOptions = TOCOptions;
type ftmlViewerBase_d_Term = Term;
type ftmlViewerBase_d_Timestamp = Timestamp;
type ftmlViewerBase_d_URI = URI;
type ftmlViewerBase_d_Var = Var;
declare const ftmlViewerBase_d_ftml_setup: typeof ftml_setup;
declare const ftmlViewerBase_d_get_server_url: typeof get_server_url;
declare const ftmlViewerBase_d_init: typeof init;
declare const ftmlViewerBase_d_initSync: typeof initSync;
declare const ftmlViewerBase_d_render_document: typeof render_document;
declare const ftmlViewerBase_d_render_fragment: typeof render_fragment;
declare const ftmlViewerBase_d_set_debug_log: typeof set_debug_log;
declare const ftmlViewerBase_d_set_server_url: typeof set_server_url;
declare namespace ftmlViewerBase_d {
  export { type ftmlViewerBase_d_AnswerClass as AnswerClass, type ftmlViewerBase_d_AnswerKind as AnswerKind, type ftmlViewerBase_d_ArchiveData as ArchiveData, type ftmlViewerBase_d_ArchiveGroupData as ArchiveGroupData, type ftmlViewerBase_d_ArchiveId as ArchiveId, type ftmlViewerBase_d_ArchiveIndex as ArchiveIndex, type ftmlViewerBase_d_Arg as Arg, type ftmlViewerBase_d_ArgMode as ArgMode, type ftmlViewerBase_d_ArgSpec as ArgSpec, type ftmlViewerBase_d_BlockFeedback as BlockFeedback, type ftmlViewerBase_d_CSS as CSS, type ftmlViewerBase_d_CheckedResult as CheckedResult, type ftmlViewerBase_d_Choice as Choice, type ftmlViewerBase_d_ChoiceBlock as ChoiceBlock, type ftmlViewerBase_d_CognitiveDimension as CognitiveDimension, type ftmlViewerBase_d_ContentURI as ContentURI, type ftmlViewerBase_d_DirectoryData as DirectoryData, type ftmlViewerBase_d_DocumentElementURI as DocumentElementURI, type ftmlViewerBase_d_DocumentOptions as DocumentOptions, type ftmlViewerBase_d_DocumentRange as DocumentRange, type ftmlViewerBase_d_DocumentURI as DocumentURI, ftmlViewerBase_d_FTMLMountHandle as FTMLMountHandle, type ftmlViewerBase_d_FileData as FileData, type ftmlViewerBase_d_FileStateSummary as FileStateSummary, type ftmlViewerBase_d_FillInSol as FillInSol, type ftmlViewerBase_d_FillInSolOption as FillInSolOption, type ftmlViewerBase_d_FillinFeedback as FillinFeedback, type ftmlViewerBase_d_FillinFeedbackKind as FillinFeedbackKind, type ftmlViewerBase_d_FragmentKind as FragmentKind, type ftmlViewerBase_d_FragmentOptions as FragmentOptions, type ftmlViewerBase_d_Gotto as Gotto, type ftmlViewerBase_d_Informal as Informal, type ftmlViewerBase_d_InitInput as InitInput, type ftmlViewerBase_d_InitOutput as InitOutput, type ftmlViewerBase_d_Instance as Instance, type ftmlViewerBase_d_Institution as Institution, ftmlViewerBase_d_IntoUnderlyingByteSource as IntoUnderlyingByteSource, ftmlViewerBase_d_IntoUnderlyingSink as IntoUnderlyingSink, ftmlViewerBase_d_IntoUnderlyingSource as IntoUnderlyingSource, type ftmlViewerBase_d_LOKind as LOKind, type ftmlViewerBase_d_Language as Language, ftmlViewerBase_d_LeptosContext as LeptosContext, type ftmlViewerBase_d_LeptosContinuation as LeptosContinuation, type ftmlViewerBase_d_ModuleURI as ModuleURI, type ftmlViewerBase_d_Name as Name, type ftmlViewerBase_d_OMDoc as OMDoc, type ftmlViewerBase_d_OMDocDeclaration as OMDocDeclaration, type ftmlViewerBase_d_OMDocDocument as OMDocDocument, type ftmlViewerBase_d_OMDocDocumentElement as OMDocDocumentElement, type ftmlViewerBase_d_OMDocExtension as OMDocExtension, type ftmlViewerBase_d_OMDocModule as OMDocModule, type ftmlViewerBase_d_OMDocMorphism as OMDocMorphism, type ftmlViewerBase_d_OMDocParagraph as OMDocParagraph, type ftmlViewerBase_d_OMDocProblem as OMDocProblem, type ftmlViewerBase_d_OMDocSection as OMDocSection, type ftmlViewerBase_d_OMDocSlide as OMDocSlide, type ftmlViewerBase_d_OMDocStructure as OMDocStructure, type ftmlViewerBase_d_OMDocSymbol as OMDocSymbol, type ftmlViewerBase_d_OMDocVariable as OMDocVariable, type ftmlViewerBase_d_ParagraphFormatting as ParagraphFormatting, type ftmlViewerBase_d_ParagraphKind as ParagraphKind, ftmlViewerBase_d_ProblemFeedback as ProblemFeedback, type ftmlViewerBase_d_ProblemFeedbackJson as ProblemFeedbackJson, type ftmlViewerBase_d_ProblemResponse as ProblemResponse, type ftmlViewerBase_d_ProblemResponseType as ProblemResponseType, type ftmlViewerBase_d_ProblemState as ProblemState, type ftmlViewerBase_d_ProblemStates as ProblemStates, type ftmlViewerBase_d_QueryFilter as QueryFilter, type ftmlViewerBase_d_Quiz as Quiz, type ftmlViewerBase_d_QuizElement as QuizElement, type ftmlViewerBase_d_QuizProblem as QuizProblem, type ftmlViewerBase_d_Regex as Regex, type ftmlViewerBase_d_SearchResult as SearchResult, type ftmlViewerBase_d_SearchResultKind as SearchResultKind, type ftmlViewerBase_d_SectionLevel as SectionLevel, type ftmlViewerBase_d_SlideElement as SlideElement, type ftmlViewerBase_d_SolutionData as SolutionData, ftmlViewerBase_d_Solutions as Solutions, type ftmlViewerBase_d_SymbolURI as SymbolURI, type ftmlViewerBase_d_SyncInitInput as SyncInitInput, type ftmlViewerBase_d_TOCElem as TOCElem, type ftmlViewerBase_d_TOCOptions as TOCOptions, type ftmlViewerBase_d_Term as Term, type ftmlViewerBase_d_Timestamp as Timestamp, type ftmlViewerBase_d_URI as URI, type ftmlViewerBase_d_Var as Var, __wbg_init as default, ftmlViewerBase_d_ftml_setup as ftml_setup, ftmlViewerBase_d_get_server_url as get_server_url, ftmlViewerBase_d_init as init, ftmlViewerBase_d_initSync as initSync, injectCss$1 as injectCss, ftmlViewerBase_d_render_document as render_document, ftmlViewerBase_d_render_fragment as render_fragment, ftmlViewerBase_d_set_debug_log as set_debug_log, ftmlViewerBase_d_set_server_url as set_server_url };
}

/**
 *
 * Execute the given code only after the FTML viewer has been initialized
 */
declare function ifStarted<R>(f: () => R): Promise<R>;
/**
 * Initializes the FTML viewer
 *
 * @param serverUrl The url of the Flams server used for requests
 * @param debug     Whether to print debug messages to the console
 */
declare function initialize(serverUrl: string, debug?: boolean): Promise<void>;
/**
 * Injects all of the css elements into the document header
 */
declare function injectCss(css: CSS[]): void;
declare const getServerUrl: typeof get_server_url;
/**
 * Configuration for rendering FTML content
 */
interface FTMLConfig {
    /**
     * whether to allow hovers
     */
    allowHovers?: boolean;
    /**
     * callback for *inserting* elements immediately after a section's title
     */
    onSectionTitle?: (uri: DocumentElementURI, lvl: SectionLevel) => LeptosContinuation | undefined;
    /**
     * callback for wrapping fragments (sections, paragraphs, problems, etc.)
     */
    onFragment?: (uri: DocumentElementURI, kind: FragmentKind) => LeptosContinuation | undefined;
    /**
     * callback for wrapping inputreferences (i.e. lazily loaded document fragments)
     */
    onInputref?: (uri: DocumentURI) => LeptosContinuation | undefined;
    problemStates?: ProblemStates | undefined;
    onProblem?: ((response: ProblemResponse) => void) | undefined;
}
/**
 * sets up a leptos context for rendering FTML documents or fragments.
 * If a context already exists, does nothing, so is cheap to call
 * {@link renderDocument} and {@link renderFragment} also inject a context
 * iff none already exists, so this is optional in every case.
 *
 * @param {HTMLElement} to The element to render into
 * @param {FTML.LeptosContinuation} then The code to execute *within* the leptos context (e.g. various calls to
 *        {@link renderDocument} or {@link renderFragment})
 * @param {FTMLConfig?} cfg Optional configuration
 * @returns {FTML.FTMLMountHandle}; its {@link FTML.FTMLMountHandle.unmount} method removes the context. Not calling
 *          this is a memory leak.
 */
declare function ftmlSetup(to: HTMLElement, then: LeptosContinuation, cfg?: FTMLConfig): FTMLMountHandle;
/**
 * render an FTML document to the provided element
 * @param {HTMLElement} to The element to render into
 * @param {FTML.DocumentOptions} document The document to render
 * @param {FTML.LeptosContext?} context The leptos context to use (if any)
 * @param {FTMLConfig?} cfg Optional configuration
 * @returns {FTML.FTMLMountHandle}; its {@link FTML.FTMLMountHandle.unmount} method removes the context. Not calling
 *          this is a memory leak.
 */
declare function renderDocument(to: HTMLElement, document: DocumentOptions, context?: LeptosContext, cfg?: FTMLConfig): FTMLMountHandle;
/**
 * render an FTML document fragment to the provided element
 * @param {HTMLElement} to The element to render into
 * @param {FTML.FragmentOptions} fragment The fragment to render
 * @param {FTML.LeptosContext?} context The leptos context to use (if any)
 * @param {FTMLConfig?} cfg Optional configuration
 * @returns {FTML.FTMLMountHandle}; its {@link FTML.FTMLMountHandle.unmount} method removes the context. Not calling
 *          this is a memory leak.
 */
declare function renderFragment(to: HTMLElement, fragment: FragmentOptions, context?: LeptosContext, cfg?: FTMLConfig): FTMLMountHandle;

export { ftmlViewerBase_d as FTML, type FTMLConfig, ftmlSetup, getServerUrl, ifStarted, initialize, injectCss, renderDocument, renderFragment };
