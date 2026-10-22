import { Check, Plus, X } from "lucide-react";
import { useState } from "react";
import {
  Button,
  Dialog,
  DialogTrigger,
  Popover,
  Radio,
  RadioGroup,
} from "react-aria-components";
import { useTranslation } from "react-i18next";
import { TAG_COLORS, type Id, type Tag, type TagColor } from "../api";
import { filterTags, findTagByName, nextTagColor } from "../board/cardState";
import TagChip from "./TagChip";
import styles from "./TagPicker.module.css";

interface Props {
  tags: Tag[];
  selectedIds: Id[];
  onToggle: (tagId: Id) => void;
  onCreate: (name: string, color: TagColor) => void;
}

/** The card's tags as chips, plus a popover to add, remove and create tags. */
function TagPicker({ tags, selectedIds, onToggle, onCreate }: Props) {
  const { t } = useTranslation();
  const selected = selectedIds
    .map((id) => tags.find((tag) => tag.id === id))
    .filter((tag): tag is Tag => tag !== undefined);

  return (
    <div className={styles.chips}>
      {selected.map((tag) => (
        <TagChip key={tag.id} color={tag.color}>
          {tag.name}
          <button
            type="button"
            className={styles.remove}
            aria-label={t("tags.remove", { name: tag.name })}
            onClick={() => onToggle(tag.id)}
          >
            <X aria-hidden="true" />
          </button>
        </TagChip>
      ))}
      <DialogTrigger>
        <Button className={styles.addButton}>
          <Plus aria-hidden="true" /> {t("tags.add")}
        </Button>
        <Popover className={styles.popover} placement="bottom start" offset={4}>
          <Dialog className={styles.dialog} aria-label={t("tags.title")}>
            <TagSearch tags={tags} selectedIds={selectedIds} onToggle={onToggle} onCreate={onCreate} />
          </Dialog>
        </Popover>
      </DialogTrigger>
    </div>
  );
}

function TagSearch({ tags, selectedIds, onToggle, onCreate }: Props) {
  const { t } = useTranslation();
  const [query, setQuery] = useState("");
  const [color, setColor] = useState<TagColor>(() => nextTagColor(tags));
  const name = query.trim();
  const exact = findTagByName(tags, name);
  const matches = filterTags(tags, query);
  const canCreate = name !== "" && !exact;

  function create() {
    if (!canCreate) return;
    onCreate(name, color);
    setQuery("");
    setColor(nextTagColor(tags));
  }

  return (
    <>
      <input
        className={styles.search}
        value={query}
        autoFocus
        placeholder={t("tags.searchPlaceholder")}
        aria-label={t("tags.search")}
        onChange={(event) => setQuery(event.target.value)}
        onKeyDown={(event) => {
          if (event.key !== "Enter") return;
          event.preventDefault();
          if (exact) {
            onToggle(exact.id);
            setQuery("");
          } else {
            create();
          }
        }}
      />

      {matches.length > 0 && (
        <ul className={styles.options}>
          {matches.map((tag) => {
            const isSelected = selectedIds.includes(tag.id);
            return (
              <li key={tag.id}>
                <button
                  type="button"
                  className={styles.option}
                  aria-pressed={isSelected}
                  onClick={() => onToggle(tag.id)}
                >
                  <span className={styles.check} aria-hidden="true">
                    {isSelected && <Check />}
                  </span>
                  <TagChip color={tag.color}>{tag.name}</TagChip>
                </button>
              </li>
            );
          })}
        </ul>
      )}

      {tags.length === 0 && name === "" && <p className={styles.hint}>{t("tags.empty")}</p>}

      {canCreate && (
        <div className={styles.create}>
          <RadioGroup
            className={styles.colors}
            orientation="horizontal"
            aria-label={t("tags.color")}
            value={color}
            onChange={(value) => setColor(value as TagColor)}
          >
            {TAG_COLORS.map((c) => (
              <Radio key={c} value={c} className={styles.swatch} data-color={c} aria-label={t(`tags.colors.${c}`)} />
            ))}
          </RadioGroup>
          <button type="button" className={styles.createButton} onClick={create}>
            {t("tags.create", { name })}
          </button>
        </div>
      )}
    </>
  );
}

export default TagPicker;
