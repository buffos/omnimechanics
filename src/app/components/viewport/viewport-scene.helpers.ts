/**
 * Pure 3D/2D Viewport Scene Policies & Helpers (`cap-interactive-canvas`):
 * - Camera spherical clamping (`INV-3`, `SC-005`)
 * - Orthographic vs Perspective camera frustum calculation (`SC-005`)
 * - 3D Clipping Plane world-space equation (`INV-5`, `SC-006`)
 * - Raycast Boundary Facet Picking & BC Assignment (`SC-007`)
 * - Vector Glyph Arrow Builders (`WF-4`)
 * - WebGL Buffer Disposal (`NFR-1`)
 */

import * as THREE from 'three';
import { MeshData } from '../../services/mesh.service';
import {
  BoundaryConditionAssignment,
  BoundaryFacetPickEvent,
  BoundaryTag,
  ClippingPlaneConfig,
  ColormapPalette,
  evaluateColormap,
  normalizeScalarFieldValues,
} from './colormaps';

export const TAG_COLORS: Record<string, number> = {
  Left: 0x3b82f6,    // Blue (Fixed support / Dirichlet)
  Right: 0xef4444,   // Red (Traction load / Neumann)
  Top: 0xf59e0b,     // Amber / Yellow
  Bottom: 0x10b981,  // Emerald green
  Hole: 0xa855f7,    // Purple / Cavity
  Front: 0x06b6d4,   // Cyan
  Back: 0x6366f1,    // Indigo
  Default: 0x94a3b8, // Slate gray
};

export interface SphericalCameraState {
  radius: number;
  theta: number;
  phi: number;
}

export interface BoundingBox3D {
  min: [number, number, number];
  max: [number, number, number];
  center: [number, number, number];
  span: [number, number, number];
  maxDim: number;
}

/**
 * Computes the 3D axis-aligned bounding box of a mesh node array.
 */
export function computeMeshBoundingBox(
  nodes: ReadonlyArray<[number, number, number]>
): BoundingBox3D {
  if (!nodes || nodes.length === 0) {
    return {
      min: [-0.5, -0.5, -0.5],
      max: [0.5, 0.5, 0.5],
      center: [0, 0, 0],
      span: [1, 1, 1],
      maxDim: 1,
    };
  }

  let minX = Infinity, minY = Infinity, minZ = Infinity;
  let maxX = -Infinity, maxY = -Infinity, maxZ = -Infinity;

  for (const [x, y, z] of nodes) {
    if (x < minX) minX = x;
    if (x > maxX) maxX = x;
    if (y < minY) minY = y;
    if (y > maxY) maxY = y;
    if (z < minZ) minZ = z;
    if (z > maxZ) maxZ = z;
  }

  const dx = maxX - minX;
  const dy = maxY - minY;
  const dz = maxZ - minZ;

  return {
    min: [minX, minY, minZ],
    max: [maxX, maxY, maxZ],
    center: [(minX + maxX) / 2, (minY + maxY) / 2, (minZ + maxZ) / 2],
    span: [dx, dy, dz],
    maxDim: Math.max(dx, dy, dz, 1.0),
  };
}

/**
 * Enforces `INV-3` spherical camera singularity clamp:
 * phi in [0.05, pi - 0.05] rad, radius in [0.5, 250].
 */
export function clampSphericalCamera(state: SphericalCameraState): SphericalCameraState {
  return {
    theta: state.theta,
    phi: Math.max(0.05, Math.min(Math.PI - 0.05, state.phi)),
    radius: Math.max(0.5, Math.min(250, state.radius)),
  };
}

/**
 * Computes auto-fit spherical camera framing for a 2D or 3D mesh (`SC-005`).
 */
export function computeAutoFitCameraState(meshData: MeshData): {
  spherical: SphericalCameraState;
  target: [number, number, number];
} {
  const bbox = computeMeshBoundingBox(meshData.nodes);
  if (meshData.dimension === 2) {
    return {
      target: bbox.center,
      spherical: clampSphericalCamera({
        theta: 0,
        phi: Math.PI / 2.05,
        radius: bbox.maxDim * 1.8,
      }),
    };
  }
  return {
    target: bbox.center,
    spherical: clampSphericalCamera({
      theta: Math.PI / 4,
      phi: Math.PI / 3,
      radius: bbox.maxDim * 2.0,
    }),
  };
}

/**
 * Computes orthographic frustum bounds (left, right, top, bottom) from spherical radius and aspect ratio (`SC-005`).
 */
