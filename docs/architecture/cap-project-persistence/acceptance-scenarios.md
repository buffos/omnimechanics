# Acceptance Scenarios: Project Persistence & Self-Contained Workspace Storage (`cap-project-persistence`)

- **Owning Capability:** [`cap-project-persistence`](../../../.okf/capabilities/cap-project-persistence.md)
- **Node PRD:** [`prd.md`](prd.md)
- **Canonical Domain Model:** [`canonical-domain-model.md`](canonical-domain-model.md)
- **Canonical Use Cases:** [`canonical-use-cases.md`](canonical-use-cases.md)
- **Canonical Contract:** [`canonical-api-cli-contract.md`](canonical-api-cli-contract.md)
- **Root Verification Policy:** `backend-boundary: required`, `frontend-integration: required`, `end-to-end: catalog-only` ([`.okf/project.md`](../../../.okf/project.md))
- **Status:** Canonical Acceptance Scenarios (2026-11-01)

---

## 1. Scenario Coverage Matrix

| Scenario ID | Title | Primary Rule / Workflow | `backend-boundary` | `frontend-integration` | `end-to-end` |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **SC-001** | Lossless Round-Trip Save and Open of Mode 2 Structural Workspace | W3, W2, FR-1, BR-1 | `required` | `required` | `catalog-only` |
| **SC-002** | Lossless Round-Trip of Mode 1 Continuum & `ContinuumDomain` Sketch | W3, W2, FR-1 | `required` | `required` | `catalog-only` |
| **SC-003** | Deterministic Canonical JSON & SHA-256 Digest Reproducibility | BR-1, INV-P3 | `required` | `not-applicable` | `not-applicable` |
| **SC-004** | Forward/Backward Schema Migration of Legacy `.bem` File | W2, Policy 1 | `required` | `required` | `catalog-only` |
| **SC-005** | Rejection of Unsupported Schema Version (`PROJECT_SCHEMA_UNSUPPORTED`) | W2, Policy 1, BR-6 | `required` | `required` | `catalog-only` |
| **SC-006** | Rejection of Corrupted JSON, Non-Finite Floats, or Tampered Checksum (`PROJECT_CORRUPT_JSON`) | W2, INV-P1, INV-P3, BR-6 | `required` | `required` | `catalog-only` |
| **SC-007** | Atomic Write Failure Preserves Existing `.bem` File (`PROJECT_IO_ERROR`) | W3, BR-5, Policy 2 | `required` | `required` | `catalog-only` |
| **SC-008** | Stale `solution_cache` Stripped When `model_revision` Mismatches | W2, BR-4, INV-P4 | `required` | `required` | `catalog-only` |
| **SC-009** | Revision-Based Dirty Tracking Across Edits, Undo/Redo, and Save | W1-W4, BR-3, INV-P5 | `required` | `required` | `catalog-only` |
| **SC-010** | Unsaved-Changes Guard Blocks Destructive New/Open/Close Until Confirmed | W1, W2, Policy 3 | `required` | `required` | `catalog-only` |
| **SC-011** | Recent Projects (MRU) Deduplication, Ordering, Pinning, and Missing-File Cleanup | W5, INV-P6 | `required` | `required` | `catalog-only` |
| **SC-012** | SVG and PNG Diagram Export Without Altering Project Session State | W6, `UC-C07` | `required` | `required` | `catalog-only` |

---

## 2. Canonical Scenarios

### SC-001: Lossless Round-Trip Save and Open of Mode 2 Structural Workspace
- **Purpose:** Prove that a complete Mode 2 workspace (`StructuralModel` with frame members, truss bars, springs, catenary cables, end releases, rigid offsets, supports, load cases, loads, lumped masses, EN 1990/1998 `Combination`s, `NationalAnnexProfile`, `SeismicSettings`, and `ui_state`) saves to `.bem` and restores identically.
- **Preconditions:** An active Mode 2 session at `revision = 14` with 2 load cases (`G`, `Q`), 3 combinations (`ULS_STR`, `SLS_CHAR`, `SEISMIC`), Greek National Annex (`GR`), and custom viewport/diagram scales.
- **Action:** Invoke `SaveProjectAs("portal_frame.bem")`, reset session via `NewProject`, and invoke `OpenProject("portal_frame.bem")`.
- **Expected Outcome:**
  - The restored `ProjectDocument` equals the saved document across all `structural` and `ui_state` fields.
  - `ProjectSessionState` shows `file_path = ".../portal_frame.bem"`, `saved_revision = 14`, `active_revision = 14`, and `is_dirty = false`.
- **Verification Surfaces:**
  - `backend-boundary`: Rust round-trip serialization/deserialization test over `project_save_as` and `project_open`.
  - `frontend-integration`: Vitest store test verifying Mode 2 model store + UI state restoration from `project_open` payload.
  - `end-to-end`: User saves a sketched frame via Header `Save As`, clicks `New`, reopens the `.bem` file, and observes identical geometry, loads, combinations, and canvas zoom.

