import {
  StructuralModel2D,
  Node2D,
  Element2D,
  FrameMemberElement,
  DistributedLoad2D,
  isLineElement,
} from './modeler.types';
import { computeSegmentIntersection } from './snapping-engine';

export class TopologicalSplitService {
  /**
   * Adds a new member from pA to pB into the model, automatically splitting any intersected
   * existing members, snapping/welding to existing nodes, and splitting the new member if it
   * crosses multiple elements.
   */
  static addMemberWithAutoSplit(
    model: StructuralModel2D,
    pA: { x: number; y: number },
    pB: { x: number; y: number },
    sectionId: string,
    materialId: string,
    tolerance = 1e-4
  ): { model: StructuralModel2D; createdNodeIds: string[]; createdElementIds: string[] } {
    const nextRevision = model.revision + 1;
    const nodes = [...model.nodes];
    const elements: Element2D[] = [...model.elements];
    const loadCases = model.load_cases.map(lc => ({
      ...lc,
      dist_loads: [...lc.dist_loads],
      point_loads: [...lc.point_loads],
      nodal_loads: [...lc.nodal_loads],
    }));

    const createdNodeIds: string[] = [];
    const createdElementIds: string[] = [];

    // 1. Get or create start node A
    let nodeA = findCoincidentNode(nodes, pA, tolerance);
    if (!nodeA) {
      const newId = `N${nodes.length + 1}`;
      nodeA = { id: newId, x: pA.x, y: pA.y };
      nodes.push(nodeA);
      createdNodeIds.push(newId);
    }

    // 2. Get or create end node B
    let nodeB = findCoincidentNode(nodes, pB, tolerance);
    if (!nodeB) {
      const newId = `N${nodes.length + 1}`;
      nodeB = { id: newId, x: pB.x, y: pB.y };
      nodes.push(nodeB);
      createdNodeIds.push(newId);
    }

    if (nodeA.id === nodeB.id) {
      throw new Error('ZERO_LENGTH_ELEMENT');
    }

    // 3. Find if nodeA or nodeB lies strictly inside an existing element, and split that element
    splitElementAtNodeIfInterior(nodeA, nodes, elements, loadCases, tolerance);
    splitElementAtNodeIfInterior(nodeB, nodes, elements, loadCases, tolerance);

    // 4. Find all intersections between segment (nodeA, nodeB) and existing elements
    // Collect split points along the segment (nodeA -> nodeB) sorted by distance from nodeA
    interface SplitEvent {
      point: { x: number; y: number };
      existingElemId: string;
      distFromA: number;
    }

    const splitEvents: SplitEvent[] = [];
    const nodeMap = new Map<string, Node2D>();
    for (const n of nodes) nodeMap.set(n.id, n);

    for (const elem of elements) {
      if (!isLineElement(elem)) continue;
      const n1 = nodeMap.get(elem.node_i);
      const n2 = nodeMap.get(elem.node_j);
      if (!n1 || !n2) continue;

      // Don't intersect elements sharing an endpoint
      if (elem.node_i === nodeA.id || elem.node_j === nodeA.id ||
          elem.node_i === nodeB.id || elem.node_j === nodeB.id) {
        continue;
      }

      const intPt = computeSegmentIntersection(nodeA, nodeB, n1, n2, tolerance);
      if (intPt) {
        // Must be interior to both
        const dA = Math.hypot(intPt.x - nodeA.x, intPt.y - nodeA.y);
        const dB = Math.hypot(intPt.x - nodeB.x, intPt.y - nodeB.y);
        const d1 = Math.hypot(intPt.x - n1.x, intPt.y - n1.y);
        const d2 = Math.hypot(intPt.x - n2.x, intPt.y - n2.y);

        if (dA > tolerance && dB > tolerance && d1 > tolerance && d2 > tolerance) {
          splitEvents.push({
            point: intPt,
            existingElemId: elem.id,
            distFromA: dA,
          });
        }
      }
    }

    // Sort split events along the new member from nodeA to nodeB
    splitEvents.sort((a, b) => a.distFromA - b.distFromA);

    // Apply splits to existing crossed members and gather split nodes along new member
    const orderedNodesAlongNewMember: Node2D[] = [nodeA];

    for (const ev of splitEvents) {
      let splitNode = findCoincidentNode(nodes, ev.point, tolerance);
      if (!splitNode) {
        const newId = `N${nodes.length + 1}`;
        splitNode = { id: newId, x: ev.point.x, y: ev.point.y };
        nodes.push(splitNode);
        nodeMap.set(newId, splitNode);
        createdNodeIds.push(newId);
      }
      orderedNodesAlongNewMember.push(splitNode);

      // Split the crossed existing element at splitNode
      const targetIdx = elements.findIndex(e => e.id === ev.existingElemId);
      if (targetIdx !== -1) {
        const oldElem = elements[targetIdx];
        if (isLineElement(oldElem)) {
          const oldId = oldElem.id;
          const sub1Id = `${oldId}_a`;
          const sub2Id = `${oldId}_b`;

          const sub1: FrameMemberElement = {
            kind: 'FrameMember',
            id: sub1Id,
            node_i: oldElem.node_i,
            node_j: splitNode.id,
            section_id: oldElem.section_id,
            material_id: oldElem.material_id,
            release_i: (oldElem as FrameMemberElement).release_i ?? { axial: false, shear: false, moment: false },
            release_j: { axial: false, shear: false, moment: false },
            offset_i: (oldElem as FrameMemberElement).offset_i ?? { dx: 0, dy: 0, frame: 'Local' },
            offset_j: { dx: 0, dy: 0, frame: 'Local' },
          };

          const sub2: FrameMemberElement = {
            kind: 'FrameMember',
            id: sub2Id,
            node_i: splitNode.id,
            node_j: oldElem.node_j,
            section_id: oldElem.section_id,
            material_id: oldElem.material_id,
            release_i: { axial: false, shear: false, moment: false },
            release_j: (oldElem as FrameMemberElement).release_j ?? { axial: false, shear: false, moment: false },
            offset_i: { dx: 0, dy: 0, frame: 'Local' },
            offset_j: (oldElem as FrameMemberElement).offset_j ?? { dx: 0, dy: 0, frame: 'Local' },
          };

          // Replace old element with sub1 and sub2
          elements.splice(targetIdx, 1, sub1, sub2);

          // Partition any distributed loads from oldElem to sub1 and sub2
          partitionDistributedLoads(loadCases, oldId, sub1Id, sub2Id);
        }
      }
    }

    orderedNodesAlongNewMember.push(nodeB);

    // 5. Create new sub-members along orderedNodesAlongNewMember
    for (let k = 0; k < orderedNodesAlongNewMember.length - 1; k++) {
      const nStart = orderedNodesAlongNewMember[k];
      const nEnd = orderedNodesAlongNewMember[k + 1];

      // Avoid duplicates
      const exists = elements.some(
        e => isLineElement(e) &&
          ((e.node_i === nStart.id && e.node_j === nEnd.id) ||
           (e.node_i === nEnd.id && e.node_j === nStart.id))
      );

      if (!exists && nStart.id !== nEnd.id) {
        const elemId = `M${elements.length + 1}`;
        const newMember: FrameMemberElement = {
          kind: 'FrameMember',
          id: elemId,
          node_i: nStart.id,
          node_j: nEnd.id,
          section_id: sectionId,
          material_id: materialId,
          release_i: { axial: false, shear: false, moment: false },
          release_j: { axial: false, shear: false, moment: false },
          offset_i: { dx: 0, dy: 0, frame: 'Local' },
          offset_j: { dx: 0, dy: 0, frame: 'Local' },
        };
        elements.push(newMember);
        createdElementIds.push(elemId);
      }
    }

    const updatedModel: StructuralModel2D = {
      ...model,
      revision: nextRevision,
      nodes,
      elements,
      load_cases: loadCases,
    };

    return {
      model: updatedModel,
      createdNodeIds,
      createdElementIds,
    };
  }
}

