## Issue Metadata

- Issue number: `031`
- Owning capability node: `/.okf/capabilities/cap-interactive-canvas.md`
- Artifact root: `docs/architecture/cap-interactive-canvas/`
- Issue file: `docs/agents/issues/done/20261103-031-shared-2d-svg-kernel-shell-dialog-dock-verification.md`
- Category: `feature`
- Execution type: `AFK`
- Review gate: `visual-review`
- Suggested state: `done`

## Parent Artifacts

- `docs/prd.md`
- `docs/architecture/application-architecture-summary.md`
- `docs/architecture/cross-capability-conventions.md`
- `.okf/capabilities/cap-interactive-canvas.md`
- `docs/architecture/cap-interactive-canvas/prd.md`
- `docs/architecture/cap-interactive-canvas/canonical-domain-model.md`
- `docs/architecture/cap-interactive-canvas/canonical-use-cases.md`
- `docs/architecture/cap-interactive-canvas/canonical-api-cli-contract.md`
- `docs/architecture/cap-interactive-canvas/acceptance-scenarios.md`
- `docs/architecture/cap-interactive-canvas/readiness-review.md`

## What to build

Implement **Vertical Slice 2** of `cap-interactive-canvas`: extract the canonical Shared 2D SVG Viewport & Tension-Side Diagram Kernel (`ViewportTransform2D`, `buildTensionSideDiagramPath`, `SemanticColorTokens` in `src/app/shared/svg-canvas-kernel.ts`) for unified parity between Mode 2 and Mode 3 (`SC-012`), enforce `DismissGuard` warning feedback (`DIALOG_DISMISS_BLOCKED`), `EXPORT_DATA_EMPTY` guards on `BottomDockComponent`, and complete the Vitest frontend-integration verification suite across `SC-001`..`SC-004` and `SC-010`..`SC-012`.

### 1. Shared 2D SVG Viewport & Tension-Side Diagram Kernel (`src/app/shared/svg-canvas-kernel.ts`):
- Implement `ViewportTransform2D` (`worldToScreen`, `screenToWorld`, pan, zoom, aspect-locked fit) guaranteeing $+y$ up in metres maps to $-v$ up on screen and $\text{screenToWorld}(\text{worldToScreen}(x, y)) = (x, y)$ within $10^{-9}$ (`SC-012`).
- Implement `buildTensionSideDiagramPath(member, samples, opts)` and `SemanticColorTokens` enforcing `BR-4` (positive sagging moment $M > 0$ always plotted on the local $-y$ bottom tension fibre) across both Mode 2 (`ModelerCanvasComponent`) and Mode 3 (`TeachingCanvasComponent`).

### 2. Shell, Dialog `DismissGuard`, Toast Deduplication & Dock Synchronization (`src/app/shared/`, `src/app/dock/`):
- Ensure `DialogService` / `DialogShellComponent` emits a bilingual warning toast when dismissal is blocked by an active `DismissGuard` (`DIALOG_DISMISS_BLOCKED`, `SC-003`) and restores keyboard focus on close.
- Ensure `BottomDockComponent` disables `CSV`/`SVG` export buttons when no transient/sweep series is present (`EXPORT_DATA_EMPTY`, `SC-011`), streams live solver convergence iterations, and synchronizes interactive chart scrubbing with `ViewportComponent` frame index (`SC-010`).
- Ensure switching between Mode 1 (`continuum`), Mode 2 (`structural`), and Mode 3 (`teaching`) pauses background harmonic/transient timers cleanly while preserving active session signals (`SC-001`) and instant `el`/`en` switching (`SC-002`).

## Acceptance criteria

