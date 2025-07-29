/**
 * Home Page Component
 * Main landing page with hero, features, stats, and footer
 */

import { FC, useEffect } from "react";
import { useWallet } from "../../hooks/wallet/useWallet";
import { useAuthStatus } from "../../hooks/api/useAuth";
import { Hero } from "../../components/sections/Hero/Hero";
import { Features } from "../../components/sections/Features/Features";
import { Stats } from "../../components/sections/Stats/Stats";
import { Footer } from "../../components/sections/Footer/Footer";
import { validateConfig } from "../../config/env";
import "./HomePage.scss";

export const HomePage: FC = () => {
  const { state } = useWallet();
  const { isAuthenticated } = useAuthStatus();

  // Validate configuration on component mount
  useEffect(() => {
    validateConfig();
  }, []);

  return (
    <div className="home-page">
      <Hero />
      <Features />
      <Stats />
      <Footer />
    </div>
  );
};
