import { describe, it, expect, beforeEach } from 'vitest';
import { DraftingViewport } from './viewport';
import { SnappingEngine, DEFAULT_SNAPPING_OPTIONS } from './snapping-engine';
import { TopologicalSplitService } from './topological-split';
import { CadTransformationService, UndoRedoStack } from './cad-transforms';
import {
  StructuralModel2D,
  Node2D,
  FrameMemberElement,
  Material2D,
  Section2D,
} from './modeler.types';

describe('Mode 2 Modeler Core Engine', () => {
  let defaultModel: StructuralModel2D;

  const defaultMat: Material2D = {
    id: 'mat1',
    name: 'C25/30',
    source: { kind: 'Preset', catalog_id: 'C25_30' },
    class: 'Concrete',
    e_gpa: 31.0,
    g_gpa: 12.9,
    density_gamma: 25.0,
    alpha_t: 1e-5,
  };

  const defaultSec: Section2D = {
    id: 'sec1',
    name: 'R 30x50',
    source: { kind: 'Custom' },
    shape: 'Rectangular',
    area_a: 0.15,
    inertia_i: 0.003125,
  };

  beforeEach(() => {
    defaultModel = {
      id: 'model1',
      name: 'Test Model',
      revision: 1,
      gravity_g: 9.81,
      nodes: [],
      materials: [defaultMat],
      sections: [defaultSec],
      elements: [],
      supports: [],
      load_cases: [
        {
          id: 'LC1',
          name: 'Dead',
          action_type: { kind: 'Permanent' },
          point_loads: [],
          dist_loads: [],
          nodal_loads: [],
          thermal_loads: [],
          settlements: [],
        },
      ],
      lumped_masses: [],
    };
  });

  describe('SC-001: Viewport & Dynamic Grid Coordinate Transform', () => {
    it('accurately transforms screen coordinates to world and back (Y-up convention)', () => {
      const vp = new DraftingViewport(1200, 800, 50, 400, 500, 1.0);

      // Origin (0,0) world maps to screen (400, 500)
      const screenOrigin = vp.worldToScreen(0, 0);
      expect(screenOrigin.x).toBe(400);
      expect(screenOrigin.y).toBe(500);

      // Screen (400, 500) maps back to world (0, 0)
      const worldOrigin = vp.screenToWorld(400, 500);
      expect(worldOrigin.x).toBeCloseTo(0, 6);
      expect(worldOrigin.y).toBeCloseTo(0, 6);

      // World point (4.0, 3.0): X moves right (+200px), Y moves up (-150px)
      const screenPt = vp.worldToScreen(4.0, 3.0);
      expect(screenPt.x).toBe(600);
      expect(screenPt.y).toBe(350);

      const roundTrip = vp.screenToWorld(screenPt.x, screenPt.y);
      expect(roundTrip.x).toBeCloseTo(4.0, 6);
      expect(roundTrip.y).toBeCloseTo(3.0, 6);
    });

    it('pans and zooms keeping anchor point invariant', () => {
      const vp = new DraftingViewport(1200, 800, 50, 400, 500, 1.0);
      const anchorScreen = { x: 600, y: 350 };
      const worldBefore = vp.screenToWorld(anchorScreen.x, anchorScreen.y);

      vp.zoomAt(anchorScreen.x, anchorScreen.y, 1.5);
      const worldAfter = vp.screenToWorld(anchorScreen.x, anchorScreen.y);

      expect(worldAfter.x).toBeCloseTo(worldBefore.x, 6);
      expect(worldAfter.y).toBeCloseTo(worldBefore.y, 6);
    });
  });

  describe('SC-002: Priority-Ordered Snapping, Ortho Lock & Exact Numeric Entry', () => {
    it('enforces NodeSnap priority over a closer grid point (< 15 px)', () => {
      // Scale: 50 px/m, origin at screen (400, 500)
      const vp = new DraftingViewport(1200, 800, 50, 400, 500, 1.0);
      // Place node at (2.0, 0.0) -> screen (500, 500)
      const nodes: Node2D[] = [{ id: 'N1', x: 2.0, y: 0.0 }];
      const elements: any[] = [];

      // Cursor at screen (508, 500) -> distance to N1 is 8 px (< 15 px)
      // Corresponding world is 2.16m, which is closer to grid point (2.0) anyway,
      // but let's test cursor at world (1.92, 0.0) -> screen (496, 500), dist to N1 is 4 px.
      // Grid point at (1.0, 0.0) is far, but nearest grid is (2.0, 0.0).
      // Now test with gridStep = 0.1 m:
      vp.gridStep = 0.1;
      // World cursor at (1.97, 0.0) -> screen (498.5, 500). Nearest grid is (2.0, 0.0).
      // Let's test node at non-grid point: (2.03, 0.0) -> screen (501.5, 500).
      const offGridNodes: Node2D[] = [{ id: 'N1', x: 2.03, y: 0.0 }];
      // Cursor at screen (500, 500) corresponds to world (2.00, 0.0) -> exact grid point!
      // Distance from cursor (500, 500) to node screen (501.5, 500) is only 1.5 px (< 15 px).
      // NodeSnap MUST win over GridSnap even though cursor is exactly on the grid!
      const snap = SnappingEngine.evaluateSnap(
        { x: 500, y: 500 },
        vp,
        offGridNodes,
        elements,
        null,
        DEFAULT_SNAPPING_OPTIONS
      );

      expect(snap.type).toBe('Node');
      expect(snap.point.x).toBe(2.03);
      expect(snap.referenceId).toBe('N1');
    });

    it('snaps to member midpoint when near midpoint', () => {
      const vp = new DraftingViewport(1200, 800, 50, 400, 500, 1.0);
      const nodes: Node2D[] = [
        { id: 'N1', x: 0.0, y: 0.0 },
        { id: 'N2', x: 6.0, y: 0.0 },
      ];
      const elements: any[] = [
        { id: 'M1', node_i: 'N1', node_j: 'N2', kind: 'FrameMember' },
      ];

      // Midpoint is at (3.0, 0.0) -> screen (550, 500)
      // Cursor near screen (553, 502) -> dist ~3.6 px (< 12 px)
      const snap = SnappingEngine.evaluateSnap(
        { x: 553, y: 502 },
        vp,
        nodes,
        elements,
        null,
        DEFAULT_SNAPPING_OPTIONS
      );

      expect(snap.type).toBe('Midpoint');
      expect(snap.point.x).toBe(3.0);
      expect(snap.point.y).toBe(0.0);
    });

    it('locks to Ortho angles (0, 90, 180, 270) when near cardinal directions', () => {
      const vp = new DraftingViewport(1200, 800, 50, 400, 500, 1.0);
      const basePoint = { x: 0.0, y: 0.0 };

      // Cursor at world (4.0, 0.2) -> angle ~2.86 deg (< 5 deg threshold)
      const screenCur = vp.worldToScreen(4.0, 0.2);
      const snap = SnappingEngine.evaluateSnap(
        screenCur,
        vp,
        [],
        [],
        basePoint,
        { ...DEFAULT_SNAPPING_OPTIONS, orthoLockEnabled: false }
      );

      expect(snap.type).toBe('Ortho');
      expect(snap.point.y).toBeCloseTo(0.0, 5);
      expect(snap.guideAngleDeg).toBe(0);
    });

    it('parses absolute, relative and polar keyboard input', () => {
      const base = { x: 2.0, y: 1.0 };

      // Absolute "6.0, 4.5"
      const absPt = SnappingEngine.parseCoordinateInput('6.0, 4.5');
      expect(absPt).toEqual({ x: 6.0, y: 4.5 });

      // Relative "@3.0, -1.0"
      const relPt = SnappingEngine.parseCoordinateInput('@3.0, -1.0', base);
      expect(relPt).toEqual({ x: 5.0, y: 0.0 });

      // Polar "@4.0<90"
      const polPt = SnappingEngine.parseCoordinateInput('@4.0<90', base);
      expect(polPt!.x).toBeCloseTo(2.0, 5);
      expect(polPt!.y).toBeCloseTo(5.0, 5);
    });

    it('rejects zero-length element with ZERO_LENGTH_ELEMENT', () => {
      expect(() => {
        SnappingEngine.validateElementLength({ x: 1.0, y: 2.0 }, { x: 1.0, y: 2.0 });
      }).toThrow('ZERO_LENGTH_ELEMENT');
    });
  });

  describe('SC-003: Member-Member Intersection with Automatic Topological Split', () => {
    it('automatically splits an existing member when a new perpendicular member crosses it', () => {
      // 1. Initial horizontal beam from (0, 0) to (6, 0)
      const { model: m1 } = TopologicalSplitService.addMemberWithAutoSplit(
        defaultModel,
        { x: 0.0, y: 0.0 },
        { x: 6.0, y: 0.0 },
        'sec1',
        'mat1'
      );
      expect(m1.nodes.length).toBe(2);
      expect(m1.elements.length).toBe(1);

      // Add a distributed load to the horizontal member
      m1.load_cases[0].dist_loads.push({
        id: 'DL1',
        element_id: m1.elements[0].id,
        qx1: 0,
        qy1: -10,
        qx2: 0,
        qy2: -10,
        a_rel: 0,
        b_rel: 1,
      });

      // 2. Draw vertical column from (3, -2) to (3, 4) crossing the beam at (3, 0)
      const { model: m2, createdNodeIds, createdElementIds } =
        TopologicalSplitService.addMemberWithAutoSplit(
          m1,
          { x: 3.0, y: -2.0 },
          { x: 3.0, y: 4.0 },
          'sec1',
          'mat1'
        );

      // Intersection node at (3, 0) must be created
      const midNode = m2.nodes.find(n => Math.abs(n.x - 3.0) < 1e-4 && Math.abs(n.y - 0.0) < 1e-4);
      expect(midNode).toBeDefined();

      // Original beam is split into 2 sub-elements; vertical member is split into 2 sub-elements
      // Total 4 elements in model
      expect(m2.elements.length).toBe(4);

      // All 4 elements meet at midNode
      const connected = m2.elements.filter(
        e => 'node_i' in e && 'node_j' in e && (e.node_i === midNode!.id || e.node_j === midNode!.id)
      );
      expect(connected.length).toBe(4);

      // Distributed load DL1 was partitioned to the sub-elements
      expect(m2.load_cases[0].dist_loads.length).toBe(2);
      expect(m2.revision).toBe(3);
    });
  });

  describe('SC-004: Marquee Selection, CAD Transforms & Monotonic Undo/Redo', () => {
    let frameModel: StructuralModel2D;

    beforeEach(() => {
      // Create a 2-node, 1-member frame from (0, 0) to (4, 0)
      const { model } = TopologicalSplitService.addMemberWithAutoSplit(
        defaultModel,
        { x: 0.0, y: 0.0 },
        { x: 4.0, y: 0.0 },
        'sec1',
        'mat1'
      );
      frameModel = model;
    });

    it('differentiates between Window (enclosed) and Crossing marquee box selection', () => {
      // Window selection (left-to-right from -1 to 2): only Node (0,0) is enclosed, not (4,0)
      const windowSel = CadTransformationService.evaluateBoxSelection(
        frameModel,
        { x: -1.0, y: -1.0 },
        { x: 2.0, y: 2.0 }
      );
      expect(windowSel.nodeIds.size).toBe(1);
      // Element is NOT selected in window selection because (4,0) is outside!
      expect(windowSel.elementIds.size).toBe(0);

      // Crossing selection (right-to-left from 2 to -1): intersects member segment!
      const crossingSel = CadTransformationService.evaluateBoxSelection(
        frameModel,
        { x: 2.0, y: -1.0 },
        { x: -1.0, y: 2.0 }
      );
      expect(crossingSel.nodeIds.size).toBe(1);
      // Element IS selected in crossing selection!
      expect(crossingSel.elementIds.size).toBe(1);
    });

    it('executes Move transform translating coordinates', () => {
      const allNodeIds = new Set(frameModel.nodes.map(n => n.id));
      const moved = CadTransformationService.moveSelection(
        frameModel,
        allNodeIds,
        new Set([frameModel.elements[0].id]),
        2.5,
        1.5
      );

      const n0 = moved.nodes.find(n => n.id === frameModel.nodes[0].id)!;
      const n1 = moved.nodes.find(n => n.id === frameModel.nodes[1].id)!;

      expect(n0.x).toBe(2.5);
      expect(n0.y).toBe(1.5);
      expect(n1.x).toBe(6.5);
      expect(n1.y).toBe(1.5);
      expect(moved.revision).toBe(frameModel.revision + 1);
    });

    it('executes Copy transform with coincident node welding', () => {
      const allNodeIds = new Set(frameModel.nodes.map(n => n.id));
      // Copy by dx = 4.0 -> start node of copy lands exactly on (4.0, 0.0), coincident with N2!
      const copied = CadTransformationService.copySelection(
        frameModel,
        allNodeIds,
        new Set([frameModel.elements[0].id]),
        4.0,
        0.0,
        1e-4
      );

      // Total nodes should be 3 (N1 at 0, N2 at 4 welded, new N3 at 8)
      expect(copied.nodes.length).toBe(3);
      expect(copied.elements.length).toBe(2);
      expect(copied.revision).toBe(frameModel.revision + 1);
    });

    it('executes Mirror transform across a 2-point axis and rejects degenerate axis', () => {
      const allNodeIds = new Set(frameModel.nodes.map(n => n.id));
      // Mirror across vertical axis x = 2 (p1=(2, 0), p2=(2, 5))
      const mirrored = CadTransformationService.mirrorSelection(
        frameModel,
        allNodeIds,
        new Set([frameModel.elements[0].id]),
        { x: 2.0, y: 0.0 },
        { x: 2.0, y: 5.0 },
        false // in-place mirror
      );

      // (0, 0) reflected across x=2 becomes (4, 0), (4, 0) becomes (0, 0)
      const n0 = mirrored.nodes.find(n => n.id === frameModel.nodes[0].id)!;
      expect(n0.x).toBe(4.0);

      // Degenerate axis rejection
      expect(() => {
        CadTransformationService.mirrorSelection(
          frameModel,
          allNodeIds,
          new Set(),
          { x: 1.0, y: 1.0 },
          { x: 1.0, y: 1.0 }
        );
      }).toThrow('DEGENERATE_MIRROR_AXIS');
    });

    it('executes Cascade Delete removing incident elements when node is deleted', () => {
      const firstNodeId = frameModel.nodes[0].id;
      const deleted = CadTransformationService.deleteSelection(
        frameModel,
        new Set([firstNodeId]),
        new Set()
      );

      expect(deleted.nodes.length).toBe(1);
      // Element incident to firstNodeId is deleted
      expect(deleted.elements.length).toBe(0);
      expect(deleted.revision).toBe(frameModel.revision + 1);
    });

    it('ensures Undo and Redo strictly increment model.revision monotonically', () => {
      const stack = new UndoRedoStack();

      const r1 = frameModel.revision; // 2
      stack.pushState(frameModel);

      // Mutate
      const allNodeIds = new Set(frameModel.nodes.map(n => n.id));
      const m2 = CadTransformationService.moveSelection(
        frameModel,
        allNodeIds,
        new Set(),
        1.0,
        1.0
      );
      const r2 = m2.revision; // 3
      expect(r2).toBeGreaterThan(r1);

      // Undo: restores previous geometry but revision MUST be > r2!
      const mUndo = stack.undo(m2)!;
      expect(mUndo).toBeDefined();
      expect(mUndo.nodes[0].x).toBe(0.0); // restored original position
      expect(mUndo.revision).toBe(r2 + 1); // 4 > 3! Monotonic invariant!

      // Redo: restores mutated geometry and revision increments again!
      const mRedo = stack.redo(mUndo)!;
      expect(mRedo).toBeDefined();
      expect(mRedo.nodes[0].x).toBe(1.0); // restored moved position
      expect(mRedo.revision).toBe(mUndo.revision + 1); // 5 > 4!
    });
  });
});
