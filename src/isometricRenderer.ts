export type TileBitmap = HTMLCanvasElement;
export type WorldMap = number[][];

export interface Camera {
  x: number;
  y: number;
}

export interface Point2D {
  x: number;
  y: number;
}

export interface IsoRendererConfig {
  tileWidth: number;
  tileHeight: number;
  tileVariants: number;
  mapWidth: number;
  mapHeight: number;
  viewOrigin: Point2D;
}

export class IsometricRenderer {
  public readonly tileWidth: number;
  public readonly tileHeight: number;
  public readonly mapWidth: number;
  public readonly mapHeight: number;
  public readonly tileSet: TileBitmap[];
  public readonly worldMap: WorldMap;
  public readonly camera: Camera;

  private readonly tileVariants: number;
  private readonly halfTileWidth: number;
  private readonly halfTileHeight: number;
  private viewOrigin: Point2D;

  constructor(config: IsoRendererConfig) {
    this.tileWidth = config.tileWidth;
    this.tileHeight = config.tileHeight;
    this.tileVariants = config.tileVariants;
    this.mapWidth = config.mapWidth;
    this.mapHeight = config.mapHeight;
    this.halfTileWidth = this.tileWidth / 2;
    this.halfTileHeight = this.tileHeight / 2;
    this.viewOrigin = { ...config.viewOrigin };

    this.tileSet = createRandomTileSet(this.tileVariants, this.tileWidth, this.tileHeight);
    this.worldMap = createWorldMap(this.mapWidth, this.mapHeight, this.tileSet.length);
    this.camera = {
      x: this.mapWidth / 2,
      y: this.mapHeight / 2,
    };
  }

  setViewOrigin(origin: Point2D): void {
    this.viewOrigin = { ...origin };
  }

  panByPixels(deltaScreenX: number, deltaScreenY: number): void {
    const deltaTileX =
      0.5 * (deltaScreenX / this.halfTileWidth + deltaScreenY / this.halfTileHeight);
    const deltaTileY =
      0.5 * (deltaScreenY / this.halfTileHeight - deltaScreenX / this.halfTileWidth);

    this.camera.x += deltaTileX;
    this.camera.y += deltaTileY;
    this.constrainCameraPosition();
  }

  render(ctx: CanvasRenderingContext2D): void {
    ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);

    const mapHeight = this.worldMap.length;
    const mapWidth = this.worldMap[0]?.length ?? 0;
    if (mapHeight === 0 || mapWidth === 0) {
      return;
    }

    const cameraPixel = this.isoToPixel(this.camera.x, this.camera.y);
    const corners = [
      this.screenToTileCoord(0, 0, cameraPixel),
      this.screenToTileCoord(ctx.canvas.width, 0, cameraPixel),
      this.screenToTileCoord(0, ctx.canvas.height, cameraPixel),
      this.screenToTileCoord(ctx.canvas.width, ctx.canvas.height, cameraPixel),
    ];

    const margin = 2;
    const minX = clamp(Math.floor(Math.min(...corners.map((c) => c.x))) - margin, 0, mapWidth - 1);
    const maxX = clamp(Math.ceil(Math.max(...corners.map((c) => c.x))) + margin, 0, mapWidth - 1);
    const minY = clamp(Math.floor(Math.min(...corners.map((c) => c.y))) - margin, 0, mapHeight - 1);
    const maxY = clamp(Math.ceil(Math.max(...corners.map((c) => c.y))) + margin, 0, mapHeight - 1);

    for (let y = minY; y <= maxY; y += 1) {
      const row = this.worldMap[y];
      if (!row) continue;

      for (let x = minX; x <= maxX; x += 1) {
        const tileIndex = row[x];
        const tile = this.tileSet[tileIndex];
        if (!tile) continue;

        const tilePixel = this.isoToPixel(x, y);
        const screenX = this.viewOrigin.x + tilePixel.x - cameraPixel.x;
        const screenY = this.viewOrigin.y + tilePixel.y - cameraPixel.y;

        if (
          screenX + this.tileWidth < 0 ||
          screenX > ctx.canvas.width ||
          screenY + this.tileHeight < 0 ||
          screenY > ctx.canvas.height
        ) {
          continue;
        }

        ctx.drawImage(tile, screenX, screenY);
      }
    }
  }

  private isoToPixel(x: number, y: number): Point2D {
    return {
      x: (x - y) * this.halfTileWidth,
      y: (x + y) * this.halfTileHeight,
    };
  }

  private screenToTileCoord(screenX: number, screenY: number, cameraPixel: Point2D): Point2D {
    const isoX = screenX - this.viewOrigin.x + cameraPixel.x;
    const isoY = screenY - this.viewOrigin.y + cameraPixel.y;
    const a = isoX / this.halfTileWidth;
    const b = isoY / this.halfTileHeight;
    return {
      x: (a + b) / 2,
      y: (b - a) / 2,
    };
  }

  private constrainCameraPosition(): void {
    this.camera.x = clamp(this.camera.x, 0, this.mapWidth - 1);
    this.camera.y = clamp(this.camera.y, 0, this.mapHeight - 1);
  }
}

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

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}
