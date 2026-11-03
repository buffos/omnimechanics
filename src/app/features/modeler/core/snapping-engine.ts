import { DraftingViewport } from './viewport';
import { Node2D, Element2D, SnapCandidateDto, SnapType, isLineElement } from './modeler.types';

export interface SnappingOptions {
  nodeSnapEnabled: boolean;
  intersectionSnapEnabled: boolean;
  midpointSnapEnabled: boolean;
  orthoLockEnabled: boolean;
  angularGuidesEnabled: boolean;
  gridSnapEnabled: boolean;
  nodeSnapTolerancePx: number; // default 15 px
  geomSnapTolerancePx: number; // default 12 px
}

export const DEFAULT_SNAPPING_OPTIONS: SnappingOptions = {
  nodeSnapEnabled: true,
  intersectionSnapEnabled: true,
  midpointSnapEnabled: true,
  orthoLockEnabled: false,
  angularGuidesEnabled: true,
  gridSnapEnabled: true,
  nodeSnapTolerancePx: 15,
  geomSnapTolerancePx: 12,
};

export class SnappingEngine {
  /**
   * Evaluates cursor position against all enabled snapping layers with strict priority
   */
  static evaluateSnap(
    screenCursor: { x: number; y: number },
    viewport: DraftingViewport,
    nodes: Node2D[],
    elements: Element2D[],
    basePoint?: { x: number; y: number } | null,
    options: SnappingOptions = DEFAULT_SNAPPING_OPTIONS
  ): SnapCandidateDto {
    const rawWorld = viewport.screenToWorld(screenCursor.x, screenCursor.y);

    const nodeMap = new Map<string, Node2D>();
    for (const n of nodes) {
      nodeMap.set(n.id, n);
    }

    // 1. Priority 1: NodeSnap (< 15 px)
    if (options.nodeSnapEnabled) {
      let bestNode: Node2D | null = null;
      let minNodeDistPx = Infinity;

      for (const n of nodes) {
        const screenPos = viewport.worldToScreen(n.x, n.y);
        const distPx = Math.hypot(screenPos.x - screenCursor.x, screenPos.y - screenCursor.y);
        if (distPx <= options.nodeSnapTolerancePx && distPx < minNodeDistPx) {
          minNodeDistPx = distPx;
          bestNode = n;
        }
      }

      if (bestNode) {
        const screenP = viewport.worldToScreen(bestNode.x, bestNode.y);
        return {
          type: 'Node',
          point: { x: bestNode.x, y: bestNode.y },
          screenPoint: screenP,
          distancePx: minNodeDistPx,
          referenceId: bestNode.id,
        };
      }
    }

    // 2. Priority 2: IntersectionSnap (< 12 px) between existing members
    if (options.intersectionSnapEnabled && elements.length > 1) {
      let bestIntPt: { x: number; y: number } | null = null;
      let minIntDistPx = Infinity;

      for (let i = 0; i < elements.length; i++) {
        const e1 = elements[i];
        if (!isLineElement(e1)) continue;
        const n1a = nodeMap.get(e1.node_i);
        const n1b = nodeMap.get(e1.node_j);
        if (!n1a || !n1b) continue;

        for (let j = i + 1; j < elements.length; j++) {
          const e2 = elements[j];
          if (!isLineElement(e2)) continue;
          const n2a = nodeMap.get(e2.node_i);
          const n2b = nodeMap.get(e2.node_j);
          if (!n2a || !n2b) continue;

          const intPt = computeSegmentIntersection(n1a, n1b, n2a, n2b);
          if (intPt) {
            const screenP = viewport.worldToScreen(intPt.x, intPt.y);
            const distPx = Math.hypot(screenP.x - screenCursor.x, screenP.y - screenCursor.y);
            if (distPx <= options.geomSnapTolerancePx && distPx < minIntDistPx) {
              minIntDistPx = distPx;
              bestIntPt = intPt;
            }
          }
        }
      }

      if (bestIntPt) {
        return {
          type: 'Intersection',
          point: bestIntPt,
          screenPoint: viewport.worldToScreen(bestIntPt.x, bestIntPt.y),
          distancePx: minIntDistPx,
        };
      }
    }

    // 3. Priority 3: MidpointSnap (< 12 px)
    if (options.midpointSnapEnabled) {
      let bestMidPt: { x: number; y: number } | null = null;
      let minMidDistPx = Infinity;
      let refElemId: string | undefined;

      for (const e of elements) {
        if (!isLineElement(e)) continue;
        const na = nodeMap.get(e.node_i);
        const nb = nodeMap.get(e.node_j);
        if (!na || !nb) continue;

        const mid = { x: (na.x + nb.x) / 2, y: (na.y + nb.y) / 2 };
        const screenP = viewport.worldToScreen(mid.x, mid.y);
        const distPx = Math.hypot(screenP.x - screenCursor.x, screenP.y - screenCursor.y);
        if (distPx <= options.geomSnapTolerancePx && distPx < minMidDistPx) {
          minMidDistPx = distPx;
          bestMidPt = mid;
          refElemId = e.id;
        }
      }

      if (bestMidPt) {
        return {
          type: 'Midpoint',
          point: bestMidPt,
          screenPoint: viewport.worldToScreen(bestMidPt.x, bestMidPt.y),
          distancePx: minMidDistPx,
          referenceId: refElemId,
        };
      }
    }

    // 4. Priority 4: Ortho Lock & Angular Guide Snap (when basePoint is present)
    if (basePoint) {
      const dx = rawWorld.x - basePoint.x;
      const dy = rawWorld.y - basePoint.y;
      const dist = Math.hypot(dx, dy);

      if (dist > 1e-4) {
        let angleDeg = (Math.atan2(dy, dx) * 180) / Math.PI;
        if (angleDeg < 0) angleDeg += 360;

        // Check Ortho Lock (0, 90, 180, 270)
        const orthoAngles = [0, 90, 180, 270, 360];
        let nearestOrtho = 0;
        let minOrthoDiff = Infinity;
        for (const oa of orthoAngles) {
          const diff = Math.abs(angleDeg - oa);
          if (diff < minOrthoDiff) {
            minOrthoDiff = diff;
            nearestOrtho = oa % 360;
          }
        }

        if (options.orthoLockEnabled || minOrthoDiff <= 5.0) {
          const rad = (nearestOrtho * Math.PI) / 180;
          const lockedPoint = {
            x: basePoint.x + dist * Math.cos(rad),
            y: basePoint.y + dist * Math.sin(rad),
          };
          const screenP = viewport.worldToScreen(lockedPoint.x, lockedPoint.y);
          return {
            type: 'Ortho',
            point: lockedPoint,
            screenPoint: screenP,
            distancePx: Math.hypot(screenP.x - screenCursor.x, screenP.y - screenCursor.y),
            guideAngleDeg: nearestOrtho,
          };
        }

        // Check Angular Guides (30, 45, 60, 120, 135, 150, 210, 225, 240, 300, 315, 330)
        if (options.angularGuidesEnabled) {
          const guideAngles = [30, 45, 60, 120, 135, 150, 210, 225, 240, 300, 315, 330];
          for (const ga of guideAngles) {
            if (Math.abs(angleDeg - ga) <= 3.5) {
              const rad = (ga * Math.PI) / 180;
              const guidePoint = {
                x: basePoint.x + dist * Math.cos(rad),
                y: basePoint.y + dist * Math.sin(rad),
              };
              const screenP = viewport.worldToScreen(guidePoint.x, guidePoint.y);
              return {
                type: 'AngularGuide',
                point: guidePoint,
                screenPoint: screenP,
                distancePx: Math.hypot(screenP.x - screenCursor.x, screenP.y - screenCursor.y),
                guideAngleDeg: ga,
              };
            }
          }
        }
      }
    }

    // 5. Priority 5: Grid Snap
    if (options.gridSnapEnabled && viewport.gridStep > 0) {
      const step = viewport.gridStep;
      const snappedWorld = {
        x: Math.round(rawWorld.x / step) * step,
        y: Math.round(rawWorld.y / step) * step,
      };
      // Normalize float precision
      snappedWorld.x = Math.round(snappedWorld.x * 1000) / 1000;
      snappedWorld.y = Math.round(snappedWorld.y * 1000) / 1000;

      const screenP = viewport.worldToScreen(snappedWorld.x, snappedWorld.y);
      return {
        type: 'Grid',
        point: snappedWorld,
        screenPoint: screenP,
        distancePx: Math.hypot(screenP.x - screenCursor.x, screenP.y - screenCursor.y),
      };
    }

    // Fallback: raw cursor position
    return {
      type: 'Grid',
      point: rawWorld,
      screenPoint: screenCursor,
      distancePx: 0,
    };
  }

