## Issue Metadata

- Issue number: `001`
- Owning capability node: `/.okf/capabilities/cap-interactive-canvas.md`
- Artifact root: `docs/architecture/cap-interactive-canvas/`
- Issue file: `docs/agents/issues/done/20261028-001-scaffold-tauri-angular-rust.md`
- Category: `feature`
- Execution type: `AFK`
- Review gate: `visual-review`
- Suggested state: `done`

## Parent Artifacts

- `.okf/project.md`
- `.okf/capabilities/cap-interactive-canvas.md`
- `.okf/capabilities/cap-solver-core.md`

## What to build

Scaffold the unified workspace integrating Tauri v2, Angular 20+ (standalone with Signals), Three.js, and a Rust computational core crate (`faer` + linear algebra foundations). Verify that the application compiles, launches a native desktop window, and renders an interactive Three.js 3D viewport canvas.

## Acceptance criteria

- [x] Workspace initialized with Tauri v2 (`src-tauri/`) and Angular frontend (`src/`).
- [x] Rust core configured with `tauri` v2 and linear algebra crate dependencies (`faer`, `nalgebra`, `rayon`, `num-complex`).
- [x] Angular frontend configured with Three.js (`three` + `@types/three`) in standalone mode.
- [x] A dedicated Three.js canvas component initializes a 3D scene (perspective camera, orbit controls, ground grid/axes, and sample test geometry with continuous inhomogeneity colormap).
- [x] Desktop app builds and executes via Tauri, demonstrating IPC connectivity between Angular and the Rust backend.

## Artifact sync required

- Application PRD: `none - initial repository scaffold`
- Application architecture summary: `none - initial repository scaffold`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-interactive-canvas.md`
- Issue registry: `required`; node `issues:` reference: `required`
- Reason/no-impact decision: `Baseline infrastructure issue establishing the Tauri v2 + Angular + Rust workspace.`

## Human review gate

- Visual review: User verifies that the Tauri desktop window launches and displays the interactive Three.js 3D viewport with smooth orbit camera controls. (Approved by user on 2026-10-28).

## Blocked by

None - can start immediately

## Artifact anchors

- Capability: `cap-interactive-canvas`
- Capability: `cap-solver-core`

## Acceptance scenarios addressed

- Initial Scaffolding and visual verification completed.

## Verification obligations

- mode: `when-supported`
- surfaces:
  - backend-boundary: `when-supported`
  - frontend-integration: `when-supported`
  - end-to-end: `visual-review`
