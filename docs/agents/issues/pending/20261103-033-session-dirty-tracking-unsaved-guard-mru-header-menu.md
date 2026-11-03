## Issue Metadata

- Issue number: `033`
- Owning capability node: `/.okf/capabilities/cap-project-persistence.md`
- Artifact root: `docs/architecture/cap-project-persistence/`
- Issue file: `docs/agents/issues/pending/20261103-033-session-dirty-tracking-unsaved-guard-mru-header-menu.md`
- Category: `feature`
- Execution type: `AFK`
- Review gate: `visual-review`
- Suggested state: `ready-for-agent`

## Parent Artifacts

- `docs/prd.md`
- `docs/architecture/application-architecture-summary.md`
- `docs/architecture/cross-capability-conventions.md`
- `.okf/capabilities/cap-project-persistence.md`
- `docs/architecture/cap-project-persistence/prd.md`
- `docs/architecture/cap-project-persistence/canonical-domain-model.md`
- `docs/architecture/cap-project-persistence/canonical-use-cases.md`
- `docs/architecture/cap-project-persistence/canonical-api-cli-contract.md`
- `docs/architecture/cap-project-persistence/acceptance-scenarios.md`
- `docs/architecture/cap-project-persistence/readiness-review.md`

## What to build

Implement **Vertical Slice 2** of `cap-project-persistence`: revision-based session dirty tracking (`project_record_revision`, `project_get_session_state`), the `UnsavedChangesGuard` modal (`Save`, `Discard`, `Cancel` / `PROJECT_UNSAVED_CHANGES`), Recent Projects (MRU) manifest management (`project_list_recent`, `project_manage_recent` with pinning and missing-file cleanup), `project_export_diagram` session preservation (`SC-012`), and the unified Header Project Menu (`New`, `Open`, `Save`, `Save As`, `Recent Projects`) across all 3 workspaces.

### 1. Session State, Dirty Tracking & MRU Backend Commands (`src-tauri/src/project/`):
- Implement `ProjectSessionState` (`file_path`, `project_title`, `saved_revision`, `active_revision`, `is_dirty: active_revision != saved_revision`) and IPC commands `project_record_revision` and `project_get_session_state` (`SC-009`).
- Enforce `PROJECT_UNSAVED_CHANGES` when `project_new` or `project_open` is invoked while `is_dirty == true` and `confirm_discard == false` (`SC-010`).
- Implement `RecentProjectsManifest` (`project_list_recent`, `project_manage_recent` supporting `Pin`, `Unpin`, `Remove`, `ClearUnpinned`, max 15 deduplicated entries ordered with pinned first then `last_opened_at` descending, and `exists_on_disk` verification) (`SC-011`).
- Implement `project_export_diagram` (`UC-C07`, `SC-012`) validating `DiagramExportSpec` (`PROJECT_EXPORT_INVALID_SPEC`) and writing standalone `.svg` or `.png` files without modifying `ProjectSessionState`.

### 2. Angular `ProjectPersistenceService`, `UnsavedChangesGuard` Modal & Header Project Menu (`src/app/shared/`, `src/app/components/header/`):
- Create `ProjectPersistenceService` coordinating workspace snapshots across Mode 1 (`MeshService`, `MaterialService`, `SolverService`, `ContinuumSketcherStore`), Mode 2 (`ModelerStore`), and UI state (`WorkspaceModeService`, `I18nService`).
- Synchronize `active_revision` on edits, undo, redo, and save so undoing back to `saved_revision` automatically clears the dirty indicator (`*` in header project badge) (`SC-009`).
- Add the **Project Menu** (`New`, `Open...`, `Save`, `Save As...`, `Recent Projects` submenu with pin/remove actions) to `HeaderComponent` plus keyboard shortcuts (`Ctrl+N`, `Ctrl+O`, `Ctrl+S`, `Ctrl+Shift+S`).
- Present the bilingual `UnsavedChangesDialogComponent` (`Save`, `Discard`, `Cancel`) whenever `New` or `Open` is triggered while `is_dirty() === true` (`SC-010`).

## Acceptance criteria

