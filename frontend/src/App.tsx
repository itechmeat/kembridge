import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import { WalletProviders } from "./providers/WalletProviders";
import { NearWalletProvider } from "./contexts/NearWalletContext.tsx";
import { useNearWallet } from "./hooks/wallet/useNearWallet";
import { setNearWalletContext } from "./services/wallet/providers/near";
import { MobileLayout } from "./components/layout/MobileLayout/MobileLayout";
import { WalletPage } from "./pages/WalletPage/WalletPage";
import { BridgePage } from "./pages/BridgePage/BridgePage";
import { AuthTestPage } from "./pages/AuthTestPage/AuthTestPage";
import { SecurityTestPage } from "./pages/SecurityTestPage/SecurityTestPage";
import { setupGlobalErrorHandlers } from "./utils/errorHandler";
import { useAuthInit } from "./hooks/api/useAuth";
import ErrorNotificationDisplay from "./components/notifications/ErrorNotificationDisplay";
import { useEffect, useCallback, useRef } from "react";
import "./styles/main.scss";
import "./pages/BridgePage/BridgePage.scss";
import "@rainbow-me/rainbowkit/styles.css";

function AppContent() {
  console.log("🏗️ App: AppContent component rendering");
  const nearWallet = useNearWallet();
  const authInit = useAuthInit();
  const previousNearWalletRef = useRef(nearWallet);

  // Memoized callback to prevent unnecessary re-renders
  const updateNearContext = useCallback(() => {
    const current = nearWallet;
    const previous = previousNearWalletRef.current;
    
    // Only update if something actually changed
    if (
      current.selector !== previous.selector ||
      current.modal !== previous.modal ||
      current.accountId !== previous.accountId ||
      current.isConnected !== previous.isConnected
    ) {
      console.log("🔗 App: Connecting NEAR context to provider...");
      console.log("📊 App: NEAR context data:", {
        selector: !!current.selector,
        modal: !!current.modal,
        accountId: current.accountId,
        isConnected: current.isConnected,
      });

      setNearWalletContext(current);
      console.log("✅ App: NEAR context connected to provider");
      previousNearWalletRef.current = current;
    }
  }, [nearWallet]);

  useEffect(() => {
    updateNearContext();
  }, [updateNearContext]);

  // Log authentication initialization status
  useEffect(() => {
    if (authInit.isInitialized) {
      console.log("🔐 App: Authentication initialized", {
        isAuthenticated: authInit.isAuthenticated,
      });
    }
  }, [authInit.isInitialized, authInit.isAuthenticated]);

  return (
    <WalletProviders>
      <Router>
        <MobileLayout>
          <Routes>
            <Route path="/" element={<WalletPage />} />
            <Route path="/bridge" element={<BridgePage />} />
            <Route path="/swap" element={<BridgePage />} />
            <Route
              path="/history"
              element={
                <div style={{ padding: "2rem", textAlign: "center" }}>
                  History Page Coming Soon
                </div>
              }
            />
            <Route
              path="/settings"
              element={
                <div style={{ padding: "2rem", textAlign: "center" }}>
                  Settings Page Coming Soon
                </div>
              }
            />
            <Route path="/auth-test" element={<AuthTestPage />} />
            <Route path="/security-test" element={<SecurityTestPage />} />
          </Routes>
        </MobileLayout>

        {/* Global Error Notification Display */}
        <ErrorNotificationDisplay maxVisible={5} position="top-right" />
      </Router>
    </WalletProviders>
  );
}

function App() {
  useEffect(() => {
    setupGlobalErrorHandlers();
    console.log("🚀 App: Initializing application...");
    console.log("📦 App: Setting up NEAR wallet provider...");
  }, []);

  return (
    <NearWalletProvider>
      <AppContent />
    </NearWalletProvider>
  );
}

export default App;
