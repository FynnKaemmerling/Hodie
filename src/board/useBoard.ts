import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { api, type BoardView, type CardSummary, type ColumnView, type Id, type Tag } from "../api";
import { useNotifications } from "../notifications/Notifications";
import * as state from "./boardState";

/**
 * One board with its columns and cards. Changes show immediately and are
 * saved in the background; if saving fails, the board is reloaded from the
 * backend, which is the source of truth.
 */
export function useBoard(boardId: Id | null) {
  const { t } = useTranslation();
  const notify = useNotifications();
  const [view, setView] = useState<BoardView | null>(null);
  const [tags, setTags] = useState<Tag[]>([]);
  const currentBoard = useRef(boardId);
  currentBoard.current = boardId;

  const reload = useCallback(async () => {
    if (boardId === null) return;
    try {
      const [loaded, loadedTags] = await Promise.all([api.getBoard(boardId), api.listTags()]);
      // Ignore answers for a board that is no longer open.
      if (currentBoard.current !== boardId) return;
      setView(loaded);
      setTags(loadedTags);
    } catch (error) {
      notify.error(error);
    }
  }, [boardId, notify]);

  useEffect(() => {
    setView(null);
    void reload();
  }, [reload]);

  const apply = useCallback(
    async (change: (view: BoardView) => BoardView, save: () => Promise<unknown>) => {
      setView((current) => current && change(current));
      try {
        await save();
        return true;
      } catch (error) {
        notify.error(error);
        await reload();
        return false;
      }
    },
    [notify, reload],
  );

  const createCard = useCallback(
    async (columnId: Id, title: string) => {
      try {
        const card = await api.createCard({ columnId, title });
        setView((current) => current && state.insertCard(current, card, columnId));
      } catch (error) {
        notify.error(error);
      }
    },
    [notify],
  );

  const renameCard = useCallback(
    (cardId: Id, title: string) =>
      apply(
        (v) => state.updateCard(v, cardId, { title }),
        () => api.updateCard(cardId, { title }),
      ),
    [apply],
  );

  const moveCard = useCallback(
    (cardId: Id, columnId: Id, index: number) =>
      apply(
        (v) => state.moveCard(v, cardId, columnId, index),
        () => api.moveCard(cardId, columnId, index),
      ),
    [apply],
  );

  const deleteCard = useCallback(
    async (card: CardSummary) => {
      const deleted = await apply(
        (v) => state.removeCard(v, card.id),
        () => api.deleteCard(card.id),
      );
      if (!deleted) return;
      notify.info(t("toast.cardDeleted", { title: card.title }), async () => {
        try {
          await api.restoreCard(card.id);
        } catch (error) {
          notify.error(error);
        }
        await reload();
      });
    },
    [apply, notify, reload, t],
  );

  const createColumn = useCallback(
    async (name: string) => {
      if (boardId === null) return;
      try {
        const column = await api.createColumn(boardId, name);
        setView((current) => current && state.insertColumn(current, { ...column, cards: [] }));
      } catch (error) {
        notify.error(error);
      }
    },
    [boardId, notify],
  );

  const renameColumn = useCallback(
    (columnId: Id, name: string) =>
      apply(
        (v) => state.updateColumn(v, columnId, { name }),
        () => api.renameColumn(columnId, name),
      ),
    [apply],
  );

  const moveColumn = useCallback(
    (columnId: Id, index: number) =>
      apply(
        (v) => state.moveColumn(v, columnId, index),
        () => api.reorderColumn(columnId, index),
      ),
    [apply],
  );

  const setDoneColumn = useCallback(
    (columnId: Id) =>
      apply(
        (v) => state.setDoneColumn(v, columnId),
        () => api.setDoneColumn(columnId),
      ),
    [apply],
  );

  const deleteColumn = useCallback(
    async (column: ColumnView) => {
      try {
        await api.deleteColumn(column.id);
      } catch (error) {
        notify.error(error);
        return;
      }
      setView((current) => current && state.removeColumn(current, column.id));
      notify.info(t("toast.columnDeleted", { name: column.name }), async () => {
        try {
          await api.restoreColumn(column.id);
        } catch (error) {
          notify.error(error);
        }
        await reload();
      });
    },
    [notify, reload, t],
  );

  return {
    view,
    tags,
    createCard,
    renameCard,
    moveCard,
    deleteCard,
    createColumn,
    renameColumn,
    moveColumn,
    setDoneColumn,
    deleteColumn,
  };
}

export type BoardActions = ReturnType<typeof useBoard>;
