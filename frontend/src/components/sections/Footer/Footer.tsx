import { FC } from "react";
import styles from "./Footer.module.scss";

export const Footer: FC = () => {
  return (
    <footer className={styles.footer}>
      <div className={styles.content}>
        <p>&copy; 2025 KEMBridge. Quantum-secured cross-chain bridge.</p>
      </div>
    </footer>
  );
};
