const TILE_WIDTH = 64;
const TILE_HEIGHT = 32;
const HALF_TILE_WIDTH = TILE_WIDTH / 2;
const HALF_TILE_HEIGHT = TILE_HEIGHT / 2;
const MAP_WIDTH = 220;
const MAP_HEIGHT = 220;
const TILE_VARIANTS = 8;
const WHEEL_LINE_HEIGHT = 16;

type TileBitmap = HTMLCanvasElement;
type WorldMap = number[][];

interface Camera {
  x: number;
  y: number;
}

interface Point2D {
  x: number;
  y: number;
}

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

/**
 * Creates a list of canvases that act as isometric tile bitmaps.
 * Each tile is seeded with random colors and tiny accents to keep them unique.
 */
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
    tiles.push(tileCanvas);
  }
  return tiles;
}

/**
 * Generates the logical layout of the world, assigning tile indices randomly.
 */
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

const tileSet = createRandomTileSet(TILE_VARIANTS, TILE_WIDTH, TILE_HEIGHT);
const worldMap = createWorldMap(MAP_WIDTH, MAP_HEIGHT, tileSet.length);

const canvas = assertCanvas(document.getElementById("isoCanvas"));
const context = assertContext(canvas);
canvas.addEventListener("wheel", handleWheel, { passive: false });

const viewOrigin: Point2D = {
  x: canvas.width / 2,
  y: 120,
};

const camera: Camera = {
  x: MAP_WIDTH / 2,
  y: MAP_HEIGHT / 2,
};

function isoToPixel(x: number, y: number): Point2D {
  return {
    x: (x - y) * HALF_TILE_WIDTH,
    y: (x + y) * HALF_TILE_HEIGHT,
  };
}

function screenToTileCoord(
  screenX: number,
  screenY: number,
  cameraPixel: Point2D,
  origin: Point2D,
): Point2D {
  const isoX = screenX - origin.x + cameraPixel.x;
  const isoY = screenY - origin.y + cameraPixel.y;
  const a = isoX / HALF_TILE_WIDTH;
  const b = isoY / HALF_TILE_HEIGHT;
  return {
    x: (a + b) / 2,
    y: (b - a) / 2,
  };
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

function constrainCameraPosition(activeCamera: Camera): void {
  activeCamera.x = clamp(activeCamera.x, 0, MAP_WIDTH - 1);
  activeCamera.y = clamp(activeCamera.y, 0, MAP_HEIGHT - 1);
}

function panCameraByPixels(deltaScreenX: number, deltaScreenY: number): void {
  const deltaTileX =
    0.5 * (deltaScreenX / HALF_TILE_WIDTH + deltaScreenY / HALF_TILE_HEIGHT);
  const deltaTileY =
    0.5 * (deltaScreenY / HALF_TILE_HEIGHT - deltaScreenX / HALF_TILE_WIDTH);

  camera.x -= deltaTileX;
  camera.y -= deltaTileY;
  constrainCameraPosition(camera);
}

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

  panCameraByPixels(pixelX, pixelY);
}

/**
 * Renders the provided map using the supplied tile bitmaps.
 */
function renderWorld(
  ctx: CanvasRenderingContext2D,
  map: WorldMap,
  tiles: TileBitmap[],
  activeCamera: Camera,
  origin: Point2D,
): void {
  ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);

  const mapHeight = map.length;
  const mapWidth = map[0]?.length ?? 0;
  if (mapHeight === 0 || mapWidth === 0) {
    return;
  }

  const cameraPixel = isoToPixel(activeCamera.x, activeCamera.y);
  const corners = [
    screenToTileCoord(0, 0, cameraPixel, origin),
    screenToTileCoord(ctx.canvas.width, 0, cameraPixel, origin),
    screenToTileCoord(0, ctx.canvas.height, cameraPixel, origin),
    screenToTileCoord(ctx.canvas.width, ctx.canvas.height, cameraPixel, origin),
  ];

  const margin = 2;
  const minX = clamp(Math.floor(Math.min(...corners.map((c) => c.x))) - margin, 0, mapWidth - 1);
  const maxX = clamp(Math.ceil(Math.max(...corners.map((c) => c.x))) + margin, 0, mapWidth - 1);
  const minY = clamp(Math.floor(Math.min(...corners.map((c) => c.y))) - margin, 0, mapHeight - 1);
  const maxY = clamp(Math.ceil(Math.max(...corners.map((c) => c.y))) + margin, 0, mapHeight - 1);

  for (let y = minY; y <= maxY; y += 1) {
    const row = map[y];
    if (!row) continue;

    for (let x = minX; x <= maxX; x += 1) {
      const tileIndex = row[x];
      const tile = tiles[tileIndex];
      if (!tile) continue;

      const tilePixel = isoToPixel(x, y);
      const screenX = origin.x + tilePixel.x - cameraPixel.x;
      const screenY = origin.y + tilePixel.y - cameraPixel.y;

      if (
        screenX + TILE_WIDTH < 0 ||
        screenX > ctx.canvas.width ||
        screenY + TILE_HEIGHT < 0 ||
        screenY > ctx.canvas.height
      ) {
        continue;
      }

      ctx.drawImage(tile, screenX, screenY);
    }
  }
}

function gameLoop(): void {
  renderWorld(context, worldMap, tileSet, camera, viewOrigin);
  requestAnimationFrame(gameLoop);
}

interface IsoEngineAPI {
  tileSet: TileBitmap[];
  worldMap: WorldMap;
  camera: Camera;
  renderWorld: () => void;
  pan: (pixelX: number, pixelY: number) => void;
}

declare global {
  interface Window {
    isoEngine: IsoEngineAPI;
  }
}

window.isoEngine = {
  tileSet,
  worldMap,
  camera,
  renderWorld: () => renderWorld(context, worldMap, tileSet, camera, viewOrigin),
  pan: panCameraByPixels,
};

gameLoop();

export {};
