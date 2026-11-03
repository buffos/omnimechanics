## Issue Metadata

- Issue number: `016`
- Owning capability node: `/.okf/capabilities/cap-frame-analysis-engine.md`
- Artifact root: `docs/architecture/cap-frame-analysis-engine/`
- Issue file: `docs/agents/issues/pending/20261101-016-frame-model-catalog-kinematic-diagnostics.md`
- Category: `feature`
- Execution type: `AFK`
- Review gate: `none`
- Suggested state: `done`

## Parent Artifacts

- `docs/prd.md`
- `docs/architecture/application-architecture-summary.md`
- `docs/architecture/cross-capability-conventions.md`
- `.okf/capabilities/cap-frame-analysis-engine.md`
- `docs/architecture/cap-frame-analysis-engine/prd.md`
- `docs/architecture/cap-frame-analysis-engine/canonical-domain-model.md`
- `docs/architecture/cap-frame-analysis-engine/canonical-use-cases.md`
- `docs/architecture/cap-frame-analysis-engine/canonical-api-cli-contract.md`
- `docs/architecture/cap-frame-analysis-engine/acceptance-scenarios.md`
- `docs/architecture/cap-frame-analysis-engine/readiness-review.md`

## What to build

Implement **Vertical Slice 1** of `cap-frame-analysis-engine`: the authoritative 2D `StructuralModel` aggregate, built-in Material & Section catalogs, domain validation invariants, and the pre-solve Kinematic Instability & Static Indeterminacy Diagnostics engine exposed via Tauri IPC.

### 1. Rust Domain Model & Catalog (`src-tauri/src/frame/`):
- **`model.rs`:**
  - `StructuralModel` aggregate root with monotonic `revision: u64`, `gravity_g: f64`, `nodes`, `materials`, `sections`, `elements`, `supports`, `load_cases`, and `lumped_masses`.
  - Elements (`Element`): `FrameMember` (with per-end `EndRelease { axial, shear, moment }` and `RigidOffset { dx, dy, frame }`), `TrussBar` (`Both | TensionOnly | CompressionOnly`), `Spring` (1-node grounded or 2-node discrete with $k_x, k_y, k_\theta$, `angle_deg`), and `Cable` (`UnstressedLength`, `SagRatio`, `Pretension`).
  - Supports (`Support`): `Fixed`, `Pinned`, `Roller { angle_deg }`, `Guided { angle_deg }`, `Elastic { kx, ky, ktheta }`, and `unilateral: bool`.
  - Loads (`Load`) inside `LoadCase`: `NodalLoad`, `MemberPointLoad`, `MemberDistributedLoad` (`Local | Global | Projected`), `SupportSettlement`, `ThermalLoad` (`dt_uniform`, `dt_gradient`), plus `self_weight: bool`.
- **`catalog.rs`:**
  - Built-in Material presets (Concrete C20/25–C50/60, Steel S235/S275/S355, Timber C24/GL24h, CableSteel Y1770/Y1860) and Section presets (IPE 80–600, HEA 100–600, HEB 100–600, hollow RHS/CHS, plus parametric Rectangular $b\times h$ and Circular $D$).
- **`validation.rs`:**
  - Enforce hard invariants `INV-01`..`INV-04` and return canonical bilingual (`el`/`en`) errors (`INVALID_ELEMENT_GEOMETRY`, `MISSING_MODEL_REFERENCE`, `DUPLICATE_NODE_SUPPORT`, `INVALID_LOAD_ASSIGNMENT`, `STALE_MODEL_REVISION`).
- **`diagnostics.rs`:**
  - Assemble elastic topological stiffness $\mathbf{K}_{ff}$, compute static indeterminacy degree $n_s$ and active DOFs, perform rank-revealing QR / SVD / pivoted Cholesky factorization to detect kinematic instability ($n_k > 0$), extract normalized nullspace mechanism mode vectors $\boldsymbol{\phi}_m$ ($\|\boldsymbol{\phi}_m\|_\infty = 1$), and classify global vs. local hinge mechanisms.
- **Tauri IPC Commands:**
  - Register `frame_upsert_model`, `frame_get_catalog`, and `frame_evaluate_diagnostics` in `src-tauri/src/lib.rs`.

## Acceptance criteria

- [x] `StructuralModel` and all child entities/value objects serialize/deserialize cleanly matching the JSON schema in `canonical-api-cli-contract.md` §2.1.
- [x] Built-in material and section catalogs (`frame_get_catalog`) return standard European steel, concrete, timber, and cable presets with accurate SI/GPa properties.
- [x] Domain validation rejects zero-length elements, rigid offsets exceeding $0.95 L$, unknown entity IDs, duplicate node supports, transverse loads on cables, and thermal gradients on sections without `depth_h` using bilingual error envelopes.
- [x] `frame_evaluate_diagnostics` computes exact static indeterminacy $n_s$ and detects mechanisms ($n_k > 0$), returning `KINEMATIC_INSTABILITY` with normalized `mechanism_modes` ($\max(|\Delta u_x|, |\Delta u_y|) = 1.0$) for 4-hinge portal frames, collinear hinges, and unrestrained trusses (`SC-002`).
- [x] Companion unit test files (`*_test.rs`) verify all invariants and mechanism modes; all files obey $\le 1000$ lines (`npm run check:limits` and `cargo test` pass).

## Artifact sync required

- Application PRD: `none - already synthesized in docs/prd.md`
- Application architecture summary: `none - matches src-tauri/src/frame/ boundary in docs/architecture/application-architecture-summary.md`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-frame-analysis-engine.md`
- Issue registry: `required`; node `issues:` reference: `required: .okf/capabilities/cap-frame-analysis-engine.md`
- Reason/no-impact decision: `Implements Slice 1 of the already-specified cap-frame-analysis-engine contract without altering product or cross-capability architecture truth.`

## Human review gate

`None` (backend domain model, catalog, and Tauri IPC boundary slice with no rendered UI/UX changes).

## Blocked by

`None - can start immediately`

## Artifact anchors

- PRD: `FR-MOD-01`, `FR-MOD-02`, `FR-MOD-03`, `FR-MOD-04`, `FR-MOD-05`, `FR-DIAG-01`, `FR-DIAG-02`
- Domain model: `StructuralModel` aggregate, `INV-01`..`INV-04`, `KinematicDiagnosticsService`
- Use cases: `UC-01 UpsertStructuralModel`, `UC-02 ListMaterialAndSectionCatalog`, `UC-03 EvaluateKinematicDiagnostics`
- Contract: `frame_upsert_model`, `frame_get_catalog`, `frame_evaluate_diagnostics`, error codes §3

## Acceptance scenarios addressed

- `SC-002`: Pre-Solve Kinematic Instability Detection and Mechanism Mode Animation Vector

## Verification obligations

- Policy source: `/.okf/project.md`

| Scenario | Backend boundary | Frontend integration | End-to-end journey |
|---|---|---|---|
| `SC-002` | `planned` | `deferred: owned by cap-sketcher-frame-modeler UI slice` | `not-applicable` |
