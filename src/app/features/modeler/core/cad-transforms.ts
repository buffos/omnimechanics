import {
  StructuralModel2D,
  Node2D,
  Element2D,
  FrameMemberElement,
  Support2D,
  SelectionState,
  isLineElement,
} from './modeler.types';
import { computeSegmentIntersection } from './snapping-engine';

export class CadTransformationService {
  /**
   * Marquee Box Selection:
   * - Left-to-right (box.x1 <= box.x2): Enclosed Window (only entities completely inside)
   * - Right-to-left (box.x1 > box.x2): Crossing (entities inside or intersecting boundary)
   */
  static evaluateBoxSelection(
    model: StructuralModel2D,
    boxStartWorld: { x: number; y: number },
    boxEndWorld: { x: number; y: number }
  ): SelectionState {
    const isCrossing = boxEndWorld.x < boxStartWorld.x;

    const minX = Math.min(boxStartWorld.x, boxEndWorld.x);
    const maxX = Math.max(boxStartWorld.x, boxEndWorld.x);
    const minY = Math.min(boxStartWorld.y, boxEndWorld.y);
    const maxY = Math.max(boxStartWorld.y, boxEndWorld.y);

    const selectedNodes = new Set<string>();
    const selectedElements = new Set<string>();

    const nodeMap = new Map<string, Node2D>();
    for (const n of model.nodes) {
      nodeMap.set(n.id, n);
      const inside = n.x >= minX && n.x <= maxX && n.y >= minY && n.y <= maxY;
      if (inside) {
        selectedNodes.add(n.id);
      }
    }

    // Box boundary segments for crossing detection
    const bTopLeft = { x: minX, y: maxY };
    const bTopRight = { x: maxX, y: maxY };
    const bBottomLeft = { x: minX, y: minY };
    const bBottomRight = { x: maxX, y: minY };

    const boxSegments = [
      [bBottomLeft, bBottomRight],
      [bBottomRight, bTopRight],
      [bTopRight, bTopLeft],
      [bTopLeft, bBottomLeft],
    ];

    for (const elem of model.elements) {
      if (!isLineElement(elem)) continue;
      const ni = nodeMap.get(elem.node_i);
      const nj = nodeMap.get(elem.node_j);
      if (!ni || !nj) continue;

      const iInside = selectedNodes.has(elem.node_i);
      const jInside = selectedNodes.has(elem.node_j);

      if (!isCrossing) {
        // Enclosed Window: BOTH endpoints must be inside
        if (iInside && jInside) {
          selectedElements.add(elem.id);
        }
      } else {
        // Crossing: either endpoint inside OR element intersects any box boundary
        if (iInside || jInside) {
          selectedElements.add(elem.id);
        } else {
          // Check segment intersection with box 4 edges
          let intersects = false;
          for (const [p1, p2] of boxSegments) {
            if (computeSegmentIntersection(ni, nj, p1, p2)) {
              intersects = true;
              break;
            }
          }
          if (intersects) {
            selectedElements.add(elem.id);
          }
        }
      }
    }

    return {
      nodeIds: selectedNodes,
      elementIds: selectedElements,
    };
  }

  /**
   * Moves selected nodes and incident elements by (dx, dy)
   */
  static moveSelection(
    model: StructuralModel2D,
    nodeIds: Set<string>,
    elementIds: Set<string>,
    dx: number,
    dy: number
  ): StructuralModel2D {
    if (nodeIds.size === 0 && elementIds.size === 0) {
      return model;
    }

    const effectiveNodeIds = new Set<string>(nodeIds);
    // Include endpoints of all explicitly selected elements
    for (const elem of model.elements) {
      if (elementIds.has(elem.id) && isLineElement(elem)) {
        effectiveNodeIds.add(elem.node_i);
        effectiveNodeIds.add(elem.node_j);
      }
    }

    const nextRev = model.revision + 1;
    const nodes = model.nodes.map(n => {
      if (effectiveNodeIds.has(n.id)) {
        return {
          ...n,
          x: Math.round((n.x + dx) * 10000) / 10000,
          y: Math.round((n.y + dy) * 10000) / 10000,
        };
      }
      return n;
    });

    return {
      ...model,
      revision: nextRev,
      nodes,
    };
  }

