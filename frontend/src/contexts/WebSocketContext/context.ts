import { createContext } from "react";
import type {
  BridgeWebSocketState,
  BridgeWebSocketActions,
} from "../../hooks/bridge/useBridgeWebSocket";

export interface WebSocketContextType
  extends BridgeWebSocketState,
    BridgeWebSocketActions {}

export const WebSocketContext = createContext<WebSocketContextType | null>(
  null
);
