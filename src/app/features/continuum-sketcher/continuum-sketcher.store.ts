import { Injectable, computed, inject, signal } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { MeshService } from '../../services/mesh.service';
import {
  BoundaryConditionSpec,
  BoundaryLoop,
  CandidateSegmentPreview,
  ConformalMeshHandoff,
  ContinuumDomain,
  Cutout,
  DomainValidationResponse,
  InclusionRegion,
  MeshingProfile,
  SketcherTool,
  Vertex2D,
} from './continuum-sketcher.types';

function createDefaultMeshingProfile(): MeshingProfile {
  return {
    target_element_size: 0.8,
    hole_refinement_factor: 0.35,
    min_angle_deg: 20.0,
    volume_element_type: 'Tri3',
    boundary_element_type: 'Line2',
  };
}

function createDefaultTemplateDomain(): ContinuumDomain {
  return {
    id: 'CD1',
    revision: 1,
    outer_boundary: {
      id: 'OuterLoop',
      orientation: 'CounterClockwise',
      closure_state: 'ClosedValid',
      vertices: [
        { id: 'v0', x: 0.0, y: 0.0 },
        { id: 'v1', x: 10.0, y: 0.0 },
        { id: 'v2', x: 10.0, y: 10.0 },
        { id: 'v3', x: 0.0, y: 10.0 },
      ],
      segments: [
        {
          id: 's0',
          start_vertex_id: 'v0',
          end_vertex_id: 'v1',
          tag: 'Bottom',
          outward_normal: [0.0, -1.0, 0.0],
        },
        {
          id: 's1',
          start_vertex_id: 'v1',
          end_vertex_id: 'v2',
          tag: 'Right',
          outward_normal: [1.0, 0.0, 0.0],
        },
        {
          id: 's2',
          start_vertex_id: 'v2',
          end_vertex_id: 'v3',
          tag: 'Top',
          outward_normal: [0.0, 1.0, 0.0],
        },
        {
          id: 's3',
          start_vertex_id: 'v3',
          end_vertex_id: 'v0',
          tag: 'Left',
          outward_normal: [-1.0, 0.0, 0.0],
        },
      ],
    },
    cutouts: [
      {
        id: 'cutout_center_hole',
        shape: {
          kind: 'Circle',
          center_x: 5.0,
          center_y: 5.0,
          radius: 1.5,
          angular_divisions: 24,
        },
        tag: 'Hole',
        local_element_size: undefined,
      },
    ],
    inclusions: [],
    tag_conditions: {
      Bottom: {
        kind: 'DirichletDisplacement',
        fix_x: true,
        fix_y: true,
        ux_m: 0.0,
        uy_m: 0.0,
      },
      Top: {
        kind: 'NeumannTraction',
        tx_kn_m2: 0.0,
        ty_kn_m2: 100.0,
        coordinate_frame: 'Global',
      },
      Left: { kind: 'Free' },
      Right: { kind: 'Free' },
      Hole: { kind: 'Free' },
    },
    meshing_profile: createDefaultMeshingProfile(),
  };
}

@Injectable({
  providedIn: 'root',
})
export class ContinuumSketcherStore {
  private readonly meshService: MeshService;

  constructor(meshService?: MeshService) {
    if (meshService) {
      this.meshService = meshService;
    } else {
      try {
        this.meshService = inject(MeshService);
      } catch {
        this.meshService = new MeshService();
      }
    }
  }

  readonly domain = signal<ContinuumDomain>(createDefaultTemplateDomain());
  readonly activeTool = signal<SketcherTool>('Select');
  readonly selectedSegmentIdx = signal<number | null>(null);
  readonly selectedCutoutId = signal<string | null>(null);
  readonly selectedInclusionId = signal<string | null>(null);

  readonly candidatePoint = signal<Vertex2D | null>(null);
  readonly candidatePreview = signal<CandidateSegmentPreview | null>(null);

  readonly meshHandoff = signal<ConformalMeshHandoff | null>(null);
  readonly validationResponse = signal<DomainValidationResponse | null>(null);

  readonly isMeshing = signal<boolean>(false);
  readonly errorMessage = signal<string | null>(null);

  readonly meshStatus = computed<'Unmeshed' | 'Current' | 'Stale'>(() => {
    const handoff = this.meshHandoff();
    if (!handoff) return 'Unmeshed';
    return handoff.domain_revision === this.domain().revision ? 'Current' : 'Stale';
  });

