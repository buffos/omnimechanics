/**
 * Canonical Shared 2D SVG Viewport & Tension-Side Diagram Kernel (`cap-interactive-canvas`):
 * - `ViewportTransform2D` value object (`worldToScreen`, `screenToWorld`, `pan`, `zoomAt`, `fitBounds`)
 *   guaranteeing +y up in metres maps to -v up on screen and exact round-trip identity (`SC-012`)
 * - `buildTensionSideDiagramPath` enforcing `INV-6` / `BR-4` (positive sagging moment M > 0 always
 *   plotted toward the local -y bottom tension fibre across Mode 2 and Mode 3) (`SC-012`)
 * - `SemanticColorTokens` shared visual palette across viewports
 */

export interface Point2D {
  readonly x: number;
  readonly y: number;
}

export interface ViewportTransform2DState {
  readonly viewWidth: number;
  readonly viewHeight: number;
  readonly margin: number;
  readonly panX: number; // screen pixel offset u_0
  readonly panY: number; // screen pixel offset v_0
  readonly scale: number; // pixels per metre s
  readonly refWorldX: number;
  readonly refWorldY: number;
}

export interface DiagramSample2D {
  /** Station coordinate along member [0, length] in metres, or normalized [0, 1] if length is omitted */
  readonly x: number;
  /** Primary ordinate value (M in kNm, Q in kN, or N in kN) */
  readonly value: number;
  /** Optional minimum ordinate when rendering Envelope bands */
  readonly minValue?: number;
}

export interface DiagramMember2D {
  readonly start: Point2D;
  readonly end: Point2D;
  readonly length?: number;
}

export interface DiagramRenderOptions2D {
  readonly kind: 'M' | 'Q' | 'N' | 'EnvelopeM' | 'EnvelopeQ' | 'EnvelopeN';
  readonly maxAbsValue: number;
  readonly maxPixelOffset?: number; // default: 70 px for beams/frames, 12 px for trusses
  readonly tensionSidePositiveM?: true; // invariant: positive M plotted toward local -y fibre
}

/**
 * Shared visual palette across Mode 1, Mode 2, and Mode 3 (`SemanticColorTokens`).
 */
export const SemanticColorTokens = {
  fixedSupportBlue: '#3b82f6',
  dirichletBlue: '#3b82f6',
  momentPositiveBlue: '#3b82f6',
  appliedLoadRed: '#ef4444',
  neumannTractionRed: '#ef4444',
  momentNegativeRed: '#ef4444',
  trussCompressionRed: '#ef4444',
  pinnedSupportEmerald: '#10b981',
  bottomBoundaryEmerald: '#10b981',
  trussTensionEmerald: '#10b981',
  topBoundaryAmber: '#f59e0b',
  newmarkCurveAmber: '#f59e0b',
  shearDiagramAmber: '#f59e0b',
  cavityPurple: '#a855f7',
  staticEbReferencePurple: '#a855f7',
  envelopePurple: '#a855f7',
  talbotCurveCyan: '#38bdf8',
} as const;

/**
 * Immutable 2D affine coordinate transform between engineering world coordinates (x, y) in metres
 * (+x right, +y up) and SVG screen coordinates (u, v) in pixels (+u right, +v down):
 *   u = panX + scale * (x - refWorldX)
 *   v = panY - scale * (y - refWorldY)
 */
export class ViewportTransform2D implements ViewportTransform2DState {
  readonly viewWidth: number;
  readonly viewHeight: number;
  readonly margin: number;
  readonly panX: number;
  readonly panY: number;
  readonly scale: number;
  readonly refWorldX: number;
  readonly refWorldY: number;

  constructor(state?: Partial<ViewportTransform2DState>) {
    this.viewWidth = state?.viewWidth ?? 1200;
    this.viewHeight = state?.viewHeight ?? 800;
    this.margin = state?.margin ?? 60;
    this.scale = Math.max(1e-6, state?.scale ?? 60);
    this.panX = state?.panX ?? 300;
    this.panY = state?.panY ?? 500;
    this.refWorldX = state?.refWorldX ?? 0;
    this.refWorldY = state?.refWorldY ?? 0;
  }

