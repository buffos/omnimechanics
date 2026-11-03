import { describe, it, expect, beforeEach, vi } from 'vitest';
import { TestBed } from '@angular/core/testing';
import { ContinuumSketcherStore } from './continuum-sketcher.store';
import { MeshService, MeshData } from '../../services/mesh.service';
import { ConformalMeshHandoff } from './continuum-sketcher.types';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (cmd: string, args: any) => {
    if (cmd === 'continuum_preview_segment') {
      return {
        is_valid: true,
        intersecting_segment_indices: [],
        snap_closure_ready: false,
      };
    }
    if (cmd === 'continuum_close_and_normalize_loop') {
      return {
        id: args.request.loop_id,
        orientation: 'CounterClockwise',
        closure_state: 'ClosedValid',
        vertices: args.request.vertices,
        segments: args.request.vertices.map((v: any, i: number, arr: any[]) => ({
          id: `s${i}`,
          start_vertex_id: v.id,
          end_vertex_id: arr[(i + 1) % arr.length].id,
          tag: `Edge_${i + 1}`,
          outward_normal: [0, -1, 0],
        })),
      };
    }
    if (cmd === 'continuum_validate_domain') {
      return {
        is_valid_for_meshing: true,
        diagnostics: [],
      };
    }
    if (cmd === 'continuum_generate_mesh') {
      const dom = args.request?.domain ?? args.domain;
      const mockHandoff: ConformalMeshHandoff = {
        domain_id: dom.id,
        domain_revision: dom.revision,
        mesh_data: {
          dimension: 2,
          nodes: [
            [0, 0, 0],
            [10, 0, 0],
            [10, 10, 0],
            [0, 10, 0],
            [5, 5, 0],
          ],
          boundary_elements: [
            { id: 0, elem_type: 'Line2', nodes: [0, 1], tag: 'Bottom', normal: [0, -1, 0] },
            { id: 1, elem_type: 'Line2', nodes: [1, 2], tag: 'Right', normal: [1, 0, 0] },
            { id: 2, elem_type: 'Line2', nodes: [2, 3], tag: 'Top', normal: [0, 1, 0] },
            { id: 3, elem_type: 'Line2', nodes: [3, 0], tag: 'Left', normal: [-1, 0, 0] },
          ],
          volume_elements: [
            { id: 0, elem_type: 'Tri3', nodes: [0, 1, 4] },
            { id: 1, elem_type: 'Tri3', nodes: [1, 2, 4] },
            { id: 2, elem_type: 'Tri3', nodes: [2, 3, 4] },
            { id: 3, elem_type: 'Tri3', nodes: [3, 0, 4] },
          ],
          stats: {
            dimension: 2,
            num_nodes: 5,
            num_boundary_elements: 4,
            num_volume_elements: 4,
          },
        },
        quality_report: {
          num_nodes: 5,
          num_boundary_elements: 4,
          num_volume_elements: 4,
          min_angle_deg: 35.0,
          max_angle_deg: 90.0,
          min_aspect_ratio: 1.0,
          max_aspect_ratio: 1.4,
          is_conformal: true,
          outward_normals_verified: true,
        },
        tag_element_counts: { Bottom: 1, Right: 1, Top: 1, Left: 1 },
      };
      return mockHandoff;
    }
    return null;
  }),
}));