### SC-002: Lossless Round-Trip of Mode 1 Continuum & `ContinuumDomain` Sketch
- **Purpose:** Verify that Mode 1 continuum parameters (`preset_id`, `mesh`, `material_field`, `boundary_conditions`), `continuum_sketch` (`ContinuumDomain` outer loop, interior holes, boundary tags), and optional `solution_cache` round-trip without loss.
- **Preconditions:** Mode 1 session with functional gradation material field, a 2D `ContinuumDomain` polygon with 1 circular hole, and a converged `solution_cache` at `model_revision = 5`.
- **Action:** Save with `include_solution_cache = true` to `"continuum_bench.bem"` and reopen.
- **Expected Outcome:** All Mode 1 preset/material/BC settings, `ContinuumDomain` loops/tags, and cached displacement/traction arrays are restored with `stale_cache_stripped = false`.
- **Verification Surfaces:** `backend-boundary: required`, `frontend-integration: required`, `end-to-end: catalog-only`.

### SC-003: Deterministic Canonical JSON & SHA-256 Digest Reproducibility
- **Purpose:** Verify that canonical JSON serialization produces deterministic key ordering, `\n` line endings, and identical `payload_sha256` hashes independent of insertion order.
- **Preconditions:** Two in-memory `ProjectDocument` instances with identical semantic content and timestamps but constructed with different hash-map insertion orders.
- **Action:** Serialize both via the canonical serializer.
- **Expected Outcome:** Both byte buffers and `integrity.payload_sha256` digests are bit-for-bit identical.
- **Verification Surfaces:** `backend-boundary: required`, `frontend-integration: not-applicable`, `end-to-end: not-applicable`.

### SC-004: Forward/Backward Schema Migration of Legacy `.bem` File
- **Purpose:** Verify that a `.bem` file written with an older migratable minor schema version is upgraded automatically on open without data loss.
- **Preconditions:** A fixture `.bem` file with an older migratable version missing newly added optional fields (e.g., `seismic_settings` or `continuum_sketch`).
- **Action:** Invoke `OpenProject` on the legacy fixture.
- **Expected Outcome:** The migration pipeline populates canonical defaults, upgrades `schema_version` in memory to `"1.0"`, returns a populated `migration_report`, and loads the project cleanly.
- **Verification Surfaces:** `backend-boundary: required`, `frontend-integration: required`, `end-to-end: catalog-only`.

### SC-005: Rejection of Unsupported Schema Version (`PROJECT_SCHEMA_UNSUPPORTED`)
- **Purpose:** Ensure `.bem` files from an incompatible future version (`"2.0"` or `"1.99"`) or invalid version (`"0.5"`) are rejected cleanly without corrupting the active session.
- **Preconditions:** Active session has a loaded project at `revision = 3` (`is_dirty = false`). Target file `"future.bem"` declares `"schema_version": "2.0"`.
- **Action:** Invoke `OpenProject("future.bem")`.
- **Expected Outcome:**
  - Command fails with `code: "PROJECT_SCHEMA_UNSUPPORTED"`, bilingual `message_el` / `message_en`, and `details.found_version = "2.0"`.
  - The active workspace at `revision = 3` remains completely untouched.
- **Verification Surfaces:** `backend-boundary: required`, `frontend-integration: required`, `end-to-end: catalog-only`.

### SC-006: Rejection of Corrupted JSON, Non-Finite Floats, or Tampered Checksum (`PROJECT_CORRUPT_JSON`)
- **Purpose:** Ensure syntax-broken JSON, `NaN`/`Infinity` values, or manually altered payload bytes that mismatch `payload_sha256` are rejected with `PROJECT_CORRUPT_JSON`.
- **Preconditions:** Three invalid fixtures: (a) truncated JSON file, (b) valid JSON with a tampered node coordinate that breaks `payload_sha256`, (c) payload containing `NaN`.
- **Action:** Attempt `OpenProject` on (a) and (b), and `SaveProject` on (c).
- **Expected Outcome:** Each call rejects with `code: "PROJECT_CORRUPT_JSON"` and specific `details.reason` (`"SyntaxError"`, `"ChecksumMismatch"`, `"NonFiniteNumber"`), leaving the active workspace untouched.
- **Verification Surfaces:** `backend-boundary: required`, `frontend-integration: required`, `end-to-end: catalog-only`.

### SC-007: Atomic Write Failure Preserves Existing `.bem` File (`PROJECT_IO_ERROR`)
- **Purpose:** Prove that an I/O failure during `SaveProject` never truncates or corrupts the previously saved `.bem` file on disk.
- **Preconditions:** `"existing.bem"` exists on disk at `revision = 5` with valid content. The target directory or file is locked/read-only, or temporary write fails.
- **Action:** Mutate workspace to `revision = 6` and invoke `SaveProject`.
- **Expected Outcome:**
  - Command fails with `code: "PROJECT_IO_ERROR"`.
  - Any temporary `.tmp` file is cleaned up.
  - `"existing.bem"` on disk retains its exact `revision = 5` content and valid SHA-256 checksum, and `session.is_dirty` remains `true`.
