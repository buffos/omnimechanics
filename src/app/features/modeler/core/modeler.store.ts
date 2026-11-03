import { Injectable, computed, signal } from '@angular/core';
import {
  StructuralModel2D,
  Node2D,
  ActiveTool,
  SelectionState,
  Material2D,
  Section2D,
  Element2D,
  FrameMemberElement,
  CableElement,
  Support2D,
  SupportKind,
  isLineElement,
  PointLoad2D,
  DistributedLoad2D,
  NodalLoad2D,
  Settlement2D,
  ThermalLoad2D,
} from './modeler.types';
import { CombinationDto, SeismicSettingsDto } from '../combinations/combinations.types';

export const DEFAULT_SEISMIC_SETTINGS: SeismicSettingsDto = {
  zone: 'Zone II (0.24g)',
  ag_g: 0.24,
  ground_type: 'B',
  importance_factor: 1.0,
  behavior_factor_q: 1.5,
  spectrum_type: 'Type1',
  direction: 'Horizontal',
};
import { DraftingViewport } from './viewport';
import { DEFAULT_SNAPPING_OPTIONS, SnappingOptions } from './snapping-engine';
import { TopologicalSplitService } from './topological-split';
import { CadTransformationService, UndoRedoStack } from './cad-transforms';

function createDefaultModel(): StructuralModel2D {
  const defaultMat: Material2D = {
    id: 'mat_c25_30',
    name: 'C25/30',
    source: { kind: 'Preset', catalog_id: 'C25_30' },
    class: 'Concrete',
    e_gpa: 31.0,
    g_gpa: 12.9,
    density_gamma: 25.0,
    alpha_t: 1e-5,
  };

  const defaultSec: Section2D = {
    id: 'sec_rect_30x50',
    name: 'R 30x50',
    source: { kind: 'Custom' },
    shape: 'Rectangular',
    area_a: 0.15,
    inertia_i: 0.003125,
    shear_area_av: 0.125,
    depth_h: 0.50,
  };

  return {
    id: 'model_1',
    name: '2D Frame Model',
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
        name: 'G (Dead Load)',
        action_type: { kind: 'Permanent' },
        self_weight: true,
        point_loads: [],
        dist_loads: [],
        nodal_loads: [],
        thermal_loads: [],
        settlements: [],
      },
    ],
    lumped_masses: [],
  };
}

@Injectable({
  providedIn: 'root',
})
export class ModelerStore {
  private undoRedo = new UndoRedoStack();

  readonly model = signal<StructuralModel2D>(createDefaultModel());
  readonly activeTool = signal<ActiveTool>('Select');
  readonly selection = signal<SelectionState>({
    nodeIds: new Set<string>(),
    elementIds: new Set<string>(),
  });
  readonly snappingOptions = signal<SnappingOptions>({ ...DEFAULT_SNAPPING_OPTIONS });
  readonly viewport = signal<DraftingViewport>(new DraftingViewport());
  readonly activeLoadCaseId = signal<string>('LC1');
  readonly currentCursorWorld = signal<{ x: number; y: number }>({ x: 0, y: 0 });

  // Eurocode combinations & seismic settings
  readonly combinations = signal<CombinationDto[]>([]);
  readonly seismicSettings = signal<SeismicSettingsDto>({ ...DEFAULT_SEISMIC_SETTINGS });
  readonly isCombinationModalOpen = signal<boolean>(false);
  readonly selectedSupportKind = signal<SupportKind>({ type: 'Fixed' });
  readonly isLoadsModalOpen = signal<boolean>(false);
  readonly isTransformModalOpen = signal<boolean>(false);
  readonly activeTransformType = signal<'move' | 'copy' | 'mirror' | 'array'>('move');
  readonly drawStartPoint = signal<{ x: number; y: number } | null>(null);

  // Undo/Redo reactivity version trigger
  private readonly historyVersion = signal<number>(0);

