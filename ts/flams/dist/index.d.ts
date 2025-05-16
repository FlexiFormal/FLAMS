type OMDoc$1 = ({ type: "Slide" } & OMDocSlide) | ({ type: "Document" } & OMDocDocument) | ({ type: "Section" } & OMDocSection) | ({ type: "DocModule" } & OMDocModule<OMDocDocumentElement>) | ({ type: "Module" } & OMDocModule<OMDocDeclaration>) | ({ type: "DocMorphism" } & OMDocMorphism<OMDocDocumentElement>) | ({ type: "Morphism" } & OMDocMorphism<OMDocDeclaration>) | ({ type: "DocStructure" } & OMDocStructure<OMDocDocumentElement>) | ({ type: "Structure" } & OMDocStructure<OMDocDeclaration>) | ({ type: "DocExtension" } & OMDocExtension<OMDocDocumentElement>) | ({ type: "Extension" } & OMDocExtension<OMDocDeclaration>) | ({ type: "SymbolDeclaration" } & OMDocSymbol) | ({ type: "Variable" } & OMDocVariable) | ({ type: "Paragraph" } & OMDocParagraph) | ({ type: "Problem" } & OMDocProblem) | { type: "Term"; uri: DocumentElementURI$1; term: Term } | { type: "DocReference"; uri: DocumentURI$1; title: string | undefined } | ({ type: "Other" } & string);

type OMDocDocumentElement = ({ type: "Slide" } & OMDocSlide) | ({ type: "Section" } & OMDocSection) | ({ type: "Module" } & OMDocModule<OMDocDocumentElement>) | ({ type: "Morphism" } & OMDocMorphism<OMDocDocumentElement>) | ({ type: "Structure" } & OMDocStructure<OMDocDocumentElement>) | ({ type: "Extension" } & OMDocExtension<OMDocDocumentElement>) | { type: "DocumentReference"; uri: DocumentURI$1; title: string | undefined } | ({ type: "Variable" } & OMDocVariable) | ({ type: "Paragraph" } & OMDocParagraph) | ({ type: "Problem" } & OMDocProblem) | { type: "TopTerm"; uri: DocumentElementURI$1; term: Term } | ({ type: "SymbolDeclaration" } & SymbolURI$1|OMDocSymbol);

interface OMDocProblem {
    uri: DocumentElementURI$1;
    sub_problem: boolean;
    autogradable: boolean;
    points: number | undefined;
    title: string | undefined;
    preconditions: [CognitiveDimension$1, SymbolURI$1][];
    objectives: [CognitiveDimension$1, SymbolURI$1][];
    uses: ModuleURI[];
    children: OMDocDocumentElement[];
}

interface OMDocParagraph {
    uri: DocumentElementURI$1;
    kind: ParagraphKind$1;
    formatting: ParagraphFormatting;
    uses: ModuleURI[];
    fors: ModuleURI[];
    title: string | undefined;
    children: OMDocDocumentElement[];
    definition_like: boolean;
}

interface OMDocVariable {
    uri: DocumentElementURI$1;
    arity: ArgSpec;
    macro_name: string | undefined;
    tp: Term | undefined;
    df: Term | undefined;
    is_seq: boolean;
}

interface OMDocSlide {
    uri: DocumentElementURI$1;
    uses: ModuleURI[];
    children: OMDocDocumentElement[];
}

interface OMDocSection {
    title: string | undefined;
    uri: DocumentElementURI$1;
    uses: ModuleURI[];
    children: OMDocDocumentElement[];
}

interface OMDocDocument {
    uri: DocumentURI$1;
    title: string | undefined;
    uses: ModuleURI[];
    children: OMDocDocumentElement[];
}

/**
 * An entry in a table of contents. Either:
 * 1. a section; the title is assumed to be an HTML string, or
 * 2. an inputref to some other document; the URI is the one for the
 *    inputref itself; not the referenced Document. For the TOC,
 *    which document is inputrefed is actually irrelevant.
 */
type TOCElem$1 = { type: "Section"; title: string | undefined; uri: DocumentElementURI$1; id: string; children: TOCElem$1[] } | { type: "SkippedSection"; children: TOCElem$1[] } | { type: "Inputref"; uri: DocumentURI$1; title: string | undefined; id: string; children: TOCElem$1[] } | { type: "Paragraph"; styles: Name$1[]; kind: ParagraphKind$1 } | { type: "Slide" };

interface OMDocSymbol {
    uri: SymbolURI$1;
    df: Term | undefined;
    tp: Term | undefined;
    arity: ArgSpec;
    macro_name: string | undefined;
}

