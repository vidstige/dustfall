import type { TileBitmap } from "./isometricRenderer";

export function createRandomTileSet(count: number, width: number, height: number): TileBitmap[] {
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
