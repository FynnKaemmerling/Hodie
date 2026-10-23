import { useCallback, useEffect, useRef, useState } from "react";
import { emit } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ArrowUp } from "lucide-react";
import { useTranslation } from "react-i18next";
import { api, type Id, type Tag } from "../api";
import { errorMessageKey } from "../api/errors";
import { CARDS_CHANGED, type CardsChangedPayload } from "../api/events";
import { findTagByName } from "../board/cardState";
import { createCardFromEntry, parseQuickEntry } from "../board/quickEntry";
import { readStoredBoard } from "../board/useBoards";
import logo from "../../src-tauri/icons/app-icon.svg";
import TagChip from "../components/TagChip";
import styles from "./QuickAdd.module.css";

interface Target {
  boardName: string;
  columnId: Id;
  columnName: string;
}

/**
 * The small window opened by the global shortcut. Enter adds the card to the
 * first column of the board last open in the main window.
 */
function QuickAdd() {
  const { t } = useTranslation();
  const [text, setText] = useState("");
  const [target, setTarget] = useState<Target | null>(null);
  const [tags, setTags] = useState<Tag[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const input = useRef<HTMLInputElement>(null);

  // Runs each time the window appears, since boards may have changed.
  const prepare = useCallback(async () => {
    setError(null);
    try {
      const boards = await api.listBoards();
      const stored = readStoredBoard();
      const board = boards.find((b) => b.id === stored) ?? boards[0];
      if (!board) {
        setTarget(null);
        setError(t("quickAdd.noBoard"));
        return;
      }
      const [view, loadedTags] = await Promise.all([api.getBoard(board.id), api.listTags()]);
      const column = view.columns[0];
      setTags(loadedTags);
      setTarget(
        column ? { boardName: view.board.name, columnId: column.id, columnName: column.name } : null,
      );
    } catch (failure) {
      setError(t(errorMessageKey(failure)));
    }
  }, [t]);

  useEffect(() => {
    const appWindow = getCurrentWindow();
    void prepare();
    const unlisten = appWindow.onFocusChanged(({ payload: focused }) => {
      if (focused) {
        void prepare();
        input.current?.focus();
        input.current?.select();
      } else {
        void appWindow.hide();
      }
    });
    return () => void unlisten.then((stop) => stop());
  }, [prepare]);

  async function submit() {
    if (!target || saving) return;
    setSaving(true);
    try {
      const { card } = await createCardFromEntry(target.columnId, text, tags);
      await emit<CardsChangedPayload>(CARDS_CHANGED, { cardId: card.id });
      setText("");
      await getCurrentWindow().hide();
    } catch (failure) {
      setError(t(errorMessageKey(failure)));
    } finally {
      setSaving(false);
    }
  }

  const entry = parseQuickEntry(text);

  return (
    <div className={styles.window} data-priority={entry.priority}>
      <div className={styles.entry}>
        <img className={styles.logo} src={logo} alt="" />
        <input
          ref={input}
          className={styles.input}
          value={text}
          autoFocus
          placeholder={t("quickAdd.placeholder")}
          aria-label={t("quickAdd.label")}
          aria-describedby="quick-add-status"
          onChange={(event) => {
            setText(event.target.value);
            setError(null);
          }}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              event.preventDefault();
              void submit();
            } else if (event.key === "Escape") {
              event.preventDefault();
              void getCurrentWindow().hide();
            }
          }}
        />
        <button
          type="button"
          className={styles.send}
          aria-label={t("quickAdd.submit")}
          disabled={!target || saving || text.trim() === ""}
          // Keeps the caret in the input.
          onMouseDown={(event) => event.preventDefault()}
          onClick={() => void submit()}
        >
          <ArrowUp aria-hidden />
        </button>
      </div>

      <div className={styles.footer} id="quick-add-status">
        {error ? (
          <span className={styles.error} role="alert">
            {error}
          </span>
        ) : (
          target && (
            <span className={styles.target}>
              {t("quickAdd.target", { board: target.boardName, column: target.columnName })}
            </span>
          )
        )}

        <span className={styles.preview}>
          {entry.priority !== "none" && (
            <span className={styles.priority}>{t(`card.priority.${entry.priority}`)}</span>
          )}
          {entry.tagNames.map((name) => (
            <TagChip key={name} color={findTagByName(tags, name)?.color ?? "gray"} size="small">
              {name}
            </TagChip>
          ))}
        </span>

        <span className={styles.hint}>{t("quickAdd.hint")}</span>
      </div>
    </div>
  );
}

export default QuickAdd;