type OMDocDeclaration = ({ type: "Symbol" } & OMDocSymbol) | ({ type: "NestedModule" } & OMDocModule<OMDocDeclaration>) | ({ type: "Structure" } & OMDocStructure<OMDocDeclaration>) | ({ type: "Morphism" } & OMDocMorphism<OMDocDeclaration>) | ({ type: "Extension" } & OMDocExtension<OMDocDeclaration>);

interface OMDocExtension<E> {
    uri: SymbolURI$1;
    target: SymbolURI$1;
    uses: ModuleURI[];
    children: E[];
}

interface OMDocStructure<E> {
    uri: SymbolURI$1;
    macro_name: string | undefined;
    uses: ModuleURI[];
    extends: ModuleURI[];
    children: E[];
    extensions: [SymbolURI$1, OMDocSymbol[]][];
}

interface OMDocMorphism<E> {
    uri: SymbolURI$1;
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
    signature: Language$1 | undefined;
    children: E[];
}

type SolutionData$1 = { Solution: { html: string; answer_class: string | undefined } } | { ChoiceBlock: ChoiceBlock } | { FillInSol: FillInSol };

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

interface ProblemFeedbackJson$1 {
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

type FillinFeedbackKind = ({ type: "Exact" } & string) | { type: "NumRange"; from: number | undefined; to: number | undefined } | ({ type: "Regex" } & string);

type CheckedResult = { type: "SingleChoice"; selected: number; choices: BlockFeedback[] } | { type: "MultipleChoice"; selected: boolean[]; choices: BlockFeedback[] } | { type: "FillinSol"; matching: number | undefined; text: string; options: FillinFeedback[] };

interface ProblemResponse$1 {
    uri: DocumentElementURI$1;
    responses: ProblemResponseType$1[];
}

/**
 * Either a list of booleans (multiple choice), a single integer (single choice),
 * or a string (fill-in-the-gaps)
 */
type ProblemResponseType$1 = boolean[] | number | string;

interface AnswerClass {
    id: string;
    feedback: string;
    kind: AnswerKind;
}

type AnswerKind = ({ type: "Class" } & number) | ({ type: "Trait" } & number);

type CognitiveDimension$1 = "Remember" | "Understand" | "Apply" | "Analyze" | "Evaluate" | "Create";

interface Quiz$1 {
    css: CSS$1[];
    title: string | undefined;
    elements: QuizElement[];
    solutions: Map<DocumentElementURI$1, string>;
    answer_classes: Map<DocumentElementURI$1, AnswerClass[]>;
}

type QuizElement = { Section: { title: string; elements: QuizElement[] } } | { Problem: QuizProblem } | { Paragraph: { html: string } };

interface QuizProblem {
    html: string;
    title_html: string | undefined;
    uri: DocumentElementURI$1;
    total_points: number | undefined;
    preconditions: [CognitiveDimension$1, SymbolURI$1][];
    objectives: [CognitiveDimension$1, SymbolURI$1][];
}

type Informal = { Term: number } | { Node: { tag: string; attributes: [string, string][]; children: Informal[] } } | { Text: string };

type Var = { Name: Name$1 } | { Ref: { declaration: DocumentElementURI$1; is_sequence: boolean | undefined } };

type ArgMode = "Normal" | "Sequence" | "Binding" | "BindingSequence";

interface Arg {
    term: Term;
    mode: ArgMode;
}

type Term = { OMID: ContentURI } | { OMV: Var } | { OMA: { head: Term; args: Arg[] } } | { Field: { record: Term; key: Name$1; owner: Term | undefined } } | { OML: { name: Name$1; df: Term | undefined; tp: Term | undefined } } | { Informal: { tag: string; attributes: [string, string][]; children: Informal[]; terms: Term[] } };

type ArchiveId$1 = string;

type SearchResultKind = "Document" | "Paragraph" | "Definition" | "Example" | "Assertion" | "Problem";

type SearchResult$1 = { Document: DocumentURI$1 } | { Paragraph: { uri: DocumentElementURI$1; fors: SymbolURI$1[]; def_like: boolean; kind: SearchResultKind } };

interface QueryFilter$1 {
    allow_documents?: boolean;
    allow_paragraphs?: boolean;
    allow_definitions?: boolean;
    allow_examples?: boolean;
    allow_assertions?: boolean;
    allow_problems?: boolean;
    definition_like_only?: boolean;
}

type SectionLevel$1 = "Part" | "Chapter" | "Section" | "Subsection" | "Subsubsection" | "Paragraph" | "Subparagraph";

type Name$1 = string;

type SlideElement$1 = { type: "Slide"; html: string } | { type: "Paragraph"; html: string } | { type: "Inputref"; uri: DocumentURI$1 } | { type: "Section"; title: string | undefined; children: SlideElement$1[] };

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
    id: ArchiveId$1;
    summary?: FileStateSummary | undefined;
}

interface ArchiveData {
    id: ArchiveId$1;
    git?: string | undefined;
    summary?: FileStateSummary | undefined;
}

