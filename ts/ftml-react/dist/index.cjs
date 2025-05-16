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

// src/index.tsx
var index_exports = {};
__export(index_exports, {
  FTMLDocument: () => FTMLDocument,
  FTMLFragment: () => FTMLFragment,
  FTMLSetup: () => FTMLSetup,
  getFlamsServer: () => getFlamsServer2,
  getServerUrl: () => getServerUrl2,
  injectCss: () => injectCss2,
  setDebugLog: () => setDebugLog2,
  setServerUrl: () => setServerUrl2
});
module.exports = __toCommonJS(index_exports);
var FTMLT = __toESM(require("@kwarc/ftml-viewer"), 1);
var import_react2 = require("react");

// src/leptos.tsx
var import_react = require("react");
var import_react_dom = require("react-dom");
var import_jsx_runtime = require("react/jsx-runtime");
var FTMLContext = (0, import_react.createContext)(void 0);
function useLeptosTunnel() {
  const [tunnel, setTunnel] = (0, import_react.useState)(void 0);
  const addTunnel = (element, node, context) => {
    const id = Math.random().toString(36).slice(2);
    setTunnel({ element, node, id, context });
    return id;
  };
  const removeTunnel = () => {
    if (tunnel) {
      try {
        tunnel.context.cleanup();
      } catch (e) {
        console.log("Error cleaning up leptos context:", e);
      }
    }
    setTunnel(void 0);
  };
  const TunnelRenderer = () => tunnel ? (0, import_react_dom.createPortal)(/* @__PURE__ */ (0, import_jsx_runtime.jsx)(FTMLContext.Provider, { value: tunnel.context, children: tunnel.node }), tunnel.element, tunnel.id) : /* @__PURE__ */ (0, import_jsx_runtime.jsx)(import_jsx_runtime.Fragment, {});
  (0, import_react.useEffect)(() => {
    return () => {
      if (tunnel) {
        try {
          tunnel.context.cleanup();
        } catch (e) {
          console.log("Error cleaning up leptos context:", e);
        }
      }
    };
  });
  return {
    addTunnel,
    removeTunnel,
    TunnelRenderer
  };
}
function useLeptosTunnels() {
  const [tunnels, setTunnels] = (0, import_react.useState)([]);
  const addTunnel = (element, node, context) => {
    const id = Math.random().toString(36).slice(2);
    setTunnels((prev) => [...prev, { element, node, id, context }]);
    return id;
  };
  const removeTunnel = (id) => {
    setTunnels((prev) => prev.filter((tunnel) => {
      if (tunnel.id === id) {
        try {
          tunnel.context.cleanup();
        } catch (e) {
          console.log("Error cleaning up leptos context:", e);
        }
      }
      return tunnel.id !== id;
    }));
  };
  const TunnelRenderer = () => /* @__PURE__ */ (0, import_jsx_runtime.jsx)(import_jsx_runtime.Fragment, { children: tunnels.map(
    (tunnel) => (0, import_react_dom.createPortal)(/* @__PURE__ */ (0, import_jsx_runtime.jsx)(FTMLContext.Provider, { value: tunnel.context, children: tunnel.node }), tunnel.element, tunnel.id)
  ) });
  (0, import_react.useEffect)(() => {
    return () => {
      tunnels.forEach((tunnel) => {
        try {
          tunnel.context.cleanup();
        } catch (e) {
          console.log("Error cleaning up leptos context:", e);
        }
      });
    };
  });
  return {
    addTunnel,
    removeTunnel,
    TunnelRenderer
  };
}