  /**
   * Copies selected nodes and elements with translation (dx, dy) and welds coincident nodes
   */
  static copySelection(
    model: StructuralModel2D,
    nodeIds: Set<string>,
    elementIds: Set<string>,
    dx: number,
    dy: number,
    weldTol = 1e-4
  ): StructuralModel2D {
    if (nodeIds.size === 0 && elementIds.size === 0) {
      return model;
    }

    const effectiveNodeIds = new Set<string>(nodeIds);
    for (const elem of model.elements) {
      if (elementIds.has(elem.id) && isLineElement(elem)) {
        effectiveNodeIds.add(elem.node_i);
        effectiveNodeIds.add(elem.node_j);
      }
    }

    const nextRev = model.revision + 1;
    const nodes = [...model.nodes];
    const elements = [...model.elements];
    const oldToNewNodeMap = new Map<string, string>();

    // Copy nodes
    for (const oldNodeId of effectiveNodeIds) {
      const oldNode = model.nodes.find(n => n.id === oldNodeId);
      if (!oldNode) continue;

      const targetX = Math.round((oldNode.x + dx) * 10000) / 10000;
      const targetY = Math.round((oldNode.y + dy) * 10000) / 10000;

      // Check weld with existing nodes
      const existingCoincident = nodes.find(
        n => Math.hypot(n.x - targetX, n.y - targetY) <= weldTol
      );

      if (existingCoincident) {
        oldToNewNodeMap.set(oldNodeId, existingCoincident.id);
      } else {
        const newId = `N${nodes.length + 1}`;
        nodes.push({ id: newId, x: targetX, y: targetY });
        oldToNewNodeMap.set(oldNodeId, newId);
      }
    }

    // Copy elements
    for (const elem of model.elements) {
      if (!elementIds.has(elem.id)) continue;
      if (!isLineElement(elem)) continue;

      const newNi = oldToNewNodeMap.get(elem.node_i) ?? elem.node_i;
      const newNj = oldToNewNodeMap.get(elem.node_j) ?? elem.node_j;

      if (newNi === newNj) continue; // Degenerate

      const newElemId = `M${elements.length + 1}`;
      elements.push({
        ...elem,
        id: newElemId,
        node_i: newNi,
        node_j: newNj,
      });
    }

    return {
      ...model,
      revision: nextRev,
      nodes,
      elements,
    };
  }

  /**
   * Mirrors selected entities across axis defined by p1 and p2
   */
  static mirrorSelection(
    model: StructuralModel2D,
    nodeIds: Set<string>,
    elementIds: Set<string>,
    p1: { x: number; y: number },
    p2: { x: number; y: number },
    copy = true,
    weldTol = 1e-4
  ): StructuralModel2D {
    const axisLen = Math.hypot(p2.x - p1.x, p2.y - p1.y);
    if (axisLen < 1e-5) {
      throw new Error('DEGENERATE_MIRROR_AXIS');
    }

    const effectiveNodeIds = new Set<string>(nodeIds);
    for (const elem of model.elements) {
      if (elementIds.has(elem.id) && isLineElement(elem)) {
        effectiveNodeIds.add(elem.node_i);
        effectiveNodeIds.add(elem.node_j);
      }
    }

    const ux = (p2.x - p1.x) / axisLen;
    const uy = (p2.y - p1.y) / axisLen;

    const mirrorPoint = (p: { x: number; y: number }) => {
      // Vector from p1 to p
      const vx = p.x - p1.x;
      const vy = p.y - p1.y;
      // Projection onto unit axis
      const dot = vx * ux + vy * uy;
      const projX = p1.x + dot * ux;
      const projY = p1.y + dot * uy;
      // Reflected point
      return {
        x: Math.round((2 * projX - p.x) * 10000) / 10000,
        y: Math.round((2 * projY - p.y) * 10000) / 10000,
      };
    };

    const nextRev = model.revision + 1;

    if (!copy) {
      // In-place mirror
      const nodes = model.nodes.map(n => {
        if (effectiveNodeIds.has(n.id)) {
          const mp = mirrorPoint(n);
          return { ...n, x: mp.x, y: mp.y };
        }
        return n;
      });
      return { ...model, revision: nextRev, nodes };
    }

    // Copy-mirror
    const nodes = [...model.nodes];
    const elements = [...model.elements];
    const oldToNewNodeMap = new Map<string, string>();

    for (const oldNodeId of effectiveNodeIds) {
      const oldNode = model.nodes.find(n => n.id === oldNodeId);
      if (!oldNode) continue;

      const mp = mirrorPoint(oldNode);

      const existingCoincident = nodes.find(
        n => Math.hypot(n.x - mp.x, n.y - mp.y) <= weldTol
      );

      if (existingCoincident) {
        oldToNewNodeMap.set(oldNodeId, existingCoincident.id);
      } else {
        const newId = `N${nodes.length + 1}`;
        nodes.push({ id: newId, x: mp.x, y: mp.y });
        oldToNewNodeMap.set(oldNodeId, newId);
      }
    }

    for (const elem of model.elements) {
      if (!elementIds.has(elem.id)) continue;
      if (!isLineElement(elem)) continue;

      const newNi = oldToNewNodeMap.get(elem.node_i) ?? elem.node_i;
      const newNj = oldToNewNodeMap.get(elem.node_j) ?? elem.node_j;
      if (newNi === newNj) continue;

      const newElemId = `M${elements.length + 1}`;
      elements.push({
        ...elem,
        id: newElemId,
        node_i: newNi,
        node_j: newNj,
      });
    }

    return {
      ...model,
      revision: nextRev,
      nodes,
      elements,
    };
  }

