import React from "react";

export interface InputProps
  extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  helperText?: string;
  variant?: "default" | "filled" | "outlined";
}

export const Input: React.FC<InputProps> = ({
  label,
  error,
  helperText,
  variant = "default",
  className = "",
  id,
  ...props
}) => {
  const inputId = id || `input-${Math.random().toString(36).substr(2, 9)}`;

  const baseClasses = "input";
  const variantClasses = `input--${variant}`;
  const errorClasses = error ? "input--error" : "";

  const classes = [baseClasses, variantClasses, errorClasses, className]
    .filter(Boolean)
    .join(" ");

  return (
    <div className="input-wrapper">
      {label && (
        <label htmlFor={inputId} className="input__label">
          {label}
        </label>
      )}
      <input id={inputId} className={classes} {...props} />
      {error && (
        <span className="input__error" role="alert">
          {error}
        </span>
      )}
      {helperText && !error && (
        <span className="input__helper">{helperText}</span>
      )}
    </div>
  );
};

export default Input;