export function computeOrthographicFrustum(
  radius: number,
  aspect: number,
  fovDeg = 45
): { left: number; right: number; top: number; bottom: number } {
  const safeAspect = Number.isFinite(aspect) && aspect > 0 ? aspect : 1.0;
  const halfHeight = Math.max(0.2, radius * Math.tan((fovDeg * Math.PI) / 360));
  const halfWidth = halfHeight * safeAspect;
  return {
    left: -halfWidth,
    right: halfWidth,
    top: halfHeight,
    bottom: -halfHeight,
  };
}

/**
 * Computes the world-space clipping plane normal and constant d (`INV-5`, `SC-006`):
 * Plane equation in Three.js: n . x + constant >= 0 is kept (unclipped).
 * When invertNormal === false, points with x_k <= planeCoord are kept (front half x_k > planeCoord clipped away).
 * Specifically, with normal n = (0, 0, -1) and constant = +planeCoord:
 *   -z + planeCoord >= 0  <=>  z <= planeCoord (front half z > planeCoord is clipped away).
 * When invertNormal === true, normal is flipped to +1 and constant = -planeCoord so z >= planeCoord is kept.
 */
export function computeWorldClippingPlane(
  config: ClippingPlaneConfig,
  nodes: ReadonlyArray<[number, number, number]>
): {
  normal: [number, number, number];
  constant: number;
  planeCoordinate: number;
} {
  const bbox = computeMeshBoundingBox(nodes);
  const axisIdx = config.axis === 'X' ? 0 : config.axis === 'Y' ? 1 : 2;
  const clampedOffset = Math.max(-1.0, Math.min(1.0, config.normalizedOffset));

  const minK = bbox.min[axisIdx];
  const maxK = bbox.max[axisIdx];
  const midK = bbox.center[axisIdx];
  const spanK = Math.max(maxK - minK, 1e-3);

  const planeCoordinate = midK + (clampedOffset * spanK) / 2.0;
  const sign = config.invertNormal ? 1 : -1;

  const normal: [number, number, number] = [0, 0, 0];
  normal[axisIdx] = sign;
  const constant = -sign * planeCoordinate;

  return {
    normal,
    constant,
    planeCoordinate,
  };
}

/**
 * Normalizes a raw mesh tag string into a canonical `BoundaryTag`.
 */
export function normalizeBoundaryTag(rawTag: string | undefined | null): BoundaryTag {
  switch (rawTag) {
    case 'Left':
    case 'Right':
    case 'Top':
    case 'Bottom':
    case 'Front':
    case 'Back':
    case 'Hole':
      return rawTag;
    default:
      return 'Default';
  }
}

/**
 * Default boundary condition assignments for standard continuum presets (`SC-007`).
 */
export function createDefaultBoundaryAssignments(
  meshData: MeshData | null,
  loadMagnitude = 50000
): Map<BoundaryTag, BoundaryConditionAssignment> {
  const map = new Map<BoundaryTag, BoundaryConditionAssignment>();
  if (!meshData) return map;

  const presentTags = new Set<BoundaryTag>();
  for (const bnd of meshData.boundary_elements) {
    presentTags.add(normalizeBoundaryTag(bnd.tag));
  }

  if (presentTags.has('Left')) {
    map.set('Left', {
      tag: 'Left',
      kind: 'Dirichlet',
      prescribedDisplacement: [0, 0, 0],
    });
  }
  if (presentTags.has('Right')) {
    map.set('Right', {
      tag: 'Right',
      kind: 'Neumann',
      prescribedTraction: [0, -loadMagnitude / 1000, 0],
    });
  }
  return map;
}

/**
 * Resolves a raycast intersection against `boundaryGroup` children into a `BoundaryFacetPickEvent`
 * or `'RAYCAST_NO_BOUNDARY_HIT'` when no boundary element is hit (`SC-007`).
 */
export function resolveBoundaryHitFromIntersections(
  intersections: ReadonlyArray<{
    object: THREE.Object3D;
    point: THREE.Vector3;
    faceIndex?: number | null;
    index?: number | null;
  }>
): { hit: BoundaryFacetPickEvent } | { error: 'RAYCAST_NO_BOUNDARY_HIT' } {
  for (const hit of intersections) {
    const rawTag = hit.object.userData?.['boundaryTag'] as string | undefined;
    if (rawTag) {
      const tag = normalizeBoundaryTag(rawTag);
      const elementIndices = hit.object.userData?.['elementIndices'] as number[] | undefined;
      const primIdx = hit.faceIndex ?? hit.index ?? 0;
      const boundaryElementIndex =
        elementIndices && elementIndices.length > 0
          ? (elementIndices[Math.min(primIdx, elementIndices.length - 1)] ?? 0)
          : 0;

      return {
        hit: {
          tag,
          boundaryElementIndex,
          worldPoint: [hit.point.x, hit.point.y, hit.point.z],
        },
      };
    }
  }
  return { error: 'RAYCAST_NO_BOUNDARY_HIT' };
}