- [ ] Editing the workspace sets `is_dirty = true` (`*` in the header project title), undoing back to `saved_revision` restores `is_dirty = false`, redoing sets `is_dirty = true`, and saving sets `saved_revision = active_revision` (`is_dirty = false`) (`SC-009`).
- [ ] Triggering `New Project` or `Open Project` while `is_dirty == true` blocks with `PROJECT_UNSAVED_CHANGES` unless confirmed via the `UnsavedChangesGuard` modal (`Cancel` aborts, `Discard` resets cleanly, `Save` saves first then proceeds) (`SC-010`).
- [ ] Opening/saving projects updates the Recent Projects (MRU) list without duplicates, keeps pinned entries at the top, and flags deleted files (`exists_on_disk = false`) with a one-click remove action (`SC-011`).
- [ ] Exporting diagrams to `.svg` or `.png` writes valid files while leaving `file_path`, `saved_revision`, `active_revision`, and `is_dirty` completely unchanged (`SC-012`).
- [ ] Vitest tests (`project-persistence.spec.ts`), `cargo test --lib`, `npx ng build`, and `npm run check:limits` pass with 0 errors.

## Artifact sync required

- Application PRD: `none - already covered in docs/prd.md`
- Application architecture summary: `none - matches docs/architecture/application-architecture-summary.md`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-project-persistence.md (transition state to implemented upon completion of 032-033)`
- Issue registry: `required`; node `issues:` reference: `required: .okf/capabilities/cap-project-persistence.md`
- Reason/no-impact decision: `Completes all 12 acceptance scenarios (SC-001..SC-012) of cap-project-persistence on the application shell.`

## Human review gate

`visual-review`: User uses the Header Project Menu to save a project (`.bem`), makes edits to observe the `*` dirty indicator, tests Undo/Redo dirty state toggling, triggers `New Project` to verify the Unsaved Changes guard modal, and reopens the project from the Recent Projects (MRU) list.

## Blocked by

- Blocked by `docs/agents/issues/pending/20261103-032-canonical-bem-schema-sha256-migration-atomic-io.md`

## Artifact anchors

- PRD: `W1`, `W2`, `W3`, `W4`, `W5`, `W6`, `BR-3`, `BR-6`
- Domain model: `ProjectSessionState`, `RecentProjectsManifest`, `UnsavedChangesGuard`, `DiagramExportSpec`, `INV-P5`, `INV-P6`
- Use cases: `UC-C01..UC-C07`, `UC-Q01`, `UC-Q02`
- Contract: `project_record_revision`, `project_get_session_state`, `project_list_recent`, `project_manage_recent`, `project_export_diagram`, error codes `PROJECT_UNSAVED_CHANGES`, `PROJECT_EXPORT_INVALID_SPEC`

## Acceptance scenarios addressed

- `SC-001`: Lossless Round-Trip Save and Open of Mode 2 Structural Workspace
- `SC-002`: Lossless Round-Trip of Mode 1 Continuum & `ContinuumDomain` Sketch
- `SC-004`: Forward/Backward Schema Migration of Legacy `.bem` File
- `SC-005`: Rejection of Unsupported Schema Version (`PROJECT_SCHEMA_UNSUPPORTED`)
- `SC-006`: Rejection of Corrupted JSON, Non-Finite Floats, or Tampered Checksum (`PROJECT_CORRUPT_JSON`)
- `SC-007`: Atomic Write Failure Preserves Existing `.bem` File (`PROJECT_IO_ERROR`)
- `SC-008`: Stale `solution_cache` Stripped When `model_revision` Mismatches
- `SC-009`: Revision-Based Dirty Tracking Across Edits, Undo/Redo, and Save
- `SC-010`: Unsaved-Changes Guard Blocks Destructive New/Open/Close Until Confirmed
- `SC-011`: Recent Projects (MRU) Deduplication, Ordering, Pinning, and Missing-File Cleanup
- `SC-012`: SVG and PNG Diagram Export Without Altering Project Session State

## Verification obligations

- Policy source: `/.okf/project.md`

| Scenario | Backend boundary | Frontend integration | End-to-end journey |
|---|---|---|---|
| `SC-001` | `implemented: Issue 032` | `planned` | `catalog-only` |
| `SC-002` | `implemented: Issue 032` | `planned` | `catalog-only` |
| `SC-004` | `implemented: Issue 032` | `planned` | `catalog-only` |
| `SC-005` | `implemented: Issue 032` | `planned` | `catalog-only` |
| `SC-006` | `implemented: Issue 032` | `planned` | `catalog-only` |
| `SC-007` | `implemented: Issue 032` | `planned` | `catalog-only` |
| `SC-008` | `implemented: Issue 032` | `planned` | `catalog-only` |
| `SC-009` | `planned` | `planned` | `catalog-only` |
| `SC-010` | `planned` | `planned` | `catalog-only` |
| `SC-011` | `planned` | `planned` | `catalog-only` |
| `SC-012` | `planned` | `planned` | `catalog-only` |
