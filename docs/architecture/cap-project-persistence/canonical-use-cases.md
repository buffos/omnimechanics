# Canonical Use-Case Model: Project Persistence & Self-Contained Workspace Storage (`cap-project-persistence`)

- **Owning Capability:** [`cap-project-persistence`](../../../.okf/capabilities/cap-project-persistence.md)
- **Node PRD:** [`prd.md`](prd.md)
- **Canonical Domain Model:** [`canonical-domain-model.md`](canonical-domain-model.md)
- **Conventions Reference:** [`cross-capability-conventions.md`](../cross-capability-conventions.md)
- **Status:** Canonical Application/Use-Case Model (2026-11-01)

---

## 1. Purpose & Application-Layer Boundaries

This document defines the transport-neutral application commands, queries, orchestration services, transaction rules, and failure semantics for `cap-project-persistence`.

**Application Service Groupings:**
1. **`ProjectLifecycleService`**: Orchestrates `NewProject`, `OpenProject`, `SaveProject`, `SaveProjectAs`, and `ValidateProjectFile`. Coordinates `ProjectDocument` assembly/parsing, `Schema Migration Policy`, SHA-256 verification, atomic disk I/O, and `ProjectSessionState` transitions.
2. **`SessionDirtyTrackerService`**: Orchestrates `RecordWorkspaceRevision` and `GetSessionState`, maintaining `(saved_revision, active_revision)` parity across workspace edits and undo/redo operations.
3. **`RecentProjectsService`**: Orchestrates `ListRecentProjects`, `RemoveRecentProject`, and `ClearRecentProjects` against the local `RecentProjectsManifest`.
4. **`DiagramExportService`**: Orchestrates `ExportDiagram` to render and write standalone `.svg` and `.png` files without side-effecting `ProjectSessionState`.

---

## 2. Canonical Commands & Queries Inventory

| ID | Name | Kind | Service | Summary |
| :--- | :--- | :--- | :--- | :--- |
| **UC-C01** | `NewProject` | Command | `ProjectLifecycleService` | Resets workspace to a clean default `ProjectDocument` after checking `UnsavedChangesGuard`. |
| **UC-C02** | `OpenProject` | Command | `ProjectLifecycleService` | Reads, validates, migrates, and restores a `.bem` file into the active workspace and updates MRU. |
| **UC-C03** | `SaveProject` | Command | `ProjectLifecycleService` | Serializes and atomically writes `ProjectDocument` to the current `file_path` (or delegates to `SaveProjectAs`). |
| **UC-C04** | `SaveProjectAs` | Command | `ProjectLifecycleService` | Serializes and atomically writes `ProjectDocument` to a newly chosen `.bem` path, updating `file_path` and MRU. |
| **UC-C05** | `RecordWorkspaceRevision` | Command | `SessionDirtyTrackerService` | Updates `active_revision` upon model/workspace mutation or undo/redo, recomputing `is_dirty`. |
| **UC-C06** | `ManageRecentProjects` | Command | `RecentProjectsService` | Pins, unpins, removes a stale entry, or clears unpinned entries in `RecentProjectsManifest`. |
| **UC-C07** | `ExportDiagram` | Command | `DiagramExportService` | Writes a print- or canvas-themed `.svg` or `.png` diagram to disk atomically. |
| **UC-Q01** | `GetSessionState` | Query | `SessionDirtyTrackerService` | Returns current `file_path`, `saved_revision`, `active_revision`, `is_dirty`, and project metadata summary. |
| **UC-Q02** | `ListRecentProjects` | Query | `RecentProjectsService` | Returns the ordered MRU list with file-existence status flags. |
| **UC-Q03** | `InspectProjectHeader` | Query | `ProjectLifecycleService` | Reads metadata, `schema_version`, and checksum validity of a `.bem` file without loading it into the active session. |

---

## 3. Detailed Command & Query Specifications

### 3.1 UC-C01: `NewProject`
- **Inputs:** `{ active_mode: WorkspaceMode, title?: String, dimensions?: "2D" | "3D", confirm_discard: bool }`
- **Preconditions:** Either `session.is_dirty == false` OR `confirm_discard == true`.
- **Orchestration:**
  1. Check `session.is_dirty()`. If `true` and `!confirm_discard`, abort with `PROJECT_UNSAVED_CHANGES`.
  2. Construct a default `ProjectDocument` with `schema_version = "1.0"`, new UUID `project_id`, `created_at = now()`, default `NationalAnnexProfile ("GR")` in `structural` (with `model.revision = 0`), and default `ui_state`.
  3. Compute `integrity.payload_sha256`.
  4. Reset `ProjectSessionState`: `file_path = None`, `saved_revision = 0`, `active_revision = 0`.
