import { useEffect, useId, useMemo, useRef, useState, type KeyboardEvent, type PointerEvent, type RefObject } from "react";
import { ImageOff } from "lucide-react";
import { useTranslation } from "react-i18next";
import { imageUrl, type Id, type MoodboardImage } from "../api";
import {
  canvasSize,
  moveRect,
  rectOf,
  resizeFromCorner,
  scaleRect,
  type Corner,
  type LayerAction,
  type Rect,
} from "../moodboard/moodboardState";
import type { Moodboard } from "../moodboard/useMoodboard";
import { isModKey, modShortcut } from "../platform";
import { useContextMenu, type ActionMenuItem } from "./ActionMenu";
import styles from "./MoodboardCanvas.module.css";

const CORNERS: Corner[] = ["nw", "ne", "sw", "se"];
/** A press only becomes a drag after the pointer moved this far. */
const DRAG_THRESHOLD = 3;
const KEY_STEP = 1;
const KEY_STEP_LARGE = 10;
const KEY_SCALE = 1.1;

interface Props {
  boardId: Id;
  moodboard: Moodboard;
  selectedId: Id | null;
  onSelect: (id: Id | null) => void;
  /** Files are dragged over the window. */
  dropping: boolean;
  scrollerRef: RefObject<HTMLDivElement | null>;
  onAddImages: () => void;
}

interface Drag {
  id: Id;
  pointerId: number;
  startX: number;
  startY: number;
  rect: Rect;
  /** Null moves the image, a corner resizes it. */
  corner: Corner | null;
  moved: boolean;
}

/** Layer actions for an image, with the ones that would change nothing disabled. */
export function layerItems(
  t: (key: string) => string,
  images: MoodboardImage[],
  id: Id,
  onLayer: (action: LayerAction) => void,
): ActionMenuItem[] {
  const index = images.findIndex((image) => image.id === id);
  const atFront = index === images.length - 1;
  const atBack = index === 0;
  return [
    { id: "front", label: t("moodboard.layer.front"), disabled: atFront, onAction: () => onLayer("front") },
    { id: "forward", label: t("moodboard.layer.forward"), disabled: atFront, onAction: () => onLayer("forward") },
    { id: "backward", label: t("moodboard.layer.backward"), disabled: atBack, onAction: () => onLayer("backward") },
    { id: "back", label: t("moodboard.layer.back"), disabled: atBack, onAction: () => onLayer("back") },
  ];
}

/** The layer action for a key press, e.g. Mod+] brings an image forward. */
function layerActionFor(event: KeyboardEvent): LayerAction | null {
  if (!isModKey(event)) return null;
  // The physical key, so it works on every keyboard layout.
  if (event.code === "BracketRight") return event.shiftKey ? "front" : "forward";
  if (event.code === "BracketLeft") return event.shiftKey ? "back" : "backward";
  return null;
}

/**
 * The surface of a moodboard: images placed freely, a frame with corner
 * handles around the selected one. It scrolls and grows with the images.
 */
