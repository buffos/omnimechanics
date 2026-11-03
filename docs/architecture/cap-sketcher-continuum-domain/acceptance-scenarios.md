# Acceptance Scenarios: Continuum Domain Sketcher (`cap-sketcher-continuum-domain`)

- **Owning Capability:** [`cap-sketcher-continuum-domain`](../../../.okf/capabilities/cap-sketcher-continuum-domain.md)
- **Upstream Artifacts:** [`prd.md`](prd.md), [`canonical-domain-model.md`](canonical-domain-model.md), [`canonical-use-cases.md`](canonical-use-cases.md), [`canonical-api-cli-contract.md`](canonical-api-cli-contract.md)
- **Root Verification Policy:** `backend-boundary: required`, `frontend-integration: required`, `end-to-end: catalog-only` ([`docs/prd.md`](../../prd.md) §8)

---

## 1. Scenario Coverage Matrix

| Scenario ID | Title | Primary Rule / Use Case | `backend-boundary` | `frontend-integration` | `end-to-end` |
| --- | --- | --- | --- | --- | --- |
| **SC-001** | $10 \times 10\text{ m}$ square plate with $R = 1.5\text{ m}$ central circular hole generates graded conformal `Tri3` + `Line2` mesh | `BR-CONT-01`, `BR-CONT-04`, `UC-CMD-07` | `required` | `required` | `catalog-only` |
| **SC-002** | Self-intersecting (bowtie) polygon boundary is rejected on loop closure and validation | `BR-CONT-02`, `UC-QRY-02`, `UC-CMD-01` | `required` | `required` | `catalog-only` |
| **SC-003** | Cutout breaching outer boundary or violating minimum clearance $d_{\min}$ is rejected | `BR-CONT-03`, `UC-CMD-02` | `required` | `required` | `catalog-only` |
| **SC-004** | Overlapping or touching interior cutouts are rejected with `HOLE_OVERLAPS_EXISTING_FEATURE` | `BR-CONT-03`, `UC-CMD-02` | `required` | `required` | `catalog-only` |
| **SC-005** | Clockwise-drawn outer boundary is automatically reversed to CCW and hole loops orient CW with outward normals away from material | `BR-CONT-01`, `BR-CONT-05`, `UC-CMD-01` | `required` | `required` | `not-applicable` |
| **SC-006** | Semantic boundary tagging and Dirichlet/Neumann/Robin boundary conditions propagate to all child `BoundaryElement` records | `BR-CONT-04`, `UC-CMD-05`, `UC-CMD-07` | `required` | `required` | `catalog-only` |
| **SC-007** | Interior `InclusionRegion` preserves conformal internal mesh interface without generating exterior BEM boundary elements | `INV-CONT-03`, `UC-CMD-04`, `UC-CMD-07` | `required` | `required` | `catalog-only` |
| **SC-008** | End-to-end handoff of sketched $10 \times 10\text{ m}$ plate with $R = 1.5\text{ m}$ hole into Mode 1 `BEMFEM` solver converges and renders stresses | `BR-CONT-04`, `BR-CONT-06`, `UC-CMD-08` | `required` | `required` | `catalog-only` |
| **SC-009** | Mutating a domain after meshing increments `revision` and blocks stale mesh handoff with `STALE_MESH_HANDOFF` | `BR-CONT-06`, `UC-CMD-08` | `required` | `required` | `catalog-only` |

---

## 2. Detailed Behavioral Scenarios

### SC-001: $10 \times 10\text{ m}$ Square Plate with $R = 1.5\text{ m}$ Central Circular Hole Generates Graded Conformal Mesh (Acceptance Seed)
- **Target Behavior:** Verify the carried-over acceptance seed from `.okf/capabilities/cap-sketcher-continuum-domain.md`.
- **Preconditions:**
  - A `ContinuumDomain` `"CD1"` with outer `BoundaryLoop` vertices $(-5, -5), (5, -5), (5, 5), (-5, 5)$ in metres (`ClosedValid`, signed area $100.0\text{ m}^2$).
  - An interior `CircularCutout` `"H1"` at $(0.0, 0.0)$ with radius $R = 1.5\text{ m}$, `angular_divisions: 32`, and tag `"Hole"`.
  - `MeshingProfile` configured with `target_element_size: 0.8`, `hole_refinement_factor: 0.35`, `min_angle_deg: 25.0`, `volume_element_type: Tri3`, `boundary_element_type: Line2`.
- **Action:**
  - Execute `GenerateConformalContinuumMesh` (`continuum_generate_mesh`).
