# Domain Glossary: Continuum Domain Sketcher (`cap-sketcher-continuum-domain`)

- **Owning Capability:** [`cap-sketcher-continuum-domain`](../../../.okf/capabilities/cap-sketcher-continuum-domain.md)
- **Parent Capability:** [`cap-geometry-canvas-sketcher`](../../../.okf/capabilities/cap-geometry-canvas-sketcher.md)
- **Shared Dependencies:** [`cap-geometry-presets`](../../../.okf/capabilities/cap-geometry-presets.md), [`cap-solver-core`](../../../.okf/capabilities/cap-solver-core.md), [`cap-material-field`](../../../.okf/capabilities/cap-material-field.md)
- **Binding Conventions:** [`cross-capability-conventions.md`](../cross-capability-conventions.md)

---

## 1. Purpose

This glossary stabilizes the domain vocabulary for sketching arbitrary 2D continuum regions, defining interior cutouts and material inclusions, tagging boundary segments for boundary value problems, validating planar topological constraints, and executing Constrained Delaunay Triangulation (CDT) handoff into conformal FEM/BEM meshes (`MeshData`) for the Mode 1 Continuum Workbench.

---

## 2. Canonical Terms

### 2.1 Actors

| Term | Category | Definition | Aliases / Discouraged | Distinctions |
| --- | --- | --- | --- | --- |
| **Engineer / Instructor** | Actor | User who sketches custom 2D continuum plates, walls, or perforated domains with holes and boundary conditions for stress analysis or classroom demonstration. | User, Modeler, Designer | Distinct from Mode 2 frame modeling; operates on 2D planar continuum regions rather than 1D skeletal members. |
| **Researcher** | Actor | User who designs non-standard 2D continuum geometries with cavities or inclusions to test hybrid BEM-FEM elastodynamic solvers beyond standard parametric presets. | Analyst, Scientist | Uses custom sketched domains in place of fixed `MeshGeneratorRequest` presets while retaining full solver compatibility. |

### 2.2 Business Objects & Geometric Entities