  /**
   * Parses keyboard numeric entries:
   * 1. Absolute Cartesian: "5.0, 3.2" or "5 3.2"
   * 2. Relative Cartesian: "@2.5, -1.0" or "@2.5 -1.0"
   * 3. Polar coordinate: "@4.0<45" or "4.0<45"
   */
  static parseCoordinateInput(
    input: string,
    basePoint?: { x: number; y: number } | null
  ): { x: number; y: number } | null {
    const raw = input.trim();
    if (!raw) return null;

    const isRelative = raw.startsWith('@');
    const clean = isRelative ? raw.slice(1).trim() : raw;
    const origin = isRelative ? (basePoint ?? { x: 0, y: 0 }) : { x: 0, y: 0 };

    // Polar format: L<theta
    if (clean.includes('<')) {
      const parts = clean.split('<');
      if (parts.length === 2) {
        const length = parseFloat(parts[0].trim());
        const angleDeg = parseFloat(parts[1].replace(/deg/i, '').trim());
        if (!isNaN(length) && !isNaN(angleDeg)) {
          const rad = (angleDeg * Math.PI) / 180;
          return {
            x: origin.x + length * Math.cos(rad),
            y: origin.y + length * Math.sin(rad),
          };
        }
      }
    }

    // Cartesian format: x, y or x y
    const delim = clean.includes(',') ? ',' : ' ';
    const parts = clean.split(delim).map(s => s.trim()).filter(Boolean);
    if (parts.length === 2) {
      const valX = parseFloat(parts[0]);
      const valY = parseFloat(parts[1]);
      if (!isNaN(valX) && !isNaN(valY)) {
        if (isRelative) {
          return { x: origin.x + valX, y: origin.y + valY };
        }
        return { x: valX, y: valY };
      }
    }

    return null;
  }

