import { useState, type CSSProperties } from "react";
import { Plus, Trash2, TriangleAlert } from "lucide-react";
import { Radio, RadioGroup } from "react-aria-components";
import { useTranslation } from "react-i18next";
import type { ColorPreset, PresetColors } from "../api";
import { customChoice, useColorPresets } from "../colorPreset";
import { BUILTIN_PRESETS, contrastIssues, modeColors } from "../palette";
import styles from "./ColorPresets.module.css";

/** Picks the colour preset and edits the ones the user created. */
function ColorPresets() {
  const { t } = useTranslation();
  const { presets, choice, select, create, update, remove } = useColorPresets();
  const active = presets?.find((preset) => customChoice(preset.id) === choice);

  return (
    <div className={styles.presets}>
      <RadioGroup
        className={styles.grid}
        aria-label={t("settings.colors.title")}
        value={choice}
        onChange={(value) => void select(value)}
      >
        {BUILTIN_PRESETS.map((preset) => (
          <PresetOption
            key={preset.name}
            value={preset.name}
            name={t(`settings.colors.builtin.${preset.name}`)}
            colors={preset.colors}
          />
        ))}
        {presets?.map((preset) => (
          <PresetOption
            key={preset.id}
            value={customChoice(preset.id)}
            name={preset.name}
            colors={preset.colors}
          />
        ))}
      </RadioGroup>

      <button type="button" className={styles.add} onClick={() => void create()}>
        <Plus className="icon-badge" aria-hidden="true" /> {t("settings.colors.add")}
      </button>

      {active && (
        <PresetEditor
          key={active.id}
          preset={active}
          onRename={(name) => update(active.id, { name })}
          onColors={(colors) => update(active.id, { colors })}
          onDelete={() => void remove(active)}
        />
      )}
    </div>
  );
}

interface OptionProps {
  value: string;
  name: string;
  colors: PresetColors;
}

/** A preset as a small preview of its light and dark side. */
function PresetOption({ value, name, colors }: OptionProps) {
  return (
    <Radio value={value} className={styles.option}>
      <span className={styles.preview} aria-hidden="true">
        <Swatch colors={colors} dark={false} />
        <Swatch colors={colors} dark />
      </span>
      <span className={styles.name}>{name}</span>
    </Radio>
  );
}

function Swatch({ colors, dark }: { colors: PresetColors; dark: boolean }) {
  const { accent, base } = modeColors(colors, dark);
  return (
    <span
      className={styles.swatch}
      data-dark={dark || undefined}
      style={{ "--swatch-accent": accent, "--swatch-base": base } as CSSProperties}
    >
      <span className={styles.swatchCard} />
      <span className={styles.swatchDot} />
    </span>
  );
}

interface EditorProps {
  preset: ColorPreset;
  onRename: (name: string) => void;
  onColors: (colors: PresetColors) => void;
  onDelete: () => void;
}

/** Name and colours of a custom preset; every change applies right away. */
function PresetEditor({ preset, onRename, onColors, onDelete }: EditorProps) {
  const { t } = useTranslation();
  const [name, setName] = useState(preset.name);

  function saveName() {
    const trimmed = name.trim();
    if (trimmed === "") setName(preset.name);
    else if (trimmed !== preset.name) onRename(trimmed);
  }

  const modes = [
    { dark: false, title: t("settings.themes.light"), accent: "lightAccent", base: "lightBase" },
    { dark: true, title: t("settings.themes.dark"), accent: "darkAccent", base: "darkBase" },
  ] as const;

  return (
    <div className={styles.editor}>
      <label className={styles.field}>
        <span className={styles.label}>{t("settings.colors.name")}</span>
        <input
          className={styles.text}
          value={name}
          onChange={(event) => setName(event.target.value)}
          onBlur={saveName}
          onKeyDown={(event) => {
            if (event.key === "Enter") event.currentTarget.blur();
          }}
        />
      </label>

      <div className={styles.modes}>
        {modes.map((mode) => {
          const issues = contrastIssues(modeColors(preset.colors, mode.dark), mode.dark);
          return (
            <fieldset key={mode.title} className={styles.mode}>
              <legend className={styles.legend}>{mode.title}</legend>
              <ColorField
                label={t("settings.colors.accent")}
                value={preset.colors[mode.accent]}
                onChange={(value) => onColors({ ...preset.colors, [mode.accent]: value })}
              />
              <ColorField
                label={t("settings.colors.base")}
                value={preset.colors[mode.base]}
                onChange={(value) => onColors({ ...preset.colors, [mode.base]: value })}
              />
              <div role="status" className={styles.issues}>
                {issues.map((issue) => (
                  <p key={issue} className={styles.issue}>
                    <TriangleAlert aria-hidden="true" />
                    {t(`settings.colors.issues.${issue}`)}
                  </p>
                ))}
              </div>
            </fieldset>
          );
        })}
      </div>

      <button type="button" className={styles.delete} onClick={onDelete}>
        <Trash2 aria-hidden="true" /> {t("settings.colors.delete")}
      </button>
    </div>
  );
}

interface FieldProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
}

function ColorField({ label, value, onChange }: FieldProps) {
  return (
    <label className={styles.color}>
      <input
        type="color"
        className={styles.picker}
        value={value}
        onChange={(event) => onChange(event.target.value)}
      />
      <span className={styles.label}>{label}</span>
      <span className={styles.hex}>{value}</span>
    </label>
  );
}

export default ColorPresets;
