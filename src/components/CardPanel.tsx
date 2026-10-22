import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { Radio, RadioGroup } from "react-aria-components";
import { useTranslation } from "react-i18next";
import type { CardDetail, Priority, Tag } from "../api";
import type { CardDetailActions } from "../board/useCardDetail";
import SubtaskList from "./SubtaskList";
import TagPicker from "./TagPicker";
import styles from "./CardPanel.module.css";

const PRIORITIES: Priority[] = ["none", "low", "medium", "high"];
const NOTES_SAVE_DELAY_MS = 600;

interface Props {
  detail: CardDetailActions;
  tags: Tag[];
  onClose: () => void;
  onArchive: (card: CardDetail) => void;
  onUnarchive: (card: CardDetail) => void;
  onDelete: (card: CardDetail) => void;
}

/** Side panel with every detail of one card. Changes save immediately. */
function CardPanel({ detail, tags, onClose, onArchive, onUnarchive, onDelete }: Props) {
  const { t, i18n } = useTranslation();
  const { card } = detail;
  const dateFormat = new Intl.DateTimeFormat(i18n.language, {
    dateStyle: "medium",
    timeStyle: "short",
  });
  const subtasksDone = card?.subtasks.filter((s) => s.done).length ?? 0;

  return (
    <aside
      className={styles.panel}
      aria-label={t("panel.title")}
      onKeyDown={(event) => {
        if (event.key === "Escape" && !event.defaultPrevented) {
          event.preventDefault();
          onClose();
        }
      }}
    >
      <header className={styles.header}>
        {card?.archivedAt != null && <span className={styles.badge}>{t("panel.archived")}</span>}
        <button
          type="button"
          className={styles.close}
          aria-label={t("panel.close")}
          onClick={onClose}
        >
          <span aria-hidden="true">×</span>
        </button>
      </header>

      {card && (
        <div className={styles.body}>
          <TitleField key={`title-${card.id}`} value={card.title} onSave={detail.setTitle} />

          <section className={styles.section}>
            <h3 className={styles.label} id="panel-priority">
              {t("panel.priority")}
            </h3>
            <RadioGroup
              className={styles.segments}
              orientation="horizontal"
              aria-labelledby="panel-priority"
              value={card.priority}
              onChange={(value) => void detail.setPriority(value as Priority)}
            >
              {PRIORITIES.map((priority) => (
                <Radio
                  key={priority}
                  value={priority}
                  className={styles.segment}
                  data-priority={priority}
                >
                  {t(`panel.priorities.${priority}`)}
                </Radio>
              ))}
            </RadioGroup>
          </section>

          <section className={styles.section}>
            <h3 className={styles.label}>{t("panel.tags")}</h3>
            <TagPicker
              tags={tags}
              selectedIds={card.tagIds}
              onToggle={(id) => void detail.toggleTag(id)}
              onCreate={(name, color) => void detail.createTag(name, color)}
            />
          </section>

          <section className={styles.section}>
            <h3 className={styles.label}>
              <label htmlFor="panel-notes">{t("panel.notes")}</label>
            </h3>
            <NotesField key={`notes-${card.id}`} value={card.notes} onSave={detail.setNotes} />
          </section>

          <section className={styles.section}>
            <h3 className={styles.label}>
              {t("panel.subtasks")}
              {card.subtasks.length > 0 && (
                <span className={styles.progress}>
                  {" "}
                  {subtasksDone}/{card.subtasks.length}
                </span>
              )}
            </h3>
            <SubtaskList
              subtasks={card.subtasks}
              onAdd={(title) => void detail.addSubtask(title)}
              onToggle={(subtask, done) => void detail.toggleSubtask(subtask, done)}
              onRename={(id, title) => void detail.renameSubtask(id, title)}
              onMove={(id, index) => void detail.moveSubtask(id, index)}
              onDelete={(subtask) => void detail.deleteSubtask(subtask)}
            />
          </section>

          <footer className={styles.footer}>
            <div className={styles.actions}>
              {card.archivedAt == null ? (
                <button type="button" className={styles.action} onClick={() => onArchive(card)}>
                  {t("panel.archive")}
                </button>
              ) : (
                <button type="button" className={styles.action} onClick={() => onUnarchive(card)}>
                  {t("panel.unarchive")}
                </button>
              )}
              <button
                type="button"
                className={styles.action}
                data-danger
                onClick={() => onDelete(card)}
              >
                {t("actions.delete")}
              </button>
            </div>
            <p className={styles.meta}>
              {t("panel.created", { date: dateFormat.format(card.createdAt) })}
              <br />
              {t("panel.updated", { date: dateFormat.format(card.updatedAt) })}
            </p>
          </footer>
        </div>
      )}
    </aside>
  );
}

/** Saves on Enter or when leaving the field; Escape restores the old title. */
function TitleField({ value, onSave }: { value: string; onSave: (title: string) => void }) {
  const { t } = useTranslation();
  const [draft, setDraft] = useState(value);
  const focused = useRef(false);
  const field = useRef<HTMLTextAreaElement>(null);

  // Follow renames from the board while not editing here.
  useEffect(() => {
    if (!focused.current) setDraft(value);
  }, [value]);

  // Grow with the text so long titles wrap instead of scrolling.
  useLayoutEffect(() => {
    const element = field.current;
    if (!element) return;
    element.style.height = "auto";
    element.style.height = `${element.scrollHeight}px`;
  }, [draft]);

  function save() {
    const trimmed = draft.trim();
    if (trimmed === "") setDraft(value);
    else if (trimmed !== value) onSave(trimmed);
  }

  return (
    <textarea
      ref={field}
      className={styles.title}
      aria-label={t("card.title")}
      rows={1}
      value={draft}
      autoFocus
      onFocus={() => (focused.current = true)}
      onChange={(event) => setDraft(event.target.value.replace(/\n/g, " "))}
      onBlur={() => {
        focused.current = false;
        save();
      }}
      onKeyDown={(event) => {
        if (event.key === "Enter") {
          event.preventDefault();
          event.currentTarget.blur();
        } else if (event.key === "Escape" && draft !== value) {
          event.preventDefault();
          setDraft(value);
        }
      }}
    />
  );
}

/** Saves shortly after typing stops, when leaving the field and on close. */
function NotesField({ value, onSave }: { value: string; onSave: (notes: string) => void }) {
  const { t } = useTranslation();
  const [draft, setDraft] = useState(value);
  const pending = useRef<string | null>(null);
  const timer = useRef<number | undefined>(undefined);
  const saveRef = useRef(onSave);
  saveRef.current = onSave;

  function flush() {
    window.clearTimeout(timer.current);
    if (pending.current !== null) {
      saveRef.current(pending.current);
      pending.current = null;
    }
  }

  // Closing the panel must not lose what was just typed.
  useEffect(() => flush, []);

  return (
    <textarea
      id="panel-notes"
      className={styles.notes}
      rows={5}
      value={draft}
      placeholder={t("panel.notesPlaceholder")}
      onChange={(event) => {
        setDraft(event.target.value);
        pending.current = event.target.value;
        window.clearTimeout(timer.current);
        timer.current = window.setTimeout(flush, NOTES_SAVE_DELAY_MS);
      }}
      onBlur={flush}
    />
  );
}

export default CardPanel;