  loadTemplatePerforatedPlate(): void {
    this.domain.set(createDefaultTemplateDomain());
    this.meshHandoff.set(null);
    this.validationResponse.set(null);
    this.selectedSegmentIdx.set(null);
    this.selectedCutoutId.set(null);
    this.selectedInclusionId.set(null);
    this.errorMessage.set(null);
  }

  resetDomain(): void {
    const emptyDomain: ContinuumDomain = {
      id: `CD_${Date.now()}`,
      revision: 1,
      outer_boundary: {
        id: 'OuterLoop',
        orientation: 'CounterClockwise',
        closure_state: 'OpenDraft',
        vertices: [],
        segments: [],
      },
      cutouts: [],
      inclusions: [],
      tag_conditions: {},
      meshing_profile: createDefaultMeshingProfile(),
    };
    this.domain.set(emptyDomain);
    this.meshHandoff.set(null);
    this.validationResponse.set(null);
    this.selectedSegmentIdx.set(null);
    this.selectedCutoutId.set(null);
    this.selectedInclusionId.set(null);
    this.errorMessage.set(null);
  }

  setTool(tool: SketcherTool): void {
    this.activeTool.set(tool);
    this.candidatePoint.set(null);
    this.candidatePreview.set(null);
  }

  async previewCandidate(candidate: Vertex2D, closingLoop = false): Promise<void> {
    this.candidatePoint.set(candidate);
    const verts = this.domain().outer_boundary.vertices;
    if (verts.length === 0) {
      this.candidatePreview.set({
        is_valid: true,
        intersecting_segment_indices: [],
        snap_closure_ready: false,
      });
      return;
    }

    try {
      const preview = await invoke<CandidateSegmentPreview>('continuum_preview_segment', {
        request: {
          existing_vertices: verts,
          candidate_point: candidate,
          closing_loop: closingLoop,
        },
      });
      this.candidatePreview.set(preview);
    } catch {
      this.candidatePreview.set({
        is_valid: true,
        intersecting_segment_indices: [],
        snap_closure_ready: false,
      });
    }
  }

  async addVertex(x: number, y: number): Promise<void> {
    const cur = this.domain();
    if (cur.outer_boundary.closure_state === 'ClosedValid') {
      return;
    }

    const verts = [...cur.outer_boundary.vertices];
    // Snap check: if clicking near first vertex and verts.length >= 3, close loop
    if (verts.length >= 3) {
      const v0 = verts[0];
      const dist = Math.hypot(x - v0.x, y - v0.y);
      if (dist < 0.4) {
        await this.closeOuterLoop();
        return;
      }
    }

    const newVert: Vertex2D = {
      id: `v${verts.length}`,
      x,
      y,
    };
    verts.push(newVert);

    this.domain.update((d) => ({
      ...d,
      revision: d.revision + 1,
      outer_boundary: {
        ...d.outer_boundary,
        vertices: verts,
      },
    }));
  }

  async closeOuterLoop(): Promise<void> {
    const cur = this.domain();
    const verts = cur.outer_boundary.vertices;
    if (verts.length < 3) {
      this.errorMessage.set('At least 3 vertices are required to close the outer boundary.');
      return;
    }

    try {
      const normalizedLoop = await invoke<BoundaryLoop>('continuum_close_and_normalize_loop', {
        request: {
          loop_id: 'OuterLoop',
          vertices: verts,
          is_hole: false,
        },
      });

      this.domain.update((d) => ({
        ...d,
        revision: d.revision + 1,
        outer_boundary: normalizedLoop,
      }));
      this.errorMessage.set(null);
      this.activeTool.set('Select');
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      this.errorMessage.set(`Loop closure rejected: ${msg}`);
    }
  }

  addCircularCutout(centerX: number, centerY: number, radius = 1.0): void {
    const newCutout: Cutout = {
      id: `cutout_${Date.now()}`,
      shape: {
        kind: 'Circle',
        center_x: centerX,
        center_y: centerY,
        radius,
        angular_divisions: 24,
      },
      tag: 'Hole',
    };

    this.domain.update((d) => ({
      ...d,
      revision: d.revision + 1,
      cutouts: [...d.cutouts, newCutout],
    }));
    this.selectedCutoutId.set(newCutout.id);
  }

