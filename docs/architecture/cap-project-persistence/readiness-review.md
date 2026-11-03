# Architecture Readiness Review: Project Persistence & Self-Contained Workspace Storage (`cap-project-persistence`)

- **Owning Capability:** [`cap-project-persistence`](../../../.okf/capabilities/cap-project-persistence.md)
- **Reviewed Artifact Set:**
  - [`domain-glossary.md`](domain-glossary.md)
  - [`prd.md`](prd.md)
  - [`canonical-domain-model.md`](canonical-domain-model.md)
  - [`canonical-use-cases.md`](canonical-use-cases.md)
  - [`canonical-api-cli-contract.md`](canonical-api-cli-contract.md)
  - [`acceptance-scenarios.md`](acceptance-scenarios.md)
- **Shared References Audited:**
  - [`cross-capability-conventions.md`](../cross-capability-conventions.md)
  - [`docs/prd.md`](../../prd.md)
  - [`docs/architecture/application-architecture-summary.md`](../application-architecture-summary.md)
- **Review Date:** 2026-11-01
- **Gate Verdict:** **READY FOR `specified` STATE** (0 High, 0 Medium, 0 Low blocking findings)

---

## 1. Findings-First Audit Summary

No High, Medium, or Low blocking findings were identified across the 6 canonical artifacts for `cap-project-persistence`.

| Severity | Count | Status |
| :--- | :--- | :--- |
| **High** | 0 | Clean |
| **Medium** | 0 | Clean |
| **Low** | 0 | Clean |

---

## 2. Cross-Document Consistency Verification

1. **Canonical Vocabulary & Ownership (`cross-capability-conventions.md` §1-§2):**
   - `ProjectDocument` (`[persistence]`), `StructuralModel` (`[frame]`), `Node`, `Element` (`FrameMember`, `TrussBar`, `Spring`, `Cable`), `EndRelease`, `RigidOffset`, `Section`, `Material`, `Support`, `Load`, `LoadCase`, `ActionType` (`[actions]`), `Combination` (`[actions]`), `NationalAnnexProfile` (`[actions]`), `SeismicSettings` (`[actions]`), and `ContinuumDomain` (`[continuum]`) are preserved with their exact canonical names and ownership boundaries across all 6 documents.
   - Dependency direction rule (`persistence serialises the models but owns no domain rules`) is enforced in [`prd.md`](prd.md) (BR-2) and [`canonical-domain-model.md`](canonical-domain-model.md) (§1).
2. **Error Codes & Bilingual Contract:**
   - `PROJECT_SCHEMA_UNSUPPORTED`, `PROJECT_CORRUPT_JSON`, `PROJECT_IO_ERROR`, `PROJECT_UNSAVED_CHANGES`, and `PROJECT_EXPORT_INVALID_SPEC` use the mandatory `{ code: SCREAMING_SNAKE, message_el, message_en, details? }` envelope across glossary, PRD, use-case model, contract, and acceptance scenarios (`SC-005`, `SC-006`, `SC-007`, `SC-010`).
3. **IPC Prefix & Command Alignment:**
   - Every Tauri IPC command in [`canonical-api-cli-contract.md`](canonical-api-cli-contract.md) uses the required `project_` prefix (`project_new`, `project_open`, `project_save`, `project_save_as`, `project_record_revision`, `project_get_session_state`, `project_list_recent`, `project_manage_recent`, `project_inspect_header`, `project_export_diagram`) and maps 1-to-1 to the commands/queries in [`canonical-use-cases.md`](canonical-use-cases.md).
4. **Revision & Stale-Result Invariants:**
   - Dirty-state tracking (`is_dirty = active_revision != saved_revision`, `SC-009`) and stale `solution_cache` rejection when `solution_cache.model_revision != structural.model.revision` (`SC-008`) strictly enforce `cross-capability-conventions.md` §2 ("Stale results are never shown as current").
5. **Verification Surface Traceability:**
   - All 12 scenarios (`SC-001` through `SC-012`) in [`acceptance-scenarios.md`](acceptance-scenarios.md) declare explicit `backend-boundary`, `frontend-integration`, and `end-to-end` verification surfaces aligned with the root policy (`backend-boundary: required`, `frontend-integration: required`, `end-to-end: catalog-only`).

---

## 3. State Policy & Capability Boundary Evaluation

- **State Evaluation Mode:** `own` (`state_policy` is absent on [`.okf/capabilities/cap-project-persistence.md`](../../../.okf/capabilities/cap-project-persistence.md), so `mode: own` applies).
- **Current Node State:** `bounded` $\to$ eligible for immediate transition to `specified`.
- **Structural Children:** None (leaf capability node).

---

## 4. Residual Risks & Implementation Guidance

1. **Large Mode 1 3D Mesh / Solution Cache Payloads (Low Residual Risk):**
   - *Context:* While Mode 2 structural frames are compact (`< 1.5 MB`), Mode 1 3D BEM/FEM meshes with cached displacement/stress fields can reach tens of megabytes in plain UTF-8 JSON.
   - *Mitigation in Spec:* `include_solution_cache` defaults to `false` on save (`prd.md` FR-1), and `SC-008` allows stripping caches cleanly. Future minor schema versions (`1.1`) can add optional `.bem.gz` transparent compression without changing the `ProjectDocument` logical schema.
2. **Cross-Platform Path Normalization in MRU Manifest (Low Residual Risk):**
   - *Context:* Windows drive-letter casing and path separators (`\` vs `/`) can cause duplicate entries in `RecentProjectsManifest` if raw string comparison is used.
   - *Mitigation in Spec:* `INV-P6` and `UC-C02`/`UC-C04` specify normalized path deduplication before upserting into `RecentProjectsManifest` (`SC-011`).
3. **Raster PNG Export Dependency (Low Residual Risk):**
   - *Context:* `project_export_diagram` supports both `Svg` and `Png` (`SC-012`).
   - *Mitigation in Spec:* The frontend canvas produces canonical standalone SVG markup (`DiagramExportSpec.svg_markup`), allowing either pure-Rust SVG rasterization (`resvg`/`tiny-skia`) or offscreen canvas rasterization behind the `project_export_diagram` boundary while keeping code files $\le 1000$ lines.

---

## 5. Application Synthesis Gate Assessment

- **Root Documents Checked:**
  - [`docs/prd.md`](../../prd.md)
  - [`docs/architecture/application-architecture-summary.md`](../application-architecture-summary.md)
- **Artifact Impact Classification:** `no impact` (Root synthesis documents already establish `cap-project-persistence` as the owner of `.bem` JSON project save/load for `StructuralModel`, `LoadCase`s, `Combination`s, and `ContinuumDomain`, plus SVG/PNG diagram export).
- **Readiness Conclusion:** `cap-project-persistence` passes the architecture readiness gate and is ready to advance from `bounded` to `specified`.