- **Expected Outcome:**
  - Validation succeeds with status `ValidForMeshing`.
  - `ConformalMeshHandoff` returns `mesh_data` with `dimension == 2`, `volume_elements` of type `Tri3` (all with positive CCW area), and `boundary_elements` of type `Line2`.
  - `quality_report.is_conformal == true` and `quality_report.min_angle_deg >= 20.0`.
  - Every `BoundaryElement` tagged `"Hole"` shares exact node indices `[n0, n1]` with an adjacent `Tri3` `VolumeElement`, lies on the circle $x^2 + y^2 = 1.5^2$ ($\pm 10^{-9}\text{ m}$), has edge length $\le 1.2 \times (0.8 \times 0.35)\text{ m}$ (graded refinement near the hole), and has outward unit normal pointing toward $(0, 0)$ (`n_x * x_mid + n_y * y_mid < 0`).
  - Exterior `BoundaryElement` records carry tags `"Bottom"`, `"Right"`, `"Top"`, `"Left"` with outward normals $[0,-1,0]$, $[1,0,0]$, $[0,1,0]$, $[-1,0,0]$.
- **Verification Surfaces:**
  - `backend-boundary`: `cargo test` invoking `continuum_generate_mesh` and verifying topological conformity, normal vectors, and grading ratio.
  - `frontend-integration`: Vitest test verifying the Continuum Sketcher store builds the $10 \times 10\text{ m}$ + $R = 1.5\text{ m}$ domain DTO and updates the mesh preview state upon IPC response.
  - `end-to-end`: User sketches the $10 \times 10\text{ m}$ square and $R = 1.5\text{ m}$ hole on the canvas, clicks "Generate Conformal Mesh", and inspects the graded `Tri3` mesh overlay and quality report.

---

### SC-002: Self-Intersecting (Bowtie) Polygon Boundary Is Rejected
- **Target Behavior:** Prevent self-intersecting polygons (`INV-CONT-02`) both during interactive preview and authoritative loop closure.
- **Preconditions:**
  - User places vertices $V_1(0, 0)$, $V_2(4, 4)$, $V_3(4, 0)$, $V_4(0, 4)$ forming a figure-8 / bowtie where segment $V_1 V_2$ intersects segment $V_3 V_4$ at $(2, 2)$.
- **Action:**
  - Attempt to preview segment $V_3 \to V_4$ (`continuum_preview_segment`) and attempt to close the loop (`continuum_validate_domain` / `continuum_generate_mesh`).
- **Expected Outcome:**
  - `continuum_preview_segment` returns `is_valid: false` with `intersecting_segment_indices: [0]`.
  - `continuum_validate_domain` and `continuum_generate_mesh` reject the domain with error code `SELF_INTERSECTING_LOOP`, bilingual `message_el` / `message_en`, and `details.intersection_point == [2.0, 2.0]`.
  - "Generate Conformal Mesh" remains disabled in the UI while the self-intersection exists.
- **Verification Surfaces:**
  - `backend-boundary`: Rust unit test verifying `SELF_INTERSECTING_LOOP` and intersection point `[2.0, 2.0]`.
  - `frontend-integration`: Vitest test verifying candidate segment intersection preview flags red error state and disables loop closure.
  - `end-to-end`: Visual check of red self-intersection warning badge on the sketcher canvas.

---

### SC-003: Cutout Breaching Outer Boundary or Violating Minimum Clearance Is Rejected
- **Target Behavior:** Enforce strict containment and minimum clearance $d_{\min}$ between interior holes and the outer boundary (`INV-CONT-03`).
- **Preconditions:**
  - A $10 \times 10\text{ m}$ outer boundary centered at $(0, 0)$ ($x, y \in [-5, 5]$) with $h = 0.8\text{ m}, \alpha_{\text{hole}} = 0.35$ ($d_{\min} = 0.028\text{ m}$).
- **Action:**
  - Case A: Attempt to add a `CircularCutout` at $(4.0, 0.0)$ with radius $R = 1.5\text{ m}$ (extends to $x = 5.5\text{ m}$, breaching the $x = 5.0\text{ m}$ right edge).
  - Case B: Attempt to add a `CircularCutout` at $(3.49, 0.0)$ with radius $R = 1.5\text{ m}$ (extends to $x = 4.99\text{ m}$, leaving clearance $0.01\text{ m} < d_{\min} = 0.028\text{ m}$).
- **Expected Outcome:**
  - Case A fails with `HOLE_OUTSIDE_OUTER_BOUNDARY`.
  - Case B fails with `CLEARANCE_BELOW_MINIMUM` and reports actual distance $0.01\text{ m}$ vs. required $0.028\text{ m}$ in `details`.
