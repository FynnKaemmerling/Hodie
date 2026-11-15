import type { Id, MoodboardImage, MoodboardView } from "../api";

/** Position and size in CSS pixels on the moodboard. */
export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export type Corner = "nw" | "ne" | "sw" | "se";

export type LayerAction = "front" | "forward" | "backward" | "back";

/** Images never get smaller than this (shorter side); matches the backend. */
export const MIN_IMAGE_SIZE = 16;

/** Free space kept right of and below the outermost image while scrolling. */
export const CANVAS_MARGIN = 160;

/** Offset between images added at once, so they do not hide each other. */
export const CASCADE_STEP = 24;

export function rectOf(image: MoodboardImage): Rect {
  return { x: image.x, y: image.y, width: image.width, height: image.height };
}

/** Moves a rectangle, keeping it inside the moodboard (no negative coordinates). */
export function moveRect(rect: Rect, dx: number, dy: number): Rect {
  return { ...rect, x: Math.max(0, rect.x + dx), y: Math.max(0, rect.y + dy) };
}

/** Smallest scale factor that keeps both sides at least `MIN_IMAGE_SIZE`. */
function minScale(rect: Rect): number {
  return Math.min(1, MIN_IMAGE_SIZE / Math.min(rect.width, rect.height));
}

/**
 * Resizes by dragging a corner by dx, dy. The aspect ratio stays and the
 * opposite corner stays where it is: the pointer's movement counts along the
 * image's diagonal, so dragging sideways or up and down works alike.
 */
export function resizeFromCorner(rect: Rect, corner: Corner, dx: number, dy: number): Rect {
  const east = corner === "ne" || corner === "se";
  const south = corner === "sw" || corner === "se";
  const outward = (east ? dx : -dx) * rect.width + (south ? dy : -dy) * rect.height;
  const diagonal = rect.width ** 2 + rect.height ** 2;
  let scale = Math.max(1 + outward / diagonal, minScale(rect));
  // Growing to the left or top stops at the moodboard's edge.
  const right = rect.x + rect.width;
  const bottom = rect.y + rect.height;
  if (!east) scale = Math.min(scale, right / rect.width);
  if (!south) scale = Math.min(scale, bottom / rect.height);
  const width = rect.width * scale;
  const height = rect.height * scale;
  return {
    x: east ? rect.x : right - width,
    y: south ? rect.y : bottom - height,
    width,
    height,
  };
}

/** Scales around the centre, e.g. by keyboard; stays inside the moodboard. */
export function scaleRect(rect: Rect, factor: number): Rect {
  const scale = Math.max(factor, minScale(rect));
  const width = rect.width * scale;
  const height = rect.height * scale;
  return {
    x: Math.max(0, rect.x + (rect.width - width) / 2),
    y: Math.max(0, rect.y + (rect.height - height) / 2),
    width,
    height,
  };
}

/** The layer (0 = back) an action moves an image to, or null if it stays. */
export function layerTarget(images: MoodboardImage[], id: Id, action: LayerAction): number | null {
  const index = images.findIndex((image) => image.id === id);
  if (index === -1) return null;
  const last = images.length - 1;
  const target = {
    front: last,
    forward: Math.min(index + 1, last),
    backward: Math.max(index - 1, 0),
    back: 0,
  }[action];
  return target === index ? null : target;
}

/** Size of the scrollable surface: at least the visible area, and always room beyond the images. */
export function canvasSize(
  rects: Rect[],
  viewport: { width: number; height: number },
): { width: number; height: number } {
  let width = viewport.width;
  let height = viewport.height;
  for (const rect of rects) {
    width = Math.max(width, rect.x + rect.width + CANVAS_MARGIN);
    height = Math.max(height, rect.y + rect.height + CANVAS_MARGIN);
  }
  return { width, height };
}

/** Where the n-th of several images added together goes. */
export function cascade(origin: { x: number; y: number }, n: number): { x: number; y: number } {
  return { x: origin.x + n * CASCADE_STEP, y: origin.y + n * CASCADE_STEP };
}

// Updates to a loaded moodboard, applied before the backend confirms them.

export function setRect(view: MoodboardView, id: Id, rect: Rect): MoodboardView {
  return {
    ...view,
    images: view.images.map((image) => (image.id === id ? { ...image, ...rect } : image)),
  };
}

export function moveToLayer(view: MoodboardView, id: Id, index: number): MoodboardView {
  const image = view.images.find((candidate) => candidate.id === id);
  if (!image) return view;
  const rest = view.images.filter((candidate) => candidate.id !== id);
  const at = Math.max(0, Math.min(index, rest.length));
  return { ...view, images: [...rest.slice(0, at), image, ...rest.slice(at)] };
}

export function addImage(view: MoodboardView, image: MoodboardImage): MoodboardView {
  return { ...view, images: [...view.images, image] };
}

export function insertImage(view: MoodboardView, image: MoodboardImage, index: number): MoodboardView {
  return moveToLayer(addImage(view, image), image.id, index);
}

export function removeImage(view: MoodboardView, id: Id): MoodboardView {
  return { ...view, images: view.images.filter((image) => image.id !== id) };
}
