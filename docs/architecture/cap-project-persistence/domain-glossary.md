# Domain Glossary: Project Persistence & Self-Contained Workspace Storage (`cap-project-persistence`)

- **Owning Capability:** [`cap-project-persistence`](../../../.okf/capabilities/cap-project-persistence.md)
- **Conventions Reference:** [`cross-capability-conventions.md`](../cross-capability-conventions.md)
- **Status:** Canonical (2026-11-01)

---

## 1. Purpose

This glossary stabilizes the canonical domain language for saving, loading, migrating, validating, and exporting OmniMechanics project workspaces across Mode 1 (Continuum Workbench), Mode 2 (Structural Modeling & Analysis), and Mode 3 (Teaching Studio). All downstream PRDs, domain models, use-case definitions, IPC/CLI contracts, and acceptance scenarios must use these terms without introducing synonyms.

---

## 2. Canonical Terms

### 2.1 Actors

| Canonical Term | Category | Definition | Discouraged Aliases |
| :--- | :--- | :--- | :--- |
| **Student** | Actor | Learner who opens teaching presets or custom structural sketches, steps through classical or DSM derivations in Greek or English, saves workspace progress, and exports M/Q/N diagrams for coursework. | User, Pupil, Viewer |
| **Engineer / Instructor** | Actor | Practitioner or educator who authors 2D structural frames or continuum domains, configures Eurocode load cases and combinations, saves reproducible `.bem` project archives, and exports high-resolution SVG/PNG diagrams. | Designer, Teacher, Modeler |
| **Researcher** | Actor | Computational mechanics specialist who executes Mode 1 BEM/FEM/hybrid elastodynamic benchmarks, optionally caches converged solution fields inside `.bem` files, and archives benchmark configurations for reproducibility. | Scientist, Analyst |

### 2.2 Core Business Objects & Value Objects

