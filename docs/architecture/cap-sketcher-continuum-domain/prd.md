# Product Requirements Document: Continuum Domain Sketcher (`cap-sketcher-continuum-domain`)

- **Owning Capability:** [`cap-sketcher-continuum-domain`](../../../.okf/capabilities/cap-sketcher-continuum-domain.md)
- **Parent Capability:** [`cap-geometry-canvas-sketcher`](../../../.okf/capabilities/cap-geometry-canvas-sketcher.md)
- **Shared Dependencies:** [`cap-geometry-presets`](../../../.okf/capabilities/cap-geometry-presets.md), [`cap-solver-core`](../../../.okf/capabilities/cap-solver-core.md), [`cap-material-field`](../../../.okf/capabilities/cap-material-field.md), [`cap-interactive-canvas`](../../../.okf/capabilities/cap-interactive-canvas.md), [`cap-project-persistence`](../../../.okf/capabilities/cap-project-persistence.md)
- **Root Synthesis Alignment:** [`docs/prd.md`](../../prd.md), [`application-architecture-summary.md`](../application-architecture-summary.md), [`cross-capability-conventions.md`](../cross-capability-conventions.md)
- **Status:** Specified

---

## 1. Purpose & Product Summary

The **Continuum Domain Sketcher** (`cap-sketcher-continuum-domain`) enables engineers, instructors, and researchers to interactively draw arbitrary 2D planar solid domains—such as irregular plates, shear walls, notched blocks, and multi-perforated panels with circular, rectangular, or polygonal cutouts and material inclusion zones—tag their boundaries with physical boundary conditions, validate topological integrity, and discretize them via Constrained Delaunay Triangulation (CDT) into conformal 2D FEM volume meshes (`Tri3`/`Quad4`) and matching 1D BEM boundary meshes (`Line2`/`Line3`).

Previously, Mode 1 (Continuum Workbench) was restricted to hardcoded parametric templates (`Beam2D`, `PlateWithHole2D`). This capability removes that restriction while preserving 100% compatibility with the `MeshData` and semantic boundary tagging contracts of `cap-geometry-presets` and `cap-solver-core`.

---

## 2. Goals & Non-Goals

### 2.1 Goals
1. **Arbitrary 2D Outer Boundary Sketching:** Allow users to define closed 2D polygonal domains via interactive canvas clicking (with grid/vertex/ortho/angle snapping) or exact coordinate entry $(x, y)$ / $(L, \theta)$, with automatic loop closure and real-time self-intersection prevention.
2. **Interior Cutouts & Material Inclusions:** Support placing circular, rectangular, and polygonal interior cutouts (holes/cavities) as well as solid material inclusion subdomains inside a closed outer boundary, enforcing strict containment and minimum clearance invariants.
3. **Semantic & Physical Boundary Tagging:** Allow every outer or cutout boundary segment to carry a semantic tag (`Left`, `Right`, `Top`, `Bottom`, `Hole`, or custom name) and physical boundary condition metadata (Dirichlet displacement restraint, Neumann surface traction, Robin/absorbing impedance, or Free).
4. **Conformal CDT Mesh Generation & Solver Handoff:** Triangulate validated domains using quality Constrained Delaunay Triangulation (Ruppert/Shewchuk refinement) with target mesh size $h$ and local hole refinement, producing `MeshData` where exterior and cavity FEM edges match BEM boundary elements 1-to-1 with exact outward unit normals, ready for immediate solving in Mode 1 (`PureBEM`, `PureFEM`, `BEMFEM`).

### 2.2 Non-Goals
1. **1D Skeletal Frame/Truss/Cable Sketching:** Handled exclusively by sibling capability `cap-sketcher-frame-modeler` and `cap-frame-analysis-engine`.
2. **3D Solid CSG or Extrusion Modeling:** 3D continuum benchmarks remain governed by `cap-geometry-presets` (`Beam3D`, `BlockWithCavity3D`) and `cap-geometry-cad-import` (STL).
3. **Freeform NURBS Lofting or Parametric Dimensional Constraint Solving:** Curved boundaries are limited to exact circular cutouts and regular polygonal approximations on the Planar Straight-Line Graph (PSLG).
4. **External CAD File Parsing (DXF/SVG):** Owned by `cap-geometry-cad-import` (which may later convert imported 2D loops into `ContinuumDomain` entities).

---

## 3. Actors & User Journeys

