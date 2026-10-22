import type { ReactNode } from "react";
import type { TagColor } from "../api";
import styles from "./TagChip.module.css";

interface Props {
  color: TagColor;
  size?: "small" | "medium";
  children: ReactNode;
}

function TagChip({ color, size = "medium", children }: Props) {
  return (
    <span className={styles.chip} data-color={color} data-size={size}>
      {children}
    </span>
  );
}

export default TagChip;
