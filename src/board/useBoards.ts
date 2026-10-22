import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { api, type Board, type Id } from "../api";
import { useNotifications } from "../notifications/Notifications";

// Temporary storage until settings live in the Rust backend.
const STORAGE_KEY = "hodie.board";

function readStoredBoard(): Id | null {
  try {
    const value = Number(localStorage.getItem(STORAGE_KEY));
    return Number.isInteger(value) && value > 0 ? value : null;
  } catch {
    return null;
  }
}

function storeBoard(id: Id | null) {
  try {
    if (id === null) localStorage.removeItem(STORAGE_KEY);
    else localStorage.setItem(STORAGE_KEY, String(id));
  } catch {
    // Not persisted; the selection still applies for this session.
  }
}

/** The list of boards and which one is open. */
export function useBoards() {
  const { t } = useTranslation();
  const notify = useNotifications();
  const [boards, setBoards] = useState<Board[] | null>(null);
  const [selectedId, setSelectedId] = useState<Id | null>(readStoredBoard);

  const select = useCallback((id: Id | null) => {
    setSelectedId(id);
    storeBoard(id);
  }, []);

  const reload = useCallback(async () => {
    try {
      setBoards(await api.listBoards());
    } catch (error) {
      notify.error(error);
    }
  }, [notify]);

  useEffect(() => {
    void reload();
  }, [reload]);

  // Keep the selection valid: fall back to the first board.
  useEffect(() => {
    if (!boards) return;
    if (selectedId === null || !boards.some((board) => board.id === selectedId)) {
      select(boards[0]?.id ?? null);
    }
  }, [boards, selectedId, select]);

  const create = useCallback(
    async (name: string) => {
      try {
        const board = await api.createBoard(name, [
          t("board.defaultColumns.open"),
          t("board.defaultColumns.doing"),
          t("board.defaultColumns.done"),
        ]);
        setBoards((current) => [...(current ?? []), board]);
        select(board.id);
      } catch (error) {
        notify.error(error);
      }
    },
    [notify, select, t],
  );

  const rename = useCallback(
    async (id: Id, name: string) => {
      setBoards((current) => current?.map((b) => (b.id === id ? { ...b, name } : b)) ?? null);
      try {
        await api.renameBoard(id, name);
      } catch (error) {
        notify.error(error);
        await reload();
      }
    },
    [notify, reload],
  );

  const remove = useCallback(
    async (board: Board) => {
      setBoards((current) => current?.filter((b) => b.id !== board.id) ?? null);
      try {
        await api.deleteBoard(board.id);
        notify.info(t("toast.boardDeleted", { name: board.name }), async () => {
          try {
            await api.restoreBoard(board.id);
            await reload();
            select(board.id);
          } catch (error) {
            notify.error(error);
          }
        });
      } catch (error) {
        notify.error(error);
        await reload();
      }
    },
    [notify, reload, select, t],
  );

  return { boards, selectedId, select, create, rename, remove };
}
