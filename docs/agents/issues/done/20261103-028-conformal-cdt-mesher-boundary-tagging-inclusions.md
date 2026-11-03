## Issue Metadata

- Issue number: `028`
- Owning capability node: `/.okf/capabilities/cap-sketcher-continuum-domain.md`
- Artifact root: `docs/architecture/cap-sketcher-continuum-domain/`
- Issue file: `docs/agents/issues/pending/20261103-028-conformal-cdt-mesher-boundary-tagging-inclusions.md`
- Category: `feature`
- Execution type: `AFK`
- Review gate: `none`
- Suggested state: `ready-for-agent`

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

Implement **Vertical Slice 2** of `cap-sketcher-continuum-domain`: the Constrained Delaunay Triangulation (CDT) and Chew/Ruppert/Shewchuk quality refinement engine in Rust (`continuum_generate_mesh`), generating graded `Tri3` / `Quad4` interior volume elements and strictly conformal `Line2` / `Line3` BEM boundary elements with semantic tag propagation, internal `InclusionRegion` interface conformity, and stale revision protection (`STALE_MESH_HANDOFF`).

### 1. Constrained PSLG & Graded CDT Mesher (`src-tauri/src/continuum/cdt_mesher.rs`):
- Subdivide the validated `outer_boundary`, `cutouts` (`Circle`, `Rectangle`, `Polygon`), and `inclusions` according to `MeshingProfile.target_element_size`, `hole_refinement_factor` $\alpha_{\text{hole}}$, and segment `local_element_size`.
- Construct the Planar Straight-Line Graph (PSLG) and perform Constrained Delaunay Triangulation with Steiner point insertion to satisfy `min_angle_deg` ($\ge 20^\circ$) and smooth size grading from holes ($h \cdot \alpha_{\text{hole}}$) to the outer boundary ($h$).
- Classify triangles via exact winding numbers: discard exterior triangles and interior `Cutout` void triangles, while retaining `InclusionRegion` interior triangles tagged with their inclusion subdomain provenance (`SC-007`).
- Support optional triangle-to-quad recombination when `volume_element_type == Quad4` and mid-side node insertion when `boundary_element_type == Line3`.

### 2. Strict BEM-FEM Conformity, Tag Propagation & Quality Report (`src-tauri/src/continuum/cdt_mesher.rs`):
- Extract `BoundaryElement` (`Line2` or `Line3`) records directly from the boundary-incident edges of `VolumeElement`s on the outer perimeter and cutout loops (`BR-CONT-04`, `SC-001`, `SC-006`) so zero hanging nodes exist.
- Ensure `InclusionRegion` internal interfaces share conformal nodes between inside/outside `VolumeElement`s without generating spurious BEM cavity `BoundaryElement`s (`SC-007`).
- Compute `MeshQualityReport` (`min_angle_deg`, `max_angle_deg`, `min_aspect_ratio`, `max_aspect_ratio`, `is_conformal`, `outward_normals_verified`) and `tag_element_counts`.
- Enforce `STALE_MESH_HANDOFF` verification helper when `handoff.domain_revision != domain.revision` (`BR-CONT-06`, `SC-009`).
- Register `continuum_generate_mesh` in `src-tauri/src/lib.rs` and verify end-to-end solver compatibility with `run_static_simulation` (`PureFEM`, `PureBEM`, `BEMFEM`) (`SC-008`).

## Acceptance criteria

- [x] A $10 \times 10\text{ m}$ square plate with a central $R = 1.5\text{ m}$ circular hole (`SC-001`) generates a `ConformalMeshHandoff` with `is_conformal == true`, `min_angle_deg >= 20.0`, graded hole boundary edges $\le 1.2 \cdot h \cdot \alpha_{\text{hole}}$, exact shared node indices between `Hole` `BoundaryElement`s and adjacent `Tri3` elements, and inward-pointing hole normals.
- [x] User-assigned semantic tags (`Left`, `Right`, `Top`, `Bottom`, `Hole`, custom tags) propagate to 100% of subdivided `BoundaryElement` records (`SC-006`).
- [x] An `InclusionRegion` ($R = 1.5\text{ m}$) populates interior `VolumeElement`s with conformal interface edges and generates zero interior `Hole` `BoundaryElement`s (`SC-007`).
- [x] Passing the `SC-001` `ConformalMeshHandoff.mesh_data` into Mode 1 `run_static_simulation` (`BEMFEM` and `PureFEM`) converges with positive displacement and von Mises stress (`SC-008`), and mismatched `domain_revision` rejects with `STALE_MESH_HANDOFF` (`SC-009`).
- [x] Companion unit and solver-handoff integration tests in `src-tauri/src/continuum/cdt_mesher_test.rs` pass 100% (`cargo test --lib`), and `npm run check:limits` passes.

## Artifact sync required

- Application PRD: `none - already covered in docs/prd.md`
- Application architecture summary: `none - matches docs/architecture/application-architecture-summary.md`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-sketcher-continuum-domain.md`
- Issue registry: `required`; node `issues:` reference: `required: .okf/capabilities/cap-sketcher-continuum-domain.md`
- Reason/no-impact decision: `Delivers backend Constrained Delaunay Triangulation and conformal MeshData handoff for cap-sketcher-continuum-domain.`

## Human review gate

`none`: Backend CDT meshing and Mode 1 solver compatibility slice; visual UI exposure is delivered in Issue `029`.

## Blocked by

- Blocked by `docs/agents/issues/pending/20261103-027-continuum-domain-polygon-cutout-validation-engine.md`

## Artifact anchors

- PRD: `WF-3`, `WF-4`, `BR-CONT-04`, `BR-CONT-05`, `BR-CONT-06`, `FR-04`, `FR-05`, `NFR-01`, `NFR-02`
- Domain model: `ConformalMeshHandoff`, `MeshQualityReport`, `MeshingProfile`, `INV-CONT-04..06`
- Use cases: `UC-CMD-04`, `UC-CMD-05`, `UC-CMD-07`, `UC-CMD-08`
- Contract: `continuum_generate_mesh`, error codes `CDT_CONVERGENCE_FAILURE`, `STALE_MESH_HANDOFF`

## Acceptance scenarios addressed

- `SC-001`: $10 \times 10\text{ m}$ square plate with $R = 1.5\text{ m}$ central circular hole generates graded conformal `Tri3` + `Line2` mesh
- `SC-006`: Semantic boundary tagging and Dirichlet/Neumann/Robin boundary conditions propagate to all child `BoundaryElement` records
- `SC-007`: Interior `InclusionRegion` preserves conformal internal mesh interface without generating exterior BEM boundary elements
- `SC-008`: End-to-end handoff of sketched $10 \times 10\text{ m}$ plate with $R = 1.5\text{ m}$ hole into Mode 1 `BEMFEM` solver converges and renders stresses
- `SC-009`: Mutating a domain after meshing increments `revision` and blocks stale mesh handoff with `STALE_MESH_HANDOFF`

## Verification obligations

- Policy source: `/.okf/project.md`

| Scenario | Backend boundary | Frontend integration | End-to-end journey |
|---|---|---|---|
| `SC-001` | `planned` | `deferred: Issue 029` | `catalog-only` |
| `SC-006` | `planned` | `deferred: Issue 029` | `catalog-only` |
| `SC-007` | `planned` | `deferred: Issue 029` | `catalog-only` |
| `SC-008` | `planned` | `deferred: Issue 029` | `catalog-only` |
| `SC-009` | `planned` | `deferred: Issue 029` | `catalog-only` |
