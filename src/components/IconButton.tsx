import type { ButtonHTMLAttributes, Ref } from "react";
import styles from "./IconButton.module.css";

/** Class for icon-only buttons that are not plain buttons (e.g. React Aria's). */
export const iconButtonClass = styles.button;

interface Props extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, "aria-label"> {
  /** Read by screen readers and shown as tooltip; the button has no text. */
  label: string;
  tone?: "danger";
  active?: boolean;
  ref?: Ref<HTMLButtonElement>;
}

/** A button that shows only its icon. */
function IconButton({ label, tone, active, className, children, ...rest }: Props) {
  return (
    <button
      type="button"
      className={`${styles.button} ${className ?? ""}`}
      aria-label={label}
      title={label}
      data-tone={tone}
      data-active={active || undefined}
      {...rest}
    >
      {children}
    </button>
  );
}

export default IconButton;