| Actor | Primary Goal in Capability |
| --- | --- |
| **Engineer / Instructor** | Sketch a custom 2D structural continuum plate (e.g., an L-shaped bracket or a $10 \times 10\text{ m}$ square plate with an $R = 1.5\text{ m}$ central hole), assign boundary restraints and tractions, inspect the graded conformal mesh, and visualize stress concentrations in Mode 1. |
| **Researcher** | Construct multi-cavity or inclusion-embedded 2D domains with controlled mesh grading around holes ($h_{\text{local}}$) to benchmark hybrid BEM-FEM iterative acceleration (`Anderson`, `Relaxation`) and harmonic Robin transmission solvers. |

---

## 4. Required Workflows

### 4.1 Workflow 1: Sketching & Closing the Outer Polygon Boundary
1. User enters the **2D Continuum Sketcher** drawer/canvas in Mode 1 (or selects "Custom Sketched Domain" in the Geometry panel).
2. User selects the **Outer Polygon Tool** and places vertices $V_1, V_2, \dots, V_n$ in metres via canvas clicks (snapping to grid, ortho, or numeric coordinates) or a quick-rectangle helper.
3. As the user previews the next candidate segment $V_n \to P_{\text{cursor}}$, the sketcher checks whether the candidate segment intersects any existing non-adjacent segment $V_i V_{i+1}$. If an intersection occurs, the candidate edge highlights red with a bilingual warning (`SELF_INTERSECTING_LOOP`).
4. When $n \ge 3$ and the user clicks within the snap radius of $V_1$, double-clicks $V_n$, or invokes **Close Loop**, the system closes $V_n \to V_1$, normalizes the vertex winding to counter-clockwise (CCW, signed area $A > 0$), auto-assigns default semantic tags (`Left`, `Right`, `Top`, `Bottom` for axis-aligned edges, or `Outer_i` otherwise), and transitions the outer loop to `ClosedValid`.

### 4.2 Workflow 2: Adding Interior Cutouts (Circular, Rectangular, Polygonal) & Inclusions
1. Given a `ClosedValid` outer boundary, the user selects **Add Cutout** (`Circle`, `Rectangle`, or `Polygon`) or **Add Inclusion Region**.
2. For a **Circular Cutout**, the user specifies center $(c_x, c_y)$, radius $R > 0$, and optional angular divisions $n_{\theta}$ (default 32, multiple of 4).
3. For a **Rectangular Cutout**, the user specifies center/corner, width $w > 0$, height $h > 0$, and optional rotation angle.
4. For a **Polygonal Cutout**, the user draws a closed inner polygon loop.
5. The system validates that:
   - The cutout/inclusion loop is simple and non-degenerate.
   - Every point of the cutout/inclusion lies strictly inside the outer `BoundaryLoop` with minimum clearance $d \ge d_{\min}$.
   - The cutout does not overlap or touch any existing `Cutout` or `InclusionRegion` closer than $d_{\min}$.
6. Valid cutouts are oriented clockwise (CW, winding number $-1$) so their outward normals point into the cavity (away from material), and default to semantic tag `Hole`.

### 4.3 Workflow 3: Tagging Boundary Segments & Assigning Boundary Conditions
1. User selects one or more `BoundarySegment` edges (or an entire `Cutout` loop) on the canvas.
2. In the **Boundary Inspector**, the user assigns:
   - **Semantic Tag:** `Left`, `Right`, `Top`, `Bottom`, `Hole`, or `Custom(name)`.
   - **Boundary Condition Type:**
     - `DirichletDisplacement`: prescribed $(u_x, u_y)$ in metres (with independent boolean locks `fix_x`, `fix_y`, default $(0, 0)$ for fixed support).
     - `NeumannTraction`: surface traction vector $(t_x, t_y)$ in $\text{kN/m}^2$ in global or local $(n, s)$ coordinates.
     - `RobinImpedance`: normal/tangential spring stiffness $(k_n, k_s)$ in $\text{kN/m}^3$ and damping impedance $(c_n, c_s)$.
     - `Free`: traction-free ($\mathbf{t} = \mathbf{0}$).
   - **Local Mesh Size Override ($h_{\text{local}}$):** optional refinement size along that segment or hole perimeter.

