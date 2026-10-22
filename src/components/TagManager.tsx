import { useState } from "react";
import { Button, Dialog, DialogTrigger, Popover } from "react-aria-components";
import { Plus, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { api, type Id, type Tag, type TagColor, type TagPatch } from "../api";
import { errorMessageKey } from "../api/errors";
import { nextTagColor } from "../board/cardState";
import InlineInput from "./InlineInput";
import TagChip from "./TagChip";
import TagColorPicker from "./TagColorPicker";
import styles from "./TagManager.module.css";

interface Props {
  tags: Tag[];
  onCreate: (name: string, color: TagColor) => void;
  onUpdate: (tagId: Id, patch: TagPatch) => void;
  onDelete: (tag: Tag) => void;
}

/** All tags with rename, recolour and delete. Changes apply to every card. */
function TagManager({ tags, onCreate, onUpdate, onDelete }: Props) {
  const { t } = useTranslation();
  const [adding, setAdding] = useState(false);

  return (
    <div className={styles.manager}>
      {tags.length === 0 ? (
        <p className={styles.empty}>{t("settings.tags.empty")}</p>
      ) : (
        <ul className={styles.list}>
          {tags.map((tag) => (
            <TagRow key={tag.id} tag={tag} onUpdate={onUpdate} onDelete={onDelete} />
          ))}
        </ul>
      )}

      {adding ? (
        <InlineInput
          label={t("settings.tags.name")}
          keepOpen
          onSubmit={(name) => onCreate(name, nextTagColor(tags))}
          onClose={() => setAdding(false)}
        />
      ) : (
        <button type="button" className={styles.add} onClick={() => setAdding(true)}>
          <Plus aria-hidden="true" /> {t("settings.tags.add")}
        </button>
      )}
    </div>
  );
}

interface RowProps {
  tag: Tag;
  onUpdate: (tagId: Id, patch: TagPatch) => void;
  onDelete: (tag: Tag) => void;
}

function TagRow({ tag, onUpdate, onDelete }: RowProps) {
  const { t } = useTranslation();
  const [renaming, setRenaming] = useState(false);
  // Cards the tag would be removed from, while asking for confirmation.
  const [confirmCount, setConfirmCount] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Deleting an unused tag loses nothing, so only ask when cards carry it.
  async function requestDelete() {
    try {
      const count = await api.countTagCards(tag.id);
      if (count === 0) onDelete(tag);
      else setConfirmCount(count);
    } catch (failure) {
      setError(t(errorMessageKey(failure)));
    }
  }

  if (confirmCount !== null) {
    return (
      <li className={styles.row} data-confirm>
        <TagChip color={tag.color}>{tag.name}</TagChip>
        <span className={styles.question} role="alert">
          {t("settings.tags.confirm", { count: confirmCount })}
        </span>
        <button type="button" className={styles.danger} autoFocus onClick={() => onDelete(tag)}>
          {t("settings.tags.confirmDelete")}
        </button>
        <button type="button" className={styles.cancel} onClick={() => setConfirmCount(null)}>
          {t("settings.tags.cancel")}
        </button>
      </li>
    );
  }

  return (
    <li className={styles.row}>
      <DialogTrigger>
        <Button
          className={styles.colorButton}
          data-color={tag.color}
          aria-label={t("settings.tags.color", { name: tag.name })}
        />
        <Popover className={styles.popover} placement="bottom start" offset={4}>
          <Dialog className={styles.colorDialog} aria-label={t("settings.tags.color", { name: tag.name })}>
            <TagColorPicker
              label={t("settings.tags.color", { name: tag.name })}
              value={tag.color}
              onChange={(color) => onUpdate(tag.id, { color })}
            />
          </Dialog>
        </Popover>
      </DialogTrigger>

      {renaming ? (
        <InlineInput
          label={t("settings.tags.name")}
          initialValue={tag.name}
          onSubmit={(name) => onUpdate(tag.id, { name })}
          onClose={() => setRenaming(false)}
        />
      ) : (
        <button
          type="button"
          className={styles.name}
          aria-label={t("settings.tags.rename", { name: tag.name })}
          onClick={() => setRenaming(true)}
        >
          <TagChip color={tag.color}>{tag.name}</TagChip>
        </button>
      )}

      {error && <span className={styles.error}>{error}</span>}

      <button
        type="button"
        className={styles.delete}
        aria-label={t("settings.tags.delete", { name: tag.name })}
        onClick={() => void requestDelete()}
      >
        <Trash2 aria-hidden="true" />
      </button>
    </li>
  );
}

export default TagManager;
