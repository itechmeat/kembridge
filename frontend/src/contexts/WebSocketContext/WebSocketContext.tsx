import { useEffect, ReactNode, FC } from "react";
import { useBridgeWebSocket } from "../../hooks/bridge/useBridgeWebSocket";
import { WebSocketContext } from "./context";

interface WebSocketProviderProps {
  children: ReactNode;
}

export const WebSocketProvider: FC<WebSocketProviderProps> = ({ children }) => {
  const webSocketState = useBridgeWebSocket();

  useEffect(() => {
    console.log("🌐 WebSocket Provider: Initializing WebSocket connection");

    return () => {
      console.log("🌐 WebSocket Provider: Cleaning up WebSocket connection");
    };
  }, []);

  return (
    <WebSocketContext.Provider value={webSocketState}>
      {children}
    </WebSocketContext.Provider>
  );
};
