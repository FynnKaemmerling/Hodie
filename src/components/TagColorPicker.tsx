import { Radio, RadioGroup } from "react-aria-components";
import { useTranslation } from "react-i18next";
import { TAG_COLORS, type TagColor } from "../api";
import styles from "./TagColorPicker.module.css";

interface Props {
  label: string;
  value: TagColor;
  onChange: (color: TagColor) => void;
}

/** Round swatches for the tag palette; arrow keys move between them. */
function TagColorPicker({ label, value, onChange }: Props) {
  const { t } = useTranslation();

  return (
    <RadioGroup
      className={styles.colors}
      orientation="horizontal"
      aria-label={label}
      value={value}
      onChange={(color) => onChange(color as TagColor)}
    >
      {TAG_COLORS.map((color) => (
        <Radio
          key={color}
          value={color}
          className={styles.swatch}
          data-color={color}
          aria-label={t(`tags.colors.${color}`)}
        />
      ))}
    </RadioGroup>
  );
}

export default TagColorPicker;
