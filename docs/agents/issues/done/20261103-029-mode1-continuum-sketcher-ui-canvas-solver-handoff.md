## Issue Metadata

- Issue number: `029`
- Owning capability node: `/.okf/capabilities/cap-sketcher-continuum-domain.md`
- Artifact root: `docs/architecture/cap-sketcher-continuum-domain/`
- Issue file: `docs/agents/issues/done/20261103-029-mode1-continuum-sketcher-ui-canvas-solver-handoff.md`
- Category: `feature`
- Execution type: `AFK`
- Review gate: `visual-review`
- Suggested state: `done`

## Parent Artifacts

- `docs/prd.md`
- `docs/architecture/application-architecture-summary.md`
- `docs/architecture/cross-capability-conventions.md`
- `.okf/capabilities/cap-sketcher-continuum-domain.md`
- `docs/architecture/cap-sketcher-continuum-domain/prd.md`
- `docs/architecture/cap-sketcher-continuum-domain/canonical-domain-model.md`
- `docs/architecture/cap-sketcher-continuum-domain/canonical-use-cases.md`
- `docs/architecture/cap-sketcher-continuum-domain/canonical-api-cli-contract.md`
- `docs/architecture/cap-sketcher-continuum-domain/acceptance-scenarios.md`
- `docs/architecture/cap-sketcher-continuum-domain/readiness-review.md`

## What to build

Implement **Vertical Slice 3** of `cap-sketcher-continuum-domain`: the interactive Angular 2D Continuum Domain Sketcher UI in Mode 1 (`src/app/features/continuum-sketcher/`), accessible directly from the Mode 1 Geometry tab, supporting outer polygon & quick-rectangle sketching, circular/rectangular/polygonal cutouts, material inclusion regions, boundary segment tagging (`Dirichlet`, `Neumann`, `Robin`, `Free`), live conformal CDT mesh preview, and 1-click handoff into the Mode 1 `MeshService` / `SolverService` pipeline.

### 1. Continuum Sketcher Signal Store (`src/app/features/continuum-sketcher/continuum-sketcher.store.ts`):
- Manage `ContinuumDomain` state, monotonic `revision`, active tool (`Select`, `OuterPolygon`, `QuickRectangle`, `CircularCutout`, `RectangularCutout`, `PolygonalCutout`, `InclusionRegion`), candidate segment intersection preview (`continuum_preview_segment`), `ConformalMeshHandoff` state (`Unmeshed`, `Current`, `Stale`), and undo/redo history.
- Automatically mark any existing `ConformalMeshHandoff` as `Stale` when `domain.revision` increments (`BR-CONT-06`, `SC-009`).

### 2. Interactive 2D Continuum Sketching Canvas & Inspector (`src/app/features/continuum-sketcher/`):
- Render the 2D SVG sketching canvas with grid/vertex/ortho snapping, candidate edge preview (highlighting red on `SELF_INTERSECTING_LOOP`), outward normal ticks (`SC-005`), cutout & inclusion overlays, color-coded boundary tags (`Left`, `Right`, `Top`, `Bottom`, `Hole`, custom), and the generated `Tri3`/`Quad4` + `Line2`/`Line3` conformal mesh overlay (`SC-001`).
- Provide the Boundary & Cutout Inspector for editing cutout parameters $(c_x, c_y, R, w, h)$, inclusion Young's modulus $(E, \nu)$, segment semantic tags, `BoundaryConditionSpec` (`DirichletDisplacement`, `NeumannTraction`, `RobinImpedance`, `Free`), and `MeshingProfile` ($h, \alpha_{\text{hole}}, \theta_{\min}$) (`SC-006`, `SC-007`).

### 3. Mode 1 Geometry Tab Exposure & Solver Handoff (`src/app/components/sidebar/geometry-tab/`):
- Add a **"Custom 2D Continuum Sketcher" («Σχεδιαστής 2D Συνεχούς Φορέα»)** mode/drawer trigger in Mode 1's `GeometryTabComponent` so users can either sketch from scratch or start from the $10 \times 10\text{ m}$ + $R = 1.5\text{ m}$ perforated plate template.
- Wire **"Generate Conformal Mesh"** (`continuum_generate_mesh`) and **"Send to Mode 1 Solver"**, pushing `ConformalMeshHandoff.mesh_data` into `MeshService` so Mode 1 `PureBEM`, `PureFEM`, and `BEMFEM` simulations run directly on the custom sketched domain and visualize stresses in the 3D/2D viewport (`SC-008`).

## Acceptance criteria