### 4.4 Workflow 4: Conformal CDT Meshing & Mode 1 Solver Handoff
1. User configures the `MeshingProfile`: target global element size $h$ (m), hole refinement factor $\alpha_{\text{hole}} \in [0.1, 1.0]$ (default $0.35$), minimum triangle angle $\theta_{\min}$ (default $25^\circ$), interior element type (`Tri3` or `Quad4`), and boundary element type (`Line2` or `Line3`).
2. User clicks **Generate Conformal Mesh**.
3. The Rust backend validates the `ContinuumDomain`, constructs the constrained Planar Straight-Line Graph (PSLG), inserts Steiner points along boundaries and interior according to local feature size and Chew/Ruppert/Shewchuk Delaunay refinement, removes exterior and hole triangles using exact winding-number classification, and extracts `BoundaryElement` records directly from the boundary-adjacent `VolumeElement` edges.
4. The UI renders the conformal 2D mesh overlay and displays the `MeshQualityReport` (node count, `VolumeElement` count, `BoundaryElement` count, min angle, `is_conformal: true`).
5. User clicks **Send to Mode 1 Solver** (or directly runs Static / Harmonic / Transient simulation in Mode 1), passing the generated `MeshData` into `cap-solver-core`.

---

## 5. Business Rules & Geometric Invariants

| Rule ID | Rule Name | Specification |
| --- | --- | --- |
| **BR-CONT-01** | **Automatic Loop Closure & Winding Normalization** | An outer `BoundaryLoop` requires $N \ge 3$ distinct vertices. Upon closure, if signed area $A = \frac{1}{2}\sum (x_i y_{i+1} - x_{i+1} y_i) < 0$, vertex order is reversed so the outer loop is strictly CCW ($A > 0$). Every `Cutout` loop is normalized to CW ($A < 0$). |
| **BR-CONT-02** | **Simple Polygon (No Self-Intersection)** | No two non-adjacent segments of any `BoundaryLoop` may intersect or overlap, and no two adjacent segments may be collinearly folded ($180^\circ$ spike). Minimum edge length is $\ell_{\min} = 10^{-4}\text{ m}$. Minimum absolute loop area is $A_{\min} = 10^{-6}\text{ m}^2$. |
| **BR-CONT-03** | **Strict Cutout Containment & Clearance** | Every `Cutout` and `InclusionRegion` must lie strictly inside the outer `BoundaryLoop` and outside all other cutouts. The minimum Euclidean distance between any vertex/edge of a cutout and the outer boundary (or another cutout) must satisfy $d \ge d_{\min} = \max(10^{-3}\text{ m}, 0.1 h_{\min})$, where $h_{\min} = h \cdot \alpha_{\text{hole}}$. |
| **BR-CONT-04** | **Strict BEM-FEM Topological Conformity** | Every exterior and cutout `BoundaryElement` (`Line2` or `Line3`) in the generated `MeshData` must share the exact node indices $(n_a, n_b)$ (and midpoint $n_m$ for `Line3`) of the corresponding boundary edge of the adjacent interior `VolumeElement` (`Tri3` or `Quad4`). Zero hanging nodes are permitted on any boundary or inclusion interface. |
| **BR-CONT-05** | **Outward Unit Normal Convention** | Every `BoundaryElement` must carry an outward unit normal $\mathbf{n} = [n_x, n_y, 0.0]$ pointing *away from the solid continuum material*: outward toward infinity on the outer loop, and inward toward the hole center on `Cutout` loops. |
| **BR-CONT-06** | **Revision Tracking & Stale Mesh Prevention** | Every mutation to vertices, cutouts, inclusions, boundary tags, or `MeshingProfile` increments `ContinuumDomain.revision`. Any previously generated `ConformalMeshHandoff` with `domain_revision < ContinuumDomain.revision` is marked `Stale` and cannot be solved until regenerated. |
| **BR-CONT-07** | **Unit & Sign Alignment** | Coordinates and mesh sizes are in metres ($\text{m}$). Boundary tractions are entered in $\text{kN/m}^2$ and converted to SI Pascals ($\text{N/m}^2$) or total load magnitude when handed off to `cap-solver-core`. All user-facing validation errors and inspector labels are bilingual (`el`/`en`). |

---

## 6. Functional & Non-Functional Requirements

