import { FC } from "react";
import { FEATURES } from "../../../constants";
import styles from "./Features.module.scss";

export const Features: FC = () => {
  return (
    <section className={styles.features}>
      <div className={styles.container}>
        <h2 className={styles.title}>Revolutionary Security</h2>

        <div className={styles.grid}>
          {FEATURES.map((feature) => (
            <div key={feature.id} className={styles.featureCard}>
              <div className={styles.icon}>{feature.icon}</div>
              <h3 className={styles.cardTitle}>{feature.title}</h3>
              <p className={styles.description}>{feature.description}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
};
