## Issue Metadata

- Issue number: `032`
- Owning capability node: `/.okf/capabilities/cap-project-persistence.md`
- Artifact root: `docs/architecture/cap-project-persistence/`
- Issue file: `docs/agents/issues/pending/20261103-032-canonical-bem-schema-sha256-migration-atomic-io.md`
- Category: `feature`
- Execution type: `AFK`
- Review gate: `none`
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

Implement **Vertical Slice 1** of `cap-project-persistence`: the authoritative Rust `src-tauri/src/project/` module providing the canonical `v1.0` `.bem` `ProjectDocument` envelope (covering Mode 1 `continuum`, `continuum_sketch`, Mode 2 `structural` + Eurocode combinations + NA profile + seismic settings, optional `solution_cache`, and `ui_state`), deterministic lexicographical JSON serialization with `SHA-256` payload digest (`SC-003`), schema migration & version validation (`SC-004`, `SC-005`), corruption/finite-number verification (`SC-006`), atomic temp-file write + rename (`SC-007`), and stale `solution_cache` stripping (`SC-008`).

### 1. Canonical `.bem` `ProjectDocument` v1.0 & Deterministic Serializer (`src-tauri/src/project/`):
- Define `ProjectDocument` (`schema_version: "1.0"`, `metadata: ProjectMetadata`, `integrity: DocumentIntegrity`, `active_mode`, `continuum`, `continuum_sketch`, `structural`, `solution_cache`, `ui_state`).
- Implement canonical JSON serializer (`BR-1`, `SC-003`) with UTF-8 encoding, `\n` line endings, 2-space indentation, recursive lexicographical key sorting on all JSON objects, finite IEEE 754 float enforcement (`no NaN/Infinity`), and `SHA-256` computation over the canonical payload (`integrity.payload_sha256`).

### 2. Schema Migration, Integrity Verification & Stale Cache Stripping (`src-tauri/src/project/`):
- Reject unsupported major/future schema versions (e.g., `"2.0"`, `"0.5"`) with `PROJECT_SCHEMA_UNSUPPORTED` (`SC-005`) without altering the active in-memory session.
- Implement deterministic `SchemaMigrationPolicy` upgrading legacy/pre-1.0 `.bem` payloads (including the earlier Mode 2 `ProjectDocument` format) to `"1.0"` and returning a `MigrationReport` (`SC-004`).
- Verify `payload_sha256` and JSON syntax on open; reject truncated JSON, non-finite numbers, or tampered payloads with `PROJECT_CORRUPT_JSON` (`details.reason`: `"SyntaxError"`, `"ChecksumMismatch"`, `"NonFiniteNumber"`) (`SC-006`).
- When `solution_cache.model_revision != structural.model.revision`, strip `solution_cache` on load and set `stale_cache_stripped: true` (`BR-4`, `SC-008`).

### 3. Atomic File Write & Tauri IPC Commands (`src-tauri/src/project/`):
- Write to `<target>.tmp`, flush to disk, and atomically rename to `<target>` so any I/O failure (`PROJECT_IO_ERROR`) cleans up the `.tmp` file and leaves the existing `.bem` file completely intact (`BR-5`, `SC-007`).
- Register Tauri IPC commands `project_new`, `project_open`, `project_save`, `project_save_as`, and `project_inspect_header` in `src-tauri/src/lib.rs` (maintaining backward compatibility with `project_save_workspace` / `project_load_workspace`).

## Acceptance criteria

