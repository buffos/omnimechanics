## Issue Metadata

- Issue number: `023`
- Owning capability node: `/.okf/capabilities/cap-sketcher-frame-modeler.md`
- Artifact root: `docs/architecture/cap-sketcher-frame-modeler/`
- Issue file: `docs/agents/issues/pending/20261102-023-mode2-drafting-canvas-snapping-splits-transforms.md`
- Category: `feature`
- Execution type: `AFK`
- Review gate: `visual-review`
- Suggested state: `done`

## Parent Artifacts

- `docs/prd.md`
- `docs/architecture/application-architecture-summary.md`
- `docs/architecture/cross-capability-conventions.md`
- `.okf/capabilities/cap-sketcher-frame-modeler.md`
- `docs/architecture/cap-sketcher-frame-modeler/prd.md`
- `docs/architecture/cap-sketcher-frame-modeler/canonical-domain-model.md`
- `docs/architecture/cap-sketcher-frame-modeler/canonical-use-cases.md`
- `docs/architecture/cap-sketcher-frame-modeler/canonical-api-cli-contract.md`
- `docs/architecture/cap-sketcher-frame-modeler/acceptance-scenarios.md`
- `docs/architecture/cap-sketcher-frame-modeler/readiness-review.md`

## What to build

Implement **Vertical Slice 1** of `cap-sketcher-frame-modeler`: bootstrap the Vitest `frontend-integration` test harness, add **Mode 2: Structural Modeling & Analysis** («Μοντελοποίηση & Ανάλυση Φορέων») to the top header switcher, and deliver the interactive 2D SVG drafting canvas with priority-ordered snapping, automatic topological member splitting at intersections, keyboard numeric entry, marquee box selection, CAD transformations (`Move`, `Copy`, `Mirror`, `LinearArray`, `Delete`), and monotonic-revision `Undo`/`Redo`.

### 1. Vitest Frontend Integration Harness (`package.json`, `vitest.config.ts`):
- Configure Vitest for pure TypeScript store, geometry, snapping, transformation, and undo/redo unit/integration testing (`npm run test:unit` / `npx vitest run`).

### 2. Pure-TS Snapping, Splitting, CAD Transform & Undo/Redo Engine (`src/app/features/modeler/core/`):
- **`snapping-engine.ts`:**
  - World $\leftrightarrow$ screen affine transform (`DraftingViewport` with pan, zoom, and $y$-up coordinate system) and selectable grid step ($0.1\text{ m}, 0.5\text{ m}, 1.0\text{ m}$).
  - Strict priority-ordered snap evaluation (`UC-Q01`): `NodeSnap` ($< 15\text{ px}$) $\rightarrow$ `IntersectionSnap` $\rightarrow$ `MidpointSnap` $\rightarrow$ `OrthoLock` ($0^\circ/90^\circ/180^\circ/270^\circ$) / `AngularGuideSnap` ($30^\circ/45^\circ/60^\circ$) $\rightarrow$ `GridSnap`, plus exact numeric coordinate `(x, y)` or `(L, θ)` parser (`SC-001`, `SC-002`).
- **`topological-split.ts`:**
  - Detect when a drawn element starts/ends on an existing member interior or crosses existing members (`UC-C01`), insert shared `Node`s, split intersected `FrameMember`s while preserving `Section`, `Material`, outer `EndRelease`s, and distributed load continuity, and record the entire gesture as a single undoable command (`SC-003`).
- **`cad-transforms.ts` & `modeler.store.ts`:**
  - Marquee box selection (left-to-right enclosed Window vs. right-to-left Crossing).
  - `Move`, `Copy`, `Mirror` across a 2-point axis, `LinearArray` (`dx, dy, count` with coincident node welding), and cascade `Delete` (`UC-C02`) (`SC-004`).
  - `UndoRedoStack` (`UC-C05`) where every mutation—including `Undo` and `Redo`—strictly increments `StructuralModel.revision` (`SC-004`).

### 3. Mode 2 Workspace Shell & Interactive SVG Canvas (`src/app/features/modeler/`):
- Add Mode 2 button to the top application header switcher (`SC-001`).
- Render the interactive 2D SVG drafting canvas (`modeler-canvas.component`), drafting & transform toolbar (`modeler-toolbar.component`), numeric coordinate input bar, live $L/\theta$ floating dimension tooltip, snap glyph indicators, and cursor $(x, y)$ status readout in Greek and English (`el`/`en`).

