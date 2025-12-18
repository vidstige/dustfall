const TILE_WIDTH = 64;
const TILE_HEIGHT = 32;
const MAP_WIDTH = 20;
const MAP_HEIGHT = 20;
const TILE_VARIANTS = 8;

type TileBitmap = HTMLCanvasElement;
type WorldMap = number[][];

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

/**
 * Renders the provided map using the supplied tile bitmaps.
 */
function renderWorld(
  ctx: CanvasRenderingContext2D,
  map: WorldMap,
  tiles: TileBitmap[],
  originX: number,
  originY: number,
): void {
  ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);

  for (let y = 0; y < map.length; y += 1) {
    const row = map[y];
    for (let x = 0; x < row.length; x += 1) {
      const tileIndex = row[x];
      const tile = tiles[tileIndex];
      if (!tile) continue;

      const screenX = originX + (x - y) * (TILE_WIDTH / 2);
      const screenY = originY + (x + y) * (TILE_HEIGHT / 2);
      ctx.drawImage(tile, screenX, screenY);
    }
  }
}

const originX = canvas.width / 2 - TILE_WIDTH / 2;
const originY = 40;

function gameLoop(): void {
  renderWorld(context, worldMap, tileSet, originX, originY);
  requestAnimationFrame(gameLoop);
}

interface IsoEngineAPI {
  tileSet: TileBitmap[];
  worldMap: WorldMap;
  renderWorld: () => void;
}

declare global {
  interface Window {
    isoEngine: IsoEngineAPI;
  }
}

window.isoEngine = {
  tileSet,
  worldMap,
  renderWorld: () => renderWorld(context, worldMap, tileSet, originX, originY),
};

gameLoop();
