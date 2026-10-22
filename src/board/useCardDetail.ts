import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { api, type CardDetail, type Id, type Priority, type Subtask, type Tag, type TagColor } from "../api";
import { useNotifications } from "../notifications/Notifications";
import * as state from "./cardState";

interface Callbacks {
  /** The board or archive mirrors the card's visible fields. */
  onSummary: (cardId: Id, fields: state.SummaryFields) => void;
  /** Saving failed; whatever mirrors the card should reload. */
  onFailed: () => void;
  /** The card no longer exists. */
  onGone: () => void;
  onTagCreated: (tag: Tag) => void;
}

/**
 * The card shown in the detail panel. Like the board, changes show
 * immediately and the card is reloaded if saving fails.
 */
export function useCardDetail(cardId: Id | null, callbacks: Callbacks) {
  const { t } = useTranslation();
  const notify = useNotifications();
  const [card, setCard] = useState<CardDetail | null>(null);
  // Successive changes build on the latest state even before a re-render.
  const cardRef = useRef<CardDetail | null>(null);
  const callbacksRef = useRef(callbacks);
  callbacksRef.current = callbacks;
  const openId = useRef(cardId);
  openId.current = cardId;

  const show = useCallback((next: CardDetail | null) => {
    cardRef.current = next;
    setCard(next);
  }, []);

  const load = useCallback(async () => {
    if (cardId === null) return;
    try {
      const loaded = await api.getCard(cardId);
      // Ignore answers for a card that is no longer open.
      if (openId.current === cardId) show(loaded);
    } catch (error) {
      if (error === "not_found") callbacksRef.current.onGone();
      else notify.error(error);
    }
  }, [cardId, notify, show]);

  useEffect(() => {
    show(null);
    void load();
  }, [load, show]);

  const change = useCallback(
    async (next: (card: CardDetail) => CardDetail, save: (card: CardDetail) => Promise<unknown>) => {
      const current = cardRef.current;
      if (!current) return false;
      const updated = next(current);
      show(updated);
      callbacksRef.current.onSummary(updated.id, state.summaryFields(updated));
      try {
        await save(updated);
        return true;
      } catch (error) {
        notify.error(error);
        await load();
        callbacksRef.current.onFailed();
        return false;
      }
    },
    [load, notify, show],
  );

  const setTitle = useCallback(
    (title: string) => change((c) => ({ ...c, title }), (c) => api.updateCard(c.id, { title })),
    [change],
  );

  const setNotes = useCallback(
    (notes: string) => change((c) => ({ ...c, notes }), (c) => api.updateCard(c.id, { notes })),
    [change],
  );

  const setPriority = useCallback(
    (priority: Priority) =>
      change((c) => ({ ...c, priority }), (c) => api.updateCard(c.id, { priority })),
    [change],
  );

  const addSubtask = useCallback(
    async (title: string) => {
      const current = cardRef.current;
      if (!current) return;
      try {
        const subtask = await api.createSubtask(current.id, title);
        await change(
          (c) => state.insertSubtask(c, subtask),
          () => Promise.resolve(),
        );
      } catch (error) {
        notify.error(error);
      }
    },
    [change, notify],
  );

  const toggleSubtask = useCallback(
    (subtask: Subtask, done: boolean) =>
      change(
        (c) => state.updateSubtask(c, subtask.id, { done }),
        () => api.updateSubtask(subtask.id, { done }),
      ),
    [change],
  );

  const renameSubtask = useCallback(
    (subtaskId: Id, title: string) =>
      change(
        (c) => state.updateSubtask(c, subtaskId, { title }),
        () => api.updateSubtask(subtaskId, { title }),
      ),
    [change],
  );

  const moveSubtask = useCallback(
    (subtaskId: Id, index: number) =>
      change(
        (c) => state.moveSubtask(c, subtaskId, index),
        () => api.reorderSubtask(subtaskId, index),
      ),
    [change],
  );

  // Subtasks are deleted for good, so undo re-creates them in place.
  const deleteSubtask = useCallback(
    async (subtask: Subtask) => {
      const current = cardRef.current;
      if (!current) return;
      const cardId = current.id;
      const index = current.subtasks.findIndex((s) => s.id === subtask.id);
      const deleted = await change(
        (c) => state.removeSubtask(c, subtask.id),
        () => api.deleteSubtask(subtask.id),
      );
      if (!deleted) return;
      notify.info(t("toast.subtaskDeleted", { title: subtask.title }), async () => {
        try {
          const restored = await api.createSubtask(cardId, subtask.title);
          await api.reorderSubtask(restored.id, index);
          if (subtask.done) await api.updateSubtask(restored.id, { done: true });
        } catch (error) {
          notify.error(error);
        }
        await load();
        const reloaded = cardRef.current;
        if (reloaded?.id === cardId) {
          callbacksRef.current.onSummary(cardId, state.summaryFields(reloaded));
        }
      });
    },
    [change, load, notify, t],
  );

  const setTags = useCallback(
    (tagIds: Id[]) =>
      change(
        (c) => ({ ...c, tagIds }),
        async (c) => {
          const sorted = await api.setCardTags(c.id, tagIds);
          // The backend sorts by name; adopt its order without another save.
          if (cardRef.current?.id === c.id) {
            show({ ...cardRef.current, tagIds: sorted });
            callbacksRef.current.onSummary(c.id, state.summaryFields(cardRef.current));
          }
        },
      ),
    [change, show],
  );

  const toggleTag = useCallback(
    (tagId: Id) => {
      const current = cardRef.current;
      if (!current) return;
      const tagIds = current.tagIds.includes(tagId)
        ? current.tagIds.filter((id) => id !== tagId)
        : [...current.tagIds, tagId];
      return setTags(tagIds);
    },
    [setTags],
  );

  /** Creates a tag and adds it to the card. */
  const createTag = useCallback(
    async (name: string, color: TagColor) => {
      try {
        const tag = await api.createTag(name, color);
        callbacksRef.current.onTagCreated(tag);
        const current = cardRef.current;
        if (current) await setTags([...current.tagIds, tag.id]);
      } catch (error) {
        notify.error(error);
      }
    },
    [notify, setTags],
  );

  return {
    card,
    setTitle,
    setNotes,
    setPriority,
    addSubtask,
    toggleSubtask,
    renameSubtask,
    moveSubtask,
    deleteSubtask,
    toggleTag,
    createTag,
  };
}

export type CardDetailActions = ReturnType<typeof useCardDetail>;