/**
 * Builds 3D/2D `VectorGlyphOverlay` arrows (`THREE.ArrowHelper`) anchored at boundary facet centroids
 * for active `BoundaryConditionAssignment` entries (`Neumann` tractions and `Dirichlet` reactions).
 */
export function buildBoundaryVectorGlyphs(
  meshData: MeshData,
  displacedNodes: ReadonlyArray<[number, number, number]>,
  assignments: ReadonlyMap<BoundaryTag, BoundaryConditionAssignment>,
  clippingPlanes: THREE.Plane[]
): THREE.Group {
  const glyphGroup = new THREE.Group();
  if (!meshData || meshData.boundary_elements.length === 0 || assignments.size === 0) {
    return glyphGroup;
  }

  const bbox = computeMeshBoundingBox(displacedNodes);
  const arrowLen = Math.max(0.35, bbox.maxDim * 0.12);

  // Sample up to 12 representative boundary elements per tag so glyphs remain clean and legible
  const byTag = new Map<BoundaryTag, number[]>();
  meshData.boundary_elements.forEach((bnd, idx) => {
    const tag = normalizeBoundaryTag(bnd.tag);
    if (!assignments.has(tag)) return;
    if (!byTag.has(tag)) byTag.set(tag, []);
    byTag.get(tag)!.push(idx);
  });

  for (const [tag, indices] of byTag.entries()) {
    const assignment = assignments.get(tag);
    if (!assignment) continue;

    const step = Math.max(1, Math.floor(indices.length / 8));
    for (let k = 0; k < indices.length; k += step) {
      const bnd = meshData.boundary_elements[indices[k]];
      if (!bnd || bnd.nodes.length === 0) continue;

      let cx = 0, cy = 0, cz = 0;
      for (const nIdx of bnd.nodes) {
        const pt = displacedNodes[nIdx] ?? [0, 0, 0];
        cx += pt[0];
        cy += pt[1];
        cz += pt[2];
      }
      cx /= bnd.nodes.length;
      cy /= bnd.nodes.length;
      cz /= bnd.nodes.length;

      let dir = new THREE.Vector3(0, -1, 0);
      let colorHex = 0xef4444; // Red for Neumann traction

      if (assignment.kind === 'Neumann') {
        const t = assignment.prescribedTraction ?? [0, -1, 0];
        const v = new THREE.Vector3(t[0], t[1], t[2]);
        if (v.lengthSq() > 1e-9) {
          dir = v.normalize();
        }
        colorHex = 0xef4444;
      } else if (assignment.kind === 'Dirichlet') {
        const n = bnd.normal ?? [-1, 0, 0];
        const v = new THREE.Vector3(-n[0], -n[1], -n[2]);
        if (v.lengthSq() > 1e-9) {
          dir = v.normalize();
        } else {
          dir = new THREE.Vector3(1, 0, 0);
        }
        colorHex = 0x3b82f6; // Blue for support reaction
      } else if (assignment.kind === 'Robin') {
        const n = bnd.normal ?? [0, -1, 0];
        const v = new THREE.Vector3(n[0], n[1], n[2]);
        if (v.lengthSq() > 1e-9) {
          dir = v.normalize();
        }
        colorHex = 0x10b981; // Emerald for Robin impedance
      }

      const origin = new THREE.Vector3(cx, cy, cz);
      const arrow = new THREE.ArrowHelper(
        dir,
        origin,
        arrowLen,
        colorHex,
        arrowLen * 0.28,
        arrowLen * 0.16
      );

      // Apply clipping planes to arrow materials
      arrow.traverse((child) => {
        const meshChild = child as THREE.Mesh;
        if (meshChild.material) {
          const mats = Array.isArray(meshChild.material) ? meshChild.material : [meshChild.material];
          for (const m of mats) {
            m.clippingPlanes = clippingPlanes;
          }
        }
      });

      glyphGroup.add(arrow);
    }
  }

  return glyphGroup;
}

