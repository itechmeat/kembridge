import { FC } from "react";
import cn from "classnames";
import styles from "./Spinner.module.scss";

export interface SpinnerProps {
  size?: "sm" | "md" | "lg";
  color?: "primary" | "secondary" | "white";
  className?: string;
}

export const Spinner: FC<SpinnerProps> = ({
  size = "md",
  color = "primary",
  className = "",
}) => {
  const classes = cn(
    styles.spinner,
    {
      [styles.sm]: size === "sm",
      [styles.md]: size === "md",
      [styles.lg]: size === "lg",
      [styles.primary]: color === "primary",
      [styles.secondary]: color === "secondary",
      [styles.white]: color === "white",
    },
    className?.trim()
  );

  return (
    <div className={classes} role="status" aria-label="Loading">
      <div className={styles.circle}></div>
      <span className="sr-only">Loading...</span>
    </div>
  );
};

export default Spinner;
