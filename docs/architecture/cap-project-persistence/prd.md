# Product Requirements Document: Project Persistence & Self-Contained Workspace Storage (`cap-project-persistence`)

- **Owning Capability:** [`cap-project-persistence`](../../../.okf/capabilities/cap-project-persistence.md)
- **Root Synthesis PRD:** [`docs/prd.md`](../../prd.md)
- **Domain Glossary:** [`domain-glossary.md`](domain-glossary.md)
- **Conventions Reference:** [`cross-capability-conventions.md`](../cross-capability-conventions.md)
- **Status:** Canonical Node PRD (2026-11-01)

---

## 1. Purpose & Product Summary

OmniMechanics serves students, structural engineers, instructors, and computational mechanics researchers across three integrated workspaces: Mode 1 (Continuum Workbench), Mode 2 (Structural Modeling & Analysis), and Mode 3 (Teaching Studio). Without reliable local persistence, users lose complex 2D structural frames, Eurocode load case permutations, continuum domain sketches, and benchmark configurations when switching tasks or closing the application.

`cap-project-persistence` provides deterministic, versioned, self-contained `.bem` project file serialization, schema migration, integrity verification, dirty-state tracking with unsaved-changes protection, recent-project (MRU) management, and high-resolution SVG/PNG diagram export via native desktop file dialogs.

---

## 2. Goals & Non-Goals

### 2.1 Goals
1. **Self-Contained Reproducibility:** Persist the complete workspace state—including Mode 1 continuum presets/meshes/materials/BCs, Mode 2 `StructuralModel` + `LoadCase`s + EN 1990/1998 `Combination`s + `NationalAnnexProfile`, `ContinuumDomain` polygon sketches, optional `solution_cache`, and `ui_state`—into a single human-readable, Git-friendly `.bem` JSON document.
2. **Schema Evolution & Integrity:** Guarantee backward compatibility across minor schema evolutions via a deterministic migration pipeline and protect users from corrupted or truncated files with SHA-256 payload verification and bilingual (`el`/`en`) error reporting (`PROJECT_SCHEMA_UNSUPPORTED`, `PROJECT_CORRUPT_JSON`, `PROJECT_IO_ERROR`).
3. **Zero Accidental Data Loss:** Track workspace revisions (`saved_revision` vs. `active_revision`) across all edits and undo/redo actions, presenting an unsaved-changes confirmation guard before `New Project`, `Open Project`, or window closure.
4. **Native Desktop Ergonomics:** Provide standard desktop file workflows (`New`, `Open`, `Save`, `Save As`, Recent Projects MRU menu with quick re-open) and publication-quality diagram export (`SVG` and `PNG`) for coursework and engineering deliverables.

### 2.2 Non-Goals (Current Planning Horizon)
- **Cloud Sync or Remote Databases:** Projects are stored strictly as local `.bem` files on the user's filesystem (`.okf/capabilities/cap-project-persistence.md`).
- **Multi-User Real-Time Collaboration:** Concurrent editing or CRDT merging of `.bem` files is out of scope.
- **Full PDF / LaTeX Calculation Report Generation:** Scheduled for a dedicated future slice (discovery Q24).
- **CAD Geometry Import (`.dxf`, `.svg`, `.stl`):** Owned by sibling capability [`cap-geometry-cad-import`](../../../.okf/capabilities/cap-geometry-cad-import.md).
- **Domain Validation Ownership:** `cap-project-persistence` validates the `.bem` envelope, schema version, and JSON/checksum integrity, and delegates structural or continuum domain invariant checks to `cap-frame-analysis-engine`, `cap-eurocode-actions-combinations`, and `cap-sketcher-continuum-domain`.

---

## 3. Actors & User Journeys

| Actor | Persistence & Export Needs |
| :--- | :--- |
| **Student** | Saves half-finished frame homework in `.bem`, reopens it from the Recent Projects list, switches between Mode 2 and Mode 3 step-by-step derivations, and exports clean M/Q/N diagrams to SVG/PNG for lab reports. |
| **Engineer / Instructor** | Prepares reusable structural problem sets or multi-story 2D seismic frames with Greek National Annex combinations, distributes self-contained `.bem` files to students or colleagues, and exports print-ready white-background SVG/PNG envelopes. |
| **Researcher** | Saves Mode 1 BEM/FEM elastodynamic configurations together with optional converged `solution_cache` fields so heavy 3D/2D benchmarks can be inspected immediately upon reopening without re-solving. |

