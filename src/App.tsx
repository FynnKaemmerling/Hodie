import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useTranslation } from "react-i18next";
import { api, type BoardView, type CardDetail, type Id, type Tag } from "./api";
import { CARDS_CHANGED, VAULT_FILES_CHANGED, type CardsChangedPayload } from "./api/events";
import ArchiveView from "./components/ArchiveView";
import BoardCanvas from "./components/BoardCanvas";
import BoardToolbar from "./components/BoardToolbar";
import CardPanel from "./components/CardPanel";
import SettingsDialog from "./components/SettingsDialog";
import SidebarToggle from "./components/SidebarToggle";
import WindowControls from "./components/WindowControls";
import Sidebar from "./components/Sidebar";
import { focusCard } from "./board/focus";
import { useArchive } from "./board/useArchive";
import { useBoard } from "./board/useBoard";
import { useBoards } from "./board/useBoards";
import { applyFilter, countCards, isFilterActive, NO_FILTER, type CardFilter } from "./board/filter";
import { useCardDetail } from "./board/useCardDetail";
import { useCardSearch } from "./board/useCardSearch";
import { isModKey } from "./platform";
import { useSidebarCollapsed } from "./sidebar";
import styles from "./App.module.css";

// A little longer than --duration-arrived, so the highlight fully fades.
const ARRIVED_MS = 2000;

/**
 * Whether a click closes the card panel: anywhere in the app except the
 * panel, a card (which opens that card instead) and the window drag areas.
 * Menus, dialogs and toasts render outside the app element and never count.
 */
function closesPanel(app: Element | null, target: EventTarget | null): boolean {
  if (!(target instanceof Element) || !app?.contains(target)) return false;
  return !target.closest("[data-card-panel], [data-card-id], [data-tauri-drag-region]");
}

