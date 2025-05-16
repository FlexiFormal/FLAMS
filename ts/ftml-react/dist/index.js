// src/index.tsx
import * as FTMLT from "@kwarc/ftml-viewer";
import { useContext, useEffect as useEffect2, useRef } from "react";

// src/leptos.tsx
import { createContext, useEffect, useState } from "react";
import { createPortal } from "react-dom";
import { Fragment, jsx } from "react/jsx-runtime";
var FTMLContext = createContext(void 0);
function useLeptosTunnel() {
  const [tunnel, setTunnel] = useState(void 0);
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
  const TunnelRenderer = () => tunnel ? createPortal(/* @__PURE__ */ jsx(FTMLContext.Provider, { value: tunnel.context, children: tunnel.node }), tunnel.element, tunnel.id) : /* @__PURE__ */ jsx(Fragment, {});
  useEffect(() => {
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
  const [tunnels, setTunnels] = useState([]);
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
  const TunnelRenderer = () => /* @__PURE__ */ jsx(Fragment, { children: tunnels.map(
    (tunnel) => createPortal(/* @__PURE__ */ jsx(FTMLContext.Provider, { value: tunnel.context, children: tunnel.node }), tunnel.element, tunnel.id)
  ) });
  useEffect(() => {
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
import { Fragment as Fragment2, jsx as jsx2, jsxs } from "react/jsx-runtime";
var setServerUrl2 = FTMLT.setServerUrl;
var injectCss2 = FTMLT.injectCss;
var getServerUrl2 = FTMLT.getServerUrl;
var getFlamsServer2 = FTMLT.getFlamsServer;
var setDebugLog2 = FTMLT.setDebugLog;
var FTMLSetup = (args) => {
  const mountRef = useRef(null);
  const main = useLeptosTunnel();
  const { addTunnel, TunnelRenderer } = useLeptosTunnels();
  useEffect2(() => {
    if (!mountRef.current) return;
    const handle = FTMLT.ftmlSetup(
      mountRef.current,
      (e, o) => {
        main.addTunnel(
          e,
          /* @__PURE__ */ jsxs(Fragment2, { children: [
            args.children,
            /* @__PURE__ */ jsx2(TunnelRenderer, {})
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
  return /* @__PURE__ */ jsxs(Fragment2, { children: [
    /* @__PURE__ */ jsx2("div", { ref: mountRef, style: { display: "contents" } }),
    /* @__PURE__ */ jsx2(main.TunnelRenderer, {})
  ] });
};
var FTMLDocument = (args) => {
  const mountRef = useRef(null);
  const { addTunnel, TunnelRenderer } = useLeptosTunnels();
  const context = useContext(FTMLContext);
  useEffect2(() => {
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
  return /* @__PURE__ */ jsxs("div", { style: { textAlign: "start" }, children: [
    /* @__PURE__ */ jsx2("div", { ref: mountRef }),
    /* @__PURE__ */ jsx2(TunnelRenderer, {})
  ] });
};
var FTMLFragment = (args) => {
  const mountRef = useRef(null);
  const { addTunnel, TunnelRenderer } = useLeptosTunnels();
  const context = useContext(FTMLContext);
  useEffect2(() => {
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
  return /* @__PURE__ */ jsxs("div", { style: { textAlign: "start" }, children: [
    /* @__PURE__ */ jsx2("div", { ref: mountRef }),
    /* @__PURE__ */ jsx2(TunnelRenderer, {})
  ] });
};
var ElemToReact = ({ elems, ctx }) => {
  const ref = useRef(null);
  useEffect2(() => {
    if (ref.current) {
      ref.current.replaceChildren(...elems);
    }
  }, []);
  return /* @__PURE__ */ jsx2(FTMLContext.Provider, { value: ctx, children: /* @__PURE__ */ jsx2("div", { ref, style: { display: "contents" } }) });
};
function elemToReact(elem, ctx) {
  const chs = Array.from(elem.childNodes);
  chs.forEach((c) => elem.removeChild(c));
  return /* @__PURE__ */ jsx2(ElemToReact, { elems: chs, ctx });
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
export {
  FTMLDocument,
  FTMLFragment,
  FTMLSetup,
  getFlamsServer2 as getFlamsServer,
  getServerUrl2 as getServerUrl,
  injectCss2 as injectCss,
  setDebugLog2 as setDebugLog,
  setServerUrl2 as setServerUrl
};
//# sourceMappingURL=index.js.map