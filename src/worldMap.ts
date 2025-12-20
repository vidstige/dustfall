export type WorldMapData = number[][];

export class WorldMap {
  public readonly width: number;
  public readonly height: number;
  public readonly data: WorldMapData;

  constructor(data: WorldMapData) {
    this.data = data;
    this.height = data.length;
    this.width = data[0]?.length ?? 0;
  }

  getTile(x: number, y: number): number | undefined {
    return this.data[y]?.[x];
  }
}
