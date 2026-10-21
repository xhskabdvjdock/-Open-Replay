import React from "react";

export type ButtonVariant = "primary" | "secondary" | "ghost" | "danger";
export type ButtonSize = "sm" | "md" | "lg";

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  loading?: boolean;
  icon?: React.ReactNode;
}

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ variant = "secondary", size = "md", loading = false, icon, children, disabled, className = "", ...rest }, ref) => {
    const base = `btn btn-${variant} btn-${size}${className ? " " + className : ""}`;
    return (
      <button ref={ref} className={base} disabled={disabled || loading} {...rest}>
        {loading ? <span className="btn-spinner" /> : icon ? <span className="btn-icon">{icon}</span> : null}
        {children && <span className="btn-label">{children}</span>}
      </button>
    );
  }
);
Button.displayName = "Button";