  readonly canUndo = computed(() => {
    this.historyVersion();
    return this.undoRedo.canUndo();
  });

  readonly canRedo = computed(() => {
    this.historyVersion();
    return this.undoRedo.canRedo();
  });

  readonly selectedElementsCount = computed(() => {
    const s = this.selection();
    return s.nodeIds.size + s.elementIds.size;
  });

  /**
   * Sets the active drawing or manipulation tool
   */
  setTool(tool: ActiveTool): void {
    this.activeTool.set(tool);
    this.drawStartPoint.set(null);
    if (tool !== 'Select' && tool !== 'Move') {
      this.clearSelection();
    }
  }

  /**
   * Clears selection
   */
  clearSelection(): void {
    this.selection.set({
      nodeIds: new Set<string>(),
      elementIds: new Set<string>(),
    });
  }

  /**
   * Selects entities via marquee box
   */
  selectBox(startWorld: { x: number; y: number }, endWorld: { x: number; y: number }, append = false): void {
    const boxSel = CadTransformationService.evaluateBoxSelection(this.model(), startWorld, endWorld);
    if (!append) {
      this.selection.set(boxSel);
    } else {
      const current = this.selection();
      const combinedNodes = new Set([...current.nodeIds, ...boxSel.nodeIds]);
      const combinedElements = new Set([...current.elementIds, ...boxSel.elementIds]);
      this.selection.set({ nodeIds: combinedNodes, elementIds: combinedElements });
    }
  }

  /**
   * Toggles selection of a specific entity
   */
  toggleEntitySelection(type: 'node' | 'element', id: string, append = false): void {
    const current = this.selection();
    const nodeIds = append ? new Set(current.nodeIds) : new Set<string>();
    const elementIds = append ? new Set(current.elementIds) : new Set<string>();

    if (type === 'node') {
      if (nodeIds.has(id)) nodeIds.delete(id);
      else nodeIds.add(id);
    } else {
      if (elementIds.has(id)) elementIds.delete(id);
      else elementIds.add(id);
    }

    this.selection.set({ nodeIds, elementIds });
  }

  /**
   * Sets grid step in meters
   */
  setGridStep(step: number): void {
    const vp = this.viewport();
    vp.gridStep = step;
    this.viewport.set(new DraftingViewport(vp.width, vp.height, vp.scale, vp.panX, vp.panY, step));
  }

  /**
   * Toggles Orthogonal Lock mode
   */
  toggleOrtho(): void {
    this.snappingOptions.update(opts => ({
      ...opts,
      orthoLockEnabled: !opts.orthoLockEnabled,
    }));
  }

  /**
   * Sketches a new member between pA and pB with auto-splitting
   */
  sketchMember(
    pA: { x: number; y: number },
    pB: { x: number; y: number },
    sectionId?: string,
    materialId?: string
  ): void {
    const curModel = this.model();
    const secId = sectionId ?? curModel.sections[0]?.id ?? 'sec_default';
    const matId = materialId ?? curModel.materials[0]?.id ?? 'mat_default';

    this.undoRedo.pushState(curModel);
    const { model: updatedModel } = TopologicalSplitService.addMemberWithAutoSplit(
      curModel,
      pA,
      pB,
      secId,
      matId
    );

    this.model.set(updatedModel);
    this.historyVersion.update(v => v + 1);
  }

  /**
   * Adds or returns an existing node at (x, y)
   */
  addNode(x: number, y: number, tolerance = 1e-4): Node2D {
    const curModel = this.model();
    const existing = curModel.nodes.find(n => Math.hypot(n.x - x, n.y - y) <= tolerance);
    if (existing) {
      return existing;
    }

    this.undoRedo.pushState(curModel);
    const nextRev = curModel.revision + 1;
    const newId = `N${curModel.nodes.length + 1}`;
    const newNode: Node2D = { id: newId, x, y };

    this.model.set({
      ...curModel,
      revision: nextRev,
      nodes: [...curModel.nodes, newNode],
    });
    this.historyVersion.update(v => v + 1);
    return newNode;
  }