- **Verification Surfaces:**
  - `backend-boundary`: Rust unit tests covering both Case A and Case B for circular, rectangular, and polygonal cutouts.
  - `frontend-integration`: Vitest test verifying cutout placement validation in the sketcher store.
  - `end-to-end`: User drags a circular hole toward the outer edge and observes the error highlight when clearance is breached.

---

### SC-004: Overlapping or Touching Interior Cutouts Are Rejected
- **Target Behavior:** Enforce mutual disjointness between multiple interior cutouts (`INV-CONT-03`).
- **Preconditions:**
  - $10 \times 10\text{ m}$ outer square with existing `CircularCutout` `"H1"` at $(-1.0, 0.0), R = 1.5\text{ m}$.
- **Action:**
  - Attempt to add a second `CircularCutout` `"H2"` at $(1.0, 0.0), R = 1.5\text{ m}$ (center distance $2.0\text{ m} < R_1 + R_2 = 3.0\text{ m}$).
- **Expected Outcome:**
  - Rejected with error code `HOLE_OVERLAPS_EXISTING_FEATURE` referencing `"H1"` and `"H2"`.
- **Verification Surfaces:**
  - `backend-boundary`: Rust unit test verifying circle-circle, circle-polygon, and polygon-polygon overlap rejection.
  - `frontend-integration`: Vitest test verifying multi-cutout collision check in the store.
  - `end-to-end`: Visual inspection of overlapping hole rejection in the UI.

---

### SC-005: Clockwise-Drawn Outer Boundary Is Normalized to CCW and Hole Loops Orient CW
- **Target Behavior:** Ensure deterministic winding normalization (`BR-CONT-01`) and outward normal orientation (`BR-CONT-05`) regardless of the direction in which the user clicked the vertices.
- **Preconditions:**
  - User clicks outer rectangle vertices in clockwise order: $(-5, -5) \to (-5, 5) \to (5, 5) \to (5, -5)$, and draws a polygonal cutout in counter-clockwise order: $(-1, -1) \to (1, -1) \to (1, 1) \to (-1, 1)$.
- **Action:**
  - Invoke `continuum_close_and_normalize_loop` on both loops and generate mesh.
- **Expected Outcome:**
  - Outer loop vertex order is reversed to CCW (signed area $+100.0\text{ m}^2$) with normals pointing outward.
  - Cutout polygon vertex order is reversed to CW (signed area $-4.0\text{ m}^2$) with normals pointing inward into the hole: bottom edge of hole ($y = -1$) has normal $[0, 1, 0]$ pointing toward $y = 0$.
- **Verification Surfaces:**
  - `backend-boundary`: Rust unit test asserting signed area signs and exact outward unit normals on every segment.
  - `frontend-integration`: Vitest test verifying normal vector arrow directions in the sketcher canvas view-model.
  - `end-to-end`: `not-applicable` (pure mathematical normalization verified at backend and view-model boundaries).

---

### SC-006: Semantic Boundary Tagging and Boundary Condition Propagation
- **Target Behavior:** Verify that user-assigned semantic tags and physical `BoundaryConditionSpec` settings (`DirichletDisplacement`, `NeumannTraction`, `RobinImpedance`) attach to segments and propagate to all generated `BoundaryElement` records (`UC-CMD-05`, `UC-CMD-07`).
- **Preconditions:**
  - Validated polygonal domain with 5 outer segments (`S1..S5`) and 1 cutout (`H1`).
- **Action:**
  - Tag `S1` as `"Left"` with `DirichletDisplacement { fix_x: true, fix_y: true, ux_m: 0.0, uy_m: 0.0 }`, `S3` as `"Right"` with `NeumannTraction { tx_kn_m2: 25.0, ty_kn_m2: -5.0, coordinate_frame: Global }`, and `S5` as `"AbsorbingWall"` with `RobinImpedance { kn_kn_m3: 1000.0, ks_kn_m3: 500.0, cn_kn_s_m3: 10.0, cs_kn_s_m3: 5.0 }`.
  - Execute `continuum_generate_mesh`.
- **Expected Outcome:**
  - Every `BoundaryElement` subdivided from `S1`, `S3`, `S5`, and `H1` carries the exact parent `tag` (`"Left"`, `"Right"`, `"AbsorbingWall"`, `"Hole"`).
  - `tag_element_counts` sums to `mesh_data.boundary_elements.len()`.