| Term | Category | Definition | Aliases / Discouraged | Distinctions |
| --- | --- | --- | --- | --- |
| **ContinuumDomain** | Business Object (Aggregate Root) | Complete 2D planar solid region specification owned by `cap-sketcher-continuum-domain`, comprising one closed outer `BoundaryLoop`, zero or more disjoint interior `Cutout` definitions, zero or more `InclusionRegion` definitions, boundary segment tags, and a `MeshingProfile`, carrying a monotonically increasing `revision`. | SketchDomain, PlateModel, PolygonModel | Strictly separate from `StructuralModel` (owned by `cap-frame-analysis-engine`). Serialized under the `continuum` section of `ProjectDocument`. |
| **BoundaryLoop** | Business Object (Entity) | Ordered sequence of 2D vertices (`Vertex2D`, $N \ge 3$) and connecting `BoundarySegment` edges forming a closed, non-self-intersecting simple polygon in metres. The outer loop is oriented counter-clockwise (CCW, winding number $+1$); interior hole loops are oriented clockwise (CW, winding number $-1$). | Contour, Polyline, Wire, Path | Not an open polyline during meshing; must be topologically closed and simple before CDT execution. |
| **Vertex2D** | Business Object (Value Object) | Planar coordinate point $(x, y)$ in metres with a stable domain-scoped string identifier (e.g., `V1`, `V2`). | Point, Corner, SketchNode | Never called `Node` in the sketcher domain to avoid collision with `StructuralModel` `Node` (`N1`) or discretized `MeshData` vertex indices. |
| **BoundarySegment** | Business Object (Entity) | Directed line (or discretized arc) edge connecting two consecutive vertices $(V_i, V_{i+1})$ along a `BoundaryLoop`, carrying a semantic boundary `tag`, an explicit outward unit normal $\mathbf{n} = (n_x, n_y, 0)$, a `BoundaryConditionSpec`, and an optional local edge refinement size $h_{\text{local}}$. | Edge, Side, Line, Wall | Distinct from `BoundaryElement` (`Line2`/`Line3`), which is one discretized mesh facet generated along a `BoundarySegment`. |
| **Cutout** | Business Object (Entity) | Interior void (cavity/hole) completely contained within the outer `BoundaryLoop` of a `ContinuumDomain`. Supported shapes: `CircularCutout` $(c_x, c_y, R, n_{\theta})$, `RectangularCutout` $(x_0, y_0, w, h, \theta_{\text{rot}})$, and `PolygonalCutout` (closed CW `BoundaryLoop`). | Hole, Void, Cavity, Perforation | Removes material completely from the interior and generates inward-facing BEM boundary elements (`normal` pointing into the cavity, away from material). Distinct from `InclusionRegion`. |
| **InclusionRegion** | Business Object (Entity) | Closed interior sub-region (`Circular`, `Rectangular`, or `Polygonal`) completely contained within the outer `BoundaryLoop` and disjoint from `Cutout` voids, across whose interface conformal mesh edges are enforced and a subdomain material override or inhomogeneity tag is assigned. | Subdomain, MaterialPatch, EmbeddedZone | Retains interior volume elements (meshed on both sides of the internal interface) and does *not* generate exterior BEM boundary elements unless explicitly solved as a multi-region interface. |
| **BoundaryConditionSpec** | Business Object (Value Object) | Physical boundary condition metadata attached to a `BoundarySegment` or named semantic `tag`: `Free` (homogeneous Neumann $\mathbf{t} = \mathbf{0}$), `DirichletDisplacement` (prescribed $u_x, u_y$ in m or fixed restraints), `NeumannTraction` (surface traction $\mathbf{t} = (t_x, t_y)$ in $\text{kN/m}^2$, converted to $\text{N/m}^2$ for solver core), or `RobinImpedance` (spring/radiation impedance $k_n, k_s, c_n, c_s$). | Support, Load, Restraint | Distinct from discrete nodal `Support` and `Load` in Mode 2 frames; acts continuously along 1D boundary edges of the 2D continuum. |
| **SemanticBoundaryTag** | Business Object (Value Object) | Canonical string label attached to `BoundarySegment` entities and propagated to every child `BoundaryElement.tag` in `MeshData`. Includes standard preset-compatible tags (`Left`, `Right`, `Top`, `Bottom`, `Hole`) and custom user tags (`Custom(String)`). | GroupLabel, PhysicalGroup | Ensures 100% compatibility with `cap-geometry-presets` and `cap-solver-core` boundary condition lookup routines. |
| **MeshingProfile** | Business Object (Value Object) | Discretization control parameters for Constrained Delaunay Triangulation: target global element size $h > 0$ (m), minimum angle threshold $\theta_{\min} \in [20^\circ, 33^\circ]$, hole boundary refinement factor $\alpha_{\text{hole}} \in (0, 1]$, interior element topology (`Tri3` or `Quad4` via triangle recombination), and boundary element order (`Line2` or `Line3`). | MeshSettings, TessellationConfig | Controls the CDT algorithm and boundary subdivision; changing `MeshingProfile` increments the `ContinuumDomain` `revision`. |
| **ConformalMeshHandoff** | Business Object (Aggregate / Read Model) | Result package produced by CDT meshing of a validated `ContinuumDomain` at a specific `revision`, containing the `cap-solver-core`-compatible `MeshData` (`nodes`, `boundary_elements`, `volume_elements`, `stats`), `MeshQualityReport`, and segment-to-element provenance mapping. | GeneratedMesh, CDTOutput | Serves as the direct input payload to Mode 1 `SimulationRequest`, `HarmonicSimulationRequest`, and `TransientSimulationRequest`. |
| **MeshQualityReport** | Metric / Reporting Term | Quantitative summary of the generated conformal mesh: element counts, minimum/maximum interior angle (deg), minimum/maximum scaled Jacobian / aspect ratio, and a strict boolean `is_conformal` topological verification flag. | MeshDiagnostics | Exposed in the UI and CLI so users can verify mesh grading around holes before launching expensive BEM-FEM iterations. |

### 2.3 Workflows, Policies & Rules