  /**
   * Projects world coordinates (x, y) in metres (+y up) to screen pixels (x: u, y: v) (+v down).
   */
  worldToScreen(x: number, y: number): Point2D {
    return {
      x: this.panX + this.scale * (x - this.refWorldX),
      y: this.panY - this.scale * (y - this.refWorldY),
    };
  }

  /**
   * Projects screen pixel coordinates (u, v) (+v down) back to engineering world coordinates (x, y) (+y up).
   */
  screenToWorld(u: number, v: number): Point2D {
    return {
      x: this.refWorldX + (u - this.panX) / this.scale,
      y: this.refWorldY - (v - this.panY) / this.scale,
    };
  }

  /**
   * Returns a new `ViewportTransform2D` translated by screen pixel delta (dx, dy).
   */
  pan(dx: number, dy: number): ViewportTransform2D {
    return new ViewportTransform2D({
      ...this,
      panX: this.panX + dx,
      panY: this.panY + dy,
    });
  }

  /**
   * Returns a new `ViewportTransform2D` zoomed around screen cursor (screenX, screenY).
   */
  zoomAt(screenX: number, screenY: number, factor: number): ViewportTransform2D {
    const clampedFactor = Math.max(0.2, Math.min(5.0, factor));
    const newScale = Math.max(10, Math.min(2000, this.scale * clampedFactor));
    const worldBefore = this.screenToWorld(screenX, screenY);

    const nextPanX = screenX - (worldBefore.x - this.refWorldX) * newScale;
    const nextPanY = screenY + (worldBefore.y - this.refWorldY) * newScale;

    return new ViewportTransform2D({
      ...this,
      scale: newScale,
      panX: nextPanX,
      panY: nextPanY,
    });
  }

  /**
   * Returns an aspect-locked `ViewportTransform2D` fitting the given world points inside the view margins.
   */
  fitBounds(points: ReadonlyArray<Point2D>, marginPx = this.margin): ViewportTransform2D {
    if (!points || points.length === 0) {
      return new ViewportTransform2D({
        ...this,
        margin: marginPx,
        panX: this.viewWidth / 2,
        panY: this.viewHeight / 2,
        scale: 60,
        refWorldX: 0,
        refWorldY: 0,
      });
    }

    let minX = Infinity, maxX = -Infinity;
    let minY = Infinity, maxY = -Infinity;
    for (const p of points) {
      if (p.x < minX) minX = p.x;
      if (p.x > maxX) maxX = p.x;
      if (p.y < minY) minY = p.y;
      if (p.y > maxY) maxY = p.y;
    }

    const spanX = Math.max(1.0, maxX - minX);
    const spanY = Math.max(1.0, maxY - minY);
    const availW = Math.max(100, this.viewWidth - marginPx * 2);
    const availH = Math.max(100, this.viewHeight - marginPx * 2);

    const nextScale = Math.max(10, Math.min(500, Math.min(availW / spanX, availH / spanY)));
    const midX = (minX + maxX) / 2;
    const midY = (minY + maxY) / 2;

    return new ViewportTransform2D({
      ...this,
      margin: marginPx,
      scale: nextScale,
      refWorldX: 0,
      refWorldY: 0,
      panX: this.viewWidth / 2 - midX * nextScale,
      panY: this.viewHeight / 2 + midY * nextScale,
    });
  }
}

/**
 * Computes the screen-space tension-side unit normal (nx, ny) for a member from screen p0 to p1.
 * For a horizontal left-to-right member (p1.x > p0.x, p1.y == p0.y):
 *   ux = 1, uy = 0  =>  (nx, ny) = (-uy, ux) = (0, +1), which points downward on screen (+v)
 *   corresponding to the local -y bottom fibre in tension (`INV-6`, `BR-4`, `SC-012`).
 */
export function computeTensionSideScreenNormal(
  p0Screen: Point2D,
  p1Screen: Point2D
): { nx: number; ny: number; lengthPx: number } {
  const dx = p1Screen.x - p0Screen.x;
  const dy = p1Screen.y - p0Screen.y;
  const lengthPx = Math.hypot(dx, dy);
  if (lengthPx < 1e-6) {
    return { nx: 0, ny: 1, lengthPx: 0 };
  }
  const ux = dx / lengthPx;
  const uy = dy / lengthPx;
  return {
    nx: -uy,
    ny: ux,
    lengthPx,
  };
}