---

## 4. Required Workflows

### 4.1 W1 — New Project with Unsaved-Changes Guard
1. User triggers `New Project` (toolbar button, menu, or `Ctrl+N` / `Cmd+N`).
2. If `is_dirty == true` (`active_revision != saved_revision`), the UI intercepts the action and presents the bilingual `UnsavedChangesGuard` modal (`Save`, `Discard`, `Cancel`).
3. If `Save` is chosen, the system executes **W3 (Save Project)** or **W4 (Save As)** first; if that save fails or is cancelled, `New Project` aborts.
4. When confirmed (`Discard` or after successful `Save`, or immediately if `is_dirty == false`), the system resets the workspace to a clean default `ProjectDocument`, resets `active_revision = 0` and `saved_revision = 0`, clears the active `file_path`, and updates the window title.

### 4.2 W2 — Open Project (`.bem`) & Schema Migration
1. User triggers `Open Project` (native file dialog filtered to `*.bem` or click on a `RecentProjectEntry`).
2. If `is_dirty == true`, the `UnsavedChangesGuard` runs first.
3. The backend reads the file from disk, parses the JSON envelope, and inspects `schema_version`:
   - If `schema_version` is unsupported (e.g., major version higher than current `"1.0"` or below `"1.0"`), returns `PROJECT_SCHEMA_UNSUPPORTED` without altering the active workspace.
   - If `schema_version` is an older supported minor version, runs the deterministic `Schema Migration Policy` chain to upgrade the document to the current schema.
4. The backend verifies `integrity.payload_sha256` (when present) and validates finite numeric values (`no NaN/Infinity`). On mismatch or syntax error, returns `PROJECT_CORRUPT_JSON`.
5. On success, the backend restores the domain state, discards any stale `solution_cache` whose `model_revision` does not match `structural.model.revision`, sets `saved_revision = active_revision`, records `file_path`, updates the `RecentProjectsManifest`, and switches the UI to `active_mode`.

### 4.3 W3 — Save Project (In-Place Atomic Save)
1. User triggers `Save Project` (`Ctrl+S` / `Cmd+S`).
2. If `ProjectSessionState.file_path` is `None` (untitled project), delegates automatically to **W4 (Save As)**.
3. Otherwise, assembles the current `ProjectDocument` snapshot from the active workspaces, updates `metadata.updated_at`, computes `integrity.payload_sha256` over canonical JSON, writes to `<file_path>.tmp`, flushes to disk, and atomically replaces `<file_path>`.
4. Sets `saved_revision = active_revision` (`is_dirty = false`) and updates the MRU entry timestamp.

### 4.4 W4 — Save Project As (`.bem`)
1. User triggers `Save As...` (`Ctrl+Shift+S` / `Cmd+Shift+S`) or saves an untitled project.
2. Opens the native OS save file dialog filtered to `.bem` (auto-appending `.bem` if omitted).
3. If the user cancels the native dialog, the operation terminates cleanly with `is_dirty` unchanged.
4. Performs the atomic write to the chosen path, updates `ProjectSessionState.file_path`, sets `saved_revision = active_revision`, and prepends the file to `RecentProjectsManifest`.

### 4.5 W5 — Recent Projects (MRU) Management
1. On startup or menu open, the UI queries `RecentProjectsManifest` (up to 15 deduplicated entries ordered by `last_opened_at` descending, with pinned items retained).
2. Clicking an entry invokes **W2 (Open Project)** for that path.
3. If the file no longer exists on disk, raises `PROJECT_IO_ERROR` and offers to remove the stale entry from the MRU list.

### 4.6 W6 — Diagram Export (`SVG` / `PNG`)
1. User triggers `Export Diagram...` from Mode 2 (M/Q/N, deformed shape, reactions, envelope, or structure geometry) or Mode 1/3 canvas.
2. User selects export parameters (`Svg` or `Png`, theme `LightPrint` or `DarkCanvas`, scale/DPI, title block & legend visibility) and chooses an output path via native save dialog.
3. The system renders and writes the standalone `.svg` or `.png` file atomically without changing `ProjectSessionState.file_path` or `saved_revision`.

---

## 5. Business Rules & Policies

