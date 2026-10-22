import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { api, type CardSummary, type Id } from "../api";
import { useNotifications } from "../notifications/Notifications";
import type { CardRef } from "./useBoard";

/** The archived cards of a board; loads only while `boardId` is set. */
export function useArchive(boardId: Id | null, onBoardChanged: () => void) {
  const { t } = useTranslation();
  const notify = useNotifications();
  const [cards, setCards] = useState<CardSummary[] | null>(null);

  const reload = useCallback(async () => {
    if (boardId === null) return;
    try {
      setCards(await api.listArchivedCards(boardId));
    } catch (error) {
      notify.error(error);
    }
  }, [boardId, notify]);

  useEffect(() => {
    setCards(null);
    void reload();
  }, [reload]);

  /** Puts the card back on the board. */
  const restore = useCallback(
    async (card: CardRef) => {
      setCards((current) => current?.filter((c) => c.id !== card.id) ?? null);
      try {
        await api.unarchiveCard(card.id);
        notify.info(t("toast.cardRestored", { title: card.title }));
      } catch (error) {
        notify.error(error);
        await reload();
      }
      onBoardChanged();
    },
    [notify, onBoardChanged, reload, t],
  );

  const remove = useCallback(
    async (card: CardRef) => {
      setCards((current) => current?.filter((c) => c.id !== card.id) ?? null);
      try {
        await api.deleteCard(card.id);
      } catch (error) {
        notify.error(error);
        await reload();
        return;
      }
      notify.info(t("toast.cardDeleted", { title: card.title }), async () => {
        try {
          await api.restoreCard(card.id);
        } catch (error) {
          notify.error(error);
        }
        await reload();
      });
    },
    [notify, reload, t],
  );

  /** Mirrors a change saved in the card panel. */
  const patchCard = useCallback((cardId: Id, change: Partial<CardSummary>) => {
    setCards((current) => current?.map((c) => (c.id === cardId ? { ...c, ...change } : c)) ?? null);
  }, []);

  return { cards, reload, restore, remove, patchCard };
}
