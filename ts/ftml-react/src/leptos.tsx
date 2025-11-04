import { createContext, ReactNode, useState } from "react";
import { createPortal } from "react-dom";
import { FTML as Base } from "@kwarc/ftml-viewer";


export const FTMLContext = createContext<Base.LeptosContext | undefined>(undefined);

interface Tunnel {
  element: Element;
  node: ReactNode;
  context:Base.LeptosContext;
  id: string; // for React keys
}


export function useLeptosTunnel() {
  const [tunnel, setTunnel] = useState<Tunnel | undefined>(undefined);

  const addTunnel = (element: Element, node: ReactNode, context:Base.LeptosContext) => {
    const id = Math.random().toString(36).slice(2);
    setTunnel({ element, node, id, context });
    return id; // Return id for later removal
  };

  const removeTunnel = () => {
    setTunnel(undefined);
  };

  const TunnelRenderer = () => (
      tunnel? 
        createPortal(<FTMLContext.Provider value={tunnel.context}>{tunnel.node}</FTMLContext.Provider>, tunnel.element, tunnel.id)
        : <></>
  );

  return {
    addTunnel,
    removeTunnel,
    TunnelRenderer
  };
}

export function useLeptosTunnels() {
  const [tunnels, setTunnels] = useState<Tunnel[]>([]);

  const addTunnel = (element: Element, node: ReactNode, context:Base.LeptosContext) => {
    const id = Math.random().toString(36).slice(2);
    setTunnels(prev => [...prev, { element, node, id, context }]);
    return id; // Return id for later removal
  };

  const removeTunnel = (id: string) => {
    setTunnels(prev => prev.filter(tunnel => {
      return tunnel.id !== id
    }));
  };

  const TunnelRenderer = () => (
    <>
      {tunnels.map(tunnel => 
        createPortal(<FTMLContext.Provider value={tunnel.context}>{tunnel.node}</FTMLContext.Provider>, tunnel.element, tunnel.id)
      )}
    </>
  );

  return {
    addTunnel,
    removeTunnel,
    TunnelRenderer
  };
}