- [x] Users can open the 2D Continuum Sketcher from Mode 1's Geometry tab, sketch or configure a $10 \times 10\text{ m}$ square plate with an $R = 1.5\text{ m}$ central hole, click "Generate Conformal Mesh", and inspect the graded `Tri3` + `Line2` mesh overlay and `MeshQualityReport` (`SC-001`).
- [x] Self-intersecting candidate segments highlight red and block loop closure (`SC-002`), and invalid cutout placements display bilingual error banners for `HOLE_OUTSIDE_OUTER_BOUNDARY`, `CLEARANCE_BELOW_MINIMUM`, and `HOLE_OVERLAPS_EXISTING_FEATURE` (`SC-003`, `SC-004`).
- [x] Users can tag boundary segments, assign `Dirichlet`, `Neumann`, or `Robin` conditions (`SC-006`), add `InclusionRegion`s (`SC-007`), and hand off the conformal mesh into Mode 1's `BEMFEM` solver to render von Mises stresses (`SC-008`).
- [x] Editing any vertex, cutout, or tag after meshing increments `revision` and marks the mesh badge `Stale` (`STALE_MESH_HANDOFF`) until regenerated (`SC-009`).
- [x] Vitest unit/integration tests (`continuum-sketcher.spec.ts`), `cargo test --lib`, `npx ng build`, and `npm run check:limits` pass with 0 errors.

## Artifact sync required

- Application PRD: `none - already covered in docs/prd.md`
- Application architecture summary: `none - matches docs/architecture/application-architecture-summary.md`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-sketcher-continuum-domain.md (transition state to implemented upon completion of 027-029)`
- Issue registry: `required`; node `issues:` reference: `required: .okf/capabilities/cap-sketcher-continuum-domain.md`
- Reason/no-impact decision: `Completes all 9 acceptance scenarios (SC-001..SC-009) of cap-sketcher-continuum-domain on the Mode 1 product surface.`

## Human review gate

`visual-review`: User opens the Continuum Domain Sketcher in Mode 1, sketches a 2D polygon with circular/polygonal cutouts and inclusions, tags boundary segments, generates the graded conformal mesh, and runs a Mode 1 BEM-FEM stress simulation.

## Blocked by

- Blocked by `docs/agents/issues/pending/20261103-028-conformal-cdt-mesher-boundary-tagging-inclusions.md`

## Artifact anchors

- PRD: `WF-1..WF-4`, `BR-CONT-01..07`, `FR-01..06`
- Domain model: `ContinuumDomain`, `ConformalMeshHandoff`, `BoundaryConditionSpec`
- Use cases: `UC-CMD-01..08`, `UC-QRY-01..02`
- Contract: `continuum_validate_domain`, `continuum_preview_segment`, `continuum_close_and_normalize_loop`, `continuum_generate_mesh`

## Acceptance scenarios addressed

- `SC-001`: $10 \times 10\text{ m}$ square plate with $R = 1.5\text{ m}$ central circular hole generates graded conformal `Tri3` + `Line2` mesh
- `SC-002`: Self-intersecting (bowtie) polygon boundary is rejected on loop closure and validation
- `SC-003`: Cutout breaching outer boundary or violating minimum clearance $d_{\min}$ is rejected
- `SC-004`: Overlapping or touching interior cutouts are rejected with `HOLE_OVERLAPS_EXISTING_FEATURE`
- `SC-005`: Clockwise-drawn outer boundary is automatically reversed to CCW and hole loops orient CW with outward normals away from material
- `SC-006`: Semantic boundary tagging and Dirichlet/Neumann/Robin boundary conditions propagate to all child `BoundaryElement` records
- `SC-007`: Interior `InclusionRegion` preserves conformal internal mesh interface without generating exterior BEM boundary elements
- `SC-008`: End-to-end handoff of sketched $10 \times 10\text{ m}$ plate with $R = 1.5\text{ m}$ hole into Mode 1 `BEMFEM` solver converges and renders stresses
- `SC-009`: Mutating a domain after meshing increments `revision` and blocks stale mesh handoff with `STALE_MESH_HANDOFF`

## Verification obligations

- Policy source: `/.okf/project.md`

| Scenario | Backend boundary | Frontend integration | End-to-end journey |
|---|---|---|---|
| `SC-001` | `implemented: Issue 028` | `planned` | `catalog-only` |
| `SC-002` | `implemented: Issue 027` | `planned` | `catalog-only` |
| `SC-003` | `implemented: Issue 027` | `planned` | `catalog-only` |
| `SC-004` | `implemented: Issue 027` | `planned` | `catalog-only` |
| `SC-005` | `implemented: Issue 027` | `planned` | `not-applicable` |
| `SC-006` | `implemented: Issue 028` | `planned` | `catalog-only` |
| `SC-007` | `implemented: Issue 028` | `planned` | `catalog-only` |
| `SC-008` | `implemented: Issue 028` | `planned` | `catalog-only` |
| `SC-009` | `implemented: Issue 028` | `planned` | `catalog-only` |