- **Verification Surfaces:**
  - `backend-boundary`: Rust unit test verifying tag preservation across boundary subdivision for both `Line2` and `Line3`.
  - `frontend-integration`: Vitest test verifying inspector tag/BC assignment updates `ContinuumDomain.tag_conditions` and increments `revision`.
  - `end-to-end`: User selects boundary segments on canvas, assigns Dirichlet/Neumann/Robin conditions in the inspector, and sees color-coded BC symbols along the segments.

---

### SC-007: Material Inclusion Region Preserves Conformal Internal Interface Without Spurious BEM Elements
- **Target Behavior:** Distinguish solid `InclusionRegion` subdomains from `Cutout` voids (`UC-CMD-04`).
- **Preconditions:**
  - $10 \times 10\text{ m}$ outer plate with a circular `InclusionRegion` `"INC1"` ($R = 1.5\text{ m}$, $E = 400\text{ GPa}$) at $(0, 0)$ and zero cutouts.
- **Action:**
  - Execute `continuum_generate_mesh`.
- **Expected Outcome:**
  - The interior of `"INC1"` ($r < 1.5\text{ m}$) is fully populated with `VolumeElement` triangles tagged with provenance `"INC1"`.
  - Triangles inside `"INC1"` and outside `"INC1"` share exact conformal edges along the circle $r = 1.5\text{ m}$ (zero hanging nodes).
  - `mesh_data.boundary_elements` contains *only* the exterior square perimeter elements (`Left`, `Right`, `Top`, `Bottom`) and zero `Hole` boundary elements.
- **Verification Surfaces:**
  - `backend-boundary`: Rust unit test verifying internal volume coverage, conformal interface nodes, and absence of interior `BoundaryElement` records.
  - `frontend-integration`: Vitest test verifying inclusion creation and subdomain styling in the sketcher store.
  - `end-to-end`: User adds an inclusion region, generates mesh, and sees interior elements colored by subdomain in the viewport.

---

### SC-008: End-to-End Handoff of Sketched Domain into Mode 1 BEM-FEM Solver
- **Target Behavior:** Prove that a sketched `ContinuumDomain` handed off via `ConformalMeshHandoff` executes cleanly in `cap-solver-core` (`SolverMode::BEMFEM`, `PureFEM`, `PureBEM`).
- **Preconditions:**
  - `ConformalMeshHandoff` generated from the $10 \times 10\text{ m}$ plate with $R = 1.5\text{ m}$ circular hole (`SC-001`).
- **Action:**
  - Hand off `handoff.mesh_data` into `run_static_simulation` with `SolverMode::BEMFEM`, reference material $E = 200\text{ GPa}, \nu = 0.3$, and downward/tensile load magnitude $10{,}000\text{ N}$.
- **Expected Outcome:**
  - `SimulationResult.converged == true`, `max_displacement > 0.0`, `max_von_mises > 0.0`, and `deformed_nodes.len() == handoff.mesh_data.nodes.len()`.
- **Verification Surfaces:**
  - `backend-boundary`: Rust integration test chaining `continuum_generate_mesh` $\to$ `run_static_simulation` (`PureFEM` and `BEMFEM`).
  - `frontend-integration`: Vitest test verifying `HandoffMeshToContinuumSolver` updates the Mode 1 active mesh signal and triggers simulation request construction.
  - `end-to-end`: User sketches the perforated plate, clicks "Generate Conformal Mesh", clicks "Solve in Mode 1", and views the 3D/2D von Mises stress contour plot in the Mode 1 viewport.

---

### SC-009: Domain Mutation Increments Revision and Blocks Stale Mesh Handoff
- **Target Behavior:** Enforce `BR-CONT-06` so stale meshes are never solved as current.
- **Preconditions:**
  - `ConformalMeshHandoff` generated at `domain.revision == 2`.
- **Action:**
  - User moves a vertex or changes hole radius $R$ from $1.5\text{ m}$ to $2.0\text{ m}$ (`domain.revision` becomes `3`), and attempts `HandoffMeshToContinuumSolver` using the `domain_revision == 2` mesh without re-meshing.
- **Expected Outcome:**
  - Handoff is rejected with `STALE_MESH_HANDOFF`, and the UI displays a "Mesh Stale — Regenerate Required" indicator until `continuum_generate_mesh` is re-run for `revision == 3`.
- **Verification Surfaces:**
  - `backend-boundary`: Rust unit test verifying `STALE_MESH_HANDOFF` when `handoff.domain_revision != domain.revision`.
  - `frontend-integration`: Vitest test verifying any mutation in the sketcher store transitions `meshHandoffState` from `Current` to `Stale`.
  - `end-to-end`: Visual check that editing a vertex after meshing marks the mesh badge stale until regenerated.
