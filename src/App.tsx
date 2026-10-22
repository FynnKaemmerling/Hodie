import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import type { CardDetail, Id } from "./api";
import ArchiveView from "./components/ArchiveView";
import BoardCanvas from "./components/BoardCanvas";
import CardPanel from "./components/CardPanel";
import Sidebar from "./components/Sidebar";
import { focusCard } from "./board/focus";
import { useArchive } from "./board/useArchive";
import { useBoard } from "./board/useBoard";
import { useBoards } from "./board/useBoards";
import { useCardDetail } from "./board/useCardDetail";
import styles from "./App.module.css";

function App() {
  const { t } = useTranslation();
  const boards = useBoards();
  const board = useBoard(boards.selectedId);
  const [showingArchive, setShowingArchive] = useState(false);
  const archive = useArchive(showingArchive ? boards.selectedId : null, board.reload);
  const [openCardId, setOpenCardId] = useState<Id | null>(null);
  const selected = boards.boards?.find((b) => b.id === boards.selectedId);

  const closeCard = useCallback(() => {
    if (openCardId !== null) focusCard(openCardId);
    setOpenCardId(null);
  }, [openCardId]);

  const detail = useCardDetail(openCardId, {
    onSummary: (cardId, fields) => {
      board.patchCard(cardId, fields);
      archive.patchCard(cardId, fields);
    },
    onFailed: () => {
      void board.reload();
      void archive.reload();
    },
    onGone: () => setOpenCardId(null),
    onTagCreated: board.addTag,
  });

  // A card deleted or archived on the board closes its panel.
  const openCardOnBoard = openCardId !== null && board.view?.columns.some((c) =>
    c.cards.some((card) => card.id === openCardId),
  );
  const openCardInArchive = archive.cards?.some((card) => card.id === openCardId) ?? false;
  useEffect(() => {
    if (detail.card && !openCardOnBoard && !openCardInArchive) setOpenCardId(null);
  }, [detail.card, openCardOnBoard, openCardInArchive]);

  function selectBoard(id: Id) {
    boards.select(id);
    setShowingArchive(false);
    setOpenCardId(null);
  }

  function showArchive() {
    setShowingArchive(true);
    setOpenCardId(null);
  }

  function archiveCard(card: CardDetail) {
    setOpenCardId(null);
    void board.archiveCard(card);
  }

  function unarchiveCard(card: CardDetail) {
    setOpenCardId(null);
    void archive.restore(card);
  }

  function deleteCard(card: CardDetail) {
    setOpenCardId(null);
    void (card.archivedAt == null ? board.deleteCard(card) : archive.remove(card));
  }

  return (
    <div className={styles.app}>
      <Sidebar
        boards={boards.boards ?? []}
        selectedId={boards.selectedId}
        onSelect={selectBoard}
        onCreate={boards.create}
        onRename={boards.rename}
        onDelete={boards.remove}
        showingArchive={showingArchive}
        onShowArchive={showArchive}
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
              <h1 className={styles.title}>
                {showingArchive ? t("archive.title", { name: selected.name }) : selected.name}
              </h1>
            </header>
            <div className={styles.content}>
              <div className={styles.view}>
                {showingArchive ? (
                  <ArchiveView
                    cards={archive.cards}
                    tags={board.tags}
                    selectedCardId={openCardId}
                    onOpen={setOpenCardId}
                    onRestore={archive.restore}
                    onDelete={archive.remove}
                  />
                ) : (
                  <BoardCanvas
                    board={board}
                    selectedCardId={openCardId}
                    onOpenCard={setOpenCardId}
                  />
                )}
              </div>
              {openCardId !== null && (
                <CardPanel
                  key={openCardId}
                  detail={detail}
                  tags={board.tags}
                  onClose={closeCard}
                  onArchive={archiveCard}
                  onUnarchive={unarchiveCard}
                  onDelete={deleteCard}
                />
              )}
            </div>
          </>
        )}
      </main>
    </div>
  );
}

export default App;
