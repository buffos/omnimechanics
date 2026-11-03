import { describe, it, expect, beforeEach } from 'vitest';
import { ModelerStore } from '../core/modeler.store';
import { simulateFallbackSolve, toBackendStructuralModel } from '../results/modeler-results.service';

describe('Issue 024: Modeler Inspector, Palettes & Live DSM Results', () => {
  let store: ModelerStore;

  beforeEach(() => {
    store = new ModelerStore();
  });

  describe('SC-005: Property Inspector, Internal Hinge Expansion & Cable Specs', () => {
    it('applies internal hinge expansion policy on joint with k >= 2 members, releasing k-1 members', () => {
      // Create a 3-member star joint around N1 (0,0)
      store.sketchMember({ x: 0, y: 0 }, { x: 5, y: 0 }); // M1: N1 -> N2
      store.sketchMember({ x: 0, y: 0 }, { x: 0, y: 4 }); // M2: N1 -> N3
      store.sketchMember({ x: 0, y: 0 }, { x: -4, y: 0 }); // M3: N1 -> N4

      const initialModel = store.model();
      const node1 = initialModel.nodes.find(n => n.x === 0 && n.y === 0)!;
      expect(node1).toBeDefined();

      // Apply internal hinge
      store.applyInternalHinge(node1.id);
      const updatedModel = store.model();

      const incidentMembers = updatedModel.elements.filter(
        e => e.kind === 'FrameMember' && (e.node_i === node1.id || e.node_j === node1.id)
      ) as any[];

      expect(incidentMembers.length).toBe(3);
      // Check moment releases: exactly 2 (k-1) should have moment released at node1, exactly 1 continuous
      const releasedCount = incidentMembers.filter(m =>
        m.node_i === node1.id ? m.release_i.moment : m.release_j.moment
      ).length;
      expect(releasedCount).toBe(2);

      const continuousCount = incidentMembers.filter(m =>
        m.node_i === node1.id ? !m.release_i.moment : !m.release_j.moment
      ).length;
      expect(continuousCount).toBe(1);
    });

    it('rejects internal hinge on joint with k < 2 with INVALID_HINGE_TARGET', () => {
      // Single cantilever member: end node has k=1
      store.sketchMember({ x: 0, y: 0 }, { x: 4, y: 0 });
      const tipNode = store.model().nodes.find(n => n.x === 4 && n.y === 0)!;

      expect(() => {
        store.applyInternalHinge(tipNode.id);
      }).toThrow('INVALID_HINGE_TARGET');
    });

    it('rejects invalid cable prestress values (<= 0) with INVALID_CABLE_SPEC', () => {
      store.sketchMember({ x: 0, y: 0 }, { x: 6, y: 2 });
      const elem = store.model().elements[0];

      // Convert to Cable with invalid pretension
      expect(() => {
        store.updateElementProperties(elem.id, {
          kind: 'Cable',
          prestress: { mode: 'Pretension', t0: -5.0 },
        } as any);
      }).toThrow('INVALID_CABLE_SPEC');

      expect(() => {
        store.updateElementProperties(elem.id, {
          kind: 'Cable',
          prestress: { mode: 'UnstressedLength', l0: 0.0 },
        } as any);
      }).toThrow('INVALID_CABLE_SPEC');
    });
  });

  describe('SC-006: Support & Load Palettes Without Spurious Member Splitting', () => {
    it('assigns Fixed, Pinned, Roller and Elastic supports to nodes', () => {
      store.sketchMember({ x: 0, y: 0 }, { x: 6, y: 0 });
      const n1 = store.model().nodes[0];
      const n2 = store.model().nodes[1];

      // Assign Fixed to n1
      store.assignSupport(n1.id, { type: 'Fixed' });
      expect(store.model().supports.length).toBe(1);
      expect(store.model().supports[0].kind.type).toBe('Fixed');

      // Assign Roller with 30 deg to n2
      store.assignSupport(n2.id, { type: 'Roller', angle_deg: 30 });
      expect(store.model().supports.length).toBe(2);
      expect(store.model().supports[1].kind).toEqual({ type: 'Roller', angle_deg: 30 });

      // Removing support
      store.removeSupport(n1.id);
      expect(store.model().supports.length).toBe(1);
      expect(store.model().supports[0].node_id).toBe(n2.id);
    });

    it('places interior point load along member without splitting element, and rejects out-of-bounds distances', () => {
      store.sketchMember({ x: 0, y: 0 }, { x: 5, y: 0 }); // Length = 5.0m
      const elem = store.model().elements[0];
      const initialElemCount = store.model().elements.length;

      // Add point load at distance 2.5m
      store.addMemberPointLoad(elem.id, 0, -25.0, 2.5);

      // Verify element count DID NOT CHANGE (no split)
      expect(store.model().elements.length).toBe(initialElemCount);
      const lc = store.model().load_cases[0];
      expect(lc.point_loads.length).toBe(1);
      expect(lc.point_loads[0].py).toBe(-25.0);
      expect(lc.point_loads[0].a_rel).toBeCloseTo(0.5, 4);

      // Rejects out of bounds a = 7.0m (> 5.0m)
      expect(() => {
        store.addMemberPointLoad(elem.id, 0, -10.0, 7.0);
      }).toThrow('LOAD_OUT_OF_MEMBER_BOUNDS');

      // Rejects out of bounds a = -1.0m (< 0)
      expect(() => {
        store.addMemberPointLoad(elem.id, 0, -10.0, -1.0);
      }).toThrow('LOAD_OUT_OF_MEMBER_BOUNDS');
    });

    it('places distributed loads, nodal loads, settlements and thermal loads', () => {
      store.sketchMember({ x: 0, y: 0 }, { x: 6, y: 0 });
      const elem = store.model().elements[0];
      const n1 = store.model().nodes[0];

      store.addMemberDistributedLoad(elem.id, 0, -15.0, 0, -15.0, 0.0, 1.0);
      store.addNodalLoad(n1.id, 10.0, 0.0, -5.0);
      store.addSupportSettlement(n1.id, 0, -0.02, 0);
      store.addThermalLoad(elem.id, 25.0, 10.0);
      store.toggleSelfWeight(true);

      const lc = store.model().load_cases[0];
      expect(lc.dist_loads.length).toBe(1);
      expect(lc.nodal_loads.length).toBe(1);
      expect(lc.settlements.length).toBe(1);
      expect(lc.thermal_loads.length).toBe(1);
      expect((lc as any).self_weight).toBe(true);

      // Backend conversion produces valid structure
      const backend = toBackendStructuralModel(store.model());
      expect(backend.load_cases[0].loads.length).toBe(4);
      expect(backend.load_cases[0].self_weight).toBe(true);
    });
  });

  describe('SC-007 & SC-009: Live Results, Hover Probe & Mechanism Detection', () => {
    it('solves simple beam and reports stable diagnostics and station moments', () => {
      store.sketchMember({ x: 0, y: 0 }, { x: 6, y: 0 });
      const n1 = store.model().nodes[0];
      const n2 = store.model().nodes[1];
      store.assignSupport(n1.id, { type: 'Pinned' });
      store.assignSupport(n2.id, { type: 'Roller', angle_deg: 0 });

      const resp = simulateFallbackSolve(store.model());
      expect(resp.diagnostics.is_stable).toBe(true);
      expect(resp.result_sets.length).toBe(1);
      const mr = resp.result_sets[0].member_results[0];
      expect(mr).toBeDefined();
      expect(mr.stations.length).toBeGreaterThan(5);
      // Eurocode positive moment at midspan
      expect(mr.extrema.max_m.value).toBeGreaterThan(0);
    });

    it('detects kinematic mechanism when structure has insufficient supports and returns mechanism modes', () => {
      store.sketchMember({ x: 0, y: 0 }, { x: 6, y: 0 });
      // Only 1 pin support on a 6m beam -> rotation mechanism
      const n1 = store.model().nodes[0];
      store.assignSupport(n1.id, { type: 'Pinned' });

      const resp = simulateFallbackSolve(store.model());
      expect(resp.diagnostics.is_stable).toBe(false);
      expect(resp.diagnostics.kinematic_instability_degree).toBeGreaterThanOrEqual(1);
      expect(resp.diagnostics.mechanism_modes.length).toBeGreaterThan(0);
      expect(resp.result_sets.length).toBe(0); // Suppressed force results
    });
  });
});
