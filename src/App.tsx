import { useTranslation } from "react-i18next";
import BoardCanvas from "./components/BoardCanvas";
import Sidebar from "./components/Sidebar";
import { useBoard } from "./board/useBoard";
import { useBoards } from "./board/useBoards";
import styles from "./App.module.css";

function App() {
  const { t } = useTranslation();
  const boards = useBoards();
  const board = useBoard(boards.selectedId);
  const selected = boards.boards?.find((b) => b.id === boards.selectedId);

  return (
    <div className={styles.app}>
      <Sidebar
        boards={boards.boards ?? []}
        selectedId={boards.selectedId}
        onSelect={boards.select}
        onCreate={boards.create}
        onRename={boards.rename}
        onDelete={boards.remove}
      />

      <main className={styles.main}>
        {boards.boards?.length === 0 && (
          <div className={styles.empty}>
            <h1 className={styles.emptyTitle}>{t("home.empty.title")}</h1>
            <p className={styles.emptyText}>{t("home.empty.description")}</p>
            <button
              type="button"
              className={styles.primary}
              onClick={() => boards.create(t("board.defaultName"))}
            >
              {t("home.empty.action")}
            </button>
          </div>
        )}

        {selected && (
          <>
            <header className={styles.toolbar}>
              <h1 className={styles.title}>{selected.name}</h1>
            </header>
            <div className={styles.content}>
              <BoardCanvas board={board} />
            </div>
          </>
        )}
      </main>
    </div>
  );
}

export default App;