/**
 * Builds a closed SVG polygon path string `d` for a 2D member's internal force diagram (`M`, `Q`, `N`, or `Envelope`).
 * Positive bending moment M > 0 is always offset in the direction of the local -y bottom tension fibre (`BR-4`, `INV-6`, `SC-012`).
 */
export function buildTensionSideDiagramPath(
  member: DiagramMember2D,
  samples: ReadonlyArray<DiagramSample2D>,
  opts: DiagramRenderOptions2D,
  transform?: Pick<ViewportTransform2D, 'worldToScreen'>
): {
  path: string;
  screenPoints: Point2D[];
  basePoints: Point2D[];
} {
  if (!samples || samples.length < 2) {
    return { path: '', screenPoints: [], basePoints: [] };
  }

  const p0 = transform
    ? transform.worldToScreen(member.start.x, member.start.y)
    : member.start;
  const p1 = transform
    ? transform.worldToScreen(member.end.x, member.end.y)
    : member.end;

  const { nx, ny, lengthPx } = computeTensionSideScreenNormal(p0, p1);
  if (lengthPx < 1e-4) {
    return { path: '', screenPoints: [], basePoints: [] };
  }

  const worldLen =
    member.length && member.length > 1e-9
      ? member.length
      : Math.max(
          Math.hypot(member.end.x - member.start.x, member.end.y - member.start.y),
          samples[samples.length - 1]?.x || 1.0
        );

  const safeMaxAbs = opts.maxAbsValue > 1e-9 ? opts.maxAbsValue : 1.0;
  const maxPx = opts.maxPixelOffset ?? 70;
  const isEnvelope =
    opts.kind === 'EnvelopeM' || opts.kind === 'EnvelopeQ' || opts.kind === 'EnvelopeN';

  const basePoints: Point2D[] = [];
  const screenPoints: Point2D[] = [];
  const lowerEnvelopePoints: Point2D[] = [];

  for (const s of samples) {
    const ratio = Math.max(0, Math.min(1, s.x / worldLen));
    const bx = p0.x + ratio * (p1.x - p0.x);
    const by = p0.y + ratio * (p1.y - p0.y);
    basePoints.push({ x: bx, y: by });

    const offsetPx = (s.value / safeMaxAbs) * maxPx;
    screenPoints.push({
      x: bx + nx * offsetPx,
      y: by + ny * offsetPx,
    });

    if (isEnvelope) {
      const minVal = s.minValue ?? 0;
      const minOffsetPx = (minVal / safeMaxAbs) * maxPx;
      lowerEnvelopePoints.push({
        x: bx + nx * minOffsetPx,
        y: by + ny * minOffsetPx,
      });
    }
  }

  if (isEnvelope && lowerEnvelopePoints.length === screenPoints.length) {
    let d = `M ${screenPoints[0].x.toFixed(2)} ${screenPoints[0].y.toFixed(2)} `;
    for (let i = 1; i < screenPoints.length; i++) {
      d += `L ${screenPoints[i].x.toFixed(2)} ${screenPoints[i].y.toFixed(2)} `;
    }
    for (let i = lowerEnvelopePoints.length - 1; i >= 0; i--) {
      d += `L ${lowerEnvelopePoints[i].x.toFixed(2)} ${lowerEnvelopePoints[i].y.toFixed(2)} `;
    }
    d += 'Z';
    return { path: d, screenPoints, basePoints };
  }

  let d = `M ${basePoints[0].x.toFixed(2)} ${basePoints[0].y.toFixed(2)} `;
  for (const pt of screenPoints) {
    d += `L ${pt.x.toFixed(2)} ${pt.y.toFixed(2)} `;
  }
  d += `L ${basePoints[basePoints.length - 1].x.toFixed(2)} ${basePoints[basePoints.length - 1].y.toFixed(2)} Z`;

  return { path: d, screenPoints, basePoints };
}
