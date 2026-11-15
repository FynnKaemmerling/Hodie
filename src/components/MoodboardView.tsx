import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import { ArrowDown, ArrowUp, BringToFront, ImagePlus, SendToBack, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { Board, Id, MoodboardImage } from "../api";
import type { Moodboard } from "../moodboard/useMoodboard";
import { isTextInput } from "../platform";
import IconButton from "./IconButton";
import MoodboardCanvas from "./MoodboardCanvas";
import styles from "./MoodboardView.module.css";

/** Formats all three system web views display; the backend checks the content. */
const IMAGE_EXTENSIONS = ["png", "jpg", "jpeg", "gif", "webp"];

/** Distance from the visible top left corner where added images appear. */
const ADD_INSET = 40;

interface Props {
  board: Board;
  moodboard: Moodboard;
  /** Shown before the title, e.g. the button that brings back the sidebar. */
  leading?: ReactNode;
  /** Shown at the end of the toolbar, e.g. the window controls. */
  trailing?: ReactNode;
}

/** A moodboard: its toolbar and the surface with the images. */
function MoodboardView({ board, moodboard, leading, trailing }: Props) {
  const { t } = useTranslation();
  const [selectedId, setSelectedId] = useState<Id | null>(null);
  const [dropping, setDropping] = useState(false);
  const scrollerRef = useRef<HTMLDivElement>(null);
  const images = moodboard.view?.images ?? [];
  const selected = images.find((image) => image.id === selectedId) ?? null;

  useEffect(() => setSelectedId(null), [board.id]);

  /** Top left of the visible part of the moodboard, in its coordinates. */
  const visibleOrigin = useCallback(() => {
    const scroller = scrollerRef.current;
    return {
      x: (scroller?.scrollLeft ?? 0) + ADD_INSET,
      y: (scroller?.scrollTop ?? 0) + ADD_INSET,
    };
  }, []);

  const add = useCallback(
    async (sources: (string | Blob)[], origin: { x: number; y: number }) => {
      const added: MoodboardImage[] = await moodboard.addImages(sources, origin);
      const last = added[added.length - 1];
      if (last) setSelectedId(last.id);
    },
    [moodboard],
  );

  // The listeners below stay subscribed and always reach the current `add`.
  const addRef = useRef(add);
  addRef.current = add;

  const pickImages = useCallback(async () => {
    const picked = await open({
      multiple: true,
      directory: false,
      filters: [{ name: t("moodboard.imageFiles"), extensions: IMAGE_EXTENSIONS }],
    });
    if (picked && picked.length > 0) await add(picked, visibleOrigin());
  }, [add, t, visibleOrigin]);

  // Files dropped from the file manager land where they are dropped.
  useEffect(() => {
    const unlisten = getCurrentWebview().onDragDropEvent(({ payload }) => {
      if (payload.type === "enter" || payload.type === "over") {
        setDropping(true);
        return;
      }
      setDropping(false);
      if (payload.type !== "drop" || payload.paths.length === 0) return;
      const scroller = scrollerRef.current;
      if (!scroller) return;
      const bounds = scroller.getBoundingClientRect();
      // The position is in physical pixels of the window.
      const x = payload.position.x / window.devicePixelRatio - bounds.left;
      const y = payload.position.y / window.devicePixelRatio - bounds.top;
      const inside = x >= 0 && y >= 0 && x <= bounds.width && y <= bounds.height;
      const origin = inside
        ? { x: x + scroller.scrollLeft, y: y + scroller.scrollTop }
        : visibleOrigin();
      void addRef.current(payload.paths, origin);
    });
    return () => void unlisten.then((stop) => stop());
  }, [visibleOrigin]);

  // Pasted images (screenshots, copied from a browser) are added too.
  useEffect(() => {
    function onPaste(event: ClipboardEvent) {
      if (isTextInput(event.target)) return;
      const files = Array.from(event.clipboardData?.files ?? []).filter((file) =>
        file.type.startsWith("image/"),
      );
      if (files.length === 0) return;
      event.preventDefault();
      void addRef.current(files, visibleOrigin());
    }
    document.addEventListener("paste", onPaste);
    return () => document.removeEventListener("paste", onPaste);
  }, [visibleOrigin]);

  const atFront = selected !== null && images[images.length - 1]?.id === selected.id;
  const atBack = selected !== null && images[0]?.id === selected.id;

  return (
    <>
      {/* The free space of the toolbar moves the frameless window. */}
      <header className={styles.toolbar} data-tauri-drag-region>
        {leading}
        <h1 className={styles.title} data-tauri-drag-region>
          {board.name}
        </h1>
        {selected && (
          <div className={styles.group} role="group" aria-label={t("moodboard.selection")}>
            <IconButton
              label={t("moodboard.layer.front")}
              disabled={atFront}
              onClick={() => moodboard.changeLayer(selected.id, "front")}
            >
              <BringToFront aria-hidden="true" />
            </IconButton>
            <IconButton
              label={t("moodboard.layer.forward")}
              disabled={atFront}
              onClick={() => moodboard.changeLayer(selected.id, "forward")}
            >
              <ArrowUp aria-hidden="true" />
            </IconButton>
            <IconButton
              label={t("moodboard.layer.backward")}
              disabled={atBack}
              onClick={() => moodboard.changeLayer(selected.id, "backward")}
            >
              <ArrowDown aria-hidden="true" />
            </IconButton>
            <IconButton
              label={t("moodboard.layer.back")}
              disabled={atBack}
              onClick={() => moodboard.changeLayer(selected.id, "back")}
            >
              <SendToBack aria-hidden="true" />
            </IconButton>
            <IconButton
              label={t("actions.delete")}
              tone="danger"
              onClick={() => {
                setSelectedId(null);
                void moodboard.remove(selected.id);
              }}
            >
              <Trash2 aria-hidden="true" />
            </IconButton>
          </div>
        )}
        <IconButton label={t("moodboard.add")} onClick={() => void pickImages()}>
          <ImagePlus aria-hidden="true" />
        </IconButton>
        {trailing}
      </header>
      <div className={styles.content}>
        <MoodboardCanvas
          boardId={board.id}
          moodboard={moodboard}
          selectedId={selected?.id ?? null}
          onSelect={setSelectedId}
          dropping={dropping}
          scrollerRef={scrollerRef}
          onAddImages={() => void pickImages()}
        />
      </div>
    </>
  );
}

export default MoodboardView;
