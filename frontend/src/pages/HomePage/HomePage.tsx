import { FC, useEffect } from "react";
import { Hero } from "../../components/sections/Hero/Hero";
import { Features } from "../../components/sections/Features/Features";
import { Stats } from "../../components/sections/Stats/Stats";
import { Footer } from "../../components/sections/Footer/Footer";
import { validateConfig } from "../../config/env";
import styles from "./HomePage.module.scss";

export const HomePage: FC = () => {
  useEffect(() => {
    validateConfig();
  }, []);

  return (
    <div className={styles.homePage}>
      <Hero />
      <Features />
      <Stats />
      <Footer />
    </div>
  );
};