describe('Mode 1 Interactive 2D Continuum Sketcher UI (Issue 029)', () => {
  let store: ContinuumSketcherStore;
  let meshService: MeshService;

  beforeEach(() => {
    meshService = new MeshService();
    store = new ContinuumSketcherStore(meshService);
  });

  it('SC-001: loads default perforated plate template with 10x10 outer loop and central hole', () => {
    store.loadTemplatePerforatedPlate();
    const domain = store.domain();

    expect(domain.id).toBe('CD1');
    expect(domain.outer_boundary.closure_state).toBe('ClosedValid');
    expect(domain.outer_boundary.vertices.length).toBe(4);
    expect(domain.cutouts.length).toBe(1);

    const hole = domain.cutouts[0];
    expect(hole.tag).toBe('Hole');
    expect(hole.shape.kind).toBe('Circle');
    if (hole.shape.kind === 'Circle') {
      expect(hole.shape.radius).toBe(1.5);
      expect(hole.shape.center_x).toBe(5.0);
      expect(hole.shape.center_y).toBe(5.0);
    }

    expect(store.meshStatus()).toBe('Unmeshed');
  });

  it('SC-006: updates segment semantic tag and boundary conditions in store', () => {
    store.loadTemplatePerforatedPlate();
    expect(store.domain().outer_boundary.segments[0].tag).toBe('Bottom');

    // Update segment 0 to 'Custom_Fixed' with Dirichlet condition
    store.updateSegmentTag(0, 'Custom_Fixed', {
      kind: 'DirichletDisplacement',
      fix_x: true,
      fix_y: true,
      ux_m: 0.0,
      uy_m: 0.0,
    });

    const updated = store.domain();
    expect(updated.outer_boundary.segments[0].tag).toBe('Custom_Fixed');
    expect(updated.tag_conditions['Custom_Fixed']).toBeDefined();
    expect(updated.tag_conditions['Custom_Fixed'].kind).toBe('DirichletDisplacement');
  });

  it('SC-007: adds and removes inclusion regions without mutating outer loop', () => {
    store.loadTemplatePerforatedPlate();
    expect(store.domain().inclusions.length).toBe(0);

    store.addCircularInclusion(2.5, 2.5, 0.8);
    expect(store.domain().inclusions.length).toBe(1);

    const inc = store.domain().inclusions[0];
    expect(inc.material_label).toBe('Steel_Inclusion');
    expect(inc.youngs_modulus_override_gpa).toBe(210.0);

    store.removeInclusion(inc.id);
    expect(store.domain().inclusions.length).toBe(0);
  });

  it('SC-008 & SC-009: generates conformal mesh and enforces stale mesh handoff protection', async () => {
    store.loadTemplatePerforatedPlate();
    expect(store.meshStatus()).toBe('Unmeshed');

    // Generate mesh
    const handoff = await store.generateMesh();
    expect(handoff).not.toBeNull();
    expect(store.meshHandoff()).toBe(handoff);
    expect(store.meshStatus()).toBe('Current');

    // Hand off to Mode 1 Solver
    const sent = store.sendToMode1Solver();
    expect(sent).toBe(true);
    expect(meshService.currentMesh()).toEqual(handoff!.mesh_data);

    // Now mutate domain (e.g. add a cutout), revision bumps
    store.addCircularCutout(8.0, 8.0, 0.5);
    expect(store.meshStatus()).toBe('Stale');

    // Stale mesh handoff must be rejected with error
    const sendStale = store.sendToMode1Solver();
    expect(sendStale).toBe(false);
    expect(store.errorMessage()).toContain('Current mesh is stale');
  });

  it('SC-002: resetDomain clears boundary and allows vertex-by-vertex drafting', async () => {
    store.resetDomain();
    expect(store.domain().outer_boundary.closure_state).toBe('OpenDraft');
    expect(store.domain().outer_boundary.vertices.length).toBe(0);

    await store.addVertex(0, 0);
    await store.addVertex(5, 0);
    await store.addVertex(5, 5);
    expect(store.domain().outer_boundary.vertices.length).toBe(3);

    // Closing outer loop
    await store.closeOuterLoop();
    expect(store.domain().outer_boundary.closure_state).toBe('ClosedValid');
  });

  it('supports trapezoidal/triangular loads and local normal-tangential frame', () => {
    store.loadTemplatePerforatedPlate();
    // Assign trapezoidal local traction on top boundary
    store.updateSegmentTag(2, 'Top', {
      kind: 'NeumannTraction',
      tx_kn_m2: 0,
      ty_kn_m2: 100,
      tx_end_kn_m2: 0,
      ty_end_kn_m2: 250,
      coordinate_frame: 'LocalNormalTangential',
    });

    const cond = store.domain().tag_conditions['Top'];
    expect(cond.kind).toBe('NeumannTraction');
    if (cond.kind === 'NeumannTraction') {
      expect(cond.coordinate_frame).toBe('LocalNormalTangential');
      expect(cond.ty_kn_m2).toBe(100);
      expect(cond.ty_end_kn_m2).toBe(250);
    }
  });

  it('supports configuring Quad4 volume element meshing profile', () => {
    store.loadTemplatePerforatedPlate();
    expect(store.domain().meshing_profile.volume_element_type).toBe('Tri3');

    store.updateMeshingProfile({ volume_element_type: 'Quad4' });
    expect(store.domain().meshing_profile.volume_element_type).toBe('Quad4');
  });

  it('SC-003 & SC-004: formats structured Tauri CanonicalError rejections and supports Undo/Redo', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    store.loadTemplatePerforatedPlate();
    const initialCutouts = store.domain().cutouts.length;

    store.addCircularCutout(9.5, 5.0, 1.5);
    expect(store.domain().cutouts.length).toBe(initialCutouts + 1);
    expect(store.canUndo()).toBe(true);

    // Undo and Redo
    store.undo();
    expect(store.domain().cutouts.length).toBe(initialCutouts);
    expect(store.canRedo()).toBe(true);
    store.redo();
    expect(store.domain().cutouts.length).toBe(initialCutouts + 1);

    // Simulate Rust CanonicalError rejection on validateDomain
    vi.mocked(invoke).mockRejectedValueOnce({
      code: 'HOLE_OUTSIDE_OUTER_BOUNDARY',
      message_el: "Η οπή 'H1' εξέρχεται του εξωτερικού περιγράμματος.",
      message_en: "Hole 'H1' lies outside the outer boundary.",
    });

    const valRes = await store.validateDomain();
    expect(valRes.is_valid_for_meshing).toBe(false);
    expect(valRes.diagnostics[0].code).toBe('HOLE_OUTSIDE_OUTER_BOUNDARY');
    expect(store.errorMessage()).toContain('[HOLE_OUTSIDE_OUTER_BOUNDARY]');
  });
});
