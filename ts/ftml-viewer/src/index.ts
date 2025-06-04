import * as FTML from "./base/ftml-viewer-base";

export { FTML };

const onStartI = FTML.init();

/**
 * 
 * Execute the given code only after the FTML viewer has been initialized
 */
export function ifStarted<R>(f: () => R): Promise<R> {
  return onStartI.then(() => f());
}

/**
 * Initializes the FTML viewer
 * 
 * @param serverUrl The url of the Flams server used for requests
 * @param debug     Whether to print debug messages to the console
 */
export async function initialize(serverUrl:string,debug?:boolean) {
  await onStartI;
  FTML.set_server_url(serverUrl);
  if (typeof window !== "undefined") {
    (window as any).FLAMS_SERVER_URL = serverUrl;
  }
  if (debug && debug) {
    FTML.set_debug_log();
  }
}

/**
 * Injects all of the css elements into the document header
 */
export function injectCss(css:FTML.CSS[]): void {
  css.forEach(c => FTML.injectCss(c))
}

export const getServerUrl = FTML.get_server_url;


/**
 * Configuration for rendering FTML content
 */
export interface FTMLConfig {

  /**
   * whether to allow hovers
   */
  allowHovers?: boolean;

  /**
   * callback for *inserting* elements immediately after a section's title
   */
  onSectionTitle?: (
    uri: FTML.DocumentElementURI,
    lvl: FTML.SectionLevel,
  ) => FTML.LeptosContinuation | undefined;

  /**
   * callback for wrapping fragments (sections, paragraphs, problems, etc.)
   */
  onFragment?: (
    uri: FTML.DocumentElementURI,
    kind: FTML.FragmentKind,
  ) => FTML.LeptosContinuation | undefined;
  /**
   * callback for wrapping inputreferences (i.e. lazily loaded document fragments)
   */
  onInputref?: (uri: FTML.DocumentURI) => FTML.LeptosContinuation | undefined;
  
  problemStates?: FTML.ProblemStates | undefined,
  onProblem?: ((response: FTML.ProblemResponse) => void) | undefined,
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
export function ftmlSetup(
  to: HTMLElement,
  then: FTML.LeptosContinuation,
  cfg?: FTMLConfig,
): FTML.FTMLMountHandle {
  return FTML.ftml_setup(
    to,
    then,
    cfg?.allowHovers,
    cfg?.onSectionTitle,
    cfg?.onFragment,
    cfg?.onInputref,
    cfg?.onProblem,
    cfg?.problemStates
  );
}

/**
 * render an FTML document to the provided element
 * @param {HTMLElement} to The element to render into
 * @param {FTML.DocumentOptions} document The document to render
 * @param {FTML.LeptosContext?} context The leptos context to use (if any)
 * @param {FTMLConfig?} cfg Optional configuration
 * @returns {FTML.FTMLMountHandle}; its {@link FTML.FTMLMountHandle.unmount} method removes the context. Not calling
 *          this is a memory leak.
 */
export function renderDocument(
  to: HTMLElement,
  document: FTML.DocumentOptions,
  context?: FTML.LeptosContext,
  cfg?: FTMLConfig,
): FTML.FTMLMountHandle {
  return FTML.render_document(
    to,
    document,
    context,
    cfg?.allowHovers,
    cfg?.onSectionTitle,
    cfg?.onFragment,
    cfg?.onInputref,
    cfg?.onProblem,
    cfg?.problemStates
  );
}

/**
 * render an FTML document fragment to the provided element
 * @param {HTMLElement} to The element to render into
 * @param {FTML.FragmentOptions} fragment The fragment to render
 * @param {FTML.LeptosContext?} context The leptos context to use (if any)
 * @param {FTMLConfig?} cfg Optional configuration
 * @returns {FTML.FTMLMountHandle}; its {@link FTML.FTMLMountHandle.unmount} method removes the context. Not calling
 *          this is a memory leak.
 */
export function renderFragment(
  to: HTMLElement,
  fragment: FTML.FragmentOptions,
  context?: FTML.LeptosContext,
  cfg?: FTMLConfig,
): FTML.FTMLMountHandle {
  return FTML.render_fragment(
    to,
    fragment,
    context,
    cfg?.allowHovers,
    cfg?.onSectionTitle,
    cfg?.onFragment,
    cfg?.onInputref,
    cfg?.onProblem,
    cfg?.problemStates
  );
}