function findCoincidentNode(
  nodes: Node2D[],
  pt: { x: number; y: number },
  tol: number
): Node2D | null {
  for (const n of nodes) {
    if (Math.hypot(n.x - pt.x, n.y - pt.y) <= tol) {
      return n;
    }
  }
  return null;
}

function splitElementAtNodeIfInterior(
  node: Node2D,
  nodes: Node2D[],
  elements: Element2D[],
  loadCases: any[],
  tol: number
): void {
  const nodeMap = new Map<string, Node2D>();
  for (const n of nodes) nodeMap.set(n.id, n);

  for (let i = 0; i < elements.length; i++) {
    const elem = elements[i];
    if (!isLineElement(elem)) continue;
    if (elem.node_i === node.id || elem.node_j === node.id) continue;

    const n1 = nodeMap.get(elem.node_i);
    const n2 = nodeMap.get(elem.node_j);
    if (!n1 || !n2) continue;

    const len = Math.hypot(n2.x - n1.x, n2.y - n1.y);
    if (len < tol) continue;

    // Check projection s along segment
    const dx = n2.x - n1.x;
    const dy = n2.y - n1.y;
    const s = ((node.x - n1.x) * dx + (node.y - n1.y) * dy) / (len * len);

    if (s > tol / len && s < 1.0 - tol / len) {
      const projX = n1.x + s * dx;
      const projY = n1.y + s * dy;
      const perpDist = Math.hypot(node.x - projX, node.y - projY);

      if (perpDist <= tol) {
        // Split element i
        const oldId = elem.id;
        const sub1Id = `${oldId}_a`;
        const sub2Id = `${oldId}_b`;

        const sub1: FrameMemberElement = {
          kind: 'FrameMember',
          id: sub1Id,
          node_i: elem.node_i,
          node_j: node.id,
          section_id: elem.section_id,
          material_id: elem.material_id,
          release_i: (elem as FrameMemberElement).release_i ?? { axial: false, shear: false, moment: false },
          release_j: { axial: false, shear: false, moment: false },
          offset_i: (elem as FrameMemberElement).offset_i ?? { dx: 0, dy: 0, frame: 'Local' },
          offset_j: { dx: 0, dy: 0, frame: 'Local' },
        };

        const sub2: FrameMemberElement = {
          kind: 'FrameMember',
          id: sub2Id,
          node_i: node.id,
          node_j: elem.node_j,
          section_id: elem.section_id,
          material_id: elem.material_id,
          release_i: { axial: false, shear: false, moment: false },
          release_j: (elem as FrameMemberElement).release_j ?? { axial: false, shear: false, moment: false },
          offset_i: { dx: 0, dy: 0, frame: 'Local' },
          offset_j: (elem as FrameMemberElement).offset_j ?? { dx: 0, dy: 0, frame: 'Local' },
        };

        elements.splice(i, 1, sub1, sub2);
        partitionDistributedLoads(loadCases, oldId, sub1Id, sub2Id);
        break;
      }
    }
  }
}

function partitionDistributedLoads(
  loadCases: any[],
  oldElemId: string,
  sub1Id: string,
  sub2Id: string
): void {
  for (const lc of loadCases) {
    const newLoads: DistributedLoad2D[] = [];
    for (const q of lc.dist_loads as DistributedLoad2D[]) {
      if (q.element_id === oldElemId) {
        // Partition 50% / 50% for default uniform load
        newLoads.push({
          ...q,
          id: `${q.id}_1`,
          element_id: sub1Id,
        });
        newLoads.push({
          ...q,
          id: `${q.id}_2`,
          element_id: sub2Id,
        });
      } else {
        newLoads.push(q);
      }
    }
    lc.dist_loads = newLoads;
  }
}
