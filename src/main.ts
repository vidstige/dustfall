import { IsometricRenderer } from "./isometricRenderer";
import { WorldMap, WorldMapData } from "./worldMap";

const TILE_WIDTH = 64;
const TILE_HEIGHT = 32;
const MAP_WIDTH = 220;
const MAP_HEIGHT = 220;
const TILE_IMAGE_URLS = [
  new URL("./assets/tile53.png", import.meta.url).toString(),
  new URL("./assets/tile100.png", import.meta.url).toString(),
  new URL("./assets/tile108.png", import.meta.url).toString(),
  new URL("./assets/tile163.png", import.meta.url).toString(),
  new URL("./assets/tile178.png", import.meta.url).toString(),
  new URL("./assets/tile192.png", import.meta.url).toString(),
  new URL("./assets/tile255.png", import.meta.url).toString(),
  new URL("./assets/tile298.png", import.meta.url).toString(),
  new URL("./assets/tile312.png", import.meta.url).toString(),
];

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
  const tileSet = await loadTileBitmaps();
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

async function loadTileBitmaps(): Promise<HTMLCanvasElement[]> {
  const images = await Promise.all(TILE_IMAGE_URLS.map(loadImage));
  return images.map((image) => imageToCanvas(image, TILE_WIDTH, TILE_HEIGHT));
}

function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = () => reject(new Error(`Failed to load tile image: ${String(src)}`));
    img.src = src;
  });
}

function imageToCanvas(image: HTMLImageElement, width: number, height: number): HTMLCanvasElement {
  const tileCanvas = document.createElement("canvas");
  tileCanvas.width = width;
  tileCanvas.height = height;
  const tileCtx = tileCanvas.getContext("2d");
  if (!tileCtx) {
    throw new Error("Unable to create tile canvas context.");
  }
  tileCtx.clearRect(0, 0, width, height);
  tileCtx.drawImage(image, 0, 0, width, height);
  return tileCanvas;
}
