# Canonical Use-Case & Application-Service Model: Continuum Domain Sketcher (`cap-sketcher-continuum-domain`)

- **Owning Capability:** [`cap-sketcher-continuum-domain`](../../../.okf/capabilities/cap-sketcher-continuum-domain.md)
- **Upstream Artifacts:** [`domain-glossary.md`](domain-glossary.md), [`prd.md`](prd.md), [`canonical-domain-model.md`](canonical-domain-model.md)
- **Shared Dependencies:** [`cap-geometry-presets`](../../../.okf/capabilities/cap-geometry-presets.md), [`cap-solver-core`](../../../.okf/capabilities/cap-solver-core.md), [`cap-project-persistence`](../../../.okf/capabilities/cap-project-persistence.md)
- **Binding Conventions:** [`cross-capability-conventions.md`](../cross-capability-conventions.md)

---

## 1. Purpose & Application-Layer Goals

This document defines the transport-neutral application-service layer for `cap-sketcher-continuum-domain`. It specifies the commands, queries, orchestration responsibilities, transaction boundaries, idempotency expectations, and failure categories that sit between the UI/CLI adapters and the `ContinuumDomain` / `ConformalMeshHandoff` domain core.

### Design Principles
1. **Intent-Based Operations:** Commands reflect explicit geometric or meshing intents (`CloseOuterBoundaryLoop`, `AddDomainCutout`, `AssignBoundarySegmentCondition`, `GenerateConformalContinuumMesh`) rather than generic property bags.
2. **Authoritative Backend Validation:** While the Angular frontend performs instant preview checks during cursor movement, every domain validation and CDT mesh generation command executes authoritatively in Rust (`src-tauri/src/mesh/` / `continuum/`).
3. **Revision-Coupled Handoff:** Mesh generation and Mode 1 solver handoff explicitly verify `ContinuumDomain.revision` so stale meshes are never passed to `cap-solver-core`.

---

## 2. Application Service Groupings

| Application Service | Responsibility | Collaborating Domain Objects & Services |
| --- | --- | --- |
| **`ContinuumSketchApplicationService`** | Manages `ContinuumDomain` creation, outer loop vertex drafting/closing, cutout and inclusion authoring, boundary segment tagging, and boundary condition assignment. | `ContinuumDomain`, `BoundaryLoop`, `Cutout`, `InclusionRegion`, `ContinuumTopologyValidator` |
| **`ContinuumMeshingApplicationService`** | Validates a `ContinuumDomain` at a target `revision`, builds the PSLG, executes Constrained Delaunay Triangulation (`Tri3`/`Quad4` + `Line2`/`Line3`), computes `MeshQualityReport`, and prepares `ConformalMeshHandoff` for Mode 1 solvers. | `ContinuumDomain`, `ConformalMeshHandoff`, `PslgBuilderService`, `ConformalDelaunayMesherService` |

---

## 3. Canonical Commands & Queries Summary

| ID | Name | Kind | Service | Mutates Revision? |
| --- | --- | --- | --- | --- |
| **UC-CMD-01** | `UpsertOuterBoundaryLoop` | Command | `ContinuumSketchApplicationService` | Yes (`revision++`) |
| **UC-CMD-02** | `AddOrUpdateDomainCutout` | Command | `ContinuumSketchApplicationService` | Yes (`revision++`) |
| **UC-CMD-03** | `RemoveDomainCutout` | Command | `ContinuumSketchApplicationService` | Yes (`revision++`) |
| **UC-CMD-04** | `AddOrUpdateInclusionRegion` | Command | `ContinuumSketchApplicationService` | Yes (`revision++`) |
| **UC-CMD-05** | `AssignBoundarySegmentTagAndCondition` | Command | `ContinuumSketchApplicationService` | Yes (`revision++`) |
| **UC-CMD-06** | `UpdateMeshingProfile` | Command | `ContinuumSketchApplicationService` | Yes (`revision++`) |
| **UC-CMD-07** | `GenerateConformalContinuumMesh` | Command | `ContinuumMeshingApplicationService` | No (binds to current `revision`) |
| **UC-CMD-08** | `HandoffMeshToContinuumSolver` | Command | `ContinuumMeshingApplicationService` | No (verifies `revision`) |
| **UC-QRY-01** | `ValidateContinuumDomain` | Query | `ContinuumSketchApplicationService` | No |
| **UC-QRY-02** | `PreviewCandidateSegmentIntersection` | Query | `ContinuumSketchApplicationService` | No |
| **UC-QRY-03** | `InspectMeshQualityAndProvenance` | Query | `ContinuumMeshingApplicationService` | No |

