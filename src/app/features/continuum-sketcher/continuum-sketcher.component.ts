import {
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  HostListener,
  computed,
  inject,
  output,
  signal,
  viewChild,
  OnInit,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ContinuumSketcherStore } from './continuum-sketcher.store';
import {
  BoundaryConditionSpec,
  BoundarySegment,
  Cutout,
  InclusionRegion,
  SketcherTool,
  Vertex2D,
} from './continuum-sketcher.types';
import { DraftingViewport } from '../modeler/core/viewport';
import { ContinuumInspectorComponent } from './continuum-inspector.component';

@Component({
  selector: 'app-continuum-sketcher',
  standalone: true,
  imports: [CommonModule, FormsModule, ContinuumInspectorComponent],
  templateUrl: './continuum-sketcher.component.html',
  styleUrl: './continuum-sketcher.component.css',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ContinuumSketcherComponent implements OnInit {
  readonly store = inject(ContinuumSketcherStore);
  readonly close = output<void>();

  readonly svgRef = viewChild<ElementRef<SVGSVGElement>>('sketchSvg');
  readonly coordInputRef = viewChild<ElementRef<HTMLInputElement>>('coordInput');

  // Engineering Viewport (Y-up convention, pan & zoom)
  readonly viewport = signal<DraftingViewport>(new DraftingViewport(1200, 800, 45, 600, 450, 0.5));

  // Mouse coordinate readout in world Cartesian meters (X to right, Y upwards)
  readonly cursorCoord = signal<{ x: number; y: number }>({ x: 0, y: 0 });

  // Coordinate entry input value
  readonly coordInputValue = signal<string>('');

  // Flyout menu states
  readonly isDrawFlyoutOpen = signal<boolean>(false);
  readonly isCutoutFlyoutOpen = signal<boolean>(false);
  readonly isInclusionFlyoutOpen = signal<boolean>(false);
  readonly isTemplateMenuOpen = signal<boolean>(false);

  // Pan interaction state
  private isPanning = false;
  private panStartScreen = { x: 0, y: 0 };

  ngOnInit(): void {
    // Center viewport around initial domain
    this.fitViewToDomain();
  }

  fitViewToDomain(): void {
    const verts = this.store.domain().outer_boundary.vertices;
    if (verts.length > 0) {
      this.viewport.update((vp) => {
        const next = vp.clone();
        next.zoomExtents(verts, 120);
        return next;
      });
    }
  }

  readonly selectedSegment = computed<BoundarySegment | null>(() => {
    const idx = this.store.selectedSegmentIdx();
    if (idx === null) return null;
    return this.store.domain().outer_boundary.segments[idx] ?? null;
  });

  readonly selectedCutout = computed<Cutout | null>(() => {
    const id = this.store.selectedCutoutId();
    if (!id) return null;
    return this.store.domain().cutouts.find((c) => c.id === id) ?? null;
  });

  readonly selectedInclusion = computed<InclusionRegion | null>(() => {
    const id = this.store.selectedInclusionId();
    if (!id) return null;
    return this.store.domain().inclusions.find((i) => i.id === id) ?? null;
  });

  // Global keyboard listener for AutoCAD-style @ entry
  @HostListener('window:keydown', ['$event'])
  handleGlobalKey(event: KeyboardEvent): void {
    if (event.target instanceof HTMLInputElement || event.target instanceof HTMLSelectElement) {
      if (event.key === 'Escape') {
        (event.target as HTMLElement).blur();
        this.coordInputValue.set('');
      }
      return;
    }

    if (event.key === '@') {
      event.preventDefault();
      const input = this.coordInputRef()?.nativeElement;
      if (input) {
        input.focus();
        this.coordInputValue.set('@');
      }
    } else if (event.key === 'Escape') {
      this.closeFlyouts();
      this.store.setTool('Select');
    }
  }

  closeFlyouts(): void {
    this.isDrawFlyoutOpen.set(false);
    this.isCutoutFlyoutOpen.set(false);
    this.isInclusionFlyoutOpen.set(false);
    this.isTemplateMenuOpen.set(false);
  }

  selectTool(tool: SketcherTool): void {
    this.closeFlyouts();
    if (tool === 'QuickPlateWithHole') {
      this.store.loadTemplatePerforatedPlate();
      this.store.setTool('Select');
      this.fitViewToDomain();
    } else if (tool === 'QuickRectangle') {
      this.store.addRectangleBoundary(10.0, 6.0);
      this.store.setTool('Select');
      this.fitViewToDomain();
    } else {
      this.store.setTool(tool);
    }
  }

  // --- Coordinate Transformations (World Y-up <-> Screen Y-down) ---
  w2s(x: number, y: number): { x: number; y: number } {
    return this.viewport().worldToScreen(x, y);
  }

  getScreenCoords(event: MouseEvent): { screenX: number; screenY: number } {
    const svg = this.svgRef()?.nativeElement;
    if (!svg) return { screenX: 0, screenY: 0 };
    const rect = svg.getBoundingClientRect();
    return {
      screenX: event.clientX - rect.left,
      screenY: event.clientY - rect.top,
    };
  }

  onSvgMouseDown(event: MouseEvent): void {
    if (event.button === 1 || event.altKey) {
      // Middle mouse button pan
      event.preventDefault();
      this.isPanning = true;
      this.panStartScreen = { x: event.clientX, y: event.clientY };
    }
  }

  onSvgMouseMove(event: MouseEvent): void {
    if (this.isPanning) {
      const dx = event.clientX - this.panStartScreen.x;
      const dy = event.clientY - this.panStartScreen.y;
      this.panStartScreen = { x: event.clientX, y: event.clientY };
      this.viewport.update((vp) => {
        const next = vp.clone();
        next.pan(dx, dy);
        return next;
      });
      return;
    }

    const { screenX, screenY } = this.getScreenCoords(event);
    const world = this.viewport().screenToWorld(screenX, screenY);

    // Grid snapping: 0.25m step (Y is positive UPWARDS)
    const snapX = Math.round(world.x * 4) / 4;
    const snapY = Math.round(world.y * 4) / 4;
    this.cursorCoord.set({ x: snapX, y: snapY });

    if (this.store.activeTool() === 'OuterPolygon') {
      const isClosing = this.isCloseSnap(snapX, snapY);
      this.store.previewCandidate({ id: 'candidate', x: snapX, y: snapY }, isClosing);
    }
  }

  onSvgMouseUp(event: MouseEvent): void {
    if (this.isPanning) {
      this.isPanning = false;
    }
  }

  onSvgWheel(event: WheelEvent): void {
    event.preventDefault();
    const { screenX, screenY } = this.getScreenCoords(event);
    const zoomFactor = event.deltaY < 0 ? 1.15 : 0.87;
    this.viewport.update((vp) => {
      const next = vp.clone();
      next.zoomAt(screenX, screenY, zoomFactor);
      return next;
    });
  }

  onSvgClick(event: MouseEvent): void {
    if (this.isPanning) return;
    this.closeFlyouts();
    const coord = this.cursorCoord();
    const tool = this.store.activeTool();

    if (tool === 'OuterPolygon') {
      this.store.addVertex(coord.x, coord.y);
    } else if (tool === 'AddCircularCutout') {
      this.store.addCircularCutout(coord.x, coord.y, 1.0);
      this.store.setTool('Select');
    } else if (tool === 'AddRectangularCutout') {
      this.store.addRectangularCutout(coord.x, coord.y, 2.0, 2.0);
      this.store.setTool('Select');
    } else if (tool === 'AddPolygonalCutout') {
      this.store.addPolygonalCutout(coord.x, coord.y, 1.2, 6);
      this.store.setTool('Select');
    } else if (tool === 'AddCircularInclusion') {
      this.store.addCircularInclusion(coord.x, coord.y, 1.2);
      this.store.setTool('Select');
    } else if (tool === 'AddRectangularInclusion') {
      this.store.addRectangularInclusion(coord.x, coord.y, 2.0, 2.0);
      this.store.setTool('Select');
    } else {
      this.store.selectedSegmentIdx.set(null);
      this.store.selectedCutoutId.set(null);
      this.store.selectedInclusionId.set(null);
    }
  }

  isCloseSnap(x: number, y: number): boolean {
    const verts = this.store.domain().outer_boundary.vertices;
    if (verts.length < 3) return false;
    const v0 = verts[0];
    return Math.hypot(x - v0.x, y - v0.y) < 0.5;
  }

  selectSegment(idx: number, e: MouseEvent): void {
    e.stopPropagation();
    this.closeFlyouts();
    this.store.selectedSegmentIdx.set(idx);
    this.store.selectedCutoutId.set(null);
    this.store.selectedInclusionId.set(null);
    this.store.inspectorTab.set('element');
  }

  selectCutout(id: string, e: MouseEvent): void {
    e.stopPropagation();
    this.closeFlyouts();
    this.store.selectedCutoutId.set(id);
    this.store.selectedSegmentIdx.set(null);
    this.store.selectedInclusionId.set(null);
    this.store.inspectorTab.set('element');
  }

  selectInclusion(id: string, e: MouseEvent): void {
    e.stopPropagation();
    this.closeFlyouts();
    this.store.selectedInclusionId.set(id);
    this.store.selectedSegmentIdx.set(null);
    this.store.selectedCutoutId.set(null);
    this.store.inspectorTab.set('element');
  }

  openMeshSettings(): void {
    this.store.inspectorTab.set('mesh');
  }

  // --- AutoCAD-Style Coordinate Input Parser (Y-up) ---
  submitCoordinateEntry(): void {
    const raw = this.coordInputValue().trim();
    if (!raw) return;

    const verts = this.store.domain().outer_boundary.vertices;
    const lastVert = verts.length > 0 ? verts[verts.length - 1] : { x: 0, y: 0 };

    if (raw.startsWith('@')) {
      const payload = raw.substring(1).trim();
      if (payload.includes('<')) {
        // Polar relative: @L<angle (angle in degrees CCW, Y is positive UP)
        const [lenStr, angStr] = payload.split('<');
        const length = parseFloat(lenStr);
        const angleDeg = parseFloat(angStr);
        if (!isNaN(length) && !isNaN(angleDeg)) {
          const rad = (angleDeg * Math.PI) / 180;
          const targetX = lastVert.x + length * Math.cos(rad);
          const targetY = lastVert.y + length * Math.sin(rad);
          this.store.addVertex(targetX, targetY);
          this.coordInputValue.set('');
        }
      } else if (payload.includes(',')) {
        // Cartesian relative: @dx,dy (dy > 0 is UPWARDS!)
        const [dxStr, dyStr] = payload.split(',');
        const dx = parseFloat(dxStr);
        const dy = parseFloat(dyStr);
        if (!isNaN(dx) && !isNaN(dy)) {
          this.store.addVertex(lastVert.x + dx, lastVert.y + dy);
          this.coordInputValue.set('');
        }
      }
    } else if (raw.includes(',')) {
      // Cartesian absolute: x,y
      const [xStr, yStr] = raw.split(',');
      const x = parseFloat(xStr);
      const y = parseFloat(yStr);
      if (!isNaN(x) && !isNaN(y)) {
        this.store.addVertex(x, y);
        this.coordInputValue.set('');
      }
    }
  }

  // --- Draft Segment Helpers (Live Dimensions in OpenDraft) ---
  getDraftSegmentLength(v1: Vertex2D, v2: Vertex2D): number {
    return Math.hypot(v2.x - v1.x, v2.y - v1.y);
  }

  getDraftSegmentMid(v1: Vertex2D, v2: Vertex2D): { x: number; y: number } {
    return this.w2s((v1.x + v2.x) / 2, (v1.y + v2.y) / 2);
  }

  getCandidateLength(cand: Vertex2D): number {
    const verts = this.store.domain().outer_boundary.vertices;
    if (verts.length === 0) return 0;
    const last = verts[verts.length - 1];
    return Math.hypot(cand.x - last.x, cand.y - last.y);
  }

  getCandidateScreenMid(cand: Vertex2D): { x: number; y: number } {
    const verts = this.store.domain().outer_boundary.vertices;
    if (verts.length === 0) return this.w2s(cand.x, cand.y);
    const last = verts[verts.length - 1];
    return this.w2s((last.x + cand.x) / 2, (last.y + cand.y) / 2);
  }

  handleSolveAndHandoff(): void {
    const ok = this.store.sendToMode1Solver();
    if (ok) {
      this.close.emit();
    }
  }

  getTagColor(tag: string): string {
    switch (tag.toLowerCase()) {
      case 'bottom':
        return 'var(--accent-primary, #0284c7)';
      case 'top':
        return 'var(--accent-danger, #ef4444)';
      case 'left':
        return 'var(--accent-success, #10b981)';
      case 'right':
        return 'var(--accent-warning, #f59e0b)';
      case 'hole':
        return '#ec4899';
      default:
        return 'var(--accent-glow, #38bdf8)';
    }
  }

  getSegmentLength(seg: BoundarySegment): number {
    const verts = this.store.domain().outer_boundary.vertices;
    const v1 = verts.find((v) => v.id === seg.start_vertex_id);
    const v2 = verts.find((v) => v.id === seg.end_vertex_id);
    if (!v1 || !v2) return 0;
    return Math.hypot(v2.x - v1.x, v2.y - v1.y);
  }

  getSegmentScreenMid(seg: BoundarySegment): { x: number; y: number } {
    const verts = this.store.domain().outer_boundary.vertices;
    const v1 = verts.find((v) => v.id === seg.start_vertex_id);
    const v2 = verts.find((v) => v.id === seg.end_vertex_id);
    if (!v1 || !v2) return { x: 0, y: 0 };
    return this.w2s((v1.x + v2.x) / 2, (v1.y + v2.y) / 2);
  }

  getNormalArrowScreenEnd(seg: BoundarySegment): { x: number; y: number } {
    const verts = this.store.domain().outer_boundary.vertices;
    const v1 = verts.find((v) => v.id === seg.start_vertex_id);
    const v2 = verts.find((v) => v.id === seg.end_vertex_id);
    if (!v1 || !v2) return { x: 0, y: 0 };
    const midX = (v1.x + v2.x) / 2;
    const midY = (v1.y + v2.y) / 2;
    const len = 0.6; // 0.6 meters outward
    return this.w2s(midX + seg.outward_normal[0] * len, midY + seg.outward_normal[1] * len);
  }

  getPolygonPointsSvg(verts: Vertex2D[]): string {
    return verts.map((v) => `${this.w2s(v.x, v.y).x},${this.w2s(v.x, v.y).y}`).join(' ');
  }

  getMeshElementPointsSvg(nodes: [number, number, number][], nodeIndices: number[]): string {
    return nodeIndices
      .map((idx) => {
        const n = nodes[idx];
        if (!n) return '0,0';
        const pt = this.w2s(n[0], n[1]);
        return `${pt.x},${pt.y}`;
      })
      .join(' ');
  }

  getSegmentTractionArrows(seg: BoundarySegment): { x1: number; y1: number; x2: number; y2: number }[] {
    const condition = this.store.domain().tag_conditions[seg.tag];
    if (!condition || condition.kind !== 'NeumannTraction') return [];
    const { tx_kn_m2, ty_kn_m2 } = condition;
    if (tx_kn_m2 === 0 && ty_kn_m2 === 0) return [];

    const verts = this.store.domain().outer_boundary.vertices;
    const v1 = verts.find((v) => v.id === seg.start_vertex_id);
    const v2 = verts.find((v) => v.id === seg.end_vertex_id);
    if (!v1 || !v2) return [];

    const mag = Math.hypot(tx_kn_m2, ty_kn_m2);
    const arrowPx = Math.min(45, Math.max(22, Math.log10(mag + 1) * 20));
    const dirX = tx_kn_m2 / mag;
    const dirY = ty_kn_m2 / mag;

    // In screen coordinates: Y is down, so positive world Y is negative screen Y
    const dx = dirX * arrowPx;
    const dy = -dirY * arrowPx;

    const arrows: { x1: number; y1: number; x2: number; y2: number }[] = [];
    for (const t of [0.25, 0.5, 0.75]) {
      const wx = v1.x + t * (v2.x - v1.x);
      const wy = v1.y + t * (v2.y - v1.y);
      const s = this.w2s(wx, wy);
      arrows.push({
        x1: s.x,
        y1: s.y,
        x2: s.x + dx,
        y2: s.y + dy,
      });
    }
    return arrows;
  }

  getSegmentTractionLabel(seg: BoundarySegment): string | null {
    const condition = this.store.domain().tag_conditions[seg.tag];
    if (!condition || condition.kind !== 'NeumannTraction') return null;
    const { tx_kn_m2, ty_kn_m2 } = condition;
    if (tx_kn_m2 === 0 && ty_kn_m2 === 0) return null;
    if (tx_kn_m2 === 0) return `ty = ${ty_kn_m2} kN/m²`;
    if (ty_kn_m2 === 0) return `tx = ${tx_kn_m2} kN/m²`;
    return `t = (${tx_kn_m2}, ${ty_kn_m2}) kN/m²`;
  }

  isSegmentFixed(seg: BoundarySegment): boolean {
    const condition = this.store.domain().tag_conditions[seg.tag];
    return !!condition && condition.kind === 'DirichletDisplacement' && (condition.fix_x || condition.fix_y);
  }
}