function MoodboardCanvas({
  boardId,
  moodboard,
  selectedId,
  onSelect,
  dropping,
  scrollerRef,
  onAddImages,
}: Props) {
  const { t } = useTranslation();
  const images = moodboard.view?.images ?? [];
  const [viewport, setViewport] = useState({ width: 0, height: 0 });
  const drag = useRef<Drag | null>(null);
  const instructionsId = useId();

  useEffect(() => {
    const scroller = scrollerRef.current;
    if (!scroller) return;
    const observer = new ResizeObserver(() =>
      setViewport({ width: scroller.clientWidth, height: scroller.clientHeight }),
    );
    observer.observe(scroller);
    return () => observer.disconnect();
  }, [scrollerRef]);

  const size = canvasSize(images.map(rectOf), viewport);
  // Tab order stays by creation; the layer only decides what lies on top.
  const inTabOrder = useMemo(() => [...images].sort((a, b) => (a.id < b.id ? -1 : 1)), [images]);
  const layerOf = new Map(images.map((image, index) => [image.id, index]));
  const selected = images.find((image) => image.id === selectedId) ?? null;

  function begin(event: PointerEvent<HTMLElement>, image: MoodboardImage, corner: Corner | null) {
    if (event.button !== 0) return;
    event.stopPropagation();
    event.currentTarget.setPointerCapture(event.pointerId);
    drag.current = {
      id: image.id,
      pointerId: event.pointerId,
      startX: event.clientX,
      startY: event.clientY,
      rect: rectOf(image),
      corner,
      moved: false,
    };
  }

  function move(event: PointerEvent<HTMLElement>) {
    const current = drag.current;
    if (!current || current.pointerId !== event.pointerId) return;
    const dx = event.clientX - current.startX;
    const dy = event.clientY - current.startY;
    if (!current.moved && Math.hypot(dx, dy) < DRAG_THRESHOLD) return;
    current.moved = true;
    moodboard.preview(
      current.id,
      current.corner
        ? resizeFromCorner(current.rect, current.corner, dx, dy)
        : moveRect(current.rect, dx, dy),
    );
  }

  function end(event: PointerEvent<HTMLElement>) {
    const current = drag.current;
    if (!current || current.pointerId !== event.pointerId) return;
    drag.current = null;
    if (current.moved) moodboard.commit(current.id);
  }

  const dragHandlers = { onPointerMove: move, onPointerUp: end, onPointerCancel: end };

  function onKeyDown(event: KeyboardEvent<HTMLButtonElement>, image: MoodboardImage) {
    const rect = rectOf(image);
    const step = event.shiftKey ? KEY_STEP_LARGE : KEY_STEP;
    const moves: Record<string, [number, number] | undefined> = {
      ArrowLeft: [-step, 0],
      ArrowRight: [step, 0],
      ArrowUp: [0, -step],
      ArrowDown: [0, step],
    };
    const layer = layerActionFor(event);
    const delta = isModKey(event) ? undefined : moves[event.key];
    if (layer) {
      moodboard.changeLayer(image.id, layer);
    } else if (delta) {
      moodboard.nudge(image.id, moveRect(rect, delta[0], delta[1]));
    } else if (event.key === "+" || event.key === "=") {
      moodboard.nudge(image.id, scaleRect(rect, KEY_SCALE));
    } else if (event.key === "-") {
      moodboard.nudge(image.id, scaleRect(rect, 1 / KEY_SCALE));
    } else if (event.key === "Delete" || event.key === "Backspace") {
      onSelect(null);
      void moodboard.remove(image.id);
    } else if (event.key === "Escape") {
      onSelect(null);
      event.currentTarget.blur();
    } else {
      return;
    }
    event.preventDefault();
  }

  return (
    <div
      ref={scrollerRef}
      className={styles.scroller}
      data-dropping={dropping || undefined}
      onPointerDown={(event) => {
        if (event.button === 0) onSelect(null);
      }}
    >
      <p id={instructionsId} className={styles.visuallyHidden}>
        {t("moodboard.instructions", {
          forward: modShortcut(t, "]"),
          backward: modShortcut(t, "["),
        })}
      </p>
      <div
        className={styles.surface}
        style={{ width: size.width, height: size.height }}
        role="group"
        aria-label={moodboard.view?.board.name}
      >
        {inTabOrder.map((image, position) => (
          <ImageItem
            key={image.id}
            boardId={boardId}
            image={image}
            images={images}
            label={t("moodboard.imageLabel", { position: position + 1, total: images.length })}
            describedBy={instructionsId}
            layer={layerOf.get(image.id) ?? 0}
            selected={image.id === selectedId}
            onPointerDown={(event) => {
              onSelect(image.id);
              event.currentTarget.focus();
              begin(event, image, null);
            }}
            dragHandlers={dragHandlers}
            onFocus={() => onSelect(image.id)}
            onKeyDown={(event) => onKeyDown(event, image)}
            onLayer={(action) => moodboard.changeLayer(image.id, action)}
            onDelete={() => {
              onSelect(null);
              void moodboard.remove(image.id);
            }}
          />
        ))}

        {selected && (
          <div
            className={styles.frame}
            style={{
              left: selected.x,
              top: selected.y,
              width: selected.width,
              height: selected.height,
              zIndex: images.length + 1,
            }}
            aria-hidden="true"
          >
            {CORNERS.map((corner) => (
              <span
                key={corner}
                className={styles.handle}
                data-corner={corner}
                onPointerDown={(event) => begin(event, selected, corner)}
                {...dragHandlers}
              />
            ))}
          </div>
        )}
      </div>

      {moodboard.view && images.length === 0 && (
        <div className={styles.empty}>
          <h2 className={styles.emptyTitle}>{t("moodboard.empty.title")}</h2>
          <p className={styles.emptyText}>
            {t("moodboard.empty.description", { paste: modShortcut(t, "V") })}
          </p>
          <button
            type="button"
            className={styles.primary}
            onPointerDown={(event) => event.stopPropagation()}
            onClick={onAddImages}
          >
            {t("moodboard.add")}
          </button>
        </div>
      )}
    </div>
  );
}

interface ItemProps {
  boardId: Id;
  image: MoodboardImage;
  images: MoodboardImage[];
  label: string;
  describedBy: string;
  layer: number;
  selected: boolean;
  onPointerDown: (event: PointerEvent<HTMLButtonElement>) => void;
  dragHandlers: {
    onPointerMove: (event: PointerEvent<HTMLElement>) => void;
    onPointerUp: (event: PointerEvent<HTMLElement>) => void;
    onPointerCancel: (event: PointerEvent<HTMLElement>) => void;
  };
  onFocus: () => void;
  onKeyDown: (event: KeyboardEvent<HTMLButtonElement>) => void;
  onLayer: (action: LayerAction) => void;
  onDelete: () => void;
}

/** One image: a button that can be focused, dragged and opened as a menu. */
function ImageItem({
  boardId,
  image,
  images,
  label,
  describedBy,
  layer,
  selected,
  onPointerDown,
  dragHandlers,
  onFocus,
  onKeyDown,
  onLayer,
  onDelete,
}: ItemProps) {
  const { t } = useTranslation();
  const [broken, setBroken] = useState(false);
  const menuItems: ActionMenuItem[] = [
    ...layerItems(t, images, image.id, onLayer),
    { id: "delete", label: t("actions.delete"), danger: true, onAction: onDelete },
  ];
  const contextMenu = useContextMenu(menuItems);

  return (
    <>
      <button
        type="button"
        className={styles.image}
        style={{
          left: image.x,
          top: image.y,
          width: image.width,
          height: image.height,
          zIndex: layer + 1,
        }}
        aria-label={label}
        aria-describedby={describedBy}
        aria-pressed={selected}
        data-selected={selected || undefined}
        onPointerDown={onPointerDown}
        {...dragHandlers}
        onFocus={onFocus}
        onKeyDown={onKeyDown}
        onContextMenu={contextMenu.onContextMenu}
      >
        {broken ? (
          <span className={styles.missing} title={t("moodboard.missing")}>
            <ImageOff aria-hidden="true" />
          </span>
        ) : (
          <img
            src={imageUrl(boardId, image.file)}
            alt=""
            draggable={false}
            decoding="async"
            onError={() => setBroken(true)}
          />
        )}
      </button>
      {contextMenu.menu}
    </>
  );
}

export default MoodboardCanvas;