### 6.1 Functional Requirements
- **FR-01:** Provide interactive 2D polygon and rectangle outer-boundary sketching with grid snap, vertex snap, midpoint snap, orthogonal lock, and exact coordinate input $(x, y)$ / $(\Delta x, \Delta y)$ / $(L, \theta)$.
- **FR-02:** Provide parametric and freeform interior cutout tools (`CircularCutout`, `RectangularCutout`, `PolygonalCutout`) and `InclusionRegion` tools with live containment/clearance preview.
- **FR-03:** Provide real-time client-side geometric validation preview and authoritative Rust backend validation returning structured bilingual errors (`SELF_INTERSECTING_LOOP`, `HOLE_OUTSIDE_OUTER_BOUNDARY`, `HOLE_OVERLAPS_EXISTING_FEATURE`, `CLEARANCE_BELOW_MINIMUM`, `DEGENERATE_POLYGON_AREA`).
- **FR-04:** Provide boundary segment tagging (`Left`, `Right`, `Top`, `Bottom`, `Hole`, custom) and boundary condition specification (`DirichletDisplacement`, `NeumannTraction`, `RobinImpedance`, `Free`).
- **FR-05:** Provide Constrained Delaunay Triangulation in Rust generating graded `Tri3` (and recombined `Quad4`) volume elements and conformal `Line2`/`Line3` boundary elements with `MeshQualityReport`.
- **FR-06:** Provide 1-click handoff of the generated `MeshData` into Mode 1 (`SimulationRequest`, `HarmonicSimulationRequest`, `TransientSimulationRequest`) and serialization under `ProjectDocument.continuum`.

### 6.2 Non-Functional Requirements
- **NFR-01 (Performance):** Geometric validation (`continuum_validate_domain`) completes in $< 5\text{ ms}$. Conformal CDT mesh generation (`continuum_generate_mesh`) for up to $10{,}000$ triangles completes in $< 250\text{ ms}$ in Rust.
- **NFR-02 (Mesh Quality):** Generated `Tri3` meshes must have strictly positive Jacobian determinants (CCW node ordering) and minimum interior angle $\ge 20^\circ$ unless constrained by user input corners sharper than $20^\circ$.
- **NFR-03 (Code Limits):** Every Rust and TypeScript file $\le 1000$ lines; every HTML template $\le 500$ lines; unit tests in companion `*_test.rs` / `*.spec.ts` files.

---

## 7. Acceptance Scenarios Summary

Full behavioral specifications are maintained in [`acceptance-scenarios.md`](acceptance-scenarios.md):
- **SC-001:** $10 \times 10\text{ m}$ square plate with central $R = 1.5\text{ m}$ circular hole validates and generates a graded conformal `Tri3` + `Line2` mesh.
- **SC-002:** Self-intersecting (bowtie) outer polygon loop is rejected with `SELF_INTERSECTING_LOOP`.
- **SC-003:** Cutout breaching or encroaching on the outer boundary clearance is rejected with `HOLE_OUTSIDE_OUTER_BOUNDARY` / `CLEARANCE_BELOW_MINIMUM`.
- **SC-004:** Overlapping or touching interior cutouts are rejected with `HOLE_OVERLAPS_EXISTING_FEATURE`.
- **SC-005:** Semantic boundary tagging and Dirichlet/Neumann/Robin assignment propagate accurately to all child `BoundaryElement` records.
- **SC-006:** Material `InclusionRegion` preserves conformal internal mesh interfaces without generating spurious BEM cavity elements.
- **SC-007:** End-to-end handoff of sketched $10 \times 10\text{ m}$ perforated plate into Mode 1 `BEMFEM` solver converges and renders stress fields.
- **SC-008:** Domain mutation increments `revision` and invalidates stale mesh handoff until re-meshing.

---

## 8. Explicit Assumptions & Artifact-Impact Classification

- **Assumptions:**
  1. Mode 1 solver compatibility uses the existing `MeshData` struct (`dimension: 2`, `nodes: Vec<[f64; 3]>`, `boundary_elements: Vec<BoundaryElement>`, `volume_elements: Vec<VolumeElement>`, `stats: MeshStats`) without breaking existing presets.
  2. When custom boundary conditions (`BoundaryConditionSpec`) are defined on custom tags, the sketcher also supports mapping standard preset tags (`Left`, `Right`, `Top`, `Bottom`, `Hole`) so existing Mode 1 solver routines work out-of-the-box, while exposing the full tag-to-BC map for solver extensions.
- **Application Synthesis Impact:** `no impact` on `docs/prd.md` and `docs/architecture/application-architecture-summary.md` — both root synthesis documents already catalog `cap-sketcher-continuum-domain` as the polygon/hole sketching $\to$ CDT mesh child of `cap-geometry-canvas-sketcher`.
