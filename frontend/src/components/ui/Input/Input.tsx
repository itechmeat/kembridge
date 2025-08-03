import { FC, InputHTMLAttributes, useId } from "react";
import cn from "classnames";
import styles from "./Input.module.scss";

export interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  helperText?: string;
  variant?: "default" | "filled" | "outlined";
}

export const Input: FC<InputProps> = ({
  label,
  error,
  helperText,
  variant = "default",
  className = "",
  id,
  ...props
}) => {
  const reactId = useId();
  const inputId = id || `input-${reactId}`;

  const variantClass =
    variant === "filled"
      ? styles.variantFilled
      : variant === "outlined"
      ? styles.variantOutlined
      : styles.variantDefault;

  const classes = cn(
    styles.input,
    variantClass,
    { [styles.hasError]: Boolean(error) },
    className?.toString().trim()
  );

  return (
    <div className={styles.wrapper}>
      {label && (
        <label htmlFor={inputId} className={styles.label}>
          {label}
        </label>
      )}
      <input id={inputId} className={classes} {...props} />
      {error ? (
        <span className={styles.error} role="alert">
          {error}
        </span>
      ) : (
        helperText && <span className={styles.helper}>{helperText}</span>
      )}
    </div>
  );
};

export default Input;
