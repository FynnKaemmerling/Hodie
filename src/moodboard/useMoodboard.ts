import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { api, type Id, type MoodboardImage, type MoodboardView } from "../api";
import { useNotifications } from "../notifications/Notifications";
import * as state from "./moodboardState";
import type { LayerAction, Rect } from "./moodboardState";

/** Keyboard nudges are saved once the keys rest this long. */
const NUDGE_SAVE_MS = 400;

type Origin = { x: number; y: number };

/**
 * One moodboard with its images. Like `useBoard`, changes show at once and
 * are saved in the background; a failed save reloads from the backend.
 * Moving and resizing shows every step (`preview`) and saves only the end
 * (`commit`), so dragging writes the file once.
 */
export function useMoodboard(boardId: Id | null) {
  const { t } = useTranslation();
  const notify = useNotifications();
  const [view, setView] = useState<MoodboardView | null>(null);
  const viewRef = useRef(view);
  viewRef.current = view;
  const currentBoard = useRef(boardId);
  currentBoard.current = boardId;
  const nudgeTimers = useRef(new Map<Id, number>());

  const reload = useCallback(async () => {
    if (boardId === null) return;
    try {
      const loaded = await api.getMoodboard(boardId);
      if (currentBoard.current === boardId) setView(loaded);
    } catch (error) {
      notify.error(error);
    }
  }, [boardId, notify]);

  useEffect(() => {
    setView(null);
    void reload();
  }, [reload]);

  const save = useCallback(
    async (action: () => Promise<unknown>) => {
      try {
        await action();
        return true;
      } catch (error) {
        notify.error(error);
        await reload();
        return false;
      }
    },
    [notify, reload],
  );

  /** Shows an image at another place or size without saving it yet. */
  const preview = useCallback((id: Id, rect: Rect) => {
    setView((current) => current && state.setRect(current, id, rect));
  }, []);

  /** Saves where an image is now shown. */
  const commit = useCallback(
    (id: Id) => {
      const timer = nudgeTimers.current.get(id);
      if (timer !== undefined) window.clearTimeout(timer);
      nudgeTimers.current.delete(id);
      const image = viewRef.current?.images.find((candidate) => candidate.id === id);
      if (!image) return;
      void save(() => api.updateMoodboardImage(id, state.rectOf(image)));
    },
    [save],
  );

  /** A step by keyboard: shown at once, saved once the keys rest. */
  const nudge = useCallback(
    (id: Id, rect: Rect) => {
      preview(id, rect);
      const timers = nudgeTimers.current;
      window.clearTimeout(timers.get(id));
      timers.set(
        id,
        window.setTimeout(() => commit(id), NUDGE_SAVE_MS),
      );
    },
    [preview, commit],
  );

  // Pending nudges are saved when the moodboard closes.
  useEffect(() => {
    const timers = nudgeTimers.current;
    return () => {
      for (const id of [...timers.keys()]) commit(id);
    };
  }, [boardId, commit]);

  /** Adds images one after another; returns the ones that made it. */
  const addImages = useCallback(
    async (
      sources: (string | Blob)[],
      origin: Origin,
    ): Promise<MoodboardImage[]> => {
      if (boardId === null) return [];
      const added: MoodboardImage[] = [];
      for (const [n, source] of sources.entries()) {
        const { x, y } = state.cascade(origin, n);
        try {
          const image =
            typeof source === "string"
              ? await api.addMoodboardImage(boardId, source, x, y)
              : await api.addMoodboardImageData(
                  boardId,
                  new Uint8Array(await source.arrayBuffer()),
                  x,
                  y,
                );
          if (currentBoard.current !== boardId) return added;
          setView((current) => current && state.addImage(current, image));
          added.push(image);
        } catch (error) {
          notify.error(error);
        }
      }
      return added;
    },
    [boardId, notify],
  );

  /** Moves an image to another layer. */
  const changeLayer = useCallback(
    (id: Id, action: LayerAction) => {
      const images = viewRef.current?.images ?? [];
      const index = state.layerTarget(images, id, action);
      if (index === null) return;
      setView((current) => current && state.moveToLayer(current, id, index));
      void save(() => api.reorderMoodboardImage(id, index));
    },
    [save],
  );

  /** Removes an image, with undo. */
  const remove = useCallback(
    async (id: Id) => {
      const images = viewRef.current?.images ?? [];
      const index = images.findIndex((image) => image.id === id);
      if (index === -1) return;
      const image = images[index];
      setView((current) => current && state.removeImage(current, id));
      const removed = await save(() => api.deleteMoodboardImage(id));
      if (!removed) return;
      notify.info(t("toast.imageDeleted"), async () => {
        try {
          await api.restoreMoodboardImage(id);
          if (currentBoard.current !== boardId) return;
          setView((current) => current && state.insertImage(current, image, index));
        } catch (error) {
          notify.error(error);
          await reload();
        }
      });
    },
    [boardId, notify, reload, save, t],
  );

  return { view, reload, preview, commit, nudge, addImages, changeLayer, remove };
}

export type Moodboard = ReturnType<typeof useMoodboard>;
