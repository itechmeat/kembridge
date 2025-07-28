import React from "react";
import "./Spinner.scss";

export interface SpinnerProps {
  size?: "sm" | "md" | "lg";
  color?: "primary" | "secondary" | "white";
  className?: string;
}

export const Spinner: React.FC<SpinnerProps> = ({
  size = "md",
  color = "primary",
  className = "",
}) => {
  const baseClasses = "spinner";
  const sizeClasses = `spinner--${size}`;
  const colorClasses = `spinner--${color}`;

  const classes = [baseClasses, sizeClasses, colorClasses, className]
    .filter(Boolean)
    .join(" ");

  return (
    <div className={classes} role="status" aria-label="Loading">
      <div className="spinner__circle"></div>
      <span className="sr-only">Loading...</span>
    </div>
  );
};

export default Spinner;
