import { useState } from "react";
import { useTranslation } from "react-i18next";
import type { Board, Id } from "../api";
import ActionMenu from "./ActionMenu";
import InlineInput from "./InlineInput";
import LanguageSwitcher from "./LanguageSwitcher";
import styles from "./Sidebar.module.css";

interface Props {
  boards: Board[];
  selectedId: Id | null;
  onSelect: (id: Id) => void;
  onCreate: (name: string) => void;
  onRename: (id: Id, name: string) => void;
  onDelete: (board: Board) => void;
  showingArchive: boolean;
  onShowArchive: () => void;
}

function Sidebar({
  boards,
  selectedId,
  onSelect,
  onCreate,
  onRename,
  onDelete,
  showingArchive,
  onShowArchive,
}: Props) {
  const { t } = useTranslation();
  const [creating, setCreating] = useState(false);
  const [renamingId, setRenamingId] = useState<Id | null>(null);

  return (
    <nav className={styles.sidebar} aria-label={t("nav.title")}>
      <div className={styles.header}>
        <span className={styles.appName}>{t("app.name")}</span>
      </div>

      <h2 className={styles.sectionTitle}>{t("nav.boards")}</h2>
      <ul className={styles.items}>
        {boards.map((board) => (
          <li key={board.id} className={styles.row}>
            {renamingId === board.id ? (
              <InlineInput
                label={t("board.name")}
                initialValue={board.name}
                onSubmit={(name) => onRename(board.id, name)}
                onClose={() => setRenamingId(null)}
              />
            ) : (
              <>
                <button
                  type="button"
                  className={styles.item}
                  aria-current={board.id === selectedId && !showingArchive ? "page" : undefined}
                  onClick={() => onSelect(board.id)}
                  onDoubleClick={() => setRenamingId(board.id)}
                  onKeyDown={(event) => {
                    if (event.key === "F2") {
                      event.preventDefault();
                      setRenamingId(board.id);
                    }
                  }}
                >
                  {board.name}
                </button>
                <div className={styles.menu}>
                  <ActionMenu
                    label={t("actions.more", { name: board.name })}
                    items={[
                      {
                        id: "rename",
                        label: t("actions.rename"),
                        onAction: () => setRenamingId(board.id),
                      },
                      {
                        id: "delete",
                        label: t("actions.delete"),
                        danger: true,
                        onAction: () => onDelete(board),
                      },
                    ]}
                  />
                </div>
              </>
            )}
          </li>
        ))}
      </ul>

      {creating ? (
        <InlineInput
          label={t("board.name")}
          placeholder={t("board.namePlaceholder")}
          onSubmit={onCreate}
          onClose={() => setCreating(false)}
        />
      ) : (
        <button type="button" className={styles.add} onClick={() => setCreating(true)}>
          <span aria-hidden="true">+</span> {t("board.add")}
        </button>
      )}

      {selectedId !== null && (
        <button
          type="button"
          className={`${styles.item} ${styles.archive}`}
          aria-current={showingArchive ? "page" : undefined}
          onClick={onShowArchive}
        >
          {t("nav.archive")}
        </button>
      )}

      <div className={styles.footer}>
        <LanguageSwitcher />
      </div>
    </nav>
  );
}

export default Sidebar;
