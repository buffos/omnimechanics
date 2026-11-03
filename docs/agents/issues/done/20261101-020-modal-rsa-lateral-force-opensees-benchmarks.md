## Issue Metadata

- Issue number: `020`
- Owning capability node: `/.okf/capabilities/cap-frame-analysis-engine.md`
- Artifact root: `docs/architecture/cap-frame-analysis-engine/`
- Issue file: `docs/agents/issues/done/20261101-020-modal-rsa-lateral-force-opensees-benchmarks.md`
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

Implement **Vertical Slice 5** (final slice) of `cap-frame-analysis-engine`: Modal eigenfrequency analysis with automatic $\ge 90\%$ effective mass expansion, EN 1998-1 Modal Response Spectrum Analysis (SRSS and Der Kiureghian CQC), EN 1998-1 Equivalent Static Lateral Force solver, and the offline OpenSeesPy verification benchmark suite in `docs/benchmarks/frame/`.

### 1. Modal, Response Spectrum & Lateral Force Solvers (`src-tauri/src/frame/dynamics.rs`):
- Assemble global mass matrix $\mathbf{M}_{ff}$ (lumped nodal translational/rotational masses or optional consistent element mass matrix) derived automatically from seismic mass rule $G + \sum \psi_{2,i} Q_i$ plus explicit `LumpedMass`es.
- Solve $(\mathbf{K}_{ff} - \omega^2 \mathbf{M}_{ff})\boldsymbol{\phi} = \mathbf{0}$ for natural frequencies $f_i$, periods $T_i$, participation factors $\Gamma_{x,i}, \Gamma_{y,i}$, and effective modal masses $M_{\text{eff},x,i}, M_{\text{eff},y,i}$, automatically expanding mode count until cumulative effective mass reaches $\ge 90\%$ of total active mass (`SC-006`).
- Implement `LateralForce` solver: compute $T_1$, base shear $F_b = S_d(T_1) m \lambda$, and story force distribution $F_i = F_b \frac{z_i m_i}{\sum z_j m_j}$, solving for static internal forces and displacements (`SC-006`).
- Implement `ResponseSpectrum` solver: scale modal responses by horizontal $S_{d,x}(T_i)$ (and optional vertical $S_{d,y}(T_i)$) and combine peak nodal displacements, support reactions, and member diagrams via selectable **SRSS** or **CQC** (Der Kiureghian correlation $\rho_{ij}$ at damping $\xi$, default $5\%$) (`SC-006`).

### 2. Offline OpenSeesPy Benchmark Suite (`docs/benchmarks/frame/` & `src-tauri/src/frame/benchmarks_test.rs`):
- Create an offline reproducible Python generator script (`scripts/generate-frame-benchmarks.py`) and commit reference CSV files in `docs/benchmarks/frame/` covering `SC-001` (linear frame with releases/offsets/thermal/settlements), `SC-003` (tension-only bracing + unilateral rollers), `SC-004` ($P\text{-}\Delta$ & Euler buckling), `SC-005` (Irvine elastic catenary), and `SC-006` (3-storey modal & CQC/SRSS response spectrum).
- Add Rust integration benchmark tests verifying all solver outputs against the committed reference CSVs with zero runtime dependency on Python/OpenSees.

## Acceptance criteria

- [x] `Modal` analysis extracts modes up to $\ge 90\%$ cumulative effective modal mass and matches analytical/reference natural periods $T_i$ and participation factors $\Gamma_i$ within $10^{-4}$ (`SC-006`).
- [x] `ResponseSpectrum` accurately combines modal responses using both `Srss` and `Cqc` (including off-diagonal coupling $\rho_{ij}$ for closely spaced modes), and `LateralForce` distributes base shear proportionally to $z_i m_i$ (`SC-006`).
- [x] Reference CSVs in `docs/benchmarks/frame/` and companion Rust benchmark tests verify `SC-001` through `SC-009` within $10^{-6}$ (linear) and $10^{-3}$ (nonlinear).
- [x] All files obey size limits ($\le 1000$ lines) and `npm run check:limits` + `cargo test` pass with 0 errors.

## Artifact sync required

- Application PRD: `none - already covered in docs/prd.md`
- Application architecture summary: `none - matches docs/architecture/application-architecture-summary.md`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-frame-analysis-engine.md` (transitions to `implemented` once all 5 slices close)
- Issue registry: `required`; node `issues:` reference: `required: .okf/capabilities/cap-frame-analysis-engine.md`
- Reason/no-impact decision: `Completes all remaining dynamic/seismic solvers and reference benchmark obligations of cap-frame-analysis-engine.`

## Human review gate

`None` (backend dynamic solver and benchmark verification slice with no rendered UI/UX changes).

## Blocked by

- Blocked by `docs/agents/issues/pending/20261101-018-async-jobs-unilateral-pdelta-buckling.md`
- Blocked by `docs/agents/issues/pending/20261101-019-irvine-elastic-catenary-cable-solver.md`

## Artifact anchors

- PRD: `FR-SOL-06`, `FR-SOL-07`, `NFR-PERF-02`, `NFR-VER-01`
- Domain model: `LumpedMass`, `AnalysisKind::Modal`, `AnalysisKind::ResponseSpectrum`, `AnalysisKind::LateralForce`
- Use cases: `UC-06 StartAnalysisJob`, `UC-08 GetAnalysisJobResult`
- Contract: `frame_start_analysis_job`, `frame_get_analysis_job_result`

## Acceptance scenarios addressed

- `SC-006`: Modal Analysis ($\ge 90\%$ Effective Mass), Response Spectrum (SRSS/CQC), and Lateral Force

## Verification obligations

- Policy source: `/.okf/project.md`

| Scenario | Backend boundary | Frontend integration | End-to-end journey |
|---|---|---|---|
| `SC-006` | `planned` | `deferred: owned by cap-sketcher-frame-modeler UI slice` | `not-applicable` |
