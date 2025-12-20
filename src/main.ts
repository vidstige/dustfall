import { IsometricRenderer } from "./isometricRenderer";
import { WorldMap, WorldMapData } from "./worldMap";
import { loadImageTileSet } from "./imageTileSet";

const TILE_WIDTH = 64;
const TILE_HEIGHT = 32;
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

let renderer: IsometricRenderer | null = null;

init().catch((error) => {
  console.error("Failed to initialize renderer", error);
});

async function init(): Promise<void> {
  const tileSet = await loadImageTileSet(TILE_WIDTH, TILE_HEIGHT);
  const worldMap = createWorldMap(MAP_WIDTH, MAP_HEIGHT, tileSet.length);

  renderer = new IsometricRenderer(
    {
      tileWidth: TILE_WIDTH,
      tileHeight: TILE_HEIGHT,
      viewOrigin: {
        x: canvas.width / 2,
        y: 120,
      },
    },
    tileSet,
    worldMap,
  );

  canvas.addEventListener("wheel", handleWheel, { passive: false });
  gameLoop();
}

function handleWheel(event: WheelEvent): void {
  if (!renderer) {
    return;
  }
  event.preventDefault();
  renderer.panByPixels(event.deltaX, event.deltaY);
}

function gameLoop(): void {
  if (!renderer) {
    return;
  }
  renderer.render(context);
  requestAnimationFrame(gameLoop);
}

function createWorldMap(width: number, height: number, tileCount: number): WorldMap {
  const data: WorldMapData = [];
  for (let y = 0; y < height; y += 1) {
    const row: number[] = [];
    for (let x = 0; x < width; x += 1) {
      row.push(Math.floor(Math.random() * tileCount));
    }
    data.push(row);
  }
  return new WorldMap(data);
}