- [ ] Mode 2 (`structural` + combinations + NA + seismic + UI state) and Mode 1 (`continuum` + `continuum_sketch` + `solution_cache`) round-trip losslessly through `project_save_as` and `project_open` (`SC-001`, `SC-002`).
- [ ] Two `ProjectDocument` instances with identical semantic content constructed with different map key orders serialize to bit-for-bit identical bytes and `payload_sha256` hashes (`SC-003`).
- [ ] Legacy `.bem` fixtures migrate cleanly to `"1.0"` (`SC-004`), whereas `"schema_version": "2.0"` rejects with `PROJECT_SCHEMA_UNSUPPORTED` without mutating the active session (`SC-005`).
- [ ] Truncated JSON, tampered coordinates breaking `payload_sha256`, or non-finite floats reject with `PROJECT_CORRUPT_JSON` (`SC-006`), and simulated write failures return `PROJECT_IO_ERROR` while preserving the previous `.bem` file on disk (`SC-007`).
- [ ] Mismatched `solution_cache.model_revision` strips the cache and sets `stale_cache_stripped == true` (`SC-008`).
- [ ] Rust unit tests in `src-tauri/src/project/project_test.rs` pass 100% (`cargo test --lib`), and `npm run check:limits` passes.

## Artifact sync required

- Application PRD: `none - already covered in docs/prd.md`
- Application architecture summary: `none - matches docs/architecture/application-architecture-summary.md`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-project-persistence.md`
- Issue registry: `required`; node `issues:` reference: `required: .okf/capabilities/cap-project-persistence.md`
- Reason/no-impact decision: `Delivers the canonical .bem v1.0 persistence engine, SHA-256 integrity, migration pipeline, and atomic file I/O.`

## Human review gate

`none`: Backend serialization, checksum integrity, schema migration, and atomic file I/O slice; desktop UI workflows are exposed in Issue `033`.

## Blocked by

- None - can start immediately

## Artifact anchors

- PRD: `W2`, `W3`, `W4`, `BR-1`, `BR-2`, `BR-4`, `BR-5`, `BR-6`, `FR-1`, `NFR-1`, `NFR-2`
- Domain model: `ProjectDocument`, `ProjectMetadata`, `DocumentIntegrity`, `SchemaMigrationPolicy`, `INV-P1..P4`
- Use cases: `UC-C01..UC-C04`, `UC-Q03`
- Contract: `project_new`, `project_open`, `project_save`, `project_save_as`, `project_inspect_header`, error codes `PROJECT_SCHEMA_UNSUPPORTED`, `PROJECT_CORRUPT_JSON`, `PROJECT_IO_ERROR`

## Acceptance scenarios addressed

- `SC-001`: Lossless Round-Trip Save and Open of Mode 2 Structural Workspace
- `SC-002`: Lossless Round-Trip of Mode 1 Continuum & `ContinuumDomain` Sketch
- `SC-003`: Deterministic Canonical JSON & SHA-256 Digest Reproducibility
- `SC-004`: Forward/Backward Schema Migration of Legacy `.bem` File
- `SC-005`: Rejection of Unsupported Schema Version (`PROJECT_SCHEMA_UNSUPPORTED`)
- `SC-006`: Rejection of Corrupted JSON, Non-Finite Floats, or Tampered Checksum (`PROJECT_CORRUPT_JSON`)
- `SC-007`: Atomic Write Failure Preserves Existing `.bem` File (`PROJECT_IO_ERROR`)
- `SC-008`: Stale `solution_cache` Stripped When `model_revision` Mismatches

## Verification obligations

- Policy source: `/.okf/project.md`

| Scenario | Backend boundary | Frontend integration | End-to-end journey |
|---|---|---|---|
| `SC-001` | `planned` | `deferred: Issue 033` | `catalog-only` |
| `SC-002` | `planned` | `deferred: Issue 033` | `catalog-only` |
| `SC-003` | `planned` | `not-applicable` | `not-applicable` |
| `SC-004` | `planned` | `deferred: Issue 033` | `catalog-only` |
| `SC-005` | `planned` | `deferred: Issue 033` | `catalog-only` |
| `SC-006` | `planned` | `deferred: Issue 033` | `catalog-only` |
| `SC-007` | `planned` | `deferred: Issue 033` | `catalog-only` |
| `SC-008` | `planned` | `deferred: Issue 033` | `catalog-only` |