  addRectangularCutout(centerX: number, centerY: number, width = 2.0, height = 2.0): void {
    const newCutout: Cutout = {
      id: `cutout_${Date.now()}`,
      shape: {
        kind: 'Rectangle',
        center_x: centerX,
        center_y: centerY,
        width,
        height,
        rotation_deg: 0.0,
      },
      tag: 'Hole',
    };

    this.domain.update((d) => ({
      ...d,
      revision: d.revision + 1,
      cutouts: [...d.cutouts, newCutout],
    }));
    this.selectedCutoutId.set(newCutout.id);
  }

  removeCutout(id: string): void {
    this.domain.update((d) => ({
      ...d,
      revision: d.revision + 1,
      cutouts: d.cutouts.filter((c) => c.id !== id),
    }));
    if (this.selectedCutoutId() === id) {
      this.selectedCutoutId.set(null);
    }
  }

  addCircularInclusion(centerX: number, centerY: number, radius = 1.2): void {
    const newInclusion: InclusionRegion = {
      id: `inclusion_${Date.now()}`,
      shape: {
        kind: 'Circle',
        center_x: centerX,
        center_y: centerY,
        radius,
        angular_divisions: 24,
      },
      material_label: 'Steel_Inclusion',
      youngs_modulus_override_gpa: 210.0,
      poisson_ratio_override: 0.3,
    };

    this.domain.update((d) => ({
      ...d,
      revision: d.revision + 1,
      inclusions: [...d.inclusions, newInclusion],
    }));
    this.selectedInclusionId.set(newInclusion.id);
  }

  removeInclusion(id: string): void {
    this.domain.update((d) => ({
      ...d,
      revision: d.revision + 1,
      inclusions: d.inclusions.filter((inc) => inc.id !== id),
    }));
    if (this.selectedInclusionId() === id) {
      this.selectedInclusionId.set(null);
    }
  }

  updateSegmentTag(segmentIdx: number, tag: string, bcSpec: BoundaryConditionSpec): void {
    const cur = this.domain();
    const segs = [...cur.outer_boundary.segments];
    if (segmentIdx < 0 || segmentIdx >= segs.length) return;

    segs[segmentIdx] = {
      ...segs[segmentIdx],
      tag,
    };

    const newTagConditions = { ...cur.tag_conditions, [tag]: bcSpec };

    this.domain.update((d) => ({
      ...d,
      revision: d.revision + 1,
      outer_boundary: {
        ...d.outer_boundary,
        segments: segs,
      },
      tag_conditions: newTagConditions,
    }));
  }

  updateMeshingProfile(profile: Partial<MeshingProfile>): void {
    this.domain.update((d) => ({
      ...d,
      revision: d.revision + 1,
      meshing_profile: {
        ...d.meshing_profile,
        ...profile,
      },
    }));
  }

  async validateDomain(): Promise<DomainValidationResponse> {
    const res = await invoke<DomainValidationResponse>('continuum_validate_domain', {
      request: { domain: this.domain() },
    });
    this.validationResponse.set(res);
    if (!res.is_valid_for_meshing) {
      const msgs = res.diagnostics.map((d) => d.message_en).join('; ');
      this.errorMessage.set(msgs);
    } else {
      this.errorMessage.set(null);
    }
    return res;
  }

  async generateMesh(): Promise<ConformalMeshHandoff | null> {
    this.isMeshing.set(true);
    this.errorMessage.set(null);

    try {
      const validation = await this.validateDomain();
      if (!validation.is_valid_for_meshing) {
        throw new Error(this.errorMessage() || 'Domain geometry validation failed');
      }

      const handoff = await invoke<ConformalMeshHandoff>('continuum_generate_mesh', {
        domain: this.domain(),
      });

      this.meshHandoff.set(handoff);
      return handoff;
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      this.errorMessage.set(`Meshing failed: ${msg}`);
      return null;
    } finally {
      this.isMeshing.set(false);
    }
  }

  sendToMode1Solver(): boolean {
    const handoff = this.meshHandoff();
    if (!handoff) {
      this.errorMessage.set('No generated mesh available. Please generate conformal mesh first.');
      return false;
    }
    if (handoff.domain_revision !== this.domain().revision) {
      this.errorMessage.set('Current mesh is stale. Please regenerate mesh before sending to solver.');
      return false;
    }

    this.meshService.setMesh(handoff.mesh_data);
    return true;
  }
}
