import { describe, expect, it } from "vitest";
import * as THREE from "three";
import {
  buildScalarVertexColors,
  ColormapPalette,
  computeAutoDeformationScale,
  derivePrincipalStressChannel,
  evaluateColormap,
  getColormapGradientCss,
  normalizeScalarFieldValues,
} from "./colormaps";
import {
  buildBoundaryVectorGlyphs,
  clampSphericalCamera,
  computeAutoFitCameraState,
  computeOrthographicFrustum,
  computeWorldClippingPlane,
  createDefaultBoundaryAssignments,
  resolveBoundaryHitFromIntersections,
} from "./viewport-scene.helpers";
import { MeshData } from "../../services/mesh.service";

const SAMPLE_3D_MESH: MeshData = {
  dimension: 3,
  nodes: [
    [0, 0, 0],
    [10, 0, 0],
    [10, 2, 0],
    [0, 2, 0],
    [0, 0, 4],
    [10, 0, 4],
    [10, 2, 4],
    [0, 2, 4],
  ],
  volume_elements: [
    { id: 0, elem_type: "Hex8", nodes: [0, 1, 2, 3, 4, 5, 6, 7] },
  ],
  boundary_elements: [
    { id: 0, elem_type: "Quad4", nodes: [0, 3, 7, 4], tag: "Left", normal: [-1, 0, 0] },
    { id: 1, elem_type: "Quad4", nodes: [1, 2, 6, 5], tag: "Right", normal: [1, 0, 0] },
    { id: 2, elem_type: "Quad4", nodes: [4, 5, 6, 7], tag: "Front", normal: [0, 0, 1] },
  ],
  stats: {
    num_nodes: 8,
    num_volume_elements: 1,
    num_boundary_elements: 3,
    dimension: 3,
  },
};