  /**
   * Applies Move CAD transform to current selection
   */
  moveSelection(dx: number, dy: number): void {
    const sel = this.selection();
    if (sel.nodeIds.size === 0 && sel.elementIds.size === 0) return;

    const curModel = this.model();
    this.undoRedo.pushState(curModel);
    const updated = CadTransformationService.moveSelection(curModel, sel.nodeIds, sel.elementIds, dx, dy);
    this.model.set(updated);
    this.historyVersion.update(v => v + 1);
  }

  /**
   * Applies Copy CAD transform to current selection
   */
  copySelection(dx: number, dy: number): void {
    const sel = this.selection();
    if (sel.nodeIds.size === 0 && sel.elementIds.size === 0) return;

    const curModel = this.model();
    this.undoRedo.pushState(curModel);
    const updated = CadTransformationService.copySelection(curModel, sel.nodeIds, sel.elementIds, dx, dy);
    this.model.set(updated);
    this.historyVersion.update(v => v + 1);
  }

  /**
   * Applies Mirror CAD transform to current selection
   */
  mirrorSelection(p1: { x: number; y: number }, p2: { x: number; y: number }, copy = true): void {
    const sel = this.selection();
    if (sel.nodeIds.size === 0 && sel.elementIds.size === 0) return;

    const curModel = this.model();
    this.undoRedo.pushState(curModel);
    const updated = CadTransformationService.mirrorSelection(curModel, sel.nodeIds, sel.elementIds, p1, p2, copy);
    this.model.set(updated);
    this.historyVersion.update(v => v + 1);
  }

  /**
   * Applies Linear Array CAD transform to current selection
   */
  linearArraySelection(dx: number, dy: number, count: number): void {
    const sel = this.selection();
    if (sel.nodeIds.size === 0 && sel.elementIds.size === 0) return;

    const curModel = this.model();
    this.undoRedo.pushState(curModel);
    const updated = CadTransformationService.linearArraySelection(curModel, sel.nodeIds, sel.elementIds, dx, dy, count);
    this.model.set(updated);
    this.historyVersion.update(v => v + 1);
  }

  /**
   * Deletes current selection
   */
  deleteSelection(): void {
    const sel = this.selection();
    if (sel.nodeIds.size === 0 && sel.elementIds.size === 0) return;

    const curModel = this.model();
    this.undoRedo.pushState(curModel);
    const updated = CadTransformationService.deleteSelection(curModel, sel.nodeIds, sel.elementIds);
    this.model.set(updated);
    this.clearSelection();
    this.historyVersion.update(v => v + 1);
  }

  /**
   * Undoes the last mutation, strictly incrementing revision
   */
  undo(): void {
    const curModel = this.model();
    const prev = this.undoRedo.undo(curModel);
    if (prev) {
      this.model.set(prev);
      this.clearSelection();
      this.historyVersion.update(v => v + 1);
    }
  }

  /**
   * Redoes the undone mutation, strictly incrementing revision
   */
  redo(): void {
    const curModel = this.model();
    const next = this.undoRedo.redo(curModel);
    if (next) {
      this.model.set(next);
      this.clearSelection();
      this.historyVersion.update(v => v + 1);
    }
  }

  /**
   * Replaces entire model state (e.g. from preset import or backend sync)
   */
  loadModel(newModel: StructuralModel2D): void {
    this.undoRedo.pushState(this.model());
    this.model.set(newModel);
    this.clearSelection();
    this.historyVersion.update(v => v + 1);
  }

