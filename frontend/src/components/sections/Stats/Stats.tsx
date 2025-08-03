import { FC } from "react";
import { STATS } from "../../../constants";
import styles from "./Stats.module.scss";

export const Stats: FC = () => {
  return (
    <section className={styles.stats}>
      <div className={styles.container}>
        <div className={styles.grid}>
          {STATS.map((stat) => (
            <div key={stat.id} className={styles.statItem}>
              <div className={styles.value}>{stat.value}</div>
              <div className={styles.label}>{stat.label}</div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
};
