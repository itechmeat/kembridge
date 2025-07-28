/**
 * Features Section Component
 * Displays key product features
 */

import { FC } from "react";
import { FEATURES } from "../../../constants";
import "./Features.scss";

export const Features: FC = () => {
  return (
    <section className="features">
      <div className="features__container">
        <h2 className="features__title">Revolutionary Security</h2>

        <div className="features__grid">
          {FEATURES.map((feature) => (
            <div key={feature.id} className="feature-card">
              <div className="feature-card__icon">{feature.icon}</div>
              <h3 className="feature-card__title">{feature.title}</h3>
              <p className="feature-card__description">{feature.description}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
};