---

## 4. Detailed Command Specifications

### 4.1 UC-CMD-01: `UpsertOuterBoundaryLoop`
- **Intent:** Define or update the closed outer polygon `BoundaryLoop` (either from interactive vertex placement or a quick-rectangle helper), normalize its winding to CCW, assign default semantic boundary tags (`Bottom`, `Right`, `Top`, `Left` for axis-aligned edges or `Outer_i`), and validate against existing cutouts.
- **Inputs:**
  - `domain_id: String`
  - `vertices: Vec<Vertex2D>` ($N \ge 3$)
  - `auto_close: bool` (true to connect $V_N \to V_1$ and normalize CCW winding)
  - `segment_tags: Option<Vec<SemanticBoundaryTag>>`
- **Preconditions:**
  - $N \ge 3$ vertices with finite coordinates in metres.
- **Application Responsibilities:**
  1. Check edge lengths ($\ell_i \ge 10^{-4}\text{ m}$) and non-self-intersection (`INV-CONT-01`, `INV-CONT-02`).
  2. Compute signed area $A$; if $|A| < 10^{-6}\text{ m}^2$, reject with `DEGENERATE_POLYGON_AREA`. If $A < 0$, reverse vertex order to enforce CCW winding ($A > 0$).
  3. Recompute outward unit normals $\mathbf{n}_i$ for all `BoundarySegment` edges.
  4. If the domain already contains `cutouts` or `inclusions`, run `ContinuumTopologyValidator` to check whether all interior features remain strictly inside the new outer boundary (`INV-CONT-03`).
  5. Increment `ContinuumDomain.revision` and mark any cached `ConformalMeshHandoff` as `Stale`.
- **Outputs:** Updated `ContinuumDomain` snapshot + `Vec<DomainValidationDiagnostic>`.

### 4.2 UC-CMD-02: `AddOrUpdateDomainCutout`
- **Intent:** Insert or modify an interior `Cutout` (`Circle`, `Rectangle`, or `Polygon`) inside a `ClosedValid` outer boundary.
- **Inputs:**
  - `domain_id: String`
  - `cutout: Cutout` (`id`, `shape`, `tag`, `local_element_size`)
- **Preconditions:**
  - `outer_boundary.closure_state == ClosedValid`.
- **Application Responsibilities:**
  1. Validate the cutout's intrinsic geometry (for `Circle`: $R \ge 10^{-3}\text{ m}$, `angular_divisions` multiple of 4 $\ge 12$; for `Rectangle`: $w, h \ge 10^{-3}\text{ m}$; for `Polygon`: simple closed polygon normalized to CW winding $A < 0$).
  2. Verify strict containment inside `outer_boundary` (`INV-CONT-03`); if any point lies on or outside `outer_boundary`, reject with `HOLE_OUTSIDE_OUTER_BOUNDARY`.
  3. Verify mutual disjointness against all other `cutouts` and `inclusions`; if overlapping, reject with `HOLE_OVERLAPS_EXISTING_FEATURE`.
  4. Verify minimum Euclidean clearance $d \ge d_{\min} = \max(10^{-3}\text{ m}, 0.1 h \alpha_{\text{hole}})$; if violated, reject with `CLEARANCE_BELOW_MINIMUM`.
  5. Upsert the cutout, ensure default tag `"Hole"` has a `BoundaryConditionSpec::Free` entry if not already configured, and increment `ContinuumDomain.revision`.
- **Outputs:** Updated `ContinuumDomain` snapshot.

### 4.3 UC-CMD-03: `RemoveDomainCutout`
- **Intent:** Delete an interior `Cutout` (or `InclusionRegion`) by ID and re-evaluate domain validity.
- **Inputs:** `domain_id: String`, `feature_id: String`.
- **Outputs:** Updated `ContinuumDomain` snapshot with incremented `revision`.

### 4.4 UC-CMD-04: `AddOrUpdateInclusionRegion`
- **Intent:** Insert or update an interior solid `InclusionRegion` with custom material stiffness override ($E, \nu$) and internal conformal interface edges.
- **Inputs:** `domain_id: String`, `inclusion: InclusionRegion`.
- **Preconditions:** `outer_boundary.closure_state == ClosedValid`, $E > 0$, $-1 < \nu < 0.5$.
- **Application Responsibilities:**
  1. Validate intrinsic shape geometry and normalize orientation to CCW.
  2. Enforce `INV-CONT-03` containment inside `outer_boundary`, disjointness from all `Cutout` voids, and clearance $d \ge d_{\min}$.
  3. Upsert inclusion and increment `ContinuumDomain.revision`.
