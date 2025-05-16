// src/index.ts
import * as FTML from "@kwarc/ftml-viewer-base";
import * as FLAMS from "@kwarc/flams";
var Window = typeof window !== "undefined" ? window : { FLAMS_SERVER_URL: "" };
function setDebugLog() {
  FTML.set_debug_log();
}
function injectCss2(css) {
  FTML.injectCss(css);
}
function getFlamsServer() {
  return new FLAMS.FLAMSServer(Window.FLAMS_SERVER_URL);
}
function setServerUrl(s) {
  Window.FLAMS_SERVER_URL = s;
  FTML.set_server_url(s);
}
function getServerUrl() {
  return Window.FLAMS_SERVER_URL;
}
function ftmlSetup(to, then, cfg) {
  return FTML.ftml_setup(
    to,
    then,
    cfg == null ? void 0 : cfg.allowHovers,
    cfg == null ? void 0 : cfg.onSectionTitle,
    cfg == null ? void 0 : cfg.onFragment,
    cfg == null ? void 0 : cfg.onInputref,
    cfg == null ? void 0 : cfg.onProblem,
    cfg == null ? void 0 : cfg.problemStates
  );
}
function renderDocument(to, document, context, cfg) {
  return FTML.render_document(
    to,
    document,
    context,
    cfg == null ? void 0 : cfg.allowHovers,
    cfg == null ? void 0 : cfg.onSectionTitle,
    cfg == null ? void 0 : cfg.onFragment,
    cfg == null ? void 0 : cfg.onInputref,
    cfg == null ? void 0 : cfg.onProblem,
    cfg == null ? void 0 : cfg.problemStates
  );
}
function renderFragment(to, fragment, context, cfg) {
  return FTML.render_fragment(
    to,
    fragment,
    context,
    cfg == null ? void 0 : cfg.allowHovers,
    cfg == null ? void 0 : cfg.onSectionTitle,
    cfg == null ? void 0 : cfg.onFragment,
    cfg == null ? void 0 : cfg.onInputref,
    cfg == null ? void 0 : cfg.onProblem,
    cfg == null ? void 0 : cfg.problemStates
  );
}
export {
  ftmlSetup,
  getFlamsServer,
  getServerUrl,
  injectCss2 as injectCss,
  renderDocument,
  renderFragment,
  setDebugLog,
  setServerUrl
};
//# sourceMappingURL=index.mjs.map