- [x] `ViewportTransform2D` and `buildTensionSideDiagramPath` pass exact round-trip identity ($\le 10^{-9}$) and tension-side moment parity ($M > 0$ plotted on local $-y$ tension side) across Mode 2 and Mode 3 (`SC-012`).
- [x] `DialogService` enforces LIFO stack ordering, blocks `Escape`/backdrop dismissal when `dismissGuard()` returns `false` (`DIALOG_DISMISS_BLOCKED`), and restores focus on close (`SC-003`).
- [x] `NotificationService` deduplicates identical active toasts and resets their 5000 ms auto-dismiss timer (`SC-004`).
- [x] Tri-mode switching (`SC-001`), bilingual `el`/`en` toggling (`SC-002`), synchronized viewport-dock playback scrubbing (`SC-010`), and convergence streaming / `EXPORT_DATA_EMPTY` guards (`SC-011`) pass automated Vitest integration tests (`interactive-canvas.spec.ts`).
- [x] All 12 scenarios (`SC-001`..`SC-012`) of `cap-interactive-canvas` are verified, `npx vitest run`, `npx ng build`, and `npm run check:limits` pass with 0 errors.

## Artifact sync required

- Application PRD: `none - already covered in docs/prd.md`
- Application architecture summary: `none - matches docs/architecture/application-architecture-summary.md`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-interactive-canvas.md (transition state to implemented upon completion of 030-031)`
- Issue registry: `required`; node `issues:` reference: `required: .okf/capabilities/cap-interactive-canvas.md`
- Reason/no-impact decision: `Completes all 12 acceptance scenarios (SC-001..SC-012) of cap-interactive-canvas.`

## Human review gate

`visual-review`: User switches between Mode 1, Mode 2, and Mode 3, toggles Greek/English, tests modal dialog guard and toast notifications, scrubs the transient response curve in the Bottom Engineering Dock to see the 3D mesh animate in sync, and verifies identical tension-side moment conventions in Mode 2 and Mode 3.

## Blocked by

- None - Issue 030 implemented and awaiting human visual review

## Artifact anchors

- PRD: `WF-1`, `WF-5`, `WF-6`, `BR-3`, `BR-4`, `BR-5`, `FR-1`, `FR-6`, `FR-7`, `NFR-2`
- Domain model: `DialogService`, `NotificationService`, `EngineeringDock`, `ViewportTransform2D`, `TensionSideDiagram`, `INV-1`, `INV-2`, `INV-6`
- Use cases: `UC-C01`, `UC-C02`, `UC-C08..UC-C10`
- Contract: `ViewportTransform2D`, `buildTensionSideDiagramPath`, `DialogRef`, `NotificationService`, error codes `DIALOG_DISMISS_BLOCKED`, `EXPORT_DATA_EMPTY`

## Acceptance scenarios addressed

- `SC-001`: Tri-Mode Workspace Switching Preserves Active Session State
- `SC-002`: Instant Bilingual Toggle (`el`/`en`) Across Shell, Viewport & Dock
- `SC-003`: Modal Dialog Stack LIFO Enforcement & DismissGuard Protection
- `SC-004`: Toast Notification Deduplication & 5000 ms Auto-Dismissal
- `SC-010`: Synchronized Transient/Harmonic Playback Scrubbing Between Viewport & Dock
- `SC-011`: Live Streaming Solver Convergence Residuals & CSV/SVG Export Guard
- `SC-012`: Shared 2D SVG World-to-Screen Transform & Tension-Side Moment Diagram Parity

## Verification obligations

- Policy source: `/.okf/project.md`

| Scenario | Backend boundary | Frontend integration | End-to-end journey |
|---|---|---|---|
| `SC-001` | `not-applicable` | `implemented: src/app/shared/interactive-canvas.spec.ts` | `catalog-only` |
| `SC-002` | `not-applicable` | `implemented: src/app/shared/interactive-canvas.spec.ts` | `catalog-only` |
| `SC-003` | `not-applicable` | `implemented: src/app/shared/interactive-canvas.spec.ts` | `catalog-only` |
| `SC-004` | `not-applicable` | `implemented: src/app/shared/interactive-canvas.spec.ts` | `catalog-only` |
| `SC-010` | `not-applicable` | `implemented: src/app/shared/interactive-canvas.spec.ts` | `catalog-only` |
| `SC-011` | `implemented: src-tauri/src/solver/` | `implemented: src/app/shared/interactive-canvas.spec.ts` | `catalog-only` |
| `SC-012` | `not-applicable` | `implemented: src/app/shared/interactive-canvas.spec.ts` | `catalog-only` |
