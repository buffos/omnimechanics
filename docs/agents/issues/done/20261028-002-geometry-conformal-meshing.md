## Issue Metadata

- Issue number: `002`
- Owning capability node: `/.okf/capabilities/cap-geometry-meshing.md`
- Artifact root: `docs/architecture/cap-geometry-meshing/`
- Issue file: `docs/agents/issues/done/20261028-002-geometry-conformal-meshing.md`
- Category: `feature`
- Execution type: `AFK`
- Review gate: `visual-review`
- Suggested state: `done`

## Parent Artifacts

- `.okf/project.md`
- `.okf/capabilities/cap-geometry-meshing.md`
- `.okf/capabilities/cap-interactive-canvas.md`

## What to build

Implement the 2D and 3D geometry modeling and conformal auto-meshing engine in the Rust core, with synchronized boundary elements for BEM and volume elements for FEM. Expose Tauri IPC commands to generate meshes for canonical benchmarks (2D rectangular beam, 2D plate with circular cavity, 3D solid beam, 3D block with cavity) with boundary tagging (Left, Right, Top, Bottom, Cavity) and render the generated meshes interactively in Three.js with wireframe, node markers, and boundary color-coding.

## Acceptance criteria

- [x] Rust core data structures for `Mesh2D` and `Mesh3D` with explicit separation and conformal linking between boundary elements (BEM) and volume elements (FEM).
- [x] Parametric meshing generators in Rust for canonical benchmark geometries:
  - 2D Cantilever / Simply Supported rectangular beam with $(nx, ny)$ discretization.
  - 2D Plate with central circular cavity with graded boundary refinement.
  - 3D Rectangular solid beam with $(nx, ny, nz)$ tetrahedral/hexahedral discretization.
  - 3D Solid block with central spherical cavity.
- [x] Boundary face/edge tagging system classifying external boundaries (e.g., `LeftFixed`, `RightLoaded`, `TopFree`, `BottomFree`, `HoleBoundary`) for downstream boundary condition application.
- [x] Tauri IPC command `generate_mesh` returning typed node arrays, element connectivity, and boundary tags.
- [x] Angular `MeshService` connecting UI controls to Rust mesher.
- [x] Three.js `ViewportComponent` updated to render actual generated 2D/3D meshes with interactive orbit controls, wireframe toggles, and color-coded boundary tags.

## Artifact sync required

- Application PRD: `none - capability foundation`
- Application architecture summary: `none - capability foundation`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-geometry-meshing.md`
- Issue registry: `required`; node `issues:` reference: `required`
- Reason/no-impact decision: `Grounding issue for geometry and conformal meshing capability.`

## Human review gate

- Visual review: User verified in the desktop window that selecting benchmarks (2D beam, 2D plate with circular hole, 3D solid beam) dynamically renders conformal meshes in Three.js with color-coded boundary tags and wireframe. Confirmed working perfectly.

## Blocked by

None - completed

## Artifact anchors

- Capability: `cap-geometry-meshing`
- Capability: `cap-interactive-canvas`

## Acceptance scenarios addressed

- Conformal boundary/volume meshing for 2D and 3D elastodynamic benchmarks.

## Verification obligations

- mode: `when-supported`
- surfaces:
  - backend-boundary: `passed` (5/5 Rust unit tests for mesh connectivity and boundary tags)
  - frontend-integration: `passed` (Angular component compilation and IPC integration)
  - end-to-end: `passed` (Visual review completed and confirmed)