  /**
   * Applies Internal Hinge Expansion Policy to a shared node:
   * Sets moment release on k - 1 incident members, keeping 1 continuous.
   * Rejects if k < 2 with INVALID_HINGE_TARGET.
   */
  applyInternalHinge(nodeId: string): void {
    const curModel = this.model();
    const incident = curModel.elements.filter(
      (e): e is FrameMemberElement =>
        e.kind === 'FrameMember' && (e.node_i === nodeId || e.node_j === nodeId)
    );

    if (incident.length < 2) {
      throw new Error('INVALID_HINGE_TARGET');
    }

    this.undoRedo.pushState(curModel);
    const nextRev = curModel.revision + 1;
    // Release moment on first k - 1 elements, keep the last one continuous
    const toReleaseIds = new Set(incident.slice(0, incident.length - 1).map(e => e.id));

    const updatedElements = curModel.elements.map(e => {
      if (!toReleaseIds.has(e.id)) return e;
      const f = e as FrameMemberElement;
      return {
        ...f,
        release_i: f.node_i === nodeId ? { ...f.release_i, moment: true } : f.release_i,
        release_j: f.node_j === nodeId ? { ...f.release_j, moment: true } : f.release_j,
      };
    });

    this.model.set({
      ...curModel,
      revision: nextRev,
      elements: updatedElements,
    });
    this.historyVersion.update(v => v + 1);
  }

  /**
   * Updates properties of an element (section, material, releases, offsets, cable spec)
   */
  updateElementProperties(elementId: string, updates: Partial<Element2D>): void {
    const curModel = this.model();
    const idx = curModel.elements.findIndex(e => e.id === elementId);
    if (idx === -1) return;

    const elem = curModel.elements[idx];
    const updated = { ...elem, ...updates } as Element2D;

    // Validate Cable prestress if applicable
    if (updated.kind === 'Cable') {
      const ps = updated.prestress;
      if (ps.mode === 'UnstressedLength' && ps.l0 <= 0) {
        throw new Error('INVALID_CABLE_SPEC');
      }
      if (ps.mode === 'SagRatio' && ps.sag_to_span <= 0) {
        throw new Error('INVALID_CABLE_SPEC');
      }
      if (ps.mode === 'Pretension' && ps.t0 <= 0) {
        throw new Error('INVALID_CABLE_SPEC');
      }
    }

    this.undoRedo.pushState(curModel);
    const nextRev = curModel.revision + 1;
    const elements = [...curModel.elements];
    elements[idx] = updated;

    this.model.set({
      ...curModel,
      revision: nextRev,
      elements,
    });
    this.historyVersion.update(v => v + 1);
  }

  /**
   * Updates coordinates of a node
   */
  updateNodePosition(nodeId: string, x: number, y: number): void {
    const curModel = this.model();
    const idx = curModel.nodes.findIndex(n => n.id === nodeId);
    if (idx === -1) return;

    this.undoRedo.pushState(curModel);
    const nextRev = curModel.revision + 1;
    const nodes = [...curModel.nodes];
    nodes[idx] = { ...nodes[idx], x, y };

    this.model.set({
      ...curModel,
      revision: nextRev,
      nodes,
    });
    this.historyVersion.update(v => v + 1);
  }

  /**
   * Assigns a support to a node
   */
  assignSupport(nodeId: string, kind: SupportKind, unilateral = false): void {
    const curModel = this.model();
    this.undoRedo.pushState(curModel);
    const nextRev = curModel.revision + 1;

    // Replace existing support on this node or add new
    const filtered = curModel.supports.filter(s => s.node_id !== nodeId);
    const newSupport: Support2D = {
      id: `sup_${nodeId}`,
      node_id: nodeId,
      kind,
      unilateral,
    };

    this.model.set({
      ...curModel,
      revision: nextRev,
      supports: [...filtered, newSupport],
    });
    this.historyVersion.update(v => v + 1);
  }

  /**
   * Removes any support from a node
   */
  removeSupport(nodeId: string): void {
    const curModel = this.model();
    if (!curModel.supports.some(s => s.node_id === nodeId)) return;

    this.undoRedo.pushState(curModel);
    const nextRev = curModel.revision + 1;
    this.model.set({
      ...curModel,
      revision: nextRev,
      supports: curModel.supports.filter(s => s.node_id !== nodeId),
    });
    this.historyVersion.update(v => v + 1);
  }

