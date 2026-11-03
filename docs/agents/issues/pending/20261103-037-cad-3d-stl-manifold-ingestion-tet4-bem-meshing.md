## Issue Metadata

- Issue number: `037`
- Owning capability node: `/.okf/capabilities/cap-geometry-cad-import.md`
- Artifact root: `.okf/capabilities/cap-geometry-cad-import.md`
- Issue file: `docs/agents/issues/pending/20261103-037-cad-3d-stl-manifold-ingestion-tet4-bem-meshing.md`
- Category: `feature`
- Execution type: `AFK`
- Review gate: `visual-review`
- Suggested state: `ready-for-agent`

## Parent Artifacts

- `docs/prd.md`
- `docs/architecture/application-architecture-summary.md`
- `docs/architecture/cross-capability-conventions.md`
- `.okf/capabilities/cap-geometry-cad-import.md`
- `.okf/capabilities/cap-geometry-meshing.md`

## What to build

Implement **Vertical Slice 2** of `cap-geometry-cad-import`: binary and ASCII **3D STL ingestion**, spatial vertex hashing, 2-manifold watertightness verification, consistent outward normal orientation (`HealedSurface3D`), conformal 3D `Tri3` surface BEM + interior `Tet4` volume mesh generation (`src-tauri/src/cad/stl_3d.rs`), and interactive 3D STL import in Mode 1's Geometry tab (`Scenario 3`).

### 1. Binary & ASCII 3D STL Parser & Manifold Healer (`src-tauri/src/cad/stl_3d.rs`):
- Parse both ASCII (`solid ... facet normal ...`) and binary STL triangle arrays.
- Reconstruct shared topological vertices and edges from triangle soup via spatial vertex hashing within tolerance $\epsilon$.
- Verify 2-manifold watertightness (every edge shared by exactly 2 triangles), orient all triangle normals consistently outward (for 3D BEM kernels), compute enclosed volume $V$, and populate `HealedSurface3D { vertices, triangles, is_watertight, volume }`.

### 2. 3D Conformal Surface BEM + Interior Tet4 Meshing & Mode 1 UI Exposure:
- Convert watertight `HealedSurface3D` into Mode 1 `MeshData` (`dimension: 3`) with outward-oriented `Tri3` `BoundaryElement`s (auto-tagged `Left`, `Right`, `Top`, `Bottom`, `Front`, `Back` by bounding-box/normal proximity) and conformal interior `Tet4` `VolumeElement`s.
- Register Tauri IPC command `cad_import_stl_3d`.
- Expose **"Import 3D STL Solid" («Εισαγωγή 3D STL Στερεού»)** in Mode 1's `GeometryTabComponent` and CAD Import dialog, displaying manifold status, triangle/tet counts, volume, and rendering the imported 3D bracket on the Three.js viewport ready for 3D BEM/FEM analysis (`Scenario 3`).

## Acceptance criteria

- [ ] Importing a binary or ASCII STL model of a 3D cantilever bracket merges shared vertices, verifies `is_watertight == true` (2-manifold closure), and orients face normals consistently outward (`Scenario 3`).
- [ ] Non-watertight STL meshes with open boundary edges are flagged with `is_watertight == false` and diagnostic open-edge counts.
- [ ] A watertight imported 3D STL generates a conformal 3D `MeshData` (`Tri3` boundary patches + `Tet4` volume elements), renders on the Mode 1 Three.js viewport, and solves in 3D static BEM/FEM analysis (`Scenario 3`).
- [ ] Rust unit tests (`stl_3d_test.rs`), Vitest tests, `npx ng build`, and `npm run check:limits` pass with 0 errors.

## Artifact sync required

- Application PRD: `none - already covered in docs/prd.md`
- Application architecture summary: `none - matches docs/architecture/application-architecture-summary.md`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-geometry-cad-import.md (transition state to implemented upon completion of 036-037; recompute roll-up cap-geometry-meshing)`
- Issue registry: `required`; node `issues:` reference: `required: .okf/capabilities/cap-geometry-cad-import.md`
- Reason/no-impact decision: `Completes all 3 acceptance scenarios (Scenarios 1..3) of cap-geometry-cad-import.`

## Human review gate

`visual-review`: User imports a 3D STL bracket in Mode 1's Geometry tab, inspects the manifold verification badge and 3D mesh on the Three.js canvas, and runs a 3D static stress analysis.

## Blocked by

- Blocked by `docs/agents/issues/pending/20261103-036-cad-2d-dxf-svg-parser-polyline-healing-handoff.md`

## Artifact anchors

- Capability spec (`.okf/capabilities/cap-geometry-cad-import.md`): Scope §2 (3D Surface & Solid CAD Ingestion), §3 (Handoff Pipeline), `HealedSurface3D`

## Acceptance scenarios addressed

- `Scenario 3`: Watertight 3D STL Ingestion

## Verification obligations

- Policy source: `/.okf/project.md`

| Scenario | Backend boundary | Frontend integration | End-to-end journey |
|---|---|---|---|
| `Scenario 3` | `planned` | `planned` | `catalog-only` |