- **Outputs:** `{ document: ProjectDocument, session: ProjectSessionState }`
- **Failures:** `PROJECT_UNSAVED_CHANGES`

### 3.2 UC-C02: `OpenProject`
- **Inputs:** `{ file_path?: String, confirm_discard: bool }` (if `file_path` is omitted, invokes native OS open dialog filtered to `*.bem`).
- **Preconditions:** Either `session.is_dirty == false` OR `confirm_discard == true`.
- **Orchestration:**
  1. Check `session.is_dirty()`. If `true` and `!confirm_discard`, abort with `PROJECT_UNSAVED_CHANGES` before reading disk.
  2. If `file_path` is `None`, open native file picker; if user cancels picker, return `Cancelled` no-op outcome without modifying session.
  3. Read raw bytes from `file_path`. Map filesystem errors (not found, permission denied) to `PROJECT_IO_ERROR`.
  4. Parse UTF-8 JSON envelope. Map syntax errors or missing required envelope keys to `PROJECT_CORRUPT_JSON`.
  5. Validate `schema_version`:
     - If unsupported (`major != 1` or `minor > CURRENT_MINOR`), abort with `PROJECT_SCHEMA_UNSUPPORTED`.
     - If `minor < CURRENT_MINOR`, execute sequential migration steps `v1.x -> v1.CURRENT_MINOR`.
  6. Verify `integrity.payload_sha256` against canonicalized payload sections and check INV-P1 (finite floats). On mismatch or `NaN`/`Infinity`, abort with `PROJECT_CORRUPT_JSON`.
  7. Check INV-P4: if `solution_cache` exists and `solution_cache.model_revision != structural.model.revision`, strip `solution_cache = None` and record `stale_cache_stripped = true`.
  8. Determine restored revision `r = structural.map(|s| s.model.revision).unwrap_or(0)`. Update `ProjectSessionState` with `file_path = Some(normalized_path)`, `saved_revision = r`, `active_revision = r`.
  9. Upsert `RecentProjectEntry` in `RecentProjectsManifest` and persist manifest.
- **Outputs:** `{ document: ProjectDocument, session: ProjectSessionState, migration_report?: MigrationReport, stale_cache_stripped: bool }`
- **Failures:** `PROJECT_UNSAVED_CHANGES`, `PROJECT_IO_ERROR`, `PROJECT_SCHEMA_UNSUPPORTED`, `PROJECT_CORRUPT_JSON`

### 3.3 UC-C03 & UC-C04: `SaveProject` & `SaveProjectAs`
- **Inputs:** `{ target_path?: String, document_payload: ProjectPayloadInput, include_solution_cache: bool }`
- **Preconditions:** `document_payload` contains finite numeric values (INV-P1).
- **Orchestration:**
  1. Resolve destination path:
     - For `SaveProject`: use `session.file_path`. If `None`, prompt native OS save dialog (`*.bem`).
     - For `SaveProjectAs`: use `target_path` if explicitly passed, else prompt native OS save dialog (`*.bem`). If user cancels dialog, return `Cancelled` no-op.
  2. Ensure path ends with `.bem` extension.
  3. Strip `solution_cache` if `include_solution_cache == false`.
  4. Update `metadata.updated_at = now()`, `metadata.app_version`, and `schema_version = "1.0"`.
  5. Canonicalize payload sections (sorted keys, 2-space indent, `\n` newlines, finite floats) and compute `integrity.payload_sha256`.
  6. Write complete canonical JSON to `<resolved_path>.<pid>.tmp`, flush OS buffers (`sync_all`), and atomically rename to `<resolved_path>`. If any I/O step fails, remove `.tmp` file if present and abort with `PROJECT_IO_ERROR` (leaving any prior `.bem` file untouched).
  7. Update `ProjectSessionState`: `file_path = Some(resolved_path)`, `saved_revision = active_revision`, `last_saved_at = Some(now())`.
  8. Upsert `RecentProjectEntry` in `RecentProjectsManifest`.
- **Outputs:** `{ session: ProjectSessionState, metadata: ProjectMetadata, integrity: DocumentIntegrity, bytes_written: u64 }`
- **Failures:** `PROJECT_CORRUPT_JSON` (if input payload contains `NaN`/`Infinity`), `PROJECT_IO_ERROR`

