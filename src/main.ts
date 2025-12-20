import { IsometricRenderer, Camera, TileBitmap, WorldMap } from "./isometricRenderer";
import { createRandomTileSet } from "./randomTileSet";

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