function App() {
  const { t } = useTranslation();
  const boards = useBoards();
  const board = useBoard(boards.selectedId);
  const [showingArchive, setShowingArchive] = useState(false);
  const archive = useArchive(showingArchive ? boards.selectedId : null, board.reload);
  const [openCardId, setOpenCardId] = useState<Id | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const selected = boards.boards?.find((b) => b.id === boards.selectedId);
  const [filter, setFilter] = useState<CardFilter>(NO_FILTER);
  const searchMatches = useCardSearch(boards.selectedId, filter.query, board.view);
  const filtering = isFilterActive(filter);
  const searchRef = useRef<HTMLInputElement>(null);
  const [sidebarCollapsed, setSidebarCollapsed] = useSidebarCollapsed();
  const [arrivedCardId, setArrivedCardId] = useState<Id | null>(null);
  const appRef = useRef<HTMLDivElement>(null);

  const filterView = useCallback(
    (view: BoardView) => (filtering ? applyFilter(view, filter, searchMatches) : view),
    [filter, filtering, searchMatches],
  );
  const counts =
    filtering && board.view
      ? { visible: countCards(filterView(board.view)), total: countCards(board.view) }
      : null;

  // Each board starts unfiltered.
  useEffect(() => setFilter(NO_FILTER), [boards.selectedId]);

  // Mod+F jumps to the search field, Mod+B toggles the sidebar.
  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (!isModKey(event) || event.shiftKey) return;
      const key = event.key.toLowerCase();
      if (key === "b") {
        event.preventDefault();
        setSidebarCollapsed(!sidebarCollapsed);
      } else if (key === "f" && searchRef.current) {
        event.preventDefault();
        searchRef.current.focus();
        searchRef.current.select();
      }
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [sidebarCollapsed, setSidebarCollapsed]);

  // The quick entry window adds cards behind this window's back.
  const reloadBoard = useRef(board.reload);
  reloadBoard.current = board.reload;
  useEffect(() => {
    const unlisten = listen<CardsChangedPayload>(CARDS_CHANGED, async ({ payload }) => {
      await reloadBoard.current();
      if (payload?.cardId !== undefined) setArrivedCardId(payload.cardId);
    });
    return () => void unlisten.then((stop) => stop());
  }, []);

  // Someone else changed vault files (through a sync service, an editor …).
  // The backend watches the folder; on focus the window also asks it to
  // check, for folders that cannot be watched.
  const reloadAll = useRef<() => void>(() => {});
  reloadAll.current = () => {
    void boards.reload();
    void board.reload();
    void archive.reload();
    void detail.reload();
  };
  useEffect(() => {
    const unlisten = listen(VAULT_FILES_CHANGED, () => reloadAll.current());
    const unfocus = getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      if (focused) api.refreshVault().catch(() => {});
    });
    return () => {
      void unlisten.then((stop) => stop());
      void unfocus.then((stop) => stop());
    };
  }, []);

  useEffect(() => {
    if (arrivedCardId === null) return;
    const timer = window.setTimeout(() => setArrivedCardId(null), ARRIVED_MS);
    return () => window.clearTimeout(timer);
  }, [arrivedCardId]);

  const sidebarToggle = sidebarCollapsed && (
    <SidebarToggle collapsed onToggle={() => setSidebarCollapsed(false)} />
  );

  const closeCard = useCallback(() => {
    if (openCardId !== null) focusCard(openCardId);
    setOpenCardId(null);
  }, [openCardId]);

  // Clicking next to the panel closes it. Listening for the click (not the
  // press) lets the title field save on blur first; the press must have
  // started outside too, so selecting text in the panel never closes it.
  useEffect(() => {
    if (openCardId === null) return;
    let pressedOutside = false;
    function onPointerDown(event: PointerEvent) {
      pressedOutside = closesPanel(appRef.current, event.target);
    }
    function onClick(event: MouseEvent) {
      if (pressedOutside && closesPanel(appRef.current, event.target)) setOpenCardId(null);
      pressedOutside = false;
    }
    document.addEventListener("pointerdown", onPointerDown);
    document.addEventListener("click", onClick);
    return () => {
      document.removeEventListener("pointerdown", onPointerDown);
      document.removeEventListener("click", onClick);
    };
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

  async function deleteTag(tag: Tag) {
    await board.deleteTag(tag);
    // The open card may carry the tag; reload it so it no longer refers to it.
    if (detail.card?.tagIds.includes(tag.id)) await detail.reload();
    void archive.reload();
  }

  return (
    <div ref={appRef} className={styles.app}>
      <Sidebar
        boards={boards.boards ?? []}
        selectedId={boards.selectedId}
        onSelect={selectBoard}
        onCreate={boards.create}
        onRename={boards.rename}
        onReorder={boards.reorder}
        onDelete={boards.remove}
        showingArchive={showingArchive}
        onShowArchive={showArchive}
        onOpenSettings={() => setSettingsOpen(true)}
        collapsed={sidebarCollapsed}
        onCollapse={() => setSidebarCollapsed(true)}
      />

      <SettingsDialog
        isOpen={settingsOpen}
        onOpenChange={setSettingsOpen}
        tags={board.tags}
        onCreateTag={board.createTag}
        onUpdateTag={board.updateTag}
        onDeleteTag={deleteTag}
      />

      <main className={styles.main}>
        {!selected && (
          <header className={styles.toolbar} data-tauri-drag-region>
            {sidebarToggle}
            <span className={styles.spacer} data-tauri-drag-region />
            <WindowControls />
          </header>
        )}

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
            {showingArchive ? (
              <header className={styles.toolbar} data-tauri-drag-region>
                {sidebarToggle}
                <h1 className={styles.title} data-tauri-drag-region>
                  {t("archive.title", { name: selected.name })}
                </h1>
                <WindowControls />
              </header>
            ) : (
              <BoardToolbar
                title={selected.name}
                tags={board.tags}
                filter={filter}
                onFilterChange={setFilter}
                counts={counts}
                searchRef={searchRef}
                leading={sidebarToggle}
                trailing={<WindowControls />}
              />
            )}
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
                    arrivedCardId={arrivedCardId}
                    onOpenCard={setOpenCardId}
                    filterView={filterView}
                    dragDisabled={filtering}
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
                  onConflictResolved={() => reloadAll.current()}
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