### 3.4 UC-C05: `RecordWorkspaceRevision`
- **Inputs:** `{ active_revision: u64 }`
- **Orchestration:**
  1. Set `session.active_revision = active_revision`.
  2. Recompute `is_dirty = session.active_revision != session.saved_revision`.
  3. Emit `ProjectDirtyStateChanged` if `is_dirty` flipped.
- **Outputs:** `ProjectSessionState`

### 3.5 UC-C07: `ExportDiagram`
- **Inputs:** `DiagramExportSpec` (`output_path?`, `format: "Svg" | "Png"`, `theme`, `width_px`, `height_px`, `dpi_scale`, `include_title_block`, `include_legend`, `svg_markup`)
- **Orchestration:**
  1. Validate `width_px`, `height_px`, `dpi_scale`, and non-empty well-formed `svg_markup`.
  2. If `output_path` is omitted, prompt native OS save dialog filtered to `.svg` or `.png`. If cancelled, return `Cancelled`.
  3. Apply requested `theme` (`LightPrint` or `DarkCanvas`), optional title block, and legend overlay.
  4. If `format == "Svg"`, write UTF-8 SVG via atomic `.tmp` rename. If `format == "Png"`, rasterize SVG at `(width_px * dpi_scale, height_px * dpi_scale)` and write PNG bytes via atomic `.tmp` rename.
  5. Preserve `ProjectSessionState` unchanged (`file_path` and `saved_revision` are never modified by diagram export).
- **Outputs:** `{ output_path: String, format: DiagramFormat, bytes_written: u64 }`
- **Failures:** `PROJECT_EXPORT_INVALID_SPEC`, `PROJECT_IO_ERROR`

---

## 4. Transaction, Idempotency & Failure Model

- **Atomicity Guarantee:** `OpenProject` never mutates `ProjectSessionState` or the active frontend workspace until schema validation, migration, and SHA-256 verification all succeed. `SaveProject` / `SaveProjectAs` never overwrite the existing `.bem` file until the `.tmp` file is completely flushed.
- **Idempotency:**
  - Calling `RecordWorkspaceRevision(r)` repeatedly with the same `r` is idempotent.
  - Calling `SaveProject` twice without intervening edits writes an updated `updated_at` header while keeping `payload_sha256` identical and `is_dirty == false`.
  - `ManageRecentProjects` upserts by canonicalized filesystem path, preventing duplicate MRU entries.

| Failure Category | Canonical Error Code | Session State Impact |
| :--- | :--- | :--- |
| Unsaved edits blocked by guard | `PROJECT_UNSAVED_CHANGES` | Active workspace & `is_dirty` preserved unchanged |
| Future or obsolete `.bem` version | `PROJECT_SCHEMA_UNSUPPORTED` | Active workspace & `is_dirty` preserved unchanged |
| Malformed JSON, `NaN`, or bad SHA-256 | `PROJECT_CORRUPT_JSON` | Active workspace & `is_dirty` preserved unchanged |
| OS file missing, locked, or disk full | `PROJECT_IO_ERROR` | Prior `.bem` file on disk & active session preserved |
| Invalid export dimensions or empty SVG | `PROJECT_EXPORT_INVALID_SPEC` | Active session preserved unchanged |

---

## 5. End-to-End Use-Case Chains

1. **Chain 1 — Authoring, Dirty Guard, and Save-Before-New:**
   `NewProject` (`saved_rev=0, active_rev=0, is_dirty=false`) $\to$ User draws frame in Mode 2 $\to$ `RecordWorkspaceRevision(1)` (`is_dirty=true`) $\to$ User clicks `New Project` (`confirm_discard=false`) $\to$ Rejects with `PROJECT_UNSAVED_CHANGES` $\to$ UI shows Guard Modal $\to$ User clicks `Save` $\to$ `SaveProjectAs("frame1.bem")` (`saved_rev=1, is_dirty=false`) $\to$ `NewProject` (`confirm_discard=false`) succeeds cleanly.
2. **Chain 2 — Edit, Undo Back to Clean State:**
   `OpenProject("beam.bem")` (`saved_rev=4, active_rev=4, is_dirty=false`) $\to$ User adds load (`RecordWorkspaceRevision(5)`, `is_dirty=true`) $\to$ User triggers Undo (`RecordWorkspaceRevision(4)`, `is_dirty=false`) $\to$ User closes app or opens another project without any false-positive unsaved-changes prompt.