  /**
   * Assigns a point load to an element at distance a (m) without splitting
   */
  addMemberPointLoad(elementId: string, px: number, py: number, a_m: number): void {
    const curModel = this.model();
    const elem = curModel.elements.find(e => e.id === elementId);
    if (!elem || !isLineElement(elem)) return;

    const n1 = curModel.nodes.find(n => n.id === elem.node_i);
    const n2 = curModel.nodes.find(n => n.id === elem.node_j);
    if (!n1 || !n2) return;

    const length = Math.hypot(n2.x - n1.x, n2.y - n1.y);
    if (a_m < -1e-4 || a_m > length + 1e-4) {
      throw new Error('LOAD_OUT_OF_MEMBER_BOUNDS');
    }

    const a_rel = length > 1e-5 ? Math.max(0, Math.min(1, a_m / length)) : 0;
    this.undoRedo.pushState(curModel);
    const nextRev = curModel.revision + 1;

    const loadCases = curModel.load_cases.map(lc => {
      if (lc.id !== this.activeLoadCaseId()) return lc;
      const newLoad: PointLoad2D = {
        id: `PL${lc.point_loads.length + 1}`,
        element_id: elementId,
        px,
        py,
        a_rel,
      };
      return { ...lc, point_loads: [...lc.point_loads, newLoad] };
    });

    this.model.set({
      ...curModel,
      revision: nextRev,
      load_cases: loadCases,
    });
    this.historyVersion.update(v => v + 1);
  }

  /**
   * Assigns a distributed load to an element
   */
  addMemberDistributedLoad(
    elementId: string,
    qx1: number,
    qy1: number,
    qx2: number,
    qy2: number,
    a_rel = 0,
    b_rel = 1
  ): void {
    const curModel = this.model();
    this.undoRedo.pushState(curModel);
    const nextRev = curModel.revision + 1;

    const loadCases = curModel.load_cases.map(lc => {
      if (lc.id !== this.activeLoadCaseId()) return lc;
      const newLoad: DistributedLoad2D = {
        id: `DL${lc.dist_loads.length + 1}`,
        element_id: elementId,
        qx1,
        qy1,
        qx2,
        qy2,
        a_rel,
        b_rel,
      };
      return { ...lc, dist_loads: [...lc.dist_loads, newLoad] };
    });

    this.model.set({
      ...curModel,
      revision: nextRev,
      load_cases: loadCases,
    });
    this.historyVersion.update(v => v + 1);
  }

  /**
   * Assigns a nodal load
   */
  addNodalLoad(nodeId: string, fx: number, fy: number, mz: number): void {
    const curModel = this.model();
    this.undoRedo.pushState(curModel);
    const nextRev = curModel.revision + 1;

    const loadCases = curModel.load_cases.map(lc => {
      if (lc.id !== this.activeLoadCaseId()) return lc;
      const newLoad: NodalLoad2D = {
        id: `NL${lc.nodal_loads.length + 1}`,
        node_id: nodeId,
        fx,
        fy,
        mz,
      };
      return { ...lc, nodal_loads: [...lc.nodal_loads, newLoad] };
    });

    this.model.set({
      ...curModel,
      revision: nextRev,
      load_cases: loadCases,
    });
    this.historyVersion.update(v => v + 1);
  }

  /**
   * Assigns a support settlement
   */
  addSupportSettlement(nodeId: string, delta_x: number, delta_y: number, theta_z: number): void {
    const curModel = this.model();
    this.undoRedo.pushState(curModel);
    const nextRev = curModel.revision + 1;

    const loadCases = curModel.load_cases.map(lc => {
      if (lc.id !== this.activeLoadCaseId()) return lc;
      const newLoad: Settlement2D = {
        id: `SET${lc.settlements.length + 1}`,
        node_id: nodeId,
        delta_x,
        delta_y,
        theta_z,
      };
      return { ...lc, settlements: [...lc.settlements, newLoad] };
    });

    this.model.set({
      ...curModel,
      revision: nextRev,
      load_cases: loadCases,
    });
    this.historyVersion.update(v => v + 1);
  }