  /**
   * Linear Array repeating selection `count` times along vector (dx, dy)
   */
  static linearArraySelection(
    model: StructuralModel2D,
    nodeIds: Set<string>,
    elementIds: Set<string>,
    dx: number,
    dy: number,
    count: number,
    weldTol = 1e-4
  ): StructuralModel2D {
    let currentModel = model;
    for (let c = 1; c < count; c++) {
      currentModel = this.copySelection(
        currentModel,
        nodeIds,
        elementIds,
        dx * c,
        dy * c,
        weldTol
      );
    }
    return currentModel;
  }

  /**
   * Cascade Delete:
   * - Deleting a node also removes incident elements and attached supports
   * - Deleting an element removes it from model
   */
  static deleteSelection(
    model: StructuralModel2D,
    nodeIds: Set<string>,
    elementIds: Set<string>
  ): StructuralModel2D {
    const nextRev = model.revision + 1;

    // Elements to delete: explicitly selected OR incident to any deleted node
    const finalElemDeleteIds = new Set<string>(elementIds);
    for (const elem of model.elements) {
      if (nodeIds.has(elem.node_i) || (elem.node_j && nodeIds.has(elem.node_j))) {
        finalElemDeleteIds.add(elem.id);
      }
    }

    const remainingNodes = model.nodes.filter(n => !nodeIds.has(n.id));
    const remainingElements = model.elements.filter(e => !finalElemDeleteIds.has(e.id));
    const remainingSupports = model.supports.filter(s => !nodeIds.has(s.node_id));

    // Also remove loads attached to deleted elements or nodes
    const remainingLoadCases = model.load_cases.map(lc => ({
      ...lc,
      point_loads: lc.point_loads.filter(p => !finalElemDeleteIds.has(p.element_id)),
      dist_loads: lc.dist_loads.filter(d => !finalElemDeleteIds.has(d.element_id)),
      nodal_loads: lc.nodal_loads.filter(n => !nodeIds.has(n.node_id)),
      thermal_loads: lc.thermal_loads.filter(t => !finalElemDeleteIds.has(t.element_id)),
      settlements: lc.settlements.filter(s => !nodeIds.has(s.node_id)),
    }));

    return {
      ...model,
      revision: nextRev,
      nodes: remainingNodes,
      elements: remainingElements,
      supports: remainingSupports,
      load_cases: remainingLoadCases,
    };
  }
}

/**
 * UndoRedoStack manager ensuring monotonic revision increment
 */
export class UndoRedoStack {
  private history: StructuralModel2D[] = [];
  private future: StructuralModel2D[] = [];
  private maxDepth = 50;

  pushState(model: StructuralModel2D): void {
    this.history.push(JSON.parse(JSON.stringify(model)));
    if (this.history.length > this.maxDepth) {
      this.history.shift();
    }
    this.future = [];
  }

  canUndo(): boolean {
    return this.history.length > 0;
  }

  canRedo(): boolean {
    return this.future.length > 0;
  }

  undo(currentModel: StructuralModel2D): StructuralModel2D | null {
    if (this.history.length === 0) return null;
    const previous = this.history.pop()!;
    this.future.push(JSON.parse(JSON.stringify(currentModel)));

    // Strict invariant: revision MUST strictly increment even on undo!
    return {
      ...previous,
      revision: currentModel.revision + 1,
    };
  }

  redo(currentModel: StructuralModel2D): StructuralModel2D | null {
    if (this.future.length === 0) return null;
    const next = this.future.pop()!;
    this.history.push(JSON.parse(JSON.stringify(currentModel)));

    // Strict invariant: revision MUST strictly increment even on redo!
    return {
      ...next,
      revision: currentModel.revision + 1,
    };
  }

  clear(): void {
    this.history = [];
    this.future = [];
  }
}
