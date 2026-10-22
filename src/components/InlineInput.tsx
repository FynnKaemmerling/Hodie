import { useRef, useState } from "react";
import styles from "./InlineInput.module.css";

interface Props {
  label: string;
  initialValue?: string;
  placeholder?: string;
  /** Called with the trimmed text when it is non-empty and changed. */
  onSubmit: (value: string) => void;
  onClose: () => void;
  /** Stay open and empty after Enter, to add several items in a row. */
  keepOpen?: boolean;
  className?: string;
}

/**
 * Text field for creating or renaming in place. Enter or leaving the field
 * saves, Escape cancels.
 */
function InlineInput({
  label,
  initialValue = "",
  placeholder,
  onSubmit,
  onClose,
  keepOpen = false,
  className,
}: Props) {
  const [value, setValue] = useState(initialValue);
  const closed = useRef(false);

  function submit(): boolean {
    const trimmed = value.trim();
    if (trimmed === "" || trimmed === initialValue.trim()) return false;
    onSubmit(trimmed);
    return true;
  }

  function close() {
    if (closed.current) return;
    closed.current = true;
    onClose();
  }

  return (
    <input
      className={`${styles.input} ${className ?? ""}`}
      aria-label={label}
      placeholder={placeholder}
      value={value}
      autoFocus
      onFocus={(event) => event.currentTarget.select()}
      onChange={(event) => setValue(event.target.value)}
      // Keep keys and pointer events from reaching drag handles and shortcuts.
      onPointerDown={(event) => event.stopPropagation()}
      onKeyDown={(event) => {
        event.stopPropagation();
        if (event.key === "Escape") {
          event.preventDefault();
          close();
        } else if (event.key === "Enter") {
          event.preventDefault();
          if (submit() && keepOpen) setValue("");
          else close();
        }
      }}
      onBlur={() => {
        if (closed.current) return;
        submit();
        close();
      }}
    />
  );
}

export default InlineInput;
