import { FC } from "react";
import { AuthStatus } from "../../components/auth";
import styles from "./AuthTestPage.module.scss";

export const AuthTestPage: FC = () => {
  return (
    <div className={styles.authTestPage}>
      <div className={styles.container}>
        <header className={styles.header}>
          <h1>Authentication Components Demo</h1>
          <p>Test the custom authentication UI components</p>
        </header>

        <div className={styles.content}>
          {/* Authentication Status */}
          <section className={styles.section}>
            <h2>Authentication Status</h2>
            <div className={styles.statusDemo}>
              <div className={styles.statusExample}>
                <AuthStatus showFullStatus />
              </div>
            </div>
          </section>
        </div>
      </div>
    </div>
  );
};
