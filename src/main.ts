import {
  IsometricRenderer,
  Camera,
  Point2D,
  TileBitmap,
  WorldMap,
} from "./isometricRenderer";

const WHEEL_LINE_HEIGHT = 16;

function assertCanvas(element: HTMLElement | null): HTMLCanvasElement {
  if (!(element instanceof HTMLCanvasElement)) {
    throw new Error("Expected a canvas element with id 'isoCanvas'.");
  }
  return element;
}

function assertContext(canvas: HTMLCanvasElement): CanvasRenderingContext2D {
  const ctx = canvas.getContext("2d");
  if (!ctx) {
    throw new Error("Unable to acquire 2D rendering context.");
  }
  return ctx;
}

const canvas = assertCanvas(document.getElementById("isoCanvas"));
const context = assertContext(canvas);

const renderer = new IsometricRenderer({
  tileWidth: 64,
  tileHeight: 32,
  tileVariants: 8,
  mapWidth: 220,
  mapHeight: 220,
  viewOrigin: {
    x: canvas.width / 2,
    y: 120,
  },
});

canvas.addEventListener("wheel", handleWheel, { passive: false });

function handleWheel(event: WheelEvent): void {
  event.preventDefault();

  const scale =
    event.deltaMode === WheelEvent.DOM_DELTA_LINE
      ? WHEEL_LINE_HEIGHT
      : event.deltaMode === WheelEvent.DOM_DELTA_PAGE
        ? canvas.height
        : 1;
  const pixelX = event.deltaX * scale;
  const pixelY = event.deltaY * scale;

  renderer.panByPixels(pixelX, pixelY);
}

function gameLoop(): void {
  renderer.render(context);
  requestAnimationFrame(gameLoop);
}

interface IsoEngineAPI {
  renderer: IsometricRenderer;
  camera: Camera;
  tileSet: TileBitmap[];
  worldMap: WorldMap;
  pan: (pixelX: number, pixelY: number) => void;
  render: () => void;
}

declare global {
  interface Window {
    isoEngine: IsoEngineAPI;
  }
}

window.isoEngine = {
  renderer,
  camera: renderer.camera,
  tileSet: renderer.tileSet,
  worldMap: renderer.worldMap,
  pan: (pixelX: number, pixelY: number) => renderer.panByPixels(pixelX, pixelY),
  render: () => renderer.render(context),
};

gameLoop();
