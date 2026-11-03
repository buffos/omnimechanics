## Issue Metadata

- Issue number: `018`
- Owning capability node: `/.okf/capabilities/cap-frame-analysis-engine.md`
- Artifact root: `docs/architecture/cap-frame-analysis-engine/`
- Issue file: `docs/agents/issues/pending/20261101-018-async-jobs-unilateral-pdelta-buckling.md`
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

Implement **Vertical Slice 3** of `cap-frame-analysis-engine`: the revision-fenced background `AnalysisJob` coordinator (`analysis://progress` telemetry and cooperative cancellation), active-set solver for tension/compression-only bars and unilateral supports, 2nd-order $P\text{-}\Delta / P\text{-}\delta$ geometric nonlinearity solver, and linear eigenvalue buckling solver ($\alpha_{cr}$ and buckling modes).

### 1. Background Job Coordinator & Revision Fencing (`src-tauri/src/frame/job_manager.rs`):
- Manage `AnalysisJob` lifecycle (`Queued -> Running -> Converged | Failed | Cancelled | Superseded`).
- Check cooperative cancellation tokens on every iteration; automatically mark in-flight jobs as `Superseded` when `StructuralModel.revision` advances (`SC-008`).
- Emit `analysis://progress` Tauri events with `job_id`, `model_revision`, `progress_percent`, `current_target_id`, `current_iteration`, `current_residual`, and bilingual status messages.
- Register `frame_start_analysis_job`, `frame_cancel_analysis_job`, and `frame_get_analysis_job_result`.

### 2. Unilateral & Tension/Compression-Only Active-Set Solver (`src-tauri/src/frame/unilateral.rs`):
- Solve each target (`LoadCase` or `Combination`) separately (`superposition_valid = false`, `INV-06`) when tension/compression-only `TrussBar`s or `unilateral: true` `Support`s are present.
- Iterate active set (up to 50 iterations, residual $< 10^{-6}$) deactivating slack/compression braces and uplifted supports (`SC-003`).

### 3. Second-Order $P\text{-}\Delta / P\text{-}\delta$ & Linear Buckling (`src-tauri/src/frame/pdelta_buckling.rs`):
- Assemble consistent $6\times 6$ Euler-Bernoulli geometric stiffness matrix $\mathbf{K}_G(N)$ (capturing both chord $P\text{-}\Delta$ and curvature $P\text{-}\delta$, adjusted for end releases).
- Iterate $\left[\mathbf{K}_E + \mathbf{K}_G(\mathbf{N}^{(k)})\right]\mathbf{u}^{(k+1)} = \mathbf{F}$ per combination until $\|\Delta\mathbf{u}\|_2 / \|\mathbf{u}\|_2 < 10^{-5}$; detect buckling divergence (`PDELTA_BUCKLING_DIVERGENCE`) if $\mathbf{K}_E + \mathbf{K}_G$ loses positive definiteness (`SC-004`).
- Solve generalized buckling eigenvalue problem $\left(\mathbf{K}_E + \alpha\,\mathbf{K}_G(\mathbf{N}_{\text{ref}})\right)\boldsymbol{\phi} = \mathbf{0}$ for lowest positive $\alpha_{cr,m}$ and normalized mode shapes $\boldsymbol{\phi}_m$ (`SC-004`).

## Acceptance criteria

- [x] Active-set solver converges for X-braced frames with `TensionOnly` bars and foundation beams on `unilateral: true` rollers, setting `superposition_valid: false` (`SC-003`).
- [x] `SecondOrderPDelta` matches analytical beam-column amplification $M_{\max} = H L \frac{\tan(k L)}{k L}$ within $10^{-3}$ relative error and returns `PDELTA_BUCKLING_DIVERGENCE` when $P \ge P_{cr}$ (`SC-004`).
- [x] `LinearBuckling` computes Euler critical load factor $\alpha_{cr,1}$ within $10^{-4}$ relative error and returns normalized buckling mode shapes (`SC-004`).
- [x] Mutating `StructuralModel` while a background job is running transitions the old job to `Superseded` and rejects stale requests with `STALE_MODEL_REVISION` (`SC-008`).
- [x] Companion unit tests (`*_test.rs`) and `npm run check:limits` pass with 0 errors.

## Artifact sync required

- Application PRD: `none - already covered in docs/prd.md`
- Application architecture summary: `none - matches docs/architecture/application-architecture-summary.md`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-frame-analysis-engine.md`
- Issue registry: `required`; node `issues:` reference: `required: .okf/capabilities/cap-frame-analysis-engine.md`
- Reason/no-impact decision: `Delivers background job execution, unilateral active-set, P-Delta, and linear buckling solvers as specified.`

## Human review gate

`None` (backend solver and async job coordinator slice with no rendered UI/UX changes).

## Blocked by

- Blocked by `docs/agents/issues/pending/20261101-017-linear-static-dsm-analytical-diagrams-envelopes.md`

## Artifact anchors

- PRD: `FR-SOL-02`, `FR-SOL-03`, `FR-SOL-04`, `NFR-PERF-02`, `NFR-REL-01`
- Domain model: `AnalysisJob` aggregate, `INV-06`, `ConvergencePolicy`
- Use cases: `UC-06 StartAnalysisJob`, `UC-07 CancelAnalysisJob`, `UC-08 GetAnalysisJobResult`
- Contract: `frame_start_analysis_job`, `frame_cancel_analysis_job`, `frame_get_analysis_job_result`, `analysis://progress`

## Acceptance scenarios addressed

- `SC-003`: Unilateral Support Uplift and Tension-Only Cross-Bracing Active-Set Iteration
- `SC-004`: Second-Order $P\text{-}\Delta / P\text{-}\delta$ Geometric Nonlinearity and Linear Eigenvalue Buckling
- `SC-008`: Background Job Cancellation, Progress Telemetry, and Revision Superseding

## Verification obligations

- Policy source: `/.okf/project.md`

| Scenario | Backend boundary | Frontend integration | End-to-end journey |
|---|---|---|---|
| `SC-003` | `planned` | `deferred: owned by cap-sketcher-frame-modeler UI slice` | `not-applicable` |
| `SC-004` | `planned` | `deferred: owned by cap-sketcher-frame-modeler UI slice` | `not-applicable` |
| `SC-008` | `planned` | `deferred: owned by cap-sketcher-frame-modeler UI slice` | `not-applicable` |
