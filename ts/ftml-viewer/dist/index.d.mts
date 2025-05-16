import * as FTML from '@kwarc/ftml-viewer-base';
import * as FLAMS from '@kwarc/flams';

/**
 * Turns on debugging messages on the console
 */
declare function setDebugLog(): void;
/**
 * Injects the given CSS rule into the header of the DOM (if adequate and not duplicate)
 */
declare function injectCss(css: FTML.CSS): void;
/**
 * Get the FLAMS server used globally
 */
declare function getFlamsServer(): FLAMS.FLAMSServer;
/**
 * Set the FLAMS server used globally
 */
declare function setServerUrl(s: string): void;
/**
 * Get the FLAMS server URL used globally
 */
declare function getServerUrl(): string;
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
    onSectionTitle?: (uri: FTML.DocumentElementURI, lvl: FTML.SectionLevel) => FTML.LeptosContinuation | undefined;
    /**
     * callback for wrapping fragments (sections, paragraphs, problems, etc.)
     */
    onFragment?: (uri: FTML.DocumentElementURI, kind: FTML.FragmentKind) => FTML.LeptosContinuation | undefined;
    /**
     * callback for wrapping inputreferences (i.e. lazily loaded document fragments)
     */
    onInputref?: (uri: FTML.DocumentURI) => FTML.LeptosContinuation | undefined;
    problemStates?: FTML.ProblemStates | undefined;
    onProblem?: ((response: FTML.ProblemResponse) => void) | undefined;
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
declare function ftmlSetup(to: HTMLElement, then: FTML.LeptosContinuation, cfg?: FTMLConfig): FTML.FTMLMountHandle;
/**
 * render an FTML document to the provided element
 * @param {HTMLElement} to The element to render into
 * @param {FTML.DocumentOptions} document The document to render
 * @param {FTML.LeptosContext?} context The leptos context to use (if any)
 * @param {FTMLConfig?} cfg Optional configuration
 * @returns {FTML.FTMLMountHandle}; its {@link FTML.FTMLMountHandle.unmount} method removes the context. Not calling
 *          this is a memory leak.
 */
declare function renderDocument(to: HTMLElement, document: FTML.DocumentOptions, context?: FTML.LeptosContext, cfg?: FTMLConfig): FTML.FTMLMountHandle;
/**
 * render an FTML document fragment to the provided element
 * @param {HTMLElement} to The element to render into
 * @param {FTML.FragmentOptions} fragment The fragment to render
 * @param {FTML.LeptosContext?} context The leptos context to use (if any)
 * @param {FTMLConfig?} cfg Optional configuration
 * @returns {FTML.FTMLMountHandle}; its {@link FTML.FTMLMountHandle.unmount} method removes the context. Not calling
 *          this is a memory leak.
 */
declare function renderFragment(to: HTMLElement, fragment: FTML.FragmentOptions, context?: FTML.LeptosContext, cfg?: FTMLConfig): FTML.FTMLMountHandle;

export { type FTMLConfig, ftmlSetup, getFlamsServer, getServerUrl, injectCss, renderDocument, renderFragment, setDebugLog, setServerUrl };