  /**
   * Rejects zero-length elements
   */
  static validateElementLength(
    p1: { x: number; y: number },
    p2: { x: number; y: number },
    minLen = 1e-4
  ): void {
    const len = Math.hypot(p2.x - p1.x, p2.y - p1.y);
    if (len < minLen) {
      throw new Error('ZERO_LENGTH_ELEMENT');
    }
  }
}

/**
 * Computes intersection between segments [p1, p2] and [p3, p4]
 */
export function computeSegmentIntersection(
  p1: { x: number; y: number },
  p2: { x: number; y: number },
  p3: { x: number; y: number },
  p4: { x: number; y: number },
  tolerance = 1e-6
): { x: number; y: number } | null {
  const d1x = p2.x - p1.x;
  const d1y = p2.y - p1.y;
  const d2x = p4.x - p3.x;
  const d2y = p4.y - p3.y;

  const denom = d1x * d2y - d1y * d2x;
  if (Math.abs(denom) < tolerance) return null; // parallel or collinear

  const s = ((p3.x - p1.x) * d2y - (p3.y - p1.y) * d2x) / denom;
  const t = ((p3.x - p1.x) * d1y - (p3.y - p1.y) * d1x) / denom;

  // True segment intersection requires s, t in [0, 1]
  if (s >= -tolerance && s <= 1 + tolerance && t >= -tolerance && t <= 1 + tolerance) {
    return {
      x: p1.x + s * d1x,
      y: p1.y + s * d1y,
    };
  }

  return null;
}