// src/index.tsx
var import_jsx_runtime2 = require("react/jsx-runtime");
var setServerUrl2 = FTMLT.setServerUrl;
var injectCss2 = FTMLT.injectCss;
var getServerUrl2 = FTMLT.getServerUrl;
var getFlamsServer2 = FTMLT.getFlamsServer;
var setDebugLog2 = FTMLT.setDebugLog;
var FTMLSetup = (args) => {
  const mountRef = (0, import_react2.useRef)(null);
  const main = useLeptosTunnel();
  const { addTunnel, TunnelRenderer } = useLeptosTunnels();
  (0, import_react2.useEffect)(() => {
    if (!mountRef.current) return;
    const handle = FTMLT.ftmlSetup(
      mountRef.current,
      (e, o) => {
        main.addTunnel(
          e,
          /* @__PURE__ */ (0, import_jsx_runtime2.jsxs)(import_jsx_runtime2.Fragment, { children: [
            args.children,
            /* @__PURE__ */ (0, import_jsx_runtime2.jsx)(TunnelRenderer, {})
          ] }),
          o
        );
      },
      toConfig(args, addTunnel)
    );
    return () => {
      handle.unmount();
    };
  }, []);
  return /* @__PURE__ */ (0, import_jsx_runtime2.jsxs)(import_jsx_runtime2.Fragment, { children: [
    /* @__PURE__ */ (0, import_jsx_runtime2.jsx)("div", { ref: mountRef, style: { display: "contents" } }),
    /* @__PURE__ */ (0, import_jsx_runtime2.jsx)(main.TunnelRenderer, {})
  ] });
};
var FTMLDocument = (args) => {
  const mountRef = (0, import_react2.useRef)(null);
  const { addTunnel, TunnelRenderer } = useLeptosTunnels();
  const context = (0, import_react2.useContext)(FTMLContext);
  (0, import_react2.useEffect)(() => {
    if (mountRef.current === null) return;
    const cont = context ? context.wasm_clone() : context;
    const handle = FTMLT.renderDocument(
      mountRef.current,
      args.document,
      cont,
      toConfig(args, addTunnel)
    );
    return () => {
      handle.unmount();
    };
  }, []);
  return /* @__PURE__ */ (0, import_jsx_runtime2.jsxs)("div", { style: { textAlign: "start" }, children: [
    /* @__PURE__ */ (0, import_jsx_runtime2.jsx)("div", { ref: mountRef }),
    /* @__PURE__ */ (0, import_jsx_runtime2.jsx)(TunnelRenderer, {})
  ] });
};
var FTMLFragment = (args) => {
  const mountRef = (0, import_react2.useRef)(null);
  const { addTunnel, TunnelRenderer } = useLeptosTunnels();
  const context = (0, import_react2.useContext)(FTMLContext);
  (0, import_react2.useEffect)(() => {
    if (!mountRef.current) return;
    const cont = context ? context.wasm_clone() : context;
    const handle = FTMLT.renderFragment(
      mountRef.current,
      args.fragment,
      cont,
      toConfig(args, addTunnel)
    );
    return () => {
      handle.unmount();
    };
  }, []);
  return /* @__PURE__ */ (0, import_jsx_runtime2.jsxs)("div", { style: { textAlign: "start" }, children: [
    /* @__PURE__ */ (0, import_jsx_runtime2.jsx)("div", { ref: mountRef }),
    /* @__PURE__ */ (0, import_jsx_runtime2.jsx)(TunnelRenderer, {})
  ] });
};
var ElemToReact = ({ elems, ctx }) => {
  const ref = (0, import_react2.useRef)(null);
  (0, import_react2.useEffect)(() => {
    if (ref.current) {
      ref.current.replaceChildren(...elems);
    }
  }, []);
  return /* @__PURE__ */ (0, import_jsx_runtime2.jsx)(FTMLContext.Provider, { value: ctx, children: /* @__PURE__ */ (0, import_jsx_runtime2.jsx)("div", { ref, style: { display: "contents" } }) });
};
function elemToReact(elem, ctx) {
  const chs = Array.from(elem.childNodes);
  chs.forEach((c) => elem.removeChild(c));
  return /* @__PURE__ */ (0, import_jsx_runtime2.jsx)(ElemToReact, { elems: chs, ctx });
}
function toConfig(config, addTunnel) {
  const otO = config.onSectionTitle;
  const onSectionTitle = otO ? (uri, lvl) => {
    const r = otO(uri, lvl);
    return r ? (elem, ctx) => {
      addTunnel(elem, r, ctx);
    } : void 0;
  } : void 0;
  const ofO = config.onFragment;
  const onFragment = ofO ? (uri, kind) => {
    const r = ofO(uri, kind);
    return r ? (elem, ctx) => {
      const ret = r(elemToReact(elem, ctx));
      return addTunnel(elem, ret, ctx);
    } : void 0;
  } : void 0;
  return {
    onSectionTitle,
    onFragment,
    problemStates: config.problemStates,
    onProblem: config.onProblem
  };
}
// Annotate the CommonJS export names for ESM import in node:
0 && (module.exports = {
  FTMLDocument,
  FTMLFragment,
  FTMLSetup,
  getFlamsServer,
  getServerUrl,
  injectCss,
  setDebugLog,
  setServerUrl
});
//# sourceMappingURL=index.cjs.map