| Term | Category | Definition | Aliases / Discouraged | Distinctions |
| --- | --- | --- | --- | --- |
| **Planar Straight-Line Graph (PSLG)** | Policy / Rule Object | Flattened collection of deduplicated vertices and non-intersecting constrained segments (outer boundary in CCW order, cutout loops in CW order, and inclusion interfaces) constructed from `ContinuumDomain` as the direct geometric input to Constrained Delaunay Triangulation. | ConstraintGraph, Wireframe | Curved `CircularCutout` boundaries are discretized into $n_{\theta}$ regular polygon segments (default $\ge 24$, multiple of 4) when building the PSLG. |
| **Simple Polygon Invariant** | Policy / Rule | Hard geometric rule requiring every `BoundaryLoop` to have zero non-adjacent edge intersections, zero zero-length edges ($\ell \ge \varepsilon_{\text{len}} = 10^{-4}\text{ m}$), and non-zero signed area ($|A| \ge 10^{-6}\text{ m}^2$). | Non-Self-Intersection Rule | Checked both interactively during vertex placement and authoritatively at the Rust backend boundary. |
| **Feature Clearance Invariant** | Policy / Rule | Hard geometric rule requiring every interior `Cutout` and `InclusionRegion` to lie strictly inside the outer `BoundaryLoop` and remain mutually disjoint from other cutouts by at least a minimum clearance distance $d_{\min} = \max(10^{-3}\text{ m}, 0.1 h_{\min})$. | Hole Margin Rule, Containment Rule | Prevents degenerate sliver triangles or topological pinching between a hole and the exterior boundary. |
| **Conformal Boundary-Volume Matching** | Policy / Rule | Topological invariant guaranteeing that every `BoundaryElement` (`Line2` or `Line3`) in `MeshData` shares the exact node indices of the exterior or hole-boundary edge of an adjacent `VolumeElement` (`Tri3` or `Quad4`), with zero hanging nodes and outward unit normals pointing away from the solid material. | Topological Conformity | Matches the exact contract of `cap-geometry-presets` so `cap-solver-core` hybrid BEM-FEM coupling matrices assemble without interpolation gaps. |

### 2.4 States & Statuses

| Term | Category | Definition | Aliases / Discouraged | Distinctions |
| --- | --- | --- | --- | --- |
| **LoopClosureState** | State / Status | Lifecycle state of a sketched `BoundaryLoop`: `OpenDraft` (fewer than 3 vertices or last vertex not yet closed to first), `ClosedValid` (closed simple polygon with valid winding), or `ClosedInvalid` (closed loop violating self-intersection or collinearity rules). | PathStatus | Only a domain whose loops are all `ClosedValid` can transition to meshing. |
| **DomainValidationStatus** | State / Status | Aggregate readiness state of a `ContinuumDomain`: `Incomplete` (outer boundary missing or `OpenDraft`), `InvalidGeometry` (violates simple polygon or clearance invariants), or `ValidForMeshing` (all loops closed, oriented, and clearance-verified). | SketchStatus | Determines whether the "Generate Conformal Mesh" action is enabled. |
| **MeshHandoffState** | State / Status | Lifecycle status of the mesh generation output relative to `ContinuumDomain.revision`: `Unmeshed`, `MeshingInProgress`, `Current` (mesh revision matches domain revision), or `Stale` (domain mutated since last mesh generation). | MeshStatus | Enforces the shared convention that stale meshes are never silently dispatched to `cap-solver-core`. |

---

## 3. Critical Semantic Distinctions

1. **`ContinuumDomain` vs. `StructuralModel`:**
   - `ContinuumDomain` (owned by `cap-sketcher-continuum-domain`) models 2D planar solid regions bounded by closed polygon loops and interior cutouts, discretized into 2D `VolumeElement` (`Tri3`/`Quad4`) and 1D `BoundaryElement` (`Line2`/`Line3`) meshes for Mode 1 BEM/FEM/hybrid solvers.
   - `StructuralModel` (owned by `cap-frame-analysis-engine`) models 1D skeletal members (`FrameMember`, `TrussBar`, `Spring`, `Cable`) connected at discrete `Node` points for Mode 2 DSM and Mode 3 classical teaching.
2. **Outer `BoundaryLoop` vs. Interior `Cutout` Normal Orientation:**
   - Outer boundary segments traverse counter-clockwise (CCW); their outward unit normal $\mathbf{n}$ points *away* from the enclosed solid toward the exterior infinity.
   - Interior `Cutout` boundary segments traverse clockwise (CW); their outward unit normal $\mathbf{n}$ points *into the hole* (away from the solid continuum material), matching the exact convention in `generate_plate_with_hole_2d` (`normal: [-cos(θ), -sin(θ), 0.0]`).
3. **`Cutout` vs. `InclusionRegion`:**
   - A `Cutout` is a material void: CDT deletes triangles inside the cutout loop and creates `BoundaryElement` records along its perimeter.
   - An `InclusionRegion` is a solid subdomain: CDT preserves constrained edges along its perimeter and triangulates its interior with `VolumeElement` records without creating exterior BEM `BoundaryElement` records on the internal interface.

---

## 4. Open Terminology Issues

None. All terms align with `docs/architecture/cross-capability-conventions.md`, `src-tauri/src/mesh/types.rs`, and `src-tauri/src/solver/types.rs`.
