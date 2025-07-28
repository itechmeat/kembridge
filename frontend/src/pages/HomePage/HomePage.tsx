/**
 * Home Page Component
 * Main landing page with hero, features, stats, and footer
 */

import { FC, useEffect } from "react";
import { useWallet } from "../../hooks/wallet/useWallet";
import { useAuth } from "../../hooks/auth/useAuth";
import { Hero } from "../../components/sections/Hero/Hero";
import { Features } from "../../components/sections/Features/Features";
import { Stats } from "../../components/sections/Stats/Stats";
import { Footer } from "../../components/sections/Footer/Footer";
import { initializeWalletService } from "../../services/wallet";
import { validateConfig } from "../../config/env";
import "./HomePage.scss";

export const HomePage: FC = () => {
  const { autoConnect } = useWallet();
  const { checkBackendHealth } = useAuth();

  // Initialize wallet service and auto-connect on component mount
  useEffect(() => {
    // Validate configuration first
    validateConfig();

    initializeWalletService();
    autoConnect();
    checkBackendHealth();
  }, [autoConnect, checkBackendHealth]);

  return (
    <div className="home-page">
      <Hero />
      <Features />
      <Stats />
      <Footer />
    </div>
  );
};