| Canonical Term | Category | Definition | Discouraged Aliases |
| :--- | :--- | :--- | :--- |
| **ProjectDocument** | Aggregate Root (`[persistence]`) | The versioned, self-contained JSON root structure persisted as a `.bem` file. Contains `schema_version`, `metadata`, `integrity`, `active_mode`, `continuum`, `continuum_sketch`, `structural`, optional `solution_cache`, and `ui_state`. | ProjectFile, WorkspaceDump, BemArchive, SaveFile |
| **SchemaVersion** | Value Object | Semantic integer or `MAJOR.MINOR` schema identifier (`schema_version`, currently `"1.0"`) declaring the structural contract of a `ProjectDocument` and governing forward/backward migration eligibility. | FileVersion, FormatVersion, VersionTag |
| **ProjectMetadata** | Value Object | Descriptive header inside `ProjectDocument` recording `project_id` (UUID), `title`, `author`, `description`, `created_at` (RFC 3339 UTC), `updated_at` (RFC 3339 UTC), `app_version`, and `dimensions` (`"2D"` or `"3D"`). | FileHeader, ProjectInfo, DocumentProperties |
| **DocumentIntegrity** | Value Object | Deterministic SHA-256 hex digest (`payload_sha256`) computed over the canonical JSON serialization of the workspace sections (`continuum`, `continuum_sketch`, `structural`, `solution_cache`, `ui_state`) to detect accidental file truncation or bit-rot. | Checksum, FileHash, CRC |
| **WorkspaceMode** | Enumeration | The active application workspace restored when opening a `ProjectDocument`: `ContinuumWorkbench` (Mode 1), `StructuralModeler` (Mode 2), or `TeachingStudio` (Mode 3). | AppTab, ActiveView, ModeId |
| **ContinuumWorkspaceSection** | Value Object (`continuum`) | Mode 1 persistence payload storing the parametric preset identifier or CAD import configuration, mesh topology parameters, homogeneous reference moduli, active inhomogeneity material field parameters, and boundary condition/load mappings. | Mode1State, BemConfig, ContinuumData |
| **ContinuumSketchSection** | Value Object (`continuum_sketch`) | Embedded [`ContinuumDomain`](../cross-capability-conventions.md) (`[continuum]`) containing outer polygon boundary loop, interior holes, vertex/segment grading, and boundary tags for Mode 1/Mode 2 2D CDT meshing. | PolygonSketch, DomainSketch, SketchData |
| **StructuralWorkspaceSection** | Value Object (`structural`) | Mode 2 & Mode 3 persistence payload encapsulating the authoritative [`StructuralModel`](../cross-capability-conventions.md) (`[frame]`, including `Node`s, `Element`s, `Section`s, `Material`s, `Support`s, `Load`s, `LoadCase`s, lumped masses, and `revision`), [`Combination`](../cross-capability-conventions.md)s (`[actions]`), [`NationalAnnexProfile`](../cross-capability-conventions.md) (`[actions]`, default `GR`), and [`SeismicSettings`](../cross-capability-conventions.md) (`[actions]`). | FrameSection, Mode2State, StructurePayload |
| **SolutionCacheSection** | Value Object (`solution_cache`) | Optional section inside `ProjectDocument` storing pre-computed Mode 1 field arrays (displacements, tractions, stresses, convergence history) or Mode 2 `ResultSet`/`Envelope` snapshots tagged with the exact model `revision` for instant offline inspection. | ResultsDump, CachedOutput, SolverDump |
| **UiStateSection** | Value Object (`ui_state`) | Non-normative presentation preferences persisted in `ProjectDocument`: active `language` (`"el"` or `"en"`), canvas viewport pan/zoom transform, grid/snap settings, diagram scale factors, and selected result target (`LoadCase`, `Combination`, or `Envelope` id). | ViewSettings, Preferences, GuiState |
| **ProjectSessionState** | Entity (`[persistence]`) | Runtime application state tracking the currently open file path (`Option<PathBuf>`), `saved_revision` (u64), `active_revision` (u64), derived `is_dirty` boolean (`active_revision != saved_revision`), and last-saved timestamp. | FileTracker, DirtyTracker, SessionInfo |
| **RecentProjectsManifest** | Aggregate Root (`[persistence]`) | Application-level local storage manifest maintaining a bounded, deduplicated Most-Recently-Used (MRU) list of [`RecentProjectEntry`](#22-core-business-objects--value-objects) items across application restarts. | MruList, HistoryFile, RecentFiles |
| **RecentProjectEntry** | Value Object | Entry in `RecentProjectsManifest` recording `file_path`, `title`, `active_mode`, `last_opened_at` (RFC 3339 UTC), and `pinned` flag. | HistoryItem, MruItem |
| **DiagramExportSpec** | Value Object | Request specification for exporting structural or continuum canvas diagrams to disk: target format (`Svg` or `Png`), resolution scale/DPI, paper/background theme (`LightPrint` or `DarkCanvas`), legend/title-block inclusion, and output file path. | ExportOptions, PlotConfig, ImageSettings |

### 2.3 Workflows, Policies & Errors

| Canonical Term | Category | Definition | Discouraged Aliases |
| :--- | :--- | :--- | :--- |
| **Canonical JSON Serialization** | Policy | Deterministic UTF-8 JSON formatting rule (lexicographically sorted object keys, 2-space indentation, finite IEEE 754 floats rounded or preserved without `NaN`/`Infinity`, normalized `\n` line endings) ensuring diff-friendly `.bem` files in version control and reproducible SHA-256 digests. | PrettyPrint, JsonDump |
| **Schema Migration Policy** | Policy | Ordered chain of pure transformation steps (`v_old -> v_new`) applied when loading a `ProjectDocument` with an older supported minor/major `schema_version`, upgrading it to current schema before domain deserialization. | VersionUpgrader, LegacyLoader |
| **UnsavedChangesGuard** | Policy | Interactive pre-flight check triggered before destructive session transitions (`New Project`, `Open Project`, application window close) whenever `is_dirty == true`, prompting the user in `el`/`en` to `Save`, `Discard`, or `Cancel`. | DirtyPrompt, CloseWarning |
| **Atomic Write Policy** | Policy | File-system write discipline that serializes `ProjectDocument` to a sibling temporary file (`<filename>.bem.tmp`), flushes OS buffers, and atomically renames it over the target `.bem` path so failed writes never corrupt an existing saved project. | SafeWrite, TempFileSave |
| **PROJECT_SCHEMA_UNSUPPORTED** | Error Code | Canonical error raised when a `.bem` file declares a `schema_version` newer than the running application supports or older than the minimum migratable version. | `ERR_BAD_VERSION`, `UNSUPPORTED_FORMAT` |
| **PROJECT_CORRUPT_JSON** | Error Code | Canonical error raised when a `.bem` file fails UTF-8 decoding, contains syntax-invalid JSON, violates structural schema constraints, fails `payload_sha256` integrity verification, or contains non-finite numbers (`NaN`/`Infinity`). | `ERR_INVALID_JSON`, `PARSE_ERROR`, `CHECKSUM_MISMATCH` |
| **PROJECT_IO_ERROR** | Error Code | Canonical error raised when the OS filesystem rejects reading or writing a `.bem` or exported diagram file (missing file, permission denied, disk full, locked path). | `ERR_FILE_IO`, `DISK_ERROR` |

---

## 3. Critical Semantic Distinctions

1. **`ProjectDocument` vs. `StructuralModel` / `ContinuumDomain`:**
   - `ProjectDocument` (`[persistence]`) is the outer persistence envelope that coordinates multi-workspace serialization, schema versioning, metadata, integrity hashes, and UI state.
   - `StructuralModel` (`[frame]`) and `ContinuumDomain` (`[continuum]`) are domain models owned by their respective capabilities and embedded inside `ProjectDocument` sections (`structural` and `continuum_sketch`). `cap-project-persistence` serializes and migrates their schema envelopes without owning structural mechanics or meshing rules.
2. **`saved_revision` vs. `active_revision`:**
   - Every mutation to the workspace increments `active_revision` (matching `StructuralModel.revision` in Mode 2 or workspace edit counter in Mode 1).
   - `saved_revision` records the value of `active_revision` at the completion of the last successful `New`, `Open`, `Save`, or `Save As` operation. `is_dirty` is strictly a derived predicate (`active_revision != saved_revision`), never an independently mutated boolean that can drift out of sync with undo/redo stacks.
3. **`SolutionCacheSection` vs. Authoritative Model State:**
   - The `solution_cache` is strictly an optional performance cache tied to a specific model `revision`. If `solution_cache.model_revision != structural.model.revision`, the cache is discarded on load and never displayed as current (`cross-capability-conventions.md` §2).
4. **Diagram Export (`.svg` / `.png`) vs. Project Save (`.bem`):**
   - Saving a `.bem` file persists the complete re-editable engineering workspace and clears `is_dirty`.
   - Exporting a diagram (`project_export_diagram`) writes a one-way visual artifact (`.svg` or `.png`) for reports or coursework and never modifies `ProjectSessionState.file_path` or `saved_revision`.

---

## 4. Open Terminology Issues

None. All vocabulary aligns with [`cross-capability-conventions.md`](../cross-capability-conventions.md) and [`discovery-notes.md`](../cap-geometry-canvas-sketcher/discovery-notes.md) (Q24).
