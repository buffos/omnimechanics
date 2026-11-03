export class DraftingViewport {
  panX: number;
  panY: number;
  scale: number; // pixels per world meter
  width: number;
  height: number;
  gridStep: number; // in meters (e.g. 0.1, 0.5, 1.0)

  constructor(
    width = 1200,
    height = 800,
    scale = 60,
    panX = 300,
    panY = 500,
    gridStep = 1.0
  ) {
    this.width = width;
    this.height = height;
    this.scale = scale;
    this.panX = panX;
    this.panY = panY;
    this.gridStep = gridStep;
  }

  /**
   * Converts screen pixel coordinates to world coordinates (Y-up convention)
   */
  screenToWorld(screenX: number, screenY: number): { x: number; y: number } {
    const x = (screenX - this.panX) / this.scale;
    const y = -(screenY - this.panY) / this.scale;
    return { x, y };
  }

  /**
   * Converts world coordinates (Y-up) to screen pixel coordinates
   */
  worldToScreen(worldX: number, worldY: number): { x: number; y: number } {
    const screenX = this.panX + worldX * this.scale;
    const screenY = this.panY - worldY * this.scale;
    return { x: screenX, y: screenY };
  }

  clone(): DraftingViewport {
    return new DraftingViewport(
      this.width,
      this.height,
      this.scale,
      this.panX,
      this.panY,
      this.gridStep
    );
  }

  /**
   * Pans viewport by pixel delta
   */
  pan(dx: number, dy: number): void {
    this.panX += dx;
    this.panY += dy;
  }

  /**
   * Zooms in or out keeping the cursor point anchored
   */
  zoomAt(screenX: number, screenY: number, factor: number): void {
    const clampedFactor = Math.max(0.2, Math.min(5.0, factor));
    const newScale = Math.max(10, Math.min(2000, this.scale * clampedFactor));

    // World point before zoom
    const worldBefore = this.screenToWorld(screenX, screenY);

    this.scale = newScale;

    // Adjust pan so world point stays at screenX, screenY
    this.panX = screenX - worldBefore.x * this.scale;
    this.panY = screenY + worldBefore.y * this.scale;
  }

  /**
   * Fits model bounds into the current viewport with margin
   */
  zoomExtents(nodes: Array<{ x: number; y: number }>, marginPx = 60): void {
    if (nodes.length === 0) {
      this.panX = this.width / 2;
      this.panY = this.height / 2;
      this.scale = 60;
      return;
    }

    let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
    for (const n of nodes) {
      if (n.x < minX) minX = n.x;
      if (n.x > maxX) maxX = n.x;
      if (n.y < minY) minY = n.y;
      if (n.y > maxY) maxY = n.y;
    }

    const spanX = Math.max(1.0, maxX - minX);
    const spanY = Math.max(1.0, maxY - minY);

    const availW = Math.max(100, this.width - marginPx * 2);
    const availH = Math.max(100, this.height - marginPx * 2);

    const scaleX = availW / spanX;
    const scaleY = availH / spanY;
    this.scale = Math.max(10, Math.min(500, Math.min(scaleX, scaleY)));

    const midWorldX = (minX + maxX) / 2;
    const midWorldY = (minY + maxY) / 2;

    this.panX = this.width / 2 - midWorldX * this.scale;
    this.panY = this.height / 2 + midWorldY * this.scale;
  }

  /**
   * Computes visible world bounds
   */
  getVisibleWorldBounds(): { minX: number; maxX: number; minY: number; maxY: number } {
    const topLeft = this.screenToWorld(0, 0);
    const bottomRight = this.screenToWorld(this.width, this.height);

    return {
      minX: Math.min(topLeft.x, bottomRight.x),
      maxX: Math.max(topLeft.x, bottomRight.x),
      minY: Math.min(topLeft.y, bottomRight.y),
      maxY: Math.max(topLeft.y, bottomRight.y),
    };
  }
}
