## Issue Metadata

- Issue number: `026`
- Owning capability node: `/.okf/capabilities/cap-sketcher-frame-modeler.md`
- Artifact root: `docs/architecture/cap-sketcher-frame-modeler/`
- Issue file: `docs/agents/issues/pending/20261102-026-mode3-classifier-handoff-preset-import-diagram-export.md`
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

Implement **Vertical Slice 4** (final slice) of `cap-sketcher-frame-modeler`: the classical method classifier & bidirectional Mode 2 $\leftrightarrow$ Mode 3 Teaching Studio handoff (`teaching_classify_model`, `teaching_open_step_by_step`, `teaching_preset_to_structural_model`, plus DSM Walkthrough fallback), `.bem` workspace save/load integration for Mode 2, and standalone publication-grade SVG/PNG diagram export.

### 1. Rust Teaching Classifier, Custom Sketch Step-by-Step Handoff & Preset Converter (`src-tauri/src/teaching/`):
- **`classifier.rs` (`teaching_classify_model`):**
  - Inspect a `StructuralModel` and `Diagnostics` to evaluate `MethodEligibility` across classical methods in priority order: `IsostaticBeamFrame` (including inclined members, 3-hinged frames, and compound beams with hinges), `TrussJointsRitter`, `ContinuousBeamClapeyronCross`, and `ForceMethod` (for static indeterminacy $1 \le n_s \le 3$) (`SC-011`).
  - Unconditionally mark classical methods `eligible: false` with bilingual (`el`/`en`) explanations when the model contains `Cable` elements (`BR-5`), unilateral nonlinearities, or $n_s > 3$, while keeping `dsm_walkthrough_available: true` for any kinematically stable structure (`SC-012`).
- **`handoff.rs` (`teaching_open_step_by_step` & `teaching_preset_to_structural_model`):**
  - Convert an eligible `StructuralModel` (and active `LoadCaseId`) into a full Mode 3 `TeachingSolutionDto` (or DSM Walkthrough step deck via `frame_generate_dsm_walkthrough`) (`SC-011`, `SC-012`).
  - Convert any Mode 3 teaching preset into an editable `StructuralModel` (`revision: 1`) so users can open and modify presets in Mode 2 (`SC-013`).

### 2. Mode 2 $\leftrightarrow$ Mode 3 UI Integration (`src/app/features/modeler/` & `src/app/features/teaching/`):
- Display the live `MethodEligibility` badge in the Mode 2 toolbar (`UC-Q04`):
  - Enable **"Open Step-by-Step in Teaching Studio"** («Άνοιγμα Βήμα-προς-Βήμα στο Teaching Studio») when eligible, transitioning to Mode 3 with the step-by-step derivation loaded (`UC-C08`, `SC-011`).
  - When classical methods are ineligible, show a bilingual popover explaining why per method and offer **"Open DSM Walkthrough in Mode 3"** (`SC-012`).
- Add **"Edit in Mode 2 Sketcher"** («Επεξεργασία στον Σχεδιαστή Mode 2») to Mode 3's sidebar/toolbar (`UC-C09`), loading the preset into Mode 2 as a live editable sketch (`SC-013`).

### 3. Publication SVG/PNG Diagram Export & Project Workspace Serialization (`UC-C10`):
- Implement deterministic standalone SVG and high-DPI PNG export of the Mode 2 canvas (geometry, support symbols, tension-side $M/Q/N$ curves, extrema tags, scale bar, and bilingual legend) (`SC-014`).
- Wire lossless JSON workspace save/load of `StructuralModel`, `Combination`s, and `DraftingViewport` (`SC-014`).

## Acceptance criteria

- [x] Sketching an eligible beam, frame, continuous beam, or truss in Mode 2 enables **"Open Step-by-Step in Teaching Studio"** and transitions to Mode 3 with the complete step-by-step derivation and diagrams (`SC-011`).
- [x] Adding a `Cable` element or exceeding classical indeterminacy bounds disables classical methods, displays a clear bilingual (`el`/`en`) explanation popover, and allows opening the **DSM Walkthrough** in Mode 3 (`SC-012`).
- [x] Clicking **"Edit in Mode 2 Sketcher"** in Mode 3 loads the teaching preset into Mode 2 as an editable `StructuralModel` (`revision: 1`) whose live DSM diagrams match the preset's analytical solution (`SC-013`).
- [x] Saving and reloading a Mode 2 project restores the exact `StructuralModel`, `Combination`s, and viewport state, and **Export Diagram (SVG / PNG)** produces self-contained diagram files with tension-side $M/Q/N$ curves, extrema, and bilingual legend (`SC-014`).
- [x] Vitest tests (`npx vitest run`), `cargo test`, `npm run check:limits`, and `npx ng build` pass with 0 errors.

## Artifact sync required

- Application PRD: `none - already covered in docs/prd.md`
- Application architecture summary: `none - matches docs/architecture/application-architecture-summary.md`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-sketcher-frame-modeler.md` (transitions to `implemented` once all 4 slices close)
- Issue registry: `required`; node `issues:` reference: `required: .okf/capabilities/cap-sketcher-frame-modeler.md`
- Reason/no-impact decision: `Completes all remaining Mode 2 <-> Mode 3 handoff, classifier, preset import, persistence, and SVG/PNG export scenarios of cap-sketcher-frame-modeler.`

## Human review gate

`visual-review`: User tests the live classifier badge in Mode 2, opens an eligible sketched structure step-by-step in Mode 3, verifies the bilingual ineligibility explanation and DSM Walkthrough on a cable/hyperstatic frame, imports a Mode 3 preset into Mode 2 for editing, and inspects exported SVG/PNG diagrams.

## Blocked by

- Blocked by `docs/agents/issues/pending/20261102-025-mode2-loadcases-eurocode-combinations-envelopes-async-jobs.md`

## Artifact anchors

- PRD: Goal 7, `WF-5`, `BR-5`
- Domain model: `TeachingHandoffCoordinatorService`, `WorkspaceExportAndPersistenceService`, `MethodEligibility`
- Use cases: `UC-C08 HandoffToMode3TeachingStudio`, `UC-C09 OpenMode3PresetInMode2`, `UC-C10 ExportDiagramOrPersistProject`, `UC-Q04 GetMethodEligibilitySummary`
- Contract: `teaching_classify_model`, `teaching_open_step_by_step`, `teaching_preset_to_structural_model`, error code `INELIGIBLE_TEACHING_METHOD`

## Acceptance scenarios addressed

- `SC-011`: Mode 3 Handoff for Eligible Structure ("Open Step-by-Step")
- `SC-012`: Mode 3 Handoff Blocked for Ineligible Structure (Cables / High Hyperstatic Degree) with DSM Walkthrough Fallback
- `SC-013`: Opening a Mode 3 Teaching Preset as an Editable Sketch in Mode 2
- `SC-014`: Project Persistence Round-Trip & Publication SVG/PNG Diagram Export

## Verification obligations

- Policy source: `/.okf/project.md`

| Scenario | Backend boundary | Frontend integration | End-to-end journey |
|---|---|---|---|
| `SC-011` | `planned` | `planned` | `planned` |
| `SC-012` | `planned` | `planned` | `planned` |
| `SC-013` | `planned` | `planned` | `planned` |
| `SC-014` | `planned` | `planned` | `planned` |
