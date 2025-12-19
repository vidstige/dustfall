import { IsometricRenderer, Camera, TileBitmap, WorldMap } from "./isometricRenderer";

const WHEEL_LINE_HEIGHT = 16;
const TILE_WIDTH = 64;
const TILE_HEIGHT = 32;
const TILE_VARIANTS = 8;
const MAP_WIDTH = 220;
const MAP_HEIGHT = 220;

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

const tileSet = createRandomTileSet(TILE_VARIANTS, TILE_WIDTH, TILE_HEIGHT);
const worldMap = createWorldMap(MAP_WIDTH, MAP_HEIGHT, tileSet.length);

const renderer = new IsometricRenderer(
  {
    tileWidth: TILE_WIDTH,
    tileHeight: TILE_HEIGHT,
    mapWidth: MAP_WIDTH,
    mapHeight: MAP_HEIGHT,
    viewOrigin: {
      x: canvas.width / 2,
      y: 120,
    },
  },
  tileSet,
  worldMap,
);

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

function createRandomTileSet(count: number, width: number, height: number): TileBitmap[] {
  const tiles: TileBitmap[] = [];
  for (let i = 0; i < count; i += 1) {
    const tileCanvas = document.createElement("canvas");
    tileCanvas.width = width;
    tileCanvas.height = height;
    const tileCtx = tileCanvas.getContext("2d");
    if (!tileCtx) {
      continue;
    }

    tileCtx.clearRect(0, 0, width, height);

    const hue = Math.floor(Math.random() * 360);
    const gradient = tileCtx.createLinearGradient(0, 0, 0, height);
    gradient.addColorStop(0, `hsl(${hue}, 60%, 70%)`);
    gradient.addColorStop(1, `hsl(${hue}, 60%, 35%)`);

    tileCtx.beginPath();
    tileCtx.moveTo(width / 2, 0);
    tileCtx.lineTo(width, height / 2);
    tileCtx.lineTo(width / 2, height);
    tileCtx.lineTo(0, height / 2);
    tileCtx.closePath();
    tileCtx.fillStyle = gradient;
    tileCtx.fill();

    // Scatter a few random strokes to give each tile unique texture.
    tileCtx.strokeStyle = `hsla(${hue}, 30%, 20%, 0.25)`;
    tileCtx.lineWidth = 1;
    for (let d = 0; d < 6; d += 1) {
      const px = Math.random() * width;
      const py = Math.random() * height;
      tileCtx.beginPath();
      tileCtx.moveTo(px, py);
      tileCtx.lineTo(px + Math.random() * 6 - 3, py + Math.random() * 6 - 3);
      tileCtx.stroke();
    }

    tiles.push(tileCanvas);
  }
  return tiles;
}

function createWorldMap(width: number, height: number, tileCount: number): WorldMap {
  const map: WorldMap = [];
  for (let y = 0; y < height; y += 1) {
    const row: number[] = [];
    for (let x = 0; x < width; x += 1) {
      row.push(Math.floor(Math.random() * tileCount));
    }
    map.push(row);
  }
  return map;
}
