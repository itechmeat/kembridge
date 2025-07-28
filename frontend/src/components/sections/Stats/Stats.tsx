/**
 * Stats Section Component
 * Displays key performance metrics
 */

import { FC } from "react";
import { STATS } from "../../../constants";
import "./Stats.scss";

export const Stats: FC = () => {
  return (
    <section className="stats">
      <div className="stats__container">
        <div className="stats__grid">
          {STATS.map((stat) => (
            <div key={stat.id} className="stat-item">
              <div className="stat-item__value">{stat.value}</div>
              <div className="stat-item__label">{stat.label}</div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
};
