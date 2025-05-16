"use strict";
var __create = Object.create;
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getProtoOf = Object.getPrototypeOf;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toESM = (mod, isNodeMode, target) => (target = mod != null ? __create(__getProtoOf(mod)) : {}, __copyProps(
  // If the importer is in node compatibility mode or this is not an ESM
  // file that has been converted to a CommonJS file using a Babel-
  // compatible transform (i.e. "__esModule" has not been set), then set
  // "default" to the CommonJS "module.exports" for node compatibility.
  isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", { value: mod, enumerable: true }) : target,
  mod
));
var __toCommonJS = (mod) => __copyProps(__defProp({}, "__esModule", { value: true }), mod);

// src/index.ts
var index_exports = {};
__export(index_exports, {
  ftmlSetup: () => ftmlSetup,
  getFlamsServer: () => getFlamsServer,
  getServerUrl: () => getServerUrl,
  injectCss: () => injectCss2,
  renderDocument: () => renderDocument,
  renderFragment: () => renderFragment,
  setDebugLog: () => setDebugLog,
  setServerUrl: () => setServerUrl
});
module.exports = __toCommonJS(index_exports);
var FTML = __toESM(require("@kwarc/ftml-viewer-base"));
var FLAMS = __toESM(require("@kwarc/flams"));
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
// Annotate the CommonJS export names for ESM import in node:
0 && (module.exports = {
  ftmlSetup,
  getFlamsServer,
  getServerUrl,
  injectCss,
  renderDocument,
  renderFragment,
  setDebugLog,
  setServerUrl
});
//# sourceMappingURL=index.js.map