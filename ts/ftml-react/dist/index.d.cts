import * as FTMLT from '@kwarc/ftml-viewer';
import * as FTML from '@kwarc/ftml-viewer-base';
import React, { ReactNode } from 'react';

/**
 * sets the server url. Reexported for **emphasis**.
 */
declare const setServerUrl: typeof FTMLT.setServerUrl;
/**
 * Injects the given CSS rule into the header of the DOM (if adequate and not duplicate)
 */
declare const injectCss: typeof FTMLT.injectCss;
/**
 * Get the FLAMS server URL used globally
 */
declare const getServerUrl: typeof FTMLT.getServerUrl;
/**
 * Get the FLAMS server used globally
 */
declare const getFlamsServer: typeof FTMLT.getFlamsServer;
/**
 * Turns on debugging messages on the console
 */
declare const setDebugLog: typeof FTMLT.setDebugLog;
/**
 * Configurables for FTML rendering.
 * Every attribute is inherited from ancestor nodes *unless explicitly overridden*.
 */
interface FTMLConfig {
    /** may return a react component to *insert* after the title of a section
     * @param uri the uri of the section
     * @param lvl the level of the section
     * @return a react component to insert
     */
    onSectionTitle?: (uri: FTML.DocumentElementURI, lvl: FTML.SectionLevel) => ReactNode | undefined;
    /**
     * may return a react component to wrap around a fragment (e.g. Section, Definition, Problem, etc.)
     * @param uri the uri of the fragment
     * @param kind the fragment's kind
     * @return a react component to wrap around its argument
     */
    onFragment?: (uri: FTML.DocumentElementURI, kind: FTML.FragmentKind) => ((ch: ReactNode) => ReactNode) | undefined;
    problemStates?: FTML.ProblemStates | undefined;
    onProblem?: ((response: FTML.ProblemResponse) => void) | undefined;
}
/**
 * See {@link FTMLConfig}
 */
interface FTMLSetupArgs extends FTMLConfig {
    children: ReactNode;
}
/**
 * Sets up Leptos' reactive system
 */
declare const FTMLSetup: React.FC<FTMLSetupArgs>;
/**
 * See {@link FTMLConfig} and {@link FTML.DocumentOptions}
 */
interface FTMLDocumentArgs extends FTMLConfig {
    document: FTML.DocumentOptions;
}
/**
 * render an FTML document
 */
declare const FTMLDocument: React.FC<FTMLDocumentArgs>;
/**
 * See {@link FTMLConfig} and {@link FTML.FragmentOptions}
 */
interface FTMLFragmentArgs extends FTMLConfig {
    fragment: FTML.FragmentOptions;
}
/**
 * render an FTML fragment
 */
declare const FTMLFragment: React.FC<FTMLFragmentArgs>;

export { type FTMLConfig, FTMLDocument, type FTMLDocumentArgs, FTMLFragment, type FTMLFragmentArgs, FTMLSetup, type FTMLSetupArgs, getFlamsServer, getServerUrl, injectCss, setDebugLog, setServerUrl };
