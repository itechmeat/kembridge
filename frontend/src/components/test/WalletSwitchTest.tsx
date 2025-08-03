import React from "react";

// Simple test component to verify wallet switching
export const WalletSwitchTest: React.FC = () => {
  const [logs, setLogs] = React.useState<string[]>([]);

  React.useEffect(() => {
    const handleAuthEvent = (e: Event) => {
      const detail = (e as CustomEvent).detail;
      const timestamp = new Date().toLocaleTimeString();
      setLogs((prev) => [
        ...prev,
        `${timestamp}: Auth changed - ${JSON.stringify(detail)}`,
      ]);
    };

    const handleStorageEvent = (e: StorageEvent) => {
      if (e.key?.includes("kembridge")) {
        const timestamp = new Date().toLocaleTimeString();
        setLogs((prev) => [
          ...prev,
          `${timestamp}: Storage changed - ${e.key}: ${
            e.newValue ? "SET" : "CLEARED"
          }`,
        ]);
      }
    };

    window.addEventListener("auth-token-changed", handleAuthEvent);
    window.addEventListener("storage", handleStorageEvent);

    return () => {
      window.removeEventListener("auth-token-changed", handleAuthEvent);
      window.removeEventListener("storage", handleStorageEvent);
    };
  }, []);

  return (
    <div style={{ padding: "1rem", maxWidth: "600px", margin: "0 auto" }}>
      <h3>Wallet Auth Event Monitor</h3>
      <div
        style={{
          height: "300px",
          overflow: "auto",
          border: "1px solid #ccc",
          padding: "1rem",
        }}
      >
        {logs.map((log, index) => (
          <div
            key={index}
            style={{ marginBottom: "0.5rem", fontSize: "0.8rem" }}
          >
            {log}
          </div>
        ))}
      </div>
      <button onClick={() => setLogs([])}>Clear Logs</button>
    </div>
  );
};