- **Verification Surfaces:** `backend-boundary: required`, `frontend-integration: required`, `end-to-end: catalog-only`.

### SC-008: Stale `solution_cache` Stripped When `model_revision` Mismatches
- **Purpose:** Enforce the cross-capability rule that stale results are never shown as current when a file's cached results do not match `structural.model.revision`.
- **Preconditions:** A `.bem` file where `structural.model.revision == 8` but `solution_cache.model_revision == 7` (with a valid outer checksum).
- **Action:** Invoke `OpenProject`.
- **Expected Outcome:** The project loads with `document.solution_cache == null` and `stale_cache_stripped == true`. The UI triggers a fresh live solve or prompts re-solve rather than displaying revision-7 diagrams.
- **Verification Surfaces:** `backend-boundary: required`, `frontend-integration: required`, `end-to-end: catalog-only`.

### SC-009: Revision-Based Dirty Tracking Across Edits, Undo/Redo, and Save
- **Purpose:** Verify that `is_dirty` is strictly derived from `active_revision != saved_revision`.
- **Preconditions:** Clean saved project at `saved_revision = 10, active_revision = 10` (`is_dirty = false`).
- **Action:**
  1. Add a node (`active_revision = 11`) $\to$ observe `is_dirty = true`.
  2. Undo (`active_revision = 10`) $\to$ observe `is_dirty = false`.
  3. Redo (`active_revision = 11`) $\to$ observe `is_dirty = true`.
  4. Save project $\to$ observe `saved_revision = 11, active_revision = 11, is_dirty = false`.
- **Expected Outcome:** Dirty state indicator (`*` in header title) and `ProjectSessionState.is_dirty` transition deterministically at every step.
- **Verification Surfaces:** `backend-boundary: required`, `frontend-integration: required`, `end-to-end: catalog-only`.

### SC-010: Unsaved-Changes Guard Blocks Destructive New/Open/Close Until Confirmed
- **Purpose:** Prevent accidental loss of unsaved work when invoking `New Project`, `Open Project`, or window close while `is_dirty == true`.
- **Preconditions:** Active workspace has unsaved edits (`saved_revision = 2, active_revision = 4, is_dirty = true`).
- **Action:**
  1. Invoke `project_new` or `project_open` with `confirm_discard = false`.
  2. In the UI guard dialog, test all three branches: `Cancel`, `Discard` (`confirm_discard = true`), and `Save` (saves first, then proceeds).
- **Expected Outcome:**
  - Unconfirmed backend call rejects with `PROJECT_UNSAVED_CHANGES`.
  - `Cancel` keeps the user in the active workspace at `revision = 4`.
  - `Discard` proceeds and resets `is_dirty = false`.
  - `Save` persists revision 4 first and then completes the `New` or `Open` transition.
- **Verification Surfaces:** `backend-boundary: required`, `frontend-integration: required`, `end-to-end: catalog-only`.

### SC-011: Recent Projects (MRU) Deduplication, Ordering, Pinning, and Missing-File Cleanup
- **Purpose:** Verify MRU lifecycle across multiple saves and opens.
- **Preconditions:** `RecentProjectsManifest` has 2 existing entries (`A.bem`, `B.bem`).
- **Action:** Open `A.bem` again, pin `B.bem`, delete `A.bem` from the OS filesystem, list recent projects, and attempt to open `A.bem` from the MRU list.
- **Expected Outcome:**
  - Reopening `A.bem` updates its `last_opened_at` without creating a duplicate entry.
  - Pinned `B.bem` stays at the top of the list.
  - `project_list_recent` flags deleted `A.bem` with `exists_on_disk = false`, and clicking it raises `PROJECT_IO_ERROR` with a one-click action to remove it from the MRU list.
- **Verification Surfaces:** `backend-boundary: required`, `frontend-integration: required`, `end-to-end: catalog-only`.

### SC-012: SVG and PNG Diagram Export Without Altering Project Session State
- **Purpose:** Verify that exporting an M/Q/N diagram or deformed shape to `.svg` or `.png` produces a valid standalone file while leaving `ProjectSessionState` untouched.
- **Preconditions:** Active Mode 2 project `"frame_a.bem"` with unsaved edits (`saved_revision = 3, active_revision = 5, is_dirty = true`).
- **Action:** Invoke `project_export_diagram` with `format = "Svg"`, `theme = "LightPrint"`, `include_title_block = true`, `include_legend = true` to `"moment_diagram.svg"`, and repeat with `format = "Png"`, `dpi_scale = 2.0` to `"moment_diagram.png"`.
- **Expected Outcome:**
  - Valid `.svg` and `.png` files are written to disk with non-zero byte sizes and expected dimensions.
  - `ProjectSessionState.file_path` remains `"frame_a.bem"`, `saved_revision` remains `3`, `active_revision` remains `5`, and `is_dirty` remains `true`.
- **Verification Surfaces:** `backend-boundary: required`, `frontend-integration: required`, `end-to-end: catalog-only`.