/**
 * Builds Three.js volume and wireframe `BufferGeometry` instances from `MeshData`.
 */
export function buildVolumeAndEdgeGeometries(
  meshData: MeshData,
  displacedNodes: ReadonlyArray<[number, number, number]>,
  colorValues: ReadonlyArray<number> | null,
  minVal: number,
  maxVal: number,
  palette: ColormapPalette,
  colormapActive: boolean
): {
  volumeGeometry: THREE.BufferGeometry;
  edgeGeometry: THREE.BufferGeometry;
} {
  const indices: number[] = [];
  const edgeIndices: number[] = [];

  const addEdge = (a: number, b: number) => {
    edgeIndices.push(a, b);
  };

  for (const elem of meshData.volume_elements) {
    const n = elem.nodes;
    switch (elem.elem_type) {
      case 'Tri3':
        indices.push(n[0], n[1], n[2]);
        addEdge(n[0], n[1]);
        addEdge(n[1], n[2]);
        addEdge(n[2], n[0]);
        break;

      case 'Quad4':
        indices.push(n[0], n[1], n[2]);
        indices.push(n[0], n[2], n[3]);
        addEdge(n[0], n[1]);
        addEdge(n[1], n[2]);
        addEdge(n[2], n[3]);
        addEdge(n[3], n[0]);
        break;

      case 'Tet4':
        indices.push(n[0], n[1], n[2]);
        indices.push(n[0], n[2], n[3]);
        indices.push(n[0], n[3], n[1]);
        indices.push(n[1], n[3], n[2]);
        addEdge(n[0], n[1]);
        addEdge(n[1], n[2]);
        addEdge(n[2], n[0]);
        addEdge(n[0], n[3]);
        addEdge(n[1], n[3]);
        addEdge(n[2], n[3]);
        break;

      case 'Hex8': {
        const hexFaces = [
          [n[0], n[1], n[2], n[3]],
          [n[4], n[7], n[6], n[5]],
          [n[0], n[4], n[5], n[1]],
          [n[2], n[6], n[7], n[3]],
          [n[0], n[3], n[7], n[4]],
          [n[1], n[5], n[6], n[2]],
        ];
        for (const f of hexFaces) {
          indices.push(f[0], f[1], f[2]);
          indices.push(f[0], f[2], f[3]);
        }
        addEdge(n[0], n[1]); addEdge(n[1], n[2]); addEdge(n[2], n[3]); addEdge(n[3], n[0]);
        addEdge(n[4], n[5]); addEdge(n[5], n[6]); addEdge(n[6], n[7]); addEdge(n[7], n[4]);
        addEdge(n[0], n[4]); addEdge(n[1], n[5]); addEdge(n[2], n[6]); addEdge(n[3], n[7]);
        break;
      }
    }
  }

  const flatPositions = new Float32Array(displacedNodes.length * 3);
  for (let i = 0; i < displacedNodes.length; i++) {
    flatPositions[i * 3 + 0] = displacedNodes[i][0];
    flatPositions[i * 3 + 1] = displacedNodes[i][1];
    flatPositions[i * 3 + 2] = displacedNodes[i][2];
  }

  const volumeGeometry = new THREE.BufferGeometry();
  volumeGeometry.setAttribute('position', new THREE.BufferAttribute(flatPositions, 3));
  volumeGeometry.setIndex(indices);

  if (colormapActive && colorValues && colorValues.length === displacedNodes.length) {
    const { normalized } = normalizeScalarFieldValues(colorValues, minVal, maxVal);
    const colors = new Float32Array(displacedNodes.length * 3);
    for (let i = 0; i < displacedNodes.length; i++) {
      const [r, g, b] = evaluateColormap(palette, normalized[i]);
      colors[i * 3 + 0] = r;
      colors[i * 3 + 1] = g;
      colors[i * 3 + 2] = b;
    }
    volumeGeometry.setAttribute('color', new THREE.BufferAttribute(colors, 3));
  }

  volumeGeometry.computeVertexNormals();

  const edgeGeometry = new THREE.BufferGeometry();
  edgeGeometry.setAttribute('position', new THREE.BufferAttribute(flatPositions, 3));
  edgeGeometry.setIndex(edgeIndices);

  return { volumeGeometry, edgeGeometry };
}

/**
 * Builds the 2D or 3D boundary facet `THREE.Group` with `boundaryTag` and `elementIndices` userData for raycast picking.
 */