- **Outputs:** Updated `ContinuumDomain` snapshot.

### 4.5 UC-CMD-05: `AssignBoundarySegmentTagAndCondition`
- **Intent:** Assign a `SemanticBoundaryTag` (`Left`, `Right`, `Top`, `Bottom`, `Hole`, or custom) and/or local mesh size $h_{\text{local}}$ to specific `BoundarySegment` or `Cutout` IDs, and bind a `BoundaryConditionSpec` (`DirichletDisplacement`, `NeumannTraction`, `RobinImpedance`, `Free`) to that tag.
- **Inputs:**
  - `domain_id: String`
  - `target_ids: Vec<String>` (segment IDs `S1..Sn` or cutout IDs `H1..Hm`)
  - `tag: SemanticBoundaryTag`
  - `condition: BoundaryConditionSpec`
  - `local_element_size: Option<f64>`
- **Application Responsibilities:**
  1. Validate that all `target_ids` exist in `ContinuumDomain`.
  2. Validate physical parameters (finite traction magnitudes, non-negative Robin stiffnesses $k_n, k_s \ge 0$ and damping $c_n, c_s \ge 0$, positive $h_{\text{local}} > 0$ when provided).
  3. Update segment/cutout tags and `tag_conditions` map; increment `ContinuumDomain.revision`.
- **Outputs:** Updated `ContinuumDomain` snapshot.

### 4.6 UC-CMD-06: `UpdateMeshingProfile`
- **Intent:** Configure target element size $h$, hole refinement factor $\alpha_{\text{hole}}$, minimum angle $\theta_{\min}$, volume element type (`Tri3`/`Quad4`), and boundary element type (`Line2`/`Line3`).
- **Inputs:** `domain_id: String`, `meshing_profile: MeshingProfile`.
- **Application Responsibilities:**
  1. Validate $h > 0$, $\alpha_{\text{hole}} \in [0.1, 1.0]$, and $\theta_{\min} \in [15.0, 32.0]$.
  2. Re-run `ContinuumTopologyValidator` because changing $h \cdot \alpha_{\text{hole}}$ updates $d_{\min}$.
  3. Increment `ContinuumDomain.revision`.
- **Outputs:** Updated `ContinuumDomain` snapshot.

### 4.7 UC-CMD-07: `GenerateConformalContinuumMesh`
- **Intent:** Execute Constrained Delaunay Triangulation on a validated `ContinuumDomain` to produce a `ConformalMeshHandoff` containing `MeshData` and `MeshQualityReport`.
- **Inputs:** `domain: ContinuumDomain`.
- **Preconditions:** `domain` passes `ContinuumTopologyValidator` (`status == ValidForMeshing`).
- **Application Responsibilities:**
  1. Run `ContinuumTopologyValidator`; if any invariant fails, abort and return the structured validation error.
  2. Invoke `PslgBuilderService` to discretize the outer loop, cutout loops, and inclusion loops into constrained segments with graded edge lengths ($h_{\text{hole}} = \alpha_{\text{hole}} h$ on cutouts, transitioning smoothly to $h$ in the interior).
  3. Invoke `ConformalDelaunayMesherService` to triangulate the interior, remove exterior and `Cutout` void triangles, retain `InclusionRegion` triangles with subdomain provenance, and extract `BoundaryElement` records (`Line2` or `Line3`) directly from boundary-adjacent `VolumeElement` edges with exact outward unit normals (`INV-CONT-04`).
  4. Verify `is_conformal == true` (every boundary element shares exact node indices with an adjacent volume element) and `min_angle_deg > 0.0`.
  5. Return `ConformalMeshHandoff` stamped with `domain_revision: domain.revision`.
- **Outputs:** `ConformalMeshHandoff`.