interface Instance$1 {
    semester: string;
    instructors?: string[] | undefined;
}

type ArchiveIndex$1 = { type: "library"; archive: ArchiveId$1; title: string; teaser?: string | undefined; thumbnail?: string | undefined } | { type: "book"; title: string; authors: string[]; file: DocumentURI$1; teaser?: string | undefined; thumbnail?: string | undefined } | { type: "paper"; title: string; authors: string[]; file: DocumentURI$1; thumbnail?: string | undefined; teaser?: string | undefined; venue?: string | undefined; venue_url?: string | undefined } | { type: "course"; title: string; landing: DocumentURI$1; acronym: string | undefined; instructors: string[]; institution: string; instances: Instance$1[]; notes: DocumentURI$1; slides?: DocumentURI$1 | undefined; thumbnail?: string | undefined; quizzes?: boolean; homeworks?: boolean; teaser?: string | undefined } | { type: "self-study"; title: string; landing: DocumentURI$1; notes: DocumentURI$1; acronym?: string | undefined; slides?: DocumentURI$1 | undefined; thumbnail?: string | undefined; teaser?: string | undefined };

type Institution$1 = { type: "university"; title: string; place: string; country: string; url: string; acronym: string; logo: string } | { type: "school"; title: string; place: string; country: string; url: string; acronym: string; logo: string };

type ParagraphKind$1 = "Definition" | "Assertion" | "Paragraph" | "Proof" | "SubProof" | "Example";

type ParagraphFormatting = "Block" | "Inline" | "Collapsed";

interface FileStateSummary {
    new: number;
    stale: number;
    deleted: number;
    up_to_date: number;
    last_built: Timestamp;
    last_changed: Timestamp;
}

type LOKind$1 = { type: "Definition" } | { type: "Example" } | ({ type: "Problem" } & CognitiveDimension$1) | ({ type: "SubProblem" } & CognitiveDimension$1);

type Language$1 = "en" | "de" | "fr" | "ro" | "ar" | "bg" | "ru" | "fi" | "tr" | "sl";

type ModuleURI = string;

type SymbolURI$1 = string;

type ContentURI = string;

type DocumentElementURI$1 = string;

type DocumentURI$1 = string;

type URI$1 = string;

type ArgSpec = ArgMode[];

type CSS$1 = { Link: string } | { Inline: string } | { Class: { name: string; css: string } };

type Timestamp = number;

type Regex = string;
declare class ProblemFeedback$1 {
  private constructor();
  free(): void;
  static from_jstring(s: string): ProblemFeedback$1 | undefined;
  to_jstring(): string | undefined;
  static from_json(arg0: ProblemFeedbackJson$1): ProblemFeedback$1;
  to_json(): ProblemFeedbackJson$1;
  correct: boolean;
  score_fraction: number;
}
declare class Solutions {
  private constructor();
  free(): void;
  static from_jstring(s: string): Solutions | undefined;
  to_jstring(): string | undefined;
  static from_solutions(solutions: SolutionData$1[]): Solutions;
  to_solutions(): SolutionData$1[];
  check_response(response: ProblemResponse$1): ProblemFeedback$1 | undefined;
  default_feedback(): ProblemFeedback$1;
}

type DocumentURI = DocumentURI$1;
type SymbolURI = SymbolURI$1;
type DocumentElementURI = DocumentElementURI$1;
type Name = Name$1;
type ProblemResponse = ProblemResponse$1;
type ProblemResponseType = ProblemResponseType$1;
type ProblemFeedback = ProblemFeedback$1;
type ProblemSolutions = Solutions;
type ParagraphKind = ParagraphKind$1;
type SectionLevel = SectionLevel$1;
type CSS = CSS$1;
type TOCElem = TOCElem$1;
type Institution = Institution$1;
type ArchiveIndex = ArchiveIndex$1;
type Instance = Instance$1;
type Language = Language$1;
type CognitiveDimension = CognitiveDimension$1;
type LOKind = LOKind$1;
type ArchiveGroup = ArchiveGroupData;
type Archive = ArchiveData;
type Directory = DirectoryData;
type File = FileData;
type SearchResult = SearchResult$1;
type QueryFilter = QueryFilter$1;
type Quiz = Quiz$1;
type SlideElement = SlideElement$1;
type ArchiveId = ArchiveId$1;
type SolutionData = SolutionData$1;
type ProblemFeedbackJson = ProblemFeedbackJson$1;
type OMDoc = OMDoc$1;
type URI = URI$1;
type DocumentURIParams = {
    uri: DocumentURI;
} | {
    a: string;
    rp: string;
} | {
    a: string;
    p?: string;
    d: string;
    l: Language;
};
type SymbolURIParams = {
    uri: SymbolURI;
} | {
    a: string;
    p?: string;
    m: string;
    s: string;
};
type DocumentElementURIParams = {
    uri: DocumentElementURI;
} | {
    a: string;
    p?: string;
    d: string;
    l: Language;
    e: string;
};
type URIParams = {
    uri: URI;
} | {
    a: string;
} | {
    a: string;
    rp: string;
} | {
    a: string;
    p?: string;
    d: string;
    l?: Language;
} | {
    a: string;
    p?: string;
    d: string;
    l?: Language;
    e: string;
} | {
    a: string;
    p?: string;
    m: string;
    l?: Language;
} | {
    a: string;
    p?: string;
    m: string;
    l?: Language;
    s: string;
};

