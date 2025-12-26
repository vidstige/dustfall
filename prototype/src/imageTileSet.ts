import type { TileBitmap } from "./isometricRenderer";

const TILE_IMAGE_URLS = [
  new URL("./assets/tile40.png", import.meta.url).toString(),
  new URL("./assets/tile53.png", import.meta.url).toString(),
  new URL("./assets/tile100.png", import.meta.url).toString(),
  new URL("./assets/tile108.png", import.meta.url).toString(),
  new URL("./assets/tile110.png", import.meta.url).toString(),
  new URL("./assets/tile147.png", import.meta.url).toString(),
  new URL("./assets/tile151.png", import.meta.url).toString(),
  new URL("./assets/tile163.png", import.meta.url).toString(),
  new URL("./assets/tile165.png", import.meta.url).toString(),
  new URL("./assets/tile178.png", import.meta.url).toString(),
  new URL("./assets/tile192.png", import.meta.url).toString(),
  new URL("./assets/tile205.png", import.meta.url).toString(),
  new URL("./assets/tile215.png", import.meta.url).toString(),
  new URL("./assets/tile229.png", import.meta.url).toString(),
  new URL("./assets/tile255.png", import.meta.url).toString(),
  new URL("./assets/tile298.png", import.meta.url).toString(),
  new URL("./assets/tile312.png", import.meta.url).toString(),
  new URL("./assets/tile417.png", import.meta.url).toString(),
  new URL("./assets/tile445.png", import.meta.url).toString(),
  new URL("./assets/tile447.png", import.meta.url).toString(),
];

export async function loadImageTileSet(
  tileWidth: number,
  tileHeight: number,
): Promise<TileBitmap[]> {
  const images = await Promise.all(TILE_IMAGE_URLS.map(loadImage));
  return images.map((img) => imageToCanvas(img, tileWidth, tileHeight));
}

function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = () => reject(new Error(`Failed to load tile image: ${String(src)}`));
    img.src = src;
  });
}

function imageToCanvas(image: HTMLImageElement, width: number, height: number): TileBitmap {
  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  const ctx = canvas.getContext("2d");
  if (!ctx) {
    throw new Error("Unable to create tile canvas context.");
  }
  ctx.clearRect(0, 0, width, height);
  ctx.drawImage(image, 0, 0, width, height);
  return canvas;
}
