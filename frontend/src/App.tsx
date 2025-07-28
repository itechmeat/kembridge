import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import { WalletProviders } from "./providers/WalletProviders";
import { NearWalletProvider } from "./contexts/NearWalletContext.tsx";
import { useNearWallet } from "./hooks/wallet/useNearWallet";
import { setNearWalletContext } from "./services/wallet/providers/near";
import { MobileLayout } from "./components/layout/MobileLayout/MobileLayout";
import { WalletPage } from "./pages/WalletPage/WalletPage";
import { setupGlobalErrorHandlers } from "./utils/errorHandler";
import { useEffect } from "react";
import "./styles/main.scss";
import "@rainbow-me/rainbowkit/styles.css";

function AppContent() {
  console.log("🏗️ App: AppContent component rendering");
  const nearWallet = useNearWallet();

  useEffect(() => {
    console.log("🔗 App: Connecting NEAR context to provider...");
    console.log("📊 App: NEAR context data:", {
      selector: !!nearWallet.selector,
      modal: !!nearWallet.modal,
      accountId: nearWallet.accountId,
      isConnected: nearWallet.isConnected,
    });

    // Connect NEAR context to provider
    setNearWalletContext(nearWallet);
    console.log("✅ App: NEAR context connected to provider");
  }, [nearWallet]);

  return (
    <WalletProviders>
      <Router>
        <MobileLayout>
          <Routes>
            <Route path="/" element={<WalletPage />} />
            <Route
              path="/swap"
              element={
                <div style={{ padding: "2rem", textAlign: "center" }}>
                  Swap Page Coming Soon
                </div>
              }
            />
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
          </Routes>
        </MobileLayout>
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
