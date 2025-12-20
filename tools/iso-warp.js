const fs = require("node:fs");
const path = require("node:path");
const { createCanvas, loadImage } = require("canvas");

function clamp(x, lo, hi) {
  return x < lo ? lo : x > hi ? hi : x;
}

function sampleBilinear(src, sw, sh, u, v) {
  const x0 = Math.floor(u);
  const y0 = Math.floor(v);
  const x1 = clamp(x0 + 1, 0, sw - 1);
  const y1 = clamp(y0 + 1, 0, sh - 1);

  const tx = u - x0;
  const ty = v - y0;

  const i00 = (y0 * sw + x0) * 4;
  const i10 = (y0 * sw + x1) * 4;
  const i01 = (y1 * sw + x0) * 4;
  const i11 = (y1 * sw + x1) * 4;

  const out = [0, 0, 0, 0];
  for (let c = 0; c < 4; c += 1) {
    const c00 = src[i00 + c];
    const c10 = src[i10 + c];
    const c01 = src[i01 + c];
    const c11 = src[i11 + c];

    const a = c00 * (1 - tx) + c10 * tx;
    const b = c01 * (1 - tx) + c11 * tx;
    out[c] = a * (1 - ty) + b * ty;
  }
  return out;
}

const TILE_WIDTH = 64;
const TILE_HEIGHT = 32;

async function main() {
  const inputPath = process.argv[2] ?? "input.png";
  const outputDir = process.argv[3] ?? "tiles";
  const scale = 1;

  const img = await loadImage(inputPath);

  await fs.promises.mkdir(outputDir, { recursive: true });

  const srcCanvas = createCanvas(img.width, img.height);
  const sctx = srcCanvas.getContext("2d");
  sctx.drawImage(img, 0, 0);
  const srcImgData = sctx.getImageData(0, 0, img.width, img.height);
  const src = srcImgData.data;
  const sw = img.width;
  const sh = img.height;

  const a = scale;
  const b = scale / 2;

  const corners = [
    [0, 0],
    [sw - 1, 0],
    [0, sh - 1],
    [sw - 1, sh - 1],
  ];

  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;
  for (const [u, v] of corners) {
    const x = a * (u - v);
    const y = b * (u + v);
    minX = Math.min(minX, x);
    minY = Math.min(minY, y);
    maxX = Math.max(maxX, x);
    maxY = Math.max(maxY, y);
  }

  const pad = 2;
  minX -= pad;
  minY -= pad;
  maxX += pad;
  maxY += pad;

  const outW = Math.ceil(maxX - minX + 1);
  const outH = Math.ceil(maxY - minY + 1);

  const outCanvas = createCanvas(outW, outH);
  const octx = outCanvas.getContext("2d");
  const outImgData = octx.createImageData(outW, outH);
  const out = outImgData.data;

  for (let py = 0; py < outH; py += 1) {
    const y = py + minY;
    const yOverB = y / b;
    for (let px = 0; px < outW; px += 1) {
      const x = px + minX;
      const xOverA = x / a;

      const u = 0.5 * (xOverA + yOverB);
      const v = 0.5 * (yOverB - xOverA);

      const di = (py * outW + px) * 4;
      if (u >= 0 && u <= sw - 1 && v >= 0 && v <= sh - 1) {
        const [r, g, b2, a2] = sampleBilinear(src, sw, sh, u, v);
        out[di + 0] = r | 0;
        out[di + 1] = g | 0;
        out[di + 2] = b2 | 0;
        out[di + 3] = a2 | 0;
      } else {
        out[di + 0] = 0;
        out[di + 1] = 0;
        out[di + 2] = 0;
        out[di + 3] = 0;
      }
    }
  }

  octx.putImageData(outImgData, 0, 0);

  const tileCanvas = createCanvas(TILE_WIDTH, TILE_HEIGHT);
  const tileCtx = tileCanvas.getContext("2d");
  let tileIndex = 0;

  for (let ty = 0; ty + TILE_HEIGHT <= outH; ty += TILE_HEIGHT) {
    for (let tx = 0; tx + TILE_WIDTH <= outW; tx += TILE_WIDTH) {
      const tileImage = octx.getImageData(tx, ty, TILE_WIDTH, TILE_HEIGHT);
      maskDiamond(tileImage);
      if (!hasOpaquePixels(tileImage)) continue;
      if (!isFullDiamond(tileImage)) continue;

      tileCtx.clearRect(0, 0, TILE_WIDTH, TILE_HEIGHT);
      tileCtx.putImageData(tileImage, 0, 0);
      const filename = path.join(outputDir, `${tileIndex}.png`);
      await fs.promises.writeFile(filename, tileCanvas.toBuffer("image/png"));
      tileIndex += 1;
    }
  }

  if (tileIndex === 0) {
    throw new Error("No tiles with visible pixels were detected.");
  }

  console.log(`Wrote ${tileIndex} tiles to ${outputDir}`);
}

function hasOpaquePixels(imageData) {
  const { data } = imageData;
  for (let i = 3; i < data.length; i += 4) {
    if (data[i] > 0) return true;
  }
  return false;
}

function maskDiamond(imageData) {
  const { data, width, height } = imageData;
  const halfW = width / 2;
  const halfH = height / 2;
  const centerX = (width - 1) / 2;
  const centerY = (height - 1) / 2;

  for (let y = 0; y < height; y += 1) {
    const yNorm = Math.abs(y - centerY) / halfH;
    for (let x = 0; x < width; x += 1) {
      const xNorm = Math.abs(x - centerX) / halfW;
      if (xNorm + yNorm > 1) {
        const idx = (y * width + x) * 4;
        data[idx + 0] = 0;
        data[idx + 1] = 0;
        data[idx + 2] = 0;
        data[idx + 3] = 0;
      }
    }
  }
}

function isFullDiamond(imageData) {
  const { data, width, height } = imageData;
  const halfW = width / 2;
  const halfH = height / 2;
  const centerX = (width - 1) / 2;
  const centerY = (height - 1) / 2;

  for (let y = 0; y < height; y += 1) {
    const yNorm = Math.abs(y - centerY) / halfH;
    for (let x = 0; x < width; x += 1) {
      const xNorm = Math.abs(x - centerX) / halfW;
      if (xNorm + yNorm <= 1) {
        const idx = (y * width + x) * 4;
        if (data[idx + 3] === 0) {
          return false;
        }
      }
    }
  }
  return true;
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