### 4.8 UC-CMD-08: `HandoffMeshToContinuumSolver`
- **Intent:** Hand off a `Current` `ConformalMeshHandoff` and its boundary condition mappings into the Mode 1 Continuum Workbench state so `run_static_simulation`, `run_harmonic_simulation`, or `run_transient_simulation` can execute immediately.
- **Inputs:** `domain: ContinuumDomain`, `handoff: ConformalMeshHandoff`.
- **Preconditions:** `handoff.domain_revision == domain.revision` and `handoff.quality_report.is_conformal == true`.
- **Application Responsibilities:**
  1. If `handoff.domain_revision != domain.revision`, reject with `STALE_MESH_HANDOFF`.
  2. Convert traction values from $\text{kN/m}^2$ to SI solver units and populate the Mode 1 active `MeshData`.
- **Outputs:** Active Mode 1 solver configuration ready for execution.

---

## 5. Detailed Query Specifications

1. **UC-QRY-01: `ValidateContinuumDomain`:**
   - **Input:** `domain: ContinuumDomain`
   - **Output:** `DomainValidationResult { status: DomainValidationStatus, outer_signed_area_m2: f64, diagnostics: Vec<DomainValidationDiagnostic> }`
2. **UC-QRY-02: `PreviewCandidateSegmentIntersection`:**
   - **Input:** `existing_vertices: Vec<Vertex2D>`, `candidate_point: Vertex2D`, `closing_loop: bool`
   - **Output:** `CandidateSegmentPreview { is_valid: bool, intersecting_segment_indices: Vec<usize>, snap_to_start: bool, projected_signed_area_m2: Option<f64> }`
3. **UC-QRY-03: `InspectMeshQualityAndProvenance`:**
   - **Input:** `handoff: ConformalMeshHandoff`
   - **Output:** `MeshQualityReport` + per-tag `BoundaryElement` counts (e.g., `Left: 20, Right: 20, Top: 20, Bottom: 20, Hole: 32`).

---

## 6. Transaction Boundaries, Idempotency & Failure Model

- **Transaction Boundaries:** Each command operates on a single `ContinuumDomain` aggregate instance in memory (and via stateless Rust IPC calls where the full `ContinuumDomain` DTO is validated and transformed deterministically).
- **Idempotency:** `ValidateContinuumDomain` and `GenerateConformalContinuumMesh` are pure, deterministic functions of the input `ContinuumDomain` (including a fixed deterministic Delaunay point-insertion order); repeating `GenerateConformalContinuumMesh` for the same `ContinuumDomain` payload yields bit-identical `MeshData`.
- **Failure Categories:**
  - **Domain Topology Errors:** `OPEN_BOUNDARY_LOOP`, `SELF_INTERSECTING_LOOP`, `DEGENERATE_POLYGON_AREA`, `ZERO_LENGTH_SEGMENT`, `HOLE_OUTSIDE_OUTER_BOUNDARY`, `HOLE_OVERLAPS_EXISTING_FEATURE`, `CLEARANCE_BELOW_MINIMUM`.
  - **Meshing & Handoff Errors:** `INVALID_MESHING_PROFILE`, `CDT_CONVERGENCE_FAILURE`, `STALE_MESH_HANDOFF`.

---

## 7. End-to-End Use-Case Chains

### Chain 1: $10 \times 10\text{ m}$ Plate with Central $R = 1.5\text{ m}$ Hole $\to$ Mode 1 BEM-FEM Solve
1. `UpsertOuterBoundaryLoop`: User creates a $10 \times 10\text{ m}$ square centered at $(0, 0)$ with vertices $(-5,-5), (5,-5), (5,5), (-5,5)$ (`revision = 1`, `ClosedValid`, tags `Bottom`, `Right`, `Top`, `Left`).
2. `AddOrUpdateDomainCutout`: User adds `CircularCutout { center_x: 0.0, center_y: 0.0, radius: 1.5, angular_divisions: 32 }` with tag `"Hole"` (`revision = 2`, clearance $d = 5.0 - 1.5 = 3.5\text{ m} \ge d_{\min}$).
3. `AssignBoundarySegmentTagAndCondition`: User assigns `DirichletDisplacement` fixed support on `"Left"` and `NeumannTraction` tensile load on `"Right"` (`revision = 3`).
4. `GenerateConformalContinuumMesh`: Backend runs CDT with $h = 0.8\text{ m}, \alpha_{\text{hole}} = 0.35$, producing graded `Tri3` volume elements and matching `Line2` boundary elements (`Left`, `Right`, `Top`, `Bottom`, `Hole`) with `is_conformal == true` (`domain_revision = 3`).
5. `HandoffMeshToContinuumSolver`: `MeshData` is handed off to Mode 1 `run_static_simulation` (`SolverMode::BEMFEM`), which converges and returns displacement and von Mises stress fields.