  /**
   * Assigns a thermal load to an element
   */
  addThermalLoad(elementId: string, delta_t_top: number, delta_t_bottom: number): void {
    const curModel = this.model();
    this.undoRedo.pushState(curModel);
    const nextRev = curModel.revision + 1;

    const loadCases = curModel.load_cases.map(lc => {
      if (lc.id !== this.activeLoadCaseId()) return lc;
      const newLoad: ThermalLoad2D = {
        id: `TH${lc.thermal_loads.length + 1}`,
        element_id: elementId,
        delta_t_top,
        delta_t_bottom,
      };
      return { ...lc, thermal_loads: [...lc.thermal_loads, newLoad] };
    });

    this.model.set({
      ...curModel,
      revision: nextRev,
      load_cases: loadCases,
    });
    this.historyVersion.update(v => v + 1);
  }

  /**
   * Toggles self weight on active load case
   */
  toggleSelfWeight(enabled?: boolean): void {
    const curModel = this.model();
    this.undoRedo.pushState(curModel);
    const nextRev = curModel.revision + 1;

    const loadCases = curModel.load_cases.map(lc => {
      if (lc.id !== this.activeLoadCaseId()) return lc;
      const cur = (lc as any).self_weight ?? false;
      return { ...lc, self_weight: enabled ?? !cur };
    });

    this.model.set({
      ...curModel,
      revision: nextRev,
      load_cases: loadCases,
    });
    this.historyVersion.update(v => v + 1);
  }

  /**
   * Adds a new LoadCase to the model
   */
  addLoadCase(name: string, action_type: any): void {
    const curModel = this.model();
    const newId = `LC${curModel.load_cases.length + 1}`;
    const nextRev = curModel.revision + 1;
    this.undoRedo.pushState(curModel);
    this.model.set({
      ...curModel,
      revision: nextRev,
      load_cases: [
        ...curModel.load_cases,
        {
          id: newId,
          name,
          action_type,
          point_loads: [],
          dist_loads: [],
          nodal_loads: [],
          thermal_loads: [],
          settlements: [],
        },
      ],
    });
    this.activeLoadCaseId.set(newId);
    this.historyVersion.update(v => v + 1);
  }

  /**
   * Deletes a LoadCase from the model (keeping at least 1)
   */
  deleteLoadCase(id: string): void {
    const curModel = this.model();
    if (curModel.load_cases.length <= 1) return;
    const nextRev = curModel.revision + 1;
    this.undoRedo.pushState(curModel);
    const filtered = curModel.load_cases.filter(lc => lc.id !== id);
    this.model.set({
      ...curModel,
      revision: nextRev,
      load_cases: filtered,
    });
    if (this.activeLoadCaseId() === id) {
      this.activeLoadCaseId.set(filtered[0].id);
    }
    this.historyVersion.update(v => v + 1);
  }

  /**
   * Updates Eurocode combinations list
   */
  setCombinations(combs: CombinationDto[]): void {
    this.combinations.set(combs);
  }

  /**
   * Deletes a combination by ID
   */
  deleteCombination(id: string): void {
    this.combinations.update(combs => combs.filter(c => c.id !== id));
  }

  /**
   * Updates an existing combination (setting provenance to UserEdited)
   */
  updateCombination(updated: CombinationDto): void {
    this.combinations.update(combs =>
      combs.map(c => (c.id === updated.id ? { ...updated, provenance: 'UserEdited' } : c))
    );
  }

  /**
   * Sets seismic settings
   */
  setSeismicSettings(settings: SeismicSettingsDto): void {
    this.seismicSettings.set(settings);
  }
}
