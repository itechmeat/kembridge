import { ButtonHTMLAttributes, FC, ReactNode } from "react";
import cn from "classnames";
import styles from "./Button.module.scss";

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "secondary" | "ghost";
  size?: "sm" | "md" | "lg";
  isLoading?: boolean;
  children: ReactNode;
  fullWidth?: boolean;
  iconOnly?: boolean;
}

export const Button: FC<ButtonProps> = ({
  variant = "primary",
  size = "md",
  isLoading = false,
  children,
  className = "",
  disabled,
  fullWidth = false,
  iconOnly = false,
  ...props
}) => {
  const classes = cn(
    styles.btn,
    {
      [styles.primary]: variant === "primary",
      [styles.secondary]: variant === "secondary",
      [styles.ghost]: variant === "ghost",
      [styles.sm]: size === "sm",
      [styles.md]: size === "md",
      [styles.lg]: size === "lg",
      [styles.loading]: isLoading,
      [styles.full]: fullWidth,
      [styles.icon]: iconOnly,
    },
    className?.trim()
  );

  return (
    <button className={classes} disabled={disabled || isLoading} {...props}>
      {isLoading ? (
        <span className={styles.spinner}>Loading...</span>
      ) : (
        children
      )}
    </button>
  );
};

export default Button;