## Acceptance criteria

- [x] Mode 2 ("Structural Modeling & Analysis" / «Μοντελοποίηση & Ανάλυση Φορέων») is reachable from the top header switcher alongside Mode 1 and Mode 3, with smooth pan/zoom, selectable grid ($0.1\text{ m}, 0.5\text{ m}, 1.0\text{ m}$), and live world $(x, y)$ readout (`SC-001`).
- [x] `SnappingEngine` enforces strict snap priority (`NodeSnap` $< 15\text{ px}$ over closer grid points, `IntersectionSnap`, `MidpointSnap`, `OrthoLock`, `AngularGuideSnap`, `GridSnap`), supports exact `(x, y)` and `(L, θ)` keyboard entry, and rejects zero-length elements with `ZERO_LENGTH_ELEMENT` (`SC-002`).
- [x] Drawing a member across or onto an existing member automatically splits the intersected member(s) at a shared `Node`, preserving section/material/outer releases and partitioning distributed loads in a single undoable step (`SC-003`).
- [x] Enclosed/crossing marquee box selection, `Move`, `Copy`, `Mirror`, `LinearArray` (with node welding), cascade `Delete`, and `Undo`/`Redo` (`Ctrl+Z`/`Ctrl+Y`) work on the canvas and keep `StructuralModel.revision` strictly increasing (`SC-004`).
- [x] Vitest unit/integration suite (`npx vitest run`), `npm run check:limits`, and `npx ng build` pass with 0 errors.

## Artifact sync required

- Application PRD: `none - already synthesized in docs/prd.md`
- Application architecture summary: `none - matches src/app/features/modeler/ in docs/architecture/application-architecture-summary.md`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-sketcher-frame-modeler.md`
- Issue registry: `required`; node `issues:` reference: `required: .okf/capabilities/cap-sketcher-frame-modeler.md`
- Reason/no-impact decision: `Delivers Mode 2 Slice 1A (Vitest harness, Mode 2 header entry, SVG drafting canvas, snapping, topological splits, CAD transforms, and undo/redo).`

## Human review gate

`visual-review`: User inspects Mode 2 header navigation, SVG grid rendering, magnetic node/midpoint/intersection snapping with auto-split, $L/\theta$ tooltips, numeric input bar, marquee box selection, CAD transforms (`Move`, `Copy`, `Mirror`, `LinearArray`), and `Undo`/`Redo` behavior.

## Blocked by

`None - can start immediately`

## Artifact anchors

- PRD: Goals 1 & 2, `WF-1`, `WF-2`, `BR-1`, `BR-3`, `NFR-1`, `NFR-3`, `NFR-4`
- Domain model: `ModelerWorkspace`, `DraftingViewport`, `SnappingPolicyService`, `TopologicalSplitService`, `CadTransformationService`, `UndoRedoStack`, `INV-1`, `INV-2`
- Use cases: `UC-Q01 EvaluateCursorSnap`, `UC-C01 SketchElementWithSnapping`, `UC-C02 ApplyCadTransformation`, `UC-C05 UndoOrRedoModelMutation`
- Contract: `SnapCandidateDto`, `ModelerStore.sketchElement`, `ModelerStore.applyTransform`, `ModelerStore.undo/redo`, error codes `ZERO_LENGTH_ELEMENT`, `DUPLICATE_ELEMENT`, `EMPTY_SELECTION`, `DEGENERATE_MIRROR_AXIS`

## Acceptance scenarios addressed

- `SC-001`: Workspace Navigation to Mode 2 & Dynamic Viewport Grid
- `SC-002`: Priority-Ordered Snapping, Ortho/Angle Guides & Exact Numeric Entry
- `SC-003`: Member-Member Intersection Snap with Automatic Topological Splitting
- `SC-004`: Box Selection, CAD Transformations (Move, Copy, Mirror, Array, Delete) & Undo/Redo

## Verification obligations

- Policy source: `/.okf/project.md`

| Scenario | Backend boundary | Frontend integration | End-to-end journey |
|---|---|---|---|
| `SC-001` | `not-applicable` | `planned` | `planned` |
| `SC-002` | `not-applicable` | `planned` | `planned` |
| `SC-003` | `planned` | `planned` | `planned` |
| `SC-004` | `not-applicable` | `planned` | `planned` |
