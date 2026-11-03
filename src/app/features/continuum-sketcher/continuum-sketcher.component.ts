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

@Component({
  selector: 'app-continuum-sketcher',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './continuum-sketcher.component.html',
  styleUrl: './continuum-sketcher.component.css',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ContinuumSketcherComponent {
  readonly store = inject(ContinuumSketcherStore);
  readonly close = output<void>();

  readonly svgRef = viewChild<ElementRef<SVGSVGElement>>('sketchSvg');
  readonly coordInputRef = viewChild<ElementRef<HTMLInputElement>>('coordInput');

  // Mouse coordinate readout in domain space (meters)
  readonly cursorCoord = signal<{ x: number; y: number }>({ x: 0, y: 0 });

  // Coordinate entry input value
  readonly coordInputValue = signal<string>('');

  // Flyout menu states
  readonly isDrawFlyoutOpen = signal<boolean>(false);
  readonly isCutoutFlyoutOpen = signal<boolean>(false);
  readonly isTemplateMenuOpen = signal<boolean>(false);

  // ViewBox bounds [minX, minY, width, height]
  readonly viewBox = computed(() => {
    const domain = this.store.domain();
    const verts = domain.outer_boundary.vertices;
    if (verts.length === 0) {
      return '-5 -5 20 20';
    }
    let minX = 0;
    let maxX = 10;
    let minY = 0;
    let maxY = 10;
    for (const v of verts) {
      if (v.x < minX) minX = v.x;
      if (v.x > maxX) maxX = v.x;
      if (v.y < minY) minY = v.y;
      if (v.y > maxY) maxY = v.y;
    }
    const pad = 2.5;
    return `${(minX - pad).toFixed(2)} ${(minY - pad).toFixed(2)} ${(maxX - minX + 2 * pad).toFixed(2)} ${(maxY - minY + 2 * pad).toFixed(2)}`;
  });

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
    this.isTemplateMenuOpen.set(false);
  }

  selectTool(tool: SketcherTool): void {
    this.closeFlyouts();
    if (tool === 'QuickPlateWithHole') {
      this.store.loadTemplatePerforatedPlate();
      this.store.setTool('Select');
    } else if (tool === 'QuickRectangle') {
      this.store.addRectangleBoundary(10.0, 6.0);
      this.store.setTool('Select');
    } else {
      this.store.setTool(tool);
    }
  }

  onSvgMouseMove(event: MouseEvent): void {
    const svg = this.svgRef()?.nativeElement;
    if (!svg) return;

    const pt = svg.createSVGPoint();
    pt.x = event.clientX;
    pt.y = event.clientY;
    const ctm = svg.getScreenCTM();
    if (!ctm) return;
    const svgP = pt.matrixTransform(ctm.inverse());

    // Snapping to 0.25m grid
    const snapX = Math.round(svgP.x * 4) / 4;
    const snapY = Math.round(svgP.y * 4) / 4;
    this.cursorCoord.set({ x: snapX, y: snapY });

    if (this.store.activeTool() === 'OuterPolygon') {
      const isClosing = this.isCloseSnap(snapX, snapY);
      this.store.previewCandidate({ id: 'candidate', x: snapX, y: snapY }, isClosing);
    }
  }

  onSvgClick(event: MouseEvent): void {
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
    } else if (tool === 'AddCircularInclusion') {
      this.store.addCircularInclusion(coord.x, coord.y, 1.2);
      this.store.setTool('Select');
    } else {
      // Clear selection if clicking empty canvas
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
  }

  selectCutout(id: string, e: MouseEvent): void {
    e.stopPropagation();
    this.closeFlyouts();
    this.store.selectedCutoutId.set(id);
    this.store.selectedSegmentIdx.set(null);
    this.store.selectedInclusionId.set(null);
  }

  selectInclusion(id: string, e: MouseEvent): void {
    e.stopPropagation();
    this.closeFlyouts();
    this.store.selectedInclusionId.set(id);
    this.store.selectedSegmentIdx.set(null);
    this.store.selectedCutoutId.set(null);
  }

  // Coordinate Input Parser: handles x,y or @dx,dy or @L<deg
  submitCoordinateEntry(): void {
    const raw = this.coordInputValue().trim();
    if (!raw) return;

    const verts = this.store.domain().outer_boundary.vertices;
    const lastVert = verts.length > 0 ? verts[verts.length - 1] : { x: 0, y: 0 };

    if (raw.startsWith('@')) {
      const payload = raw.substring(1).trim();
      if (payload.includes('<')) {
        // Polar: @L<angle
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
        // Relative cartesian: @dx,dy
        const [dxStr, dyStr] = payload.split(',');
        const dx = parseFloat(dxStr);
        const dy = parseFloat(dyStr);
        if (!isNaN(dx) && !isNaN(dy)) {
          this.store.addVertex(lastVert.x + dx, lastVert.y + dy);
          this.coordInputValue.set('');
        }
      }
    } else if (raw.includes(',')) {
      // Absolute cartesian: x,y
      const [xStr, yStr] = raw.split(',');
      const x = parseFloat(xStr);
      const y = parseFloat(yStr);
      if (!isNaN(x) && !isNaN(y)) {
        this.store.addVertex(x, y);
        this.coordInputValue.set('');
      }
    }
  }

  updateSegmentTag(tag: string): void {
    const idx = this.store.selectedSegmentIdx();
    if (idx === null) return;
    const currentSpec = this.store.domain().tag_conditions[tag] ?? { kind: 'Free' };
    this.store.updateSegmentTag(idx, tag, currentSpec);
  }

  updateSegmentBcKind(kind: string): void {
    const seg = this.selectedSegment();
    const idx = this.store.selectedSegmentIdx();
    if (!seg || idx === null) return;

    let spec: BoundaryConditionSpec = { kind: 'Free' };
    if (kind === 'DirichletDisplacement') {
      spec = {
        kind: 'DirichletDisplacement',
        fix_x: true,
        fix_y: true,
        ux_m: 0.0,
        uy_m: 0.0,
      };
    } else if (kind === 'NeumannTraction') {
      spec = {
        kind: 'NeumannTraction',
        tx_kn_m2: 0.0,
        ty_kn_m2: 100.0,
        coordinate_frame: 'Global',
      };
    } else if (kind === 'RobinImpedance') {
      spec = {
        kind: 'RobinImpedance',
        kn_kn_m3: 10000.0,
        ks_kn_m3: 5000.0,
        cn_kn_s_m3: 0.0,
        cs_kn_s_m3: 0.0,
      };
    }

    this.store.updateSegmentTag(idx, seg.tag, spec);
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
        return '#3b82f6'; // blue
      case 'top':
        return '#ef4444'; // red
      case 'left':
        return '#10b981'; // emerald
      case 'right':
        return '#f59e0b'; // amber
      case 'hole':
        return '#ec4899'; // pink
      default:
        return '#8b5cf6'; // violet
    }
  }

  getNormalArrowEnd(seg: BoundarySegment): { x: number; y: number } {
    const verts = this.store.domain().outer_boundary.vertices;
    const v1 = verts.find((v) => v.id === seg.start_vertex_id);
    const v2 = verts.find((v) => v.id === seg.end_vertex_id);
    if (!v1 || !v2) return { x: 0, y: 0 };
    const midX = (v1.x + v2.x) / 2;
    const midY = (v1.y + v2.y) / 2;
    const len = 0.5; // arrow length in meters
    return {
      x: midX + seg.outward_normal[0] * len,
      y: midY + seg.outward_normal[1] * len,
    };
  }

  getSegmentMid(seg: BoundarySegment): { x: number; y: number } {
    const verts = this.store.domain().outer_boundary.vertices;
    const v1 = verts.find((v) => v.id === seg.start_vertex_id);
    const v2 = verts.find((v) => v.id === seg.end_vertex_id);
    if (!v1 || !v2) return { x: 0, y: 0 };
    return {
      x: (v1.x + v2.x) / 2,
      y: (v1.y + v2.y) / 2,
    };
  }
}
