import { describe, expect, it } from "vitest";
import type { MoodboardImage, MoodboardView } from "../api";
import {
  CANVAS_MARGIN,
  MIN_IMAGE_SIZE,
  canvasSize,
  cascade,
  insertImage,
  layerTarget,
  moveRect,
  moveToLayer,
  removeImage,
  resizeFromCorner,
  scaleRect,
  setRect,
  type Rect,
} from "./moodboardState";

const rect: Rect = { x: 100, y: 50, width: 200, height: 100 };

function image(id: string, extra: Partial<MoodboardImage> = {}): MoodboardImage {
  return { id, file: `${id}.png`, x: 0, y: 0, width: 10, height: 10, ...extra };
}

function view(...ids: string[]): MoodboardView {
  return {
    board: { id: "m", name: "Ideen", kind: "moodboard" },
    images: ids.map((id) => image(id)),
  };
}

const order = (v: MoodboardView) => v.images.map((i) => i.id);

describe("moveRect", () => {
  it("moves and stops at the top left edge", () => {
    expect(moveRect(rect, 10, -20)).toEqual({ ...rect, x: 110, y: 30 });
    expect(moveRect(rect, -500, -500)).toEqual({ ...rect, x: 0, y: 0 });
  });
});

describe("resizeFromCorner", () => {
  function expectRect(actual: Rect, expected: Rect) {
    for (const key of ["x", "y", "width", "height"] as const) {
      expect(actual[key], key).toBeCloseTo(expected[key]);
    }
  }

  it("keeps the aspect ratio and the opposite corner", () => {
    expectRect(resizeFromCorner(rect, "se", 100, 50), { x: 100, y: 50, width: 300, height: 150 });
    expectRect(resizeFromCorner(rect, "nw", -100, -50), { x: 0, y: 0, width: 300, height: 150 });
    expectRect(resizeFromCorner(rect, "ne", 0, -25), { x: 100, y: 45, width: 210, height: 105 });
  });

  it("follows sideways drags along the diagonal", () => {
    expectRect(resizeFromCorner(rect, "se", 100, 0), { x: 100, y: 50, width: 280, height: 140 });
    expectRect(resizeFromCorner(rect, "sw", 100, 0), { x: 180, y: 50, width: 120, height: 60 });
  });

  it("never gets smaller than the minimum", () => {
    const small = resizeFromCorner(rect, "se", -1000, -1000);
    expect(small.height).toBeCloseTo(MIN_IMAGE_SIZE);
    expect(small.width).toBeCloseTo(MIN_IMAGE_SIZE * 2);
    expect(small.x).toBe(100);
  });

  it("stops growing at the left and top edge", () => {
    // 50 px room above means at most 150 high.
    expectRect(resizeFromCorner(rect, "nw", -1000, -1000), { x: 0, y: 0, width: 300, height: 150 });
  });
});

describe("scaleRect", () => {
  it("scales around the centre", () => {
    expect(scaleRect(rect, 1.5)).toEqual({ x: 50, y: 25, width: 300, height: 150 });
    expect(scaleRect(rect, 0.5)).toEqual({ x: 150, y: 75, width: 100, height: 50 });
  });

  it("stays inside the moodboard and above the minimum", () => {
    const grown = scaleRect({ x: 0, y: 0, width: 100, height: 100 }, 2);
    expect(grown).toEqual({ x: 0, y: 0, width: 200, height: 200 });
    expect(scaleRect(rect, 0.01).height).toBe(MIN_IMAGE_SIZE);
  });
});

describe("layerTarget", () => {
  const images = view("a", "b", "c").images;

  it("finds the new layer", () => {
    expect(layerTarget(images, "a", "front")).toBe(2);
    expect(layerTarget(images, "a", "forward")).toBe(1);
    expect(layerTarget(images, "c", "backward")).toBe(1);
    expect(layerTarget(images, "b", "back")).toBe(0);
  });

  it("returns null when nothing would change", () => {
    expect(layerTarget(images, "c", "front")).toBeNull();
    expect(layerTarget(images, "c", "forward")).toBeNull();
    expect(layerTarget(images, "a", "backward")).toBeNull();
    expect(layerTarget(images, "a", "back")).toBeNull();
    expect(layerTarget(images, "x", "front")).toBeNull();
  });
});

describe("canvasSize", () => {
  it("fills the viewport and grows beyond the outermost image", () => {
    const viewport = { width: 800, height: 600 };
    expect(canvasSize([], viewport)).toEqual(viewport);
    expect(canvasSize([rect], viewport)).toEqual(viewport);
    expect(canvasSize([rect, { x: 900, y: 10, width: 100, height: 700 }], viewport)).toEqual({
      width: 1000 + CANVAS_MARGIN,
      height: 710 + CANVAS_MARGIN,
    });
  });
});

describe("cascade", () => {
  it("offsets each further image", () => {
    expect(cascade({ x: 10, y: 20 }, 0)).toEqual({ x: 10, y: 20 });
    expect(cascade({ x: 10, y: 20 }, 2)).toEqual({ x: 58, y: 68 });
  });
});

describe("view updates", () => {
  it("sets a rectangle", () => {
    const changed = setRect(view("a", "b"), "b", rect);
    expect(changed.images[1]).toMatchObject(rect);
    expect(changed.images[0]).toEqual(image("a"));
  });

  it("moves between layers", () => {
    expect(order(moveToLayer(view("a", "b", "c"), "a", 2))).toEqual(["b", "c", "a"]);
    expect(order(moveToLayer(view("a", "b", "c"), "c", 0))).toEqual(["c", "a", "b"]);
    expect(order(moveToLayer(view("a", "b"), "x", 0))).toEqual(["a", "b"]);
  });

  it("removes and puts back on the same layer", () => {
    const without = removeImage(view("a", "b", "c"), "b");
    expect(order(without)).toEqual(["a", "c"]);
    expect(order(insertImage(without, image("b"), 1))).toEqual(["a", "b", "c"]);
    expect(order(insertImage(without, image("b"), 9))).toEqual(["a", "c", "b"]);
  });
});
