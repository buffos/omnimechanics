import {
  ChangeDetectionStrategy,
  Component,
  inject,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ContinuumSketcherStore } from './continuum-sketcher.store';
import { BoundaryConditionSpec, BoundarySegment, Cutout, InclusionRegion } from './continuum-sketcher.types';

@Component({
  selector: 'app-continuum-inspector',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './continuum-inspector.component.html',
  styleUrl: './continuum-inspector.component.css',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ContinuumInspectorComponent {
  readonly store = inject(ContinuumSketcherStore);

  get selectedSegment(): BoundarySegment | null {
    const idx = this.store.selectedSegmentIdx();
    if (idx === null) return null;
    return this.store.domain().outer_boundary.segments[idx] ?? null;
  }

  get selectedCutout(): Cutout | null {
    const id = this.store.selectedCutoutId();
    if (!id) return null;
    return this.store.domain().cutouts.find((c) => c.id === id) ?? null;
  }

  get selectedInclusion(): InclusionRegion | null {
    const id = this.store.selectedInclusionId();
    if (!id) return null;
    return this.store.domain().inclusions.find((i) => i.id === id) ?? null;
  }

  getSegmentLength(seg: BoundarySegment): number {
    const verts = this.store.domain().outer_boundary.vertices;
    const v1 = verts.find((v) => v.id === seg.start_vertex_id);
    const v2 = verts.find((v) => v.id === seg.end_vertex_id);
    if (!v1 || !v2) return 0;
    return Math.hypot(v2.x - v1.x, v2.y - v1.y);
  }

  updateSegmentTag(tag: string): void {
    const idx = this.store.selectedSegmentIdx();
    if (idx === null) return;
    const currentSpec = this.store.domain().tag_conditions[tag] ?? { kind: 'Free' };
    this.store.updateSegmentTag(idx, tag, currentSpec);
  }

  updateSegmentBcKind(kind: string): void {
    const seg = this.selectedSegment;
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

  updateSelectedCutoutRadius(r: number): void {
    const cut = this.selectedCutout;
    if (!cut || cut.shape.kind !== 'Circle') return;
    this.store.updateCutout(cut.id, {
      shape: { ...cut.shape, radius: Math.max(0.05, r) },
    });
  }

  updateSelectedCutoutCenter(cx: number, cy: number): void {
    const cut = this.selectedCutout;
    if (!cut) return;
    if (cut.shape.kind === 'Circle' || cut.shape.kind === 'Rectangle') {
      this.store.updateCutout(cut.id, {
        shape: { ...cut.shape, center_x: cx, center_y: cy },
      });
    }
  }

  updateSelectedCutoutRectDims(w: number, h: number): void {
    const cut = this.selectedCutout;
    if (!cut || cut.shape.kind !== 'Rectangle') return;
    this.store.updateCutout(cut.id, {
      shape: { ...cut.shape, width: Math.max(0.1, w), height: Math.max(0.1, h) },
    });
  }

  updateSelectedInclusionRadius(r: number): void {
    const inc = this.selectedInclusion;
    if (!inc || inc.shape.kind !== 'Circle') return;
    this.store.updateInclusion(inc.id, {
      shape: { ...inc.shape, radius: Math.max(0.05, r) },
    });
  }

  updateSelectedInclusionCenter(cx: number, cy: number): void {
    const inc = this.selectedInclusion;
    if (!inc) return;
    if (inc.shape.kind === 'Circle' || inc.shape.kind === 'Rectangle') {
      this.store.updateInclusion(inc.id, {
        shape: { ...inc.shape, center_x: cx, center_y: cy },
      });
    }
  }

  updateSelectedInclusionRectDims(w: number, h: number): void {
    const inc = this.selectedInclusion;
    if (!inc || inc.shape.kind !== 'Rectangle') return;
    this.store.updateInclusion(inc.id, {
      shape: { ...inc.shape, width: Math.max(0.1, w), height: Math.max(0.1, h) },
    });
  }

  updateSelectedInclusionMaterial(label: string, eGpa: number, nu: number): void {
    const inc = this.selectedInclusion;
    if (!inc) return;
    this.store.updateInclusion(inc.id, {
      material_label: label,
      youngs_modulus_override_gpa: eGpa,
      poisson_ratio_override: nu,
    });
  }
}