1. **Deterministic Canonical Serialization (BR-1):** `.bem` files must serialize with UTF-8 encoding, lexicographically sorted keys, 2-space indentation, `\n` line endings, and finite IEEE 754 numbers. Identical workspace states saved at the same timestamp must produce byte-identical payloads and matching `payload_sha256` hashes.
2. **Persistence Owns No Mechanics Logic (BR-2):** Per [`cross-capability-conventions.md`](../cross-capability-conventions.md) §2, `cap-project-persistence` serializes and migrates `StructuralModel`, `LoadCase`s, `Combination`s, `NationalAnnexProfile`, and `ContinuumDomain`, but never re-implements structural analysis, combination factor generation, or meshing algorithms.
3. **Revision-Based Dirty Tracking (BR-3):** `is_dirty` is true if and only if `active_revision != saved_revision`. Undoing back to the exact revision that was last saved must automatically restore `is_dirty == false`.
4. **Stale Solution Cache Rejection (BR-4):** When loading a `.bem` file containing `solution_cache`, if `solution_cache.model_revision != structural.model.revision`, the cache is stripped during load and a non-blocking diagnostic notice is returned so stale results are never shown as current.
5. **Atomic File Replacement (BR-5):** Writing a `.bem` file must never truncate the existing target file directly; it must write and flush a temporary sibling file first and rename atomically. If disk I/O fails midway, the previous `.bem` file remains intact.
6. **Bilingual Error Contract (BR-6):** Every error returned across Tauri IPC (`PROJECT_SCHEMA_UNSUPPORTED`, `PROJECT_CORRUPT_JSON`, `PROJECT_IO_ERROR`, `PROJECT_UNSAVED_CHANGES`) must carry both `message_el` (Greek) and `message_en` (English) plus structured `details`.

---

## 6. Functional & Non-Functional Requirements

- **FR-1 (`ProjectDocument` Completeness):** Must support round-trip lossless persistence of:
  - `schema_version` (`"1.0"`), `metadata`, `integrity`
  - `active_mode` (`ContinuumWorkbench`, `StructuralModeler`, `TeachingStudio`)
  - `continuum` (Mode 1 preset, mesh, material inhomogeneity field, boundary conditions)
  - `continuum_sketch` (`ContinuumDomain` outer boundary loop, holes, boundary tags)
  - `structural` (`StructuralModel` with nodes, elements, end releases, rigid offsets, library/custom sections & materials, supports, loads, load cases, lumped masses, `revision`; `Combination`s with `LimitState` and `origin`; `NationalAnnexProfile`; `SeismicSettings`)
  - `solution_cache` (optional toggle on save; default `false` for compact files, selectable `true` for benchmark/result archiving)
  - `ui_state` (active language, viewport pan/zoom, snap/grid settings, diagram scale factors, active result target)
- **NFR-1 (Save/Load Latency):** Saving or loading a typical Mode 2 project (500 nodes, 1,000 members, 50 combinations, without heavy 3D continuum mesh cache) must complete in `< 50 ms` on standard desktop SSD storage.
- **NFR-2 (File Size Compactness):** A 500-node / 1,000-member structural project without cached continuum field arrays must serialize to `< 1.5 MB` uncompressed UTF-8 JSON.

---

## 7. Acceptance Scenarios Summary

Canonical scenarios `SC-001` through `SC-012` are specified in [`acceptance-scenarios.md`](acceptance-scenarios.md), covering full Mode 2 and Mode 1/ContinuumSketch round-trips, legacy schema migration, unsupported future schema rejection, corrupt JSON/checksum rejection, atomic write failure recovery, dirty-state tracking across edit/undo/save, unsaved-changes guard flows, MRU list lifecycle, and SVG/PNG diagram export.

---

## 8. Explicit Assumptions & Artifact Impact

- **Assumptions:**
  - `.bem` files use uncompressed UTF-8 JSON in schema `"1.0"` so engineers and instructors can inspect diffs in Git; optional gzip compression can be introduced in a future minor schema version without breaking the envelope.
- **Artifact Impact Classification:** `no impact` on root [`docs/prd.md`](../../prd.md) and [`docs/architecture/application-architecture-summary.md`](../application-architecture-summary.md) because both root synthesis documents already declare `cap-project-persistence` as the owner of JSON project save/load (`ProjectDocument`) and diagram export (discovery Q24).
