## Issue Metadata

- Issue number: `025`
- Owning capability node: `/.okf/capabilities/cap-sketcher-frame-modeler.md`
- Artifact root: `docs/architecture/cap-sketcher-frame-modeler/`
- Issue file: `docs/agents/issues/pending/20261102-025-mode2-loadcases-eurocode-combinations-envelopes-async-jobs.md`
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

Implement **Vertical Slice 3** of `cap-sketcher-frame-modeler`: multi-`LoadCase` management, EN 1990 / EN 1998 combination & response spectrum configuration modals (`actions_*`), the Result Target Selector (`LoadCase` / `Combination` / `Envelope` max-min with pointwise governing combination readout), Analysis Kind selector (`LinearStatic`, `SecondOrderPDelta`, `LinearBuckling`, `Modal`, `ResponseSpectrum`, `LateralForce`), and background nonlinear/dynamic job coordination (`frame_start_analysis_job`, `analysis://progress`, `frame_cancel_analysis_job`) with curved catenary cable rendering.

### 1. Load Cases, Eurocode Combinations & Seismic Settings UI (`src/app/features/modeler/combinations/`):
- Manage multiple `LoadCase`s tagged with `ActionType` (`Permanent`, `Imposed A..H`, `Snow`, `Wind`, `Temperature`, `Seismic`).
- Integrate with `actions_get_default_na_profile`, `actions_generate_combinations`, `actions_save_custom_combination`, `actions_delete_or_reset_combination`, and `actions_compute_design_spectrum` (`UC-C07`).
- Provide modal dialogs for configuring the Greek National Annex (`GR`) $\gamma/\xi/\psi$ factors, editing/adding combinations (`Generated` $\to$ `UserEdited`, `Manual`), and previewing the EN 1998-1 Type 1 Design Spectrum $S_d(T)$ curve.

### 2. Governing Envelope Visualization & Result Target Selector (`src/app/features/modeler/results/`):
- Add toolbar selectors for `AnalysisKind` (`LinearStatic`, `SecondOrderPDelta`, `LinearBuckling`, `Modal`, `ResponseSpectrum`, `LateralForce`) and `ResultTarget` (`LoadCase`, `Combination`, `Envelope` max/min) (`SC-010`).
- Render shaded dual upper/lower ($\max/\min$) tension-side envelope curves on the SVG canvas and display both max/min values and their exact governing `CombinationId` and formula in `CrossSectionHoverProbe` and the Member End-Force Table (`SC-010`).

### 3. Background Nonlinear/Dynamic Job Coordinator & Catenary Curve Rendering:
- When `analysis_kind != LinearStatic` or the model contains `Cable` or `unilateral` elements, immediately display the synchronous `linear_preview` with a non-blocking progress pill (`analysis://progress` iteration & percentage), dispatch `frame_start_analysis_job`, and swap in the converged `ResultSet` / `Envelope` upon completion (`SC-008`).
- Cancel in-flight jobs via `frame_cancel_analysis_job` and discard stale events (`job.model_revision < current.revision`, `STALE_REVISION_SUPERSEDED`) whenever the user edits the model or runs `Undo`/`Redo` (`SC-008`).
- Display the bilingual non-superposition badge (`BR-4`) when nonlinear elements or $P\text{-}\Delta$ are active, render `CableResult.curve_points_global` as true sagged catenary curves (with slack warning badges), and render buckling/modal mode shapes.

## Acceptance criteria

- [x] Users can create multiple `LoadCase`s, generate/edit EN 1990 & EN 1998 combinations (`GR` defaults), configure `SeismicSettings` with live $S_d(T)$ spectrum preview, and switch the canvas target between any `LoadCase`, `Combination`, or `Envelope` (`SC-010`).
- [x] Selecting an `Envelope` renders shaded upper/lower ($\max/\min$) tension-side diagrams and shows the governing combination ID and formula at every hovered station and end-force table row (`SC-010`).
- [x] Enabling `SecondOrderPDelta`, `LinearBuckling`, `Modal`, `ResponseSpectrum`, `LateralForce`, or adding a `Cable` / `unilateral` support shows the immediate linear preview, streams progress from `analysis://progress`, replaces the canvas diagrams upon convergence, and displays the bilingual non-superposition notice (`SC-008`).
- [x] Mutating the model while a background job is `Running` cancels the in-flight job and discards any late events with `model_revision < current.revision` (`SC-008`).
- [x] Vitest tests (`npx vitest run`), `cargo test`, `npm run check:limits`, and `npx ng build` pass with 0 errors.

## Artifact sync required

- Application PRD: `none - already covered in docs/prd.md`
- Application architecture summary: `none - matches docs/architecture/application-architecture-summary.md`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-sketcher-frame-modeler.md`
- Issue registry: `required`; node `issues:` reference: `required: .okf/capabilities/cap-sketcher-frame-modeler.md`
- Reason/no-impact decision: `Delivers multi-load-case, Eurocode combinations/envelopes, and background nonlinear/seismic job integration in Mode 2.`

## Human review gate

`visual-review`: User configures multiple load cases and EN 1990/1998 combinations in Mode 2, inspects shaded max/min envelopes with governing combination tooltips, runs P-Δ, buckling, catenary cable, and modal/seismic analyses, and verifies the live progress indicator and curved cable rendering.

## Blocked by

- Blocked by `docs/agents/issues/pending/20261102-021-eurocode-na-profile-en1990-combination-engine.md`
- Blocked by `docs/agents/issues/pending/20261102-022-en1998-seismic-spectra-mass-lateral-cqc.md`
- Blocked by `docs/agents/issues/pending/20261102-024-palettes-inspector-live-linear-dsm-diagrams-mechanisms.md`

## Artifact anchors

- PRD: Goals 5 & 6, `WF-3`, `WF-4`, `BR-3`, `BR-4`
- Domain model: `LiveAnalysisCoordinator`, `ResultOverlayState`, `INV-5`
- Use cases: `UC-C06 CoordinateLiveAnalysis`, `UC-C07 GenerateOrUpdateCombinations`, `UC-Q02 ProbeMemberCrossSection`, `UC-Q03 GetMemberEndForceTable`
- Contract: `frame_start_analysis_job`, `frame_cancel_analysis_job`, `analysis://progress`, `actions_generate_combinations`, `actions_compute_design_spectrum`, error code `STALE_REVISION_SUPERSEDED`

## Acceptance scenarios addressed

- `SC-008`: Background Nonlinear Solve Progress, Replacement & Cancellation on Rapid Edit
- `SC-010`: LoadCase, EN 1990/1998 Combination & Governing Envelope Inspection

## Verification obligations

- Policy source: `/.okf/project.md`

| Scenario | Backend boundary | Frontend integration | End-to-end journey |
|---|---|---|---|
| `SC-008` | `implemented: src-tauri/src/frame/` | `planned` | `planned` |
| `SC-010` | `planned` | `planned` | `planned` |