export function buildBoundaryElementGroup(
  meshData: MeshData,
  displacedNodes: ReadonlyArray<[number, number, number]>,
  pickedTag: BoundaryTag | null,
  clippingPlanes: THREE.Plane[]
): THREE.Group {
  const group = new THREE.Group();

  if (meshData.dimension === 2) {
    const groupedLines = new Map<BoundaryTag, { coords: number[]; elemIndices: number[] }>();
    meshData.boundary_elements.forEach((bnd, bndIdx) => {
      const tag = normalizeBoundaryTag(bnd.tag);
      if (!groupedLines.has(tag)) {
        groupedLines.set(tag, { coords: [], elemIndices: [] });
      }
      const entry = groupedLines.get(tag)!;
      for (const nodeIdx of bnd.nodes) {
        const pt = displacedNodes[nodeIdx];
        entry.coords.push(pt[0], pt[1], pt[2] + 0.01);
      }
      entry.elemIndices.push(bndIdx);
    });

    for (const [tag, { coords, elemIndices }] of groupedLines.entries()) {
      const lineGeom = new THREE.BufferGeometry();
      lineGeom.setAttribute('position', new THREE.Float32BufferAttribute(coords, 3));
      const isPicked = pickedTag === tag;
      const colorVal = isPicked ? 0xffffff : (TAG_COLORS[tag] ?? TAG_COLORS['Default']);
      const lineMat = new THREE.LineBasicMaterial({
        color: colorVal,
        linewidth: isPicked ? 5 : 3,
      });
      const lineSegments = new THREE.LineSegments(lineGeom, lineMat);
      lineSegments.userData = { boundaryTag: tag, elementIndices: elemIndices };
      group.add(lineSegments);
    }
  } else {
    const groupedTriangles = new Map<BoundaryTag, { coords: number[]; elemIndices: number[] }>();
    meshData.boundary_elements.forEach((bnd, bndIdx) => {
      const tag = normalizeBoundaryTag(bnd.tag);
      if (!groupedTriangles.has(tag)) {
        groupedTriangles.set(tag, { coords: [], elemIndices: [] });
      }
      const entry = groupedTriangles.get(tag)!;
      if (bnd.nodes.length >= 3) {
        const p0 = displacedNodes[bnd.nodes[0]];
        const p1 = displacedNodes[bnd.nodes[1]];
        const p2 = displacedNodes[bnd.nodes[2]];
        entry.coords.push(p0[0], p0[1], p0[2], p1[0], p1[1], p1[2], p2[0], p2[1], p2[2]);
        entry.elemIndices.push(bndIdx);

        if (bnd.nodes.length === 4) {
          const p3 = displacedNodes[bnd.nodes[3]];
          entry.coords.push(p0[0], p0[1], p0[2], p2[0], p2[1], p2[2], p3[0], p3[1], p3[2]);
          entry.elemIndices.push(bndIdx);
        }
      }
    });

    for (const [tag, { coords, elemIndices }] of groupedTriangles.entries()) {
      const geom = new THREE.BufferGeometry();
      geom.setAttribute('position', new THREE.Float32BufferAttribute(coords, 3));
      geom.computeVertexNormals();

      const isPicked = pickedTag === tag;
      const colorVal = TAG_COLORS[tag] ?? TAG_COLORS['Default'];
      const mat = new THREE.MeshStandardMaterial({
        color: colorVal,
        emissive: isPicked ? 0xffffff : 0x000000,
        emissiveIntensity: isPicked ? 0.35 : 0.0,
        side: THREE.DoubleSide,
        transparent: true,
        opacity: isPicked ? 0.92 : 0.65,
        depthWrite: false,
        clippingPlanes,
      });

      const mesh = new THREE.Mesh(geom, mat);
      mesh.userData = { boundaryTag: tag, elementIndices: elemIndices };
      group.add(mesh);
    }
  }

  return group;
}

/**
 * Recursively disposes all `BufferGeometry` and `Material` instances inside a Three.js `Object3D` (`NFR-1`).
 */
export function disposeThreeObject(root: THREE.Object3D | null): void {
  if (!root) return;
  root.traverse((obj) => {
    const anyObj = obj as {
      geometry?: THREE.BufferGeometry;
      material?: THREE.Material | THREE.Material[];
    };
    if (anyObj.geometry) {
      anyObj.geometry.dispose();
    }
    if (anyObj.material) {
      if (Array.isArray(anyObj.material)) {
        for (const m of anyObj.material) {
          m.dispose();
        }
      } else {
        anyObj.material.dispose();
      }
    }
  });
}