declare class FLAMSServer {
    _url: string;
    constructor(url: string);
    get url(): string;
    /**
     * All institutions and `archive.json`-registered documents
     */
    index(): Promise<[
        Institution[],
        ArchiveIndex[]
    ] | undefined>;
    /**
     * Full-text search for documents, assuming the given filter
     */
    searchDocs(query: string, filter: QueryFilter, numResults: number): Promise<[number, SearchResult][] | undefined>;
    /**
     * Full-text search for (definitions of) symbols
     */
    searchSymbols(query: string, numResults: number): Promise<[SymbolURI, [number, SearchResult][]][] | undefined>;
    /**
     * List all archives/groups in the given group (or at top-level, if undefined)
     */
    backendGroupEntries(in_entry?: string): Promise<[ArchiveGroup[], Archive[]] | undefined>;
    /**
     * List all directories/files in the given archive at path (or at top-level, if undefined)
     */
    backendArchiveEntries(archive: string, in_path?: string): Promise<[Directory[], File[]] | undefined>;
    /**
     * SPARQL query
     */
    query(sparql: String): Promise<any>;
    /**
     * Get all dependencies of the given archive (excluding meta-inf archives)
     */
    archiveDependencies(archives: ArchiveId[]): Promise<ArchiveId[] | undefined>;
    /**
     * Return the TOC of the given document
     */
    contentToc(uri: DocumentURIParams): Promise<[CSS[], TOCElem[]] | undefined>;
    /**
     * Get all learning objects for the given symbol; if problems === true, this includes Problems and Subproblems;
     * otherwise, only definitions and examples.
     */
    learningObjects(uri: SymbolURIParams, problems?: boolean): Promise<[[string, LOKind]] | undefined>;
    /**
     * Get the quiz in the given document.
     */
    quiz(uri: DocumentURIParams): Promise<Quiz | undefined>;
    /**
     * Return slides for the given document / section
     */
    slides(uri: URIParams): Promise<[CSS[], SlideElement[]] | undefined>;
    /**
     * Batch grade an arrray of <solution,response[]> pairs.
     * Each of the responses will be graded against the corresponding solution, and the resulting
     * feedback returned at the same position. If *any* of the responses is malformed,
     * the whole batch will fail.
     * A SolutionData[] can be obtained from Solutions.to_solutions(). A ProblemFeedbackJson
     * can be turned into a "proper" ProblemFeedback using ProblemFeedback.from_json().
     */
    batchGrade(...submissions: [SolutionData[], (ProblemResponse | undefined)[]][]): Promise<(ProblemFeedbackJson[])[] | undefined>;
    /**
     * Get the solution for the problem with the given URI. As string, so it can be
     * deserialized by the ts binding for the WASM datastructure
     */
    solution(uri: DocumentElementURIParams): Promise<string | undefined>;
    omdoc(uri: URIParams): Promise<[CSS[], OMDoc] | undefined>;
    contentDocument(uri: DocumentURIParams): Promise<[DocumentURI, CSS[], string] | undefined>;
    contentFragment(uri: URIParams): Promise<[CSS[], string] | undefined>;
    rawGetRequest<TRequest extends Record<string, unknown>, TResponse>(endpoint: string, request: TRequest): Promise<TResponse | undefined>;
    private getRequestI;
    rawPostRequest<TRequest extends Record<string, unknown>, TResponse>(endpoint: string, request: TRequest): Promise<TResponse | undefined>;
    private postRequestI;
}

export { type Archive, type ArchiveGroup, type ArchiveId, type ArchiveIndex, type CSS, type CognitiveDimension, type Directory, type DocumentElementURI, type DocumentElementURIParams, type DocumentURI, type DocumentURIParams, FLAMSServer, type File, type Instance, type Institution, type LOKind, type Language, type Name, type OMDoc, type ParagraphKind, type ProblemFeedback, type ProblemFeedbackJson, type ProblemResponse, type ProblemResponseType, type ProblemSolutions, type QueryFilter, type Quiz, type SearchResult, type SectionLevel, type SlideElement, type SolutionData, type SymbolURI, type SymbolURIParams, type TOCElem, type URI, type URIParams };