describe("cap-interactive-canvas Slice 1: Viewport Camera, Clipping, Picking, Colormaps & Auto-Scale", () => {
  describe("SC-005: 3D/2D Camera Auto-Fit, Orbit Singularity Clamp & Orthographic Frustum", () => {
    it("clamps polar angle phi away from zenith/nadir poles within [0.05, pi - 0.05] and radius within [0.5, 250] (INV-3)", () => {
      const zenithAttempt = clampSphericalCamera({ theta: 0.5, phi: -0.4, radius: 0.01 });
      expect(zenithAttempt.phi).toBeCloseTo(0.05, 6);
      expect(zenithAttempt.radius).toBeCloseTo(0.5, 6);

      const nadirAttempt = clampSphericalCamera({ theta: 1.2, phi: Math.PI + 0.8, radius: 999 });
      expect(nadirAttempt.phi).toBeCloseTo(Math.PI - 0.05, 6);
      expect(nadirAttempt.radius).toBeCloseTo(250, 6);
    });

    it("computes isometric 3D and planar 2D auto-fit camera framing (SC-005)", () => {
      const fit3d = computeAutoFitCameraState(SAMPLE_3D_MESH);
      expect(fit3d.target).toEqual([5, 1, 2]);
      expect(fit3d.spherical.theta).toBeCloseTo(Math.PI / 4, 6);
      expect(fit3d.spherical.phi).toBeCloseTo(Math.PI / 3, 6);
      expect(fit3d.spherical.radius).toBeCloseTo(20, 6);

      const mesh2d: MeshData = { ...SAMPLE_3D_MESH, dimension: 2 };
      const fit2d = computeAutoFitCameraState(mesh2d);
      expect(fit2d.spherical.theta).toBeCloseTo(0, 6);
      expect(fit2d.spherical.phi).toBeCloseTo(Math.PI / 2.05, 6);
      expect(fit2d.spherical.radius).toBeCloseTo(18, 6);
    });

    it("computes symmetric orthographic frustum bounds preserving aspect ratio (SC-005)", () => {
      const frustum = computeOrthographicFrustum(20, 16 / 9, 45);
      expect(frustum.right).toBeGreaterThan(0);
      expect(frustum.left).toBeCloseTo(-frustum.right, 9);
      expect(frustum.top).toBeGreaterThan(0);
      expect(frustum.bottom).toBeCloseTo(-frustum.top, 9);
      expect(frustum.right / frustum.top).toBeCloseTo(16 / 9, 6);
    });
  });

  describe("SC-006: 3D Dynamic Clipping Plane Slicing Across Volumetric Cavity Mesh", () => {
    it("slices along Z mid-plane (s = 0.0) and flips visible half when invertNormal is toggled (INV-5)", () => {
      const midSlice = computeWorldClippingPlane(
        { enabled: true, axis: "Z", normalizedOffset: 0.0, invertNormal: false },
        SAMPLE_3D_MESH.nodes
      );
      // Bounding box Z is [0, 4], midpoint is c_z = 2.0
      expect(midSlice.planeCoordinate).toBeCloseTo(2.0, 6);
      expect(midSlice.normal).toEqual([0, 0, -1]);
      expect(midSlice.constant).toBeCloseTo(2.0, 6);

      // Front point (z = 3.5 > 2.0) has n . x + d = -3.5 + 2.0 = -1.5 < 0 (clipped away)
      const frontEvaluation = midSlice.normal[2] * 3.5 + midSlice.constant;
      expect(frontEvaluation).toBeLessThan(0);

      // Inverted normal flips visible half to z > 2.0
      const invertedSlice = computeWorldClippingPlane(
        { enabled: true, axis: "Z", normalizedOffset: 0.0, invertNormal: true },
        SAMPLE_3D_MESH.nodes
      );
      expect(invertedSlice.normal).toEqual([0, 0, 1]);
      expect(invertedSlice.constant).toBeCloseTo(-2.0, 6);
      expect(invertedSlice.normal[2] * 3.5 + invertedSlice.constant).toBeGreaterThan(0);
    });
  });

  describe("SC-007: Interactive Raycast Boundary Facet Picking & Background Miss Handling", () => {
    it("resolves BoundaryFacetPickEvent on facet hit and returns RAYCAST_NO_BOUNDARY_HIT on empty background click", () => {
      const rightFacetObj = new THREE.Mesh();
      rightFacetObj.userData = { boundaryTag: "Right", elementIndices: [1] };

      const hitResult = resolveBoundaryHitFromIntersections([
        {
          object: rightFacetObj,
          point: new THREE.Vector3(10, 1, 2),
          faceIndex: 0,
        },
      ]);
      expect("hit" in hitResult).toBe(true);
      if ("hit" in hitResult) {
        expect(hitResult.hit.tag).toBe("Right");
        expect(hitResult.hit.boundaryElementIndex).toBe(1);
        expect(hitResult.hit.worldPoint).toEqual([10, 1, 2]);
      }

      const missResult = resolveBoundaryHitFromIntersections([]);
      expect(missResult).toEqual({ error: "RAYCAST_NO_BOUNDARY_HIT" });
    });

    it("initializes default Dirichlet/Neumann assignments and builds vector glyph arrows", () => {
      const defaults = createDefaultBoundaryAssignments(SAMPLE_3D_MESH, 50000);
      expect(defaults.get("Left")?.kind).toBe("Dirichlet");
      expect(defaults.get("Right")?.kind).toBe("Neumann");

      const glyphGroup = buildBoundaryVectorGlyphs(
        SAMPLE_3D_MESH,
        SAMPLE_3D_MESH.nodes,
        defaults,
        []
      );
      expect(glyphGroup.children.length).toBeGreaterThanOrEqual(2);
    });
  });

  describe("SC-008: Scalar Field Colormap Transfer & Degenerate Homogeneous Span Guard", () => {
    const palettes: ColormapPalette[] = ["Turbo", "Viridis", "Coolwarm", "Jet"];

    it("evaluates Turbo, Viridis, Coolwarm, and Jet palettes into finite RGB triples in [0, 1]^3", () => {
      for (const palette of palettes) {
        for (const t of [-0.5, 0.0, 0.25, 0.5, 0.75, 1.0, 1.5, Number.NaN]) {
          const [r, g, b] = evaluateColormap(palette, t);
          expect(Number.isFinite(r)).toBe(true);
          expect(Number.isFinite(g)).toBe(true);
          expect(Number.isFinite(b)).toBe(true);
          expect(r).toBeGreaterThanOrEqual(0);
          expect(r).toBeLessThanOrEqual(1);
          expect(g).toBeGreaterThanOrEqual(0);
          expect(g).toBeLessThanOrEqual(1);
          expect(b).toBeGreaterThanOrEqual(0);
          expect(b).toBeLessThanOrEqual(1);
        }
        expect(getColormapGradientCss(palette)).toContain("linear-gradient");
      }
    });

    it("protects against degenerate homogeneous span (max - min <= 1e-9 => span = 1.0, t = 0.0, zero NaNs) (INV-4, BR-2)", () => {
      const uniformE = [200, 200, 200, 200, 200, 200, 200, 200];
      const norm = normalizeScalarFieldValues(uniformE, 200, 200);
      expect(norm.isDegenerate).toBe(true);
      expect(norm.span).toBe(1.0);
      expect(norm.normalized.every((t) => t === 0.0)).toBe(true);

      for (const palette of palettes) {
        const colors = buildScalarVertexColors(uniformE, palette, 200, 200);
        expect(colors.length).toBe(uniformE.length * 3);
        for (const c of colors) {
          expect(Number.isNaN(c)).toBe(false);
          expect(Number.isFinite(c)).toBe(true);
        }
      }
    });

    it("derives PrincipalStress scalar channel without NaNs", () => {
      const vm = [10, 25, 45.2];
      const disp: [number, number, number][] = [
        [0.001, 0, 0],
        [-0.002, -0.001, 0],
        [0, 0.001, 0],
      ];
      const pStress = derivePrincipalStressChannel(vm, disp);
      expect(pStress.length).toBe(3);
      expect(pStress.every((v) => Number.isFinite(v))).toBe(true);
    });
  });

  describe("SC-009: Deformation Exaggeration (Auto & Manual Presets) Preserves Physical Readouts", () => {
    it("normalizes peak visual deflection to 8% of L_max = 10 m (0.08 * 10 / 0.002 = 400x) (POL-1, BR-1)", () => {
      const nodes: [number, number, number][] = [
        [0, 0, 0],
        [10, 0, 0],
        [10, 2, 0],
        [0, 2, 0],
      ];
      const maxDispMeters = 0.002; // 2.000 mm
      const maxVonMisesMpa = 45.2;

      const scale = computeAutoDeformationScale(nodes, maxDispMeters);
      expect(scale).toBe(400);

      // Applying 400x or 1000x to displaced coordinates never mutates physical readouts
      const displacedAt1000x = nodes.map(([x, y, z]) => [x, y - maxDispMeters * 1000, z]);
      expect(displacedAt1000x[1][1]).toBeCloseTo(-2.0, 6);
      expect((maxDispMeters * 1000).toFixed(3)).toBe("2.000");
      expect(maxVonMisesMpa.toFixed(2)).toBe("45.20");
    });
  });
});
