import {
  Component,
  ChangeDetectionStrategy,
  inject,
  signal,
  computed,
  ElementRef,
  viewChild,
  HostListener,
  AfterViewInit,
  DestroyRef,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { ModelerStore } from '../core/modeler.store';
import { ModelerResultsService } from '../results/modeler-results.service';
import { SnappingEngine } from '../core/snapping-engine';
import {
  SnapCandidateDto,
  isLineElement,
  FrameMemberElement,
  SupportKind,
} from '../core/modeler.types';
import { DraftingViewport } from '../core/viewport';

function isNotNull<T>(val: T | null | undefined): val is T {
  return val !== null && val !== undefined;
}

@Component({
  selector: 'app-modeler-canvas',
  standalone: true,
  imports: [CommonModule],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './modeler-canvas.component.html',
  styleUrl: './modeler-canvas.component.css',
})
export class ModelerCanvasComponent implements AfterViewInit {
  readonly store = inject(ModelerStore);
  readonly resultsService = inject(ModelerResultsService);
  private readonly destroyRef = inject(DestroyRef);
  readonly svgRef = viewChild<ElementRef<SVGSVGElement>>('svgCanvas');

  ngAfterViewInit(): void {
    const svgEl = this.svgRef()?.nativeElement;
    if (!svgEl) return;

    const updateDimensions = () => {
      const rect = svgEl.getBoundingClientRect();
      const w = Math.round(rect.width);
      const h = Math.round(rect.height);
      if (w > 0 && h > 0) {
        const vp = this.viewport();
        if (vp.width !== w || vp.height !== h) {
          this.store.viewport.set(new DraftingViewport(w, h, vp.scale, vp.panX, vp.panY, vp.gridStep));
        }
      }
    };

    updateDimensions();

    if (typeof ResizeObserver !== 'undefined') {
      const ro = new ResizeObserver(() => updateDimensions());
      ro.observe(svgEl);
      this.destroyRef.onDestroy(() => ro.disconnect());
    }
  }

  // Mouse drag & gesture state
  isPanning = signal<boolean>(false);
  isPanMode = signal<boolean>(false);
  isSpacePressed = signal<boolean>(false);
  readonly isEffectivePan = computed(() => this.isPanMode() || this.isSpacePressed());
  lastMouseScreen = signal<{ x: number; y: number }>({ x: 0, y: 0 });

  // Interactive Node Dragging state
  isNodeDragging = signal<boolean>(false);
  draggedNodeId = signal<string | null>(null);
  previewDragNode = signal<{ id: string; x: number; y: number } | null>(null);

  // Drawing state
  readonly drawStartPoint = this.store.drawStartPoint;
  currentSnap = signal<SnapCandidateDto | null>(null);

  // Marquee selection state
  isMarqueeActive = signal<boolean>(false);
  marqueeStartWorld = signal<{ x: number; y: number } | null>(null);
  marqueeCurrentWorld = signal<{ x: number; y: number } | null>(null);

  // Computed properties
  readonly viewport = this.store.viewport;
  readonly model = this.store.model;
  readonly selection = this.store.selection;

  readonly gridLines = computed(() => {
    const vp = this.viewport();
    const w = vp.width || 1920;
    const h = vp.height || 1080;
    const step = vp.gridStep;
    if (step <= 0) return [];

    const bounds = vp.getVisibleWorldBounds();

    // Extend 2 steps buffer on all sides so lines seamlessly enter/leave during pan
    const startX = Math.floor(bounds.minX / step) * step - step * 2;
    const endX = Math.ceil(bounds.maxX / step) * step + step * 2;
    const startY = Math.floor(bounds.minY / step) * step - step * 2;
    const endY = Math.ceil(bounds.maxY / step) * step + step * 2;

    const numX = Math.round((endX - startX) / step);
    const numY = Math.round((endY - startY) / step);
    if (numX > 400 || numY > 400) {
      return [];
    }

    const lines: Array<{ x1: number; y1: number; x2: number; y2: number; isMajor: boolean }> = [];

    // Vertical lines spanning from top to bottom (y: 0 to h) across entire screen
    for (let x = startX; x <= endX; x += step) {
      const isMajor = Math.abs(Math.round(x) - x) < 1e-4;
      const screenX = vp.worldToScreen(x, 0).x;
      lines.push({ x1: screenX, y1: 0, x2: screenX, y2: h, isMajor });
    }

    // Horizontal lines spanning from left to right (x: 0 to w) across entire screen
    for (let y = startY; y <= endY; y += step) {
      const isMajor = Math.abs(Math.round(y) - y) < 1e-4;
      const screenY = vp.worldToScreen(0, y).y;
      lines.push({ x1: 0, y1: screenY, x2: w, y2: screenY, isMajor });
    }

    return lines;
  });

  readonly originScreen = computed(() => {
    return this.viewport().worldToScreen(0, 0);
  });

  readonly drawnNodes = computed(() => {
    const vp = this.viewport();
    const m = this.model();
    const sel = this.selection();
    const isStable = this.resultsService.isStable();
    const mechDisps = this.resultsService.mechanismDisplacements();
    const drag = this.previewDragNode();

    return m.nodes.map(n => {
      let wx = n.x;
      let wy = n.y;
      if (drag && drag.id === n.id) {
        wx = drag.x;
        wy = drag.y;
      } else if (!isStable && mechDisps.has(n.id)) {
        const md = mechDisps.get(n.id)!;
        wx += md.dx;
        wy += md.dy;
      }
      const p = vp.worldToScreen(wx, wy);
      return {
        id: n.id,
        x: p.x,
        y: p.y,
        worldX: wx,
        worldY: wy,
        isSelected: sel.nodeIds.has(n.id),
      };
    });
  });

  readonly drawnElements = computed(() => {
    const vp = this.viewport();
    const m = this.model();
    const sel = this.selection();
    const isStable = this.resultsService.isStable();
    const mechDisps = this.resultsService.mechanismDisplacements();
    const drag = this.previewDragNode();

    const nodeMap = new Map<string, { x: number; y: number }>();
    for (const n of m.nodes) {
      let wx = n.x;
      let wy = n.y;
      if (drag && drag.id === n.id) {
        wx = drag.x;
        wy = drag.y;
      } else if (!isStable && mechDisps.has(n.id)) {
        const md = mechDisps.get(n.id)!;
        wx += md.dx;
        wy += md.dy;
      }
      nodeMap.set(n.id, { x: wx, y: wy });
    }

    return m.elements
      .map(e => {
        if (!isLineElement(e)) {
          return null;
        }
        const ni = nodeMap.get(e.node_i);
        const nj = nodeMap.get(e.node_j);
        if (!ni || !nj) return null;

        const p1 = vp.worldToScreen(ni.x, ni.y);
        const p2 = vp.worldToScreen(nj.x, nj.y);
        const isSelected = sel.elementIds.has(e.id);

        // Check if catenary curved points exist from analysis results
        let curvePath: string | undefined;
        let isSlack = false;
        if (e.kind === 'Cable') {
          const rs = this.resultsService.currentResultSet();
          const cr = rs?.cable_results?.find(c => c.element_id === e.id);
          if (cr && cr.curve_points_global && cr.curve_points_global.length > 1) {
            isSlack = cr.is_slack;
            let d = '';
            for (let k = 0; k < cr.curve_points_global.length; k++) {
              const cp = vp.worldToScreen(cr.curve_points_global[k].x, cr.curve_points_global[k].y);
              d += k === 0 ? `M ${cp.x} ${cp.y} ` : `L ${cp.x} ${cp.y} `;
            }
            curvePath = d;
          }
        }

        const worldLen = Math.hypot(nj.x - ni.x, nj.y - ni.y);
        const dx = p2.x - p1.x;
        const dy = p2.y - p1.y;
        const sLen = Math.hypot(dx, dy);
        let upX = 0;
        let upY = -1;
        if (sLen > 1e-4) {
          const ux = dx / sLen;
          const uy = dy / sLen;
          // Perpendicular normal: (uy, -ux)
          upX = uy;
          upY = -ux;
          // Ensure up vector strictly points towards the upper screen hemisphere
          if (upY > 0 || (Math.abs(upY) < 1e-5 && upX > 0)) {
            upX = -upX;
            upY = -upY;
          }
        }
        const downX = -upX;
        const downY = -upY;

        const midX = (p1.x + p2.x) / 2;
        const midY = (p1.y + p2.y) / 2;
        const offsetDist = 14;

        return {
          id: e.id,
          kind: e.kind,
          node_i: e.node_i,
          node_j: e.node_j,
          x1: p1.x,
          y1: p1.y,
          x2: p2.x,
          y2: p2.y,
          curvePath,
          isSlack,
          isSelected,
          midX,
          midY,
          idX: midX + upX * offsetDist,
          idY: midY + upY * offsetDist,
          lengthLabel: `${worldLen.toFixed(2)} m`,
          lengthX: midX + downX * offsetDist,
          lengthY: midY + downY * offsetDist,
        };
      })
      .filter(isNotNull);
  });

  readonly drawnSupports = computed(() => {
    const vp = this.viewport();
    const m = this.model();
    const drag = this.previewDragNode();
    const nodeMap = new Map(m.nodes.map(n => [n.id, drag && drag.id === n.id ? { ...n, x: drag.x, y: drag.y } : n]));

    return m.supports
      .map(s => {
        const node = nodeMap.get(s.node_id);
        if (!node) return null;
        const p = vp.worldToScreen(node.x, node.y);
        const type = s.kind.type;
        const angle = 'angle_deg' in s.kind ? s.kind.angle_deg : 0;
        return {
          id: s.id,
          nodeId: s.node_id,
          x: p.x,
          y: p.y,
          type,
          angle,
          unilateral: s.unilateral,
        };
      })
      .filter(isNotNull);
  });

  readonly drawnReleases = computed(() => {
    const vp = this.viewport();
    const m = this.model();
    const nodeMap = new Map(m.nodes.map(n => [n.id, n]));
    const releases: Array<{ id: string; x: number; y: number }> = [];

    for (const e of m.elements) {
      if (e.kind !== 'FrameMember') continue;
      const f = e as FrameMemberElement;
      const ni = nodeMap.get(f.node_i);
      const nj = nodeMap.get(f.node_j);
      if (!ni || !nj) continue;

      const p1 = vp.worldToScreen(ni.x, ni.y);
      const p2 = vp.worldToScreen(nj.x, nj.y);
      const len = Math.hypot(p2.x - p1.x, p2.y - p1.y);
      if (len < 10) continue;

      const ux = (p2.x - p1.x) / len;
      const uy = (p2.y - p1.y) / len;

      if (f.release_i.moment) {
        releases.push({
          id: `${f.id}_rel_i`,
          x: p1.x + ux * 10,
          y: p1.y + uy * 10,
        });
      }
      if (f.release_j.moment) {
        releases.push({
          id: `${f.id}_rel_j`,
          x: p2.x - ux * 10,
          y: p2.y - uy * 10,
        });
      }
    }
    return releases;
  });

  readonly drawnLoads = computed(() => {
    const vp = this.viewport();
    const m = this.model();
    const activeLc = m.load_cases.find(lc => lc.id === this.store.activeLoadCaseId());
    if (!activeLc) return { nodal: [], memberPoints: [], dists: [] };

    const nodeMap = new Map(m.nodes.map(n => [n.id, n]));
    const elemMap = new Map(m.elements.map(e => [e.id, e]));

    // Nodal loads
    const nodal = activeLc.nodal_loads
      .map(nl => {
        const node = nodeMap.get(nl.node_id);
        if (!node) return null;
        const p = vp.worldToScreen(node.x, node.y);
        return {
          id: nl.id,
          x: p.x,
          y: p.y,
          fx: nl.fx,
          fy: nl.fy,
          mz: nl.mz,
        };
      })
      .filter(isNotNull);

    // Member point loads
    const memberPoints = activeLc.point_loads
      .map(pl => {
        const elem = elemMap.get(pl.element_id);
        if (!elem || !isLineElement(elem)) return null;
        const ni = nodeMap.get(elem.node_i);
        const nj = nodeMap.get(elem.node_j);
        if (!ni || !nj) return null;

        const p1 = vp.worldToScreen(ni.x, ni.y);
        const p2 = vp.worldToScreen(nj.x, nj.y);
        const px = p1.x + pl.a_rel * (p2.x - p1.x);
        const py = p1.y + pl.a_rel * (p2.y - p1.y);
        return {
          id: pl.id,
          x: px,
          y: py,
          loadVal: pl.py !== 0 ? pl.py : pl.px,
        };
      })
      .filter(isNotNull);

    // Distributed loads
    const dists = activeLc.dist_loads
      .map(dl => {
        const elem = elemMap.get(dl.element_id);
        if (!elem || !isLineElement(elem)) return null;
        const ni = nodeMap.get(elem.node_i);
        const nj = nodeMap.get(elem.node_j);
        if (!ni || !nj) return null;

        const p1 = vp.worldToScreen(ni.x, ni.y);
        const p2 = vp.worldToScreen(nj.x, nj.y);

        const dx = p2.x - p1.x;
        const dy = p2.y - p1.y;
        const len = Math.hypot(dx, dy);
        if (len < 1e-4) return null;

        const ux = dx / len;
        const uy = dy / len;
        // Normal pointing towards "above / local +y": in screen coords, (uy, -ux) points up for horizontal beam
        let nx = uy;
        let ny = -ux;

        const ax = p1.x + dl.a_rel * dx;
        const ay = p1.y + dl.a_rel * dy;
        const bx = p1.x + dl.b_rel * dx;
        const by = p1.y + dl.b_rel * dy;

        const q1Abs = Math.abs(dl.qy1);
        const q2Abs = Math.abs(dl.qy2);
        const qMax = Math.max(q1Abs, q2Abs);

        if (qMax < 1e-6) return null;

        // If load is positive (upward in global), flip normal
        if (dl.qy1 > 0 || (dl.qy1 === 0 && dl.qy2 > 0)) {
          nx = -nx;
          ny = -ny;
        }

        const maxPx = Math.min(45, Math.max(22, qMax * 1.6));
        const h1 = (q1Abs / qMax) * maxPx;
        const h2 = (q2Abs / qMax) * maxPx;

        const top1X = ax + nx * h1;
        const top1Y = ay + ny * h1;
        const top2X = bx + nx * h2;
        const top2Y = by + ny * h2;

        // Generate polygon outline and interior tick/hatch lines
        let path = `M ${ax.toFixed(1)} ${ay.toFixed(1)} L ${top1X.toFixed(1)} ${top1Y.toFixed(1)} L ${top2X.toFixed(1)} ${top2Y.toFixed(1)} L ${bx.toFixed(1)} ${by.toFixed(1)} Z`;

        // Add 5 intermediate transverse ticks across the load span
        const numTicks = 5;
        for (let t = 1; t < numTicks; t++) {
          const frac = t / numTicks;
          const baseX = ax + frac * (bx - ax);
          const baseY = ay + frac * (by - ay);
          const topX = top1X + frac * (top2X - top1X);
          const topY = top1Y + frac * (top2Y - top1Y);
          path += ` M ${topX.toFixed(1)} ${topY.toFixed(1)} L ${baseX.toFixed(1)} ${baseY.toFixed(1)}`;
        }

        const isUniform = Math.abs(dl.qy1 - dl.qy2) < 1e-4;
        const label = isUniform
          ? `${dl.qy1} kN/m`
          : `${dl.qy1} → ${dl.qy2} kN/m`;

        const midTopX = (top1X + top2X) / 2 + nx * 8;
        const midTopY = (top1Y + top2Y) / 2 + ny * 8;

        return {
          id: dl.id,
          path,
          label,
          labelX: midTopX,
          labelY: midTopY,
        };
      })
      .filter(isNotNull);

    return { nodal, memberPoints, dists };
  });

  readonly drawnDiagrams = computed(() => {
    const diagType = this.resultsService.activeDiagram();
    const isStable = this.resultsService.isStable();
    if (!isStable || diagType === 'None' || diagType === 'Deformed') return [];

    const vp = this.viewport();
    const m = this.model();
    const nodeMap = new Map(m.nodes.map(n => [n.id, n]));
    const diagrams: Array<{
      id: string;
      path: string;
      extremaLabel?: string;
      extremaX?: number;
      extremaY?: number;
    }> = [];

    const scale = this.resultsService.diagramScale() * 0.8;

    // 1. Envelope Mode (if envelope exists and has envelopes)
    const env = this.resultsService.currentEnvelope();
    if (this.resultsService.resultTarget() === 'Envelope' && env && env.member_envelopes.length > 0) {
      for (const me of env.member_envelopes) {
        const elem = m.elements.find(e => e.id === me.element_id);
        if (!elem || !isLineElement(elem)) continue;
        const ni = nodeMap.get(elem.node_i);
        const nj = nodeMap.get(elem.node_j);
        if (!ni || !nj) continue;

        const p1 = vp.worldToScreen(ni.x, ni.y);
        const p2 = vp.worldToScreen(nj.x, nj.y);
        const L = Math.hypot(p2.x - p1.x, p2.y - p1.y);
        if (L < 10 || me.stations.length < 2) continue;

        const ux = (p2.x - p1.x) / L;
        const uy = (p2.y - p1.y) / L;
        const nx = -uy;
        const ny = ux;

        const ptsUpper: Array<{ x: number; y: number }> = [];
        const ptsLower: Array<{ x: number; y: number }> = [];
        const effLen = (me as any).length_eff_m || Math.hypot(nj.x - ni.x, nj.y - ni.y) || me.stations[me.stations.length - 1]?.x_m || 1.0;

        for (const st of me.stations) {
          const t = Math.max(0, Math.min(1, st.x_m / effLen));
          const baseX = p1.x + t * (p2.x - p1.x);
          const baseY = p1.y + t * (p2.y - p1.y);

          const maxV = diagType === 'M' ? st.max_m : diagType === 'Q' ? st.max_q : st.max_n;
          const minV = diagType === 'M' ? st.min_m : diagType === 'Q' ? st.min_q : st.min_n;

          ptsUpper.push({ x: baseX + nx * maxV * scale, y: baseY + ny * maxV * scale });
          ptsLower.push({ x: baseX + nx * minV * scale, y: baseY + ny * minV * scale });
        }

        let d = `M ${ptsUpper[0].x} ${ptsUpper[0].y} `;
        for (let i = 1; i < ptsUpper.length; i++) d += `L ${ptsUpper[i].x} ${ptsUpper[i].y} `;
        for (let i = ptsLower.length - 1; i >= 0; i--) d += `L ${ptsLower[i].x} ${ptsLower[i].y} `;
        d += 'Z';

        const mid = Math.floor(ptsUpper.length / 2);
        diagrams.push({
          id: `env_${me.element_id}`,
          path: d,
          extremaLabel: `Env [${Math.round(me.stations[mid]?.max_m || 0)} / ${Math.round(me.stations[mid]?.min_m || 0)}]`,
          extremaX: ptsUpper[mid]?.x,
          extremaY: ptsUpper[mid]?.y,
        });
      }
      return diagrams;
    }

    // 2. Single ResultSet Mode
    const rs = this.resultsService.currentResultSet();
    if (!rs) return [];

    for (const mr of rs.member_results) {
      const elem = m.elements.find(e => e.id === mr.element_id);
      if (!elem || !isLineElement(elem)) continue;
      const ni = nodeMap.get(elem.node_i);
      const nj = nodeMap.get(elem.node_j);
      if (!ni || !nj) continue;

      const p1 = vp.worldToScreen(ni.x, ni.y);
      const p2 = vp.worldToScreen(nj.x, nj.y);
      const L = Math.hypot(p2.x - p1.x, p2.y - p1.y);
      if (L < 10) continue;

      const ux = (p2.x - p1.x) / L;
      const uy = (p2.y - p1.y) / L;
      const nx = -uy;
      const ny = ux;

      const stations = mr.stations;
      if (stations.length < 2) continue;

      const pointsDiagram: Array<{ x: number; y: number }> = [];
      const pointsBase: Array<{ x: number; y: number }> = [];

      for (const st of stations) {
        const t = st.x_m / mr.length_eff_m;
        const baseX = p1.x + t * (p2.x - p1.x);
        const baseY = p1.y + t * (p2.y - p1.y);
        pointsBase.push({ x: baseX, y: baseY });

        let val = 0;
        if (diagType === 'M') val = st.m_knm;
        else if (diagType === 'Q') val = st.q_kn;
        else if (diagType === 'N') val = st.n_kn;

        const offsetPx = val * scale;
        pointsDiagram.push({
          x: baseX + nx * offsetPx,
          y: baseY + ny * offsetPx,
        });
      }

      let d = `M ${pointsBase[0].x} ${pointsBase[0].y} `;
      for (const p of pointsDiagram) {
        d += `L ${p.x} ${p.y} `;
      }
      for (let i = pointsBase.length - 1; i >= 0; i--) {
        d += `L ${pointsBase[i].x} ${pointsBase[i].y} `;
      }
      d += 'Z';

      let extremaLabel: string | undefined;
      let extX: number | undefined;
      let extY: number | undefined;

      if (diagType === 'M' && mr.extrema?.max_m) {
        const valMax = Math.round(mr.extrema.max_m.value * 10) / 10;
        extremaLabel = `Mmax=${valMax}`;
        const midIdx = Math.floor(pointsDiagram.length / 2);
        extX = pointsDiagram[midIdx]?.x;
        extY = pointsDiagram[midIdx]?.y;
      }

      diagrams.push({
        id: `diag_${mr.element_id}`,
        path: d,
        extremaLabel,
        extremaX: extX,
        extremaY: extY,
      });
    }

    return diagrams;
  });

  readonly drawnReactions = computed(() => {
    const isStable = this.resultsService.isStable();
    const rs = this.resultsService.currentResultSet();
    if (!isStable || !rs) return [];

    const vp = this.viewport();
    const m = this.model();
    const nodeMap = new Map(m.nodes.map(n => [n.id, n]));

    return rs.support_reactions
      .map(sr => {
        const node = nodeMap.get(sr.node_id);
        if (!node) return null;
        const p = vp.worldToScreen(node.x, node.y);
        return {
          id: sr.support_id,
          x: p.x,
          y: p.y,
          rx: Math.round(sr.rx_kn * 10) / 10,
          ry: Math.round(sr.ry_kn * 10) / 10,
          mz: Math.round(sr.mz_knm * 10) / 10,
        };
      })
      .filter(isNotNull);
  });

  readonly rubberBandLine = computed(() => {
    const start = this.drawStartPoint();
    const snap = this.currentSnap();
    if (!start || !snap) return null;

    const vp = this.viewport();
    const p1 = vp.worldToScreen(start.x, start.y);
    const p2 = snap.screenPoint;

    const dx = snap.point.x - start.x;
    const dy = snap.point.y - start.y;
    const length = Math.hypot(dx, dy);
    let angleDeg = (Math.atan2(dy, dx) * 180) / Math.PI;
    if (angleDeg < 0) angleDeg += 360;

    return {
      x1: p1.x,
      y1: p1.y,
      x2: p2.x,
      y2: p2.y,
      lengthM: length.toFixed(2),
      angleDeg: angleDeg.toFixed(1),
      midX: (p1.x + p2.x) / 2,
      midY: (p1.y + p2.y) / 2 - 14,
    };
  });

  readonly marqueeRect = computed(() => {
    if (!this.isMarqueeActive()) return null;
    const startW = this.marqueeStartWorld();
    const curW = this.marqueeCurrentWorld();
    if (!startW || !curW) return null;

    const vp = this.viewport();
    const p1 = vp.worldToScreen(startW.x, startW.y);
    const p2 = vp.worldToScreen(curW.x, curW.y);

    const x = Math.min(p1.x, p2.x);
    const y = Math.min(p1.y, p2.y);
    const w = Math.abs(p2.x - p1.x);
    const h = Math.abs(p2.y - p1.y);
    const isCrossing = curW.x < startW.x;

    return { x, y, width: w, height: h, isCrossing };
  });

  onNodeMouseDown(e: MouseEvent, nodeId: string): void {
    if (e.button !== 0) return;
    if (this.isEffectivePan()) return;
    const tool = this.store.activeTool();

    if (tool === 'PlaceSupport') {
      e.stopPropagation();
      this.store.assignSupport(nodeId, this.store.selectedSupportKind());
      return;
    }

    if (tool === 'Select' || tool === 'Move') {
      e.stopPropagation();
      if (!this.store.selection().nodeIds.has(nodeId) && !e.shiftKey) {
        this.store.toggleEntitySelection('node', nodeId, false);
      }
      this.draggedNodeId.set(nodeId);
      this.isNodeDragging.set(true);
      const node = this.model().nodes.find(n => n.id === nodeId);
      if (node) {
        this.previewDragNode.set({ id: nodeId, x: node.x, y: node.y });
      }
    }
  }

  onMouseDown(e: MouseEvent): void {
    if (this.isEffectivePan() || e.button === 1 || (e.button === 0 && e.altKey)) {
      this.isPanning.set(true);
      this.lastMouseScreen.set({ x: e.clientX, y: e.clientY });
      return;
    }

    if (e.button !== 0) return;

    const rect = this.svgRef()?.nativeElement.getBoundingClientRect();
    if (!rect) return;
    const screenX = e.clientX - rect.left;
    const screenY = e.clientY - rect.top;

    const snap = SnappingEngine.evaluateSnap(
      { x: screenX, y: screenY },
      this.viewport(),
      this.model().nodes,
      this.model().elements,
      this.drawStartPoint(),
      this.store.snappingOptions()
    );

    const tool = this.store.activeTool();

    if (tool === 'DrawMember' || tool === 'DrawTruss' || tool === 'DrawCable') {
      const curStart = this.drawStartPoint();
      if (!curStart) {
        this.drawStartPoint.set(snap.point);
      } else {
        const len = Math.hypot(snap.point.x - curStart.x, snap.point.y - curStart.y);
        if (len > 1e-4) {
          this.store.sketchMember(curStart, snap.point);
          this.drawStartPoint.set(snap.point);
        }
      }
    } else if (tool === 'Select') {
      this.isMarqueeActive.set(true);
      const worldPt = this.viewport().screenToWorld(screenX, screenY);
      this.marqueeStartWorld.set(worldPt);
      this.marqueeCurrentWorld.set(worldPt);
    }
  }

  onMouseMove(e: MouseEvent): void {
    const rect = this.svgRef()?.nativeElement.getBoundingClientRect();
    if (!rect) return;
    const screenX = e.clientX - rect.left;
    const screenY = e.clientY - rect.top;

    if (this.isPanning()) {
      const last = this.lastMouseScreen();
      const dx = e.clientX - last.x;
      const dy = e.clientY - last.y;
      const vp = this.viewport().clone();
      vp.pan(dx, dy);
      this.store.viewport.set(vp);
      this.lastMouseScreen.set({ x: e.clientX, y: e.clientY });
      return;
    }

    const snap = SnappingEngine.evaluateSnap(
      { x: screenX, y: screenY },
      this.viewport(),
      this.model().nodes,
      this.model().elements,
      this.drawStartPoint(),
      this.store.snappingOptions()
    );

    this.currentSnap.set(snap);
    this.store.currentCursorWorld.set(snap.point);

    if (this.isNodeDragging() && this.draggedNodeId()) {
      this.previewDragNode.set({
        id: this.draggedNodeId()!,
        x: Math.round(snap.point.x * 1000) / 1000,
        y: Math.round(snap.point.y * 1000) / 1000,
      });
      return;
    }

    if (this.isMarqueeActive()) {
      const worldCur = this.viewport().screenToWorld(screenX, screenY);
      this.marqueeCurrentWorld.set(worldCur);
    }

    if (this.store.activeTool() === 'Select' && !this.isMarqueeActive()) {
      this.checkMemberProbe(snap.point);
    }
  }

  private checkMemberProbe(worldCursor: { x: number; y: number }): void {
    const m = this.model();
    const nodeMap = new Map(m.nodes.map(n => [n.id, n]));

    for (const elem of m.elements) {
      if (!isLineElement(elem)) continue;
      const n1 = nodeMap.get(elem.node_i);
      const n2 = nodeMap.get(elem.node_j);
      if (!n1 || !n2) continue;

      const len = Math.hypot(n2.x - n1.x, n2.y - n1.y);
      if (len < 1e-4) continue;

      const dx = n2.x - n1.x;
      const dy = n2.y - n1.y;
      const s = ((worldCursor.x - n1.x) * dx + (worldCursor.y - n1.y) * dy) / (len * len);

      if (s >= 0 && s <= 1.0) {
        const projX = n1.x + s * dx;
        const projY = n1.y + s * dy;
        const dist = Math.hypot(worldCursor.x - projX, worldCursor.y - projY);

        if (dist <= 0.25) {
          this.resultsService.probeCrossSection(elem.id, Math.round(s * len * 100) / 100);
          return;
        }
      }
    }
    this.resultsService.clearProbe();
  }

  onMouseUp(e: MouseEvent): void {
    if (this.isPanning()) {
      this.isPanning.set(false);
      return;
    }

    if (this.isNodeDragging()) {
      const p = this.previewDragNode();
      if (p) {
        this.store.updateNodePosition(p.id, p.x, p.y);
      }
      this.previewDragNode.set(null);
      this.isNodeDragging.set(false);
      this.draggedNodeId.set(null);
      return;
    }

    if (this.isMarqueeActive()) {
      const startW = this.marqueeStartWorld();
      const curW = this.marqueeCurrentWorld();
      if (startW && curW) {
        const dx = Math.abs(curW.x - startW.x);
        const dy = Math.abs(curW.y - startW.y);
        if (dx > 0.05 || dy > 0.05) {
          this.store.selectBox(startW, curW, e.shiftKey);
        } else if (!e.shiftKey) {
          this.store.clearSelection();
        }
      }
      this.isMarqueeActive.set(false);
      this.marqueeStartWorld.set(null);
      this.marqueeCurrentWorld.set(null);
    }
  }

  zoomIn(): void {
    const vp = this.viewport();
    vp.zoomAt(vp.width / 2, vp.height / 2, 1.25);
    this.store.viewport.set(new DraftingViewport(vp.width, vp.height, vp.scale, vp.panX, vp.panY, vp.gridStep));
  }

  zoomOut(): void {
    const vp = this.viewport();
    vp.zoomAt(vp.width / 2, vp.height / 2, 0.8);
    this.store.viewport.set(new DraftingViewport(vp.width, vp.height, vp.scale, vp.panX, vp.panY, vp.gridStep));
  }

  zoomFit(): void {
    const nodes = this.store.model().nodes;
    const vp = this.store.viewport();
    if (nodes.length === 0) {
      this.store.viewport.set(new DraftingViewport());
      return;
    }

    let minX = Infinity;
    let maxX = -Infinity;
    let minY = Infinity;
    let maxY = -Infinity;

    for (const n of nodes) {
      if (n.x < minX) minX = n.x;
      if (n.x > maxX) maxX = n.x;
      if (n.y < minY) minY = n.y;
      if (n.y > maxY) maxY = n.y;
    }

    const spanX = Math.max(1.0, maxX - minX);
    const spanY = Math.max(1.0, maxY - minY);
    const margin = 1.3;

    const scaleX = (vp.width * 0.8) / (spanX * margin);
    const scaleY = (vp.height * 0.8) / (spanY * margin);
    const newScale = Math.min(scaleX, scaleY, 150);

    const midX = (minX + maxX) / 2;
    const midY = (minY + maxY) / 2;
    const newPanX = vp.width / 2 - midX * newScale;
    const newPanY = vp.height / 2 + midY * newScale;

    this.store.viewport.set(new DraftingViewport(vp.width, vp.height, newScale, newPanX, newPanY, vp.gridStep));
  }

  togglePanMode(): void {
    this.isPanMode.update(v => !v);
  }

  onWheel(e: WheelEvent): void {
    e.preventDefault();
    const rect = this.svgRef()?.nativeElement.getBoundingClientRect();
    if (!rect) return;
    const screenX = e.clientX - rect.left;
    const screenY = e.clientY - rect.top;

    const factor = e.deltaY < 0 ? 1.15 : 0.85;
    const vp = this.viewport().clone();
    vp.zoomAt(screenX, screenY, factor);
    this.store.viewport.set(vp);
  }

  @HostListener('window:keydown', ['$event'])
  onKeyDown(e: KeyboardEvent): void {
    const targetTag = (e.target as HTMLElement)?.tagName?.toLowerCase();
    if (targetTag === 'input' || targetTag === 'textarea' || targetTag === 'select') {
      return;
    }

    if (e.code === 'Space' || e.key === ' ') {
      e.preventDefault();
      this.isSpacePressed.set(true);
      return;
    }

    if (e.key === 'Escape') {
      this.drawStartPoint.set(null);
      this.store.clearSelection();
    } else if (e.key === 'F8') {
      e.preventDefault();
      this.store.toggleOrtho();
    } else if (e.key === 'Delete') {
      this.store.deleteSelection();
    } else if (e.ctrlKey && e.key.toLowerCase() === 'z') {
      e.preventDefault();
      this.store.undo();
    } else if (e.ctrlKey && e.key.toLowerCase() === 'y') {
      e.preventDefault();
      this.store.redo();
    }
  }

  @HostListener('window:keyup', ['$event'])
  onKeyUp(e: KeyboardEvent): void {
    if (e.code === 'Space' || e.key === ' ') {
      this.isSpacePressed.set(false);
    }
  }
}
