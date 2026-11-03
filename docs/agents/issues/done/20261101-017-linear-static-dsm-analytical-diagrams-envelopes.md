## Issue Metadata

- Issue number: `017`
- Owning capability node: `/.okf/capabilities/cap-frame-analysis-engine.md`
- Artifact root: `docs/architecture/cap-frame-analysis-engine/`
- Issue file: `docs/agents/issues/pending/20261101-017-linear-static-dsm-analytical-diagrams-envelopes.md`
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

Implement **Vertical Slice 2** of `cap-frame-analysis-engine`: the high-performance (< 50 ms) Linear Static Direct Stiffness Method (DSM) solver, analytical interior member diagram evaluator ($M(x), Q(x), N(x)$ under Greek sign conventions), pointwise combination envelope generator, and pedagogical DSM Walkthrough generator.

### 1. Linear Static DSM Engine (`src-tauri/src/frame/`):
- **`element_stiffness.rs`:**
  - Local $6\times 6$ Euler-Bernoulli stiffness $\mathbf{k}_e$ and fixed-end force vectors $\mathbf{f}_{0,e}$ for point loads, partial trapezoidal loads (`Local | Global | Projected`), self-weight, and thermal loads ($\Delta T_{\text{uniform}}, \Delta T_{\text{gradient}}$).
  - Exact static condensation of axial, shear, and moment `EndRelease`s ($\mathbf{k}_c = \mathbf{k}_{aa} - \mathbf{k}_{ar}\mathbf{k}_{rr}^{-1}\mathbf{k}_{ra}$, $\mathbf{f}_{0,c} = \mathbf{f}_{0,a} - \mathbf{k}_{ar}\mathbf{k}_{rr}^{-1}\mathbf{f}_{0,r}$).
  - Rigid end offset transformation $\mathbf{T}_{\text{off}}$ and global rotation transformation $\mathbf{T}_e$.
  - `TrussBar` ($EA/L$) and `Spring` ($k_x, k_y, k_\theta$) stiffness contributions.
- **`linear_solver.rs`:**
  - Global assembly of $\mathbf{K}$ and load vectors $\mathbf{F}$, inclined support rotation transformations (`Roller { angle_deg }`, `Guided { angle_deg }`), elastic support springs, and prescribed `SupportSettlement` ($\mathbf{F}_f - \mathbf{K}_{fs}\mathbf{u}_s$).
  - Fast Cholesky solution via `faer` (`< 50 ms` for 500 nodes / 1000 members), reaction recovery $\mathbf{R}_s$, and linear superposition across `Combination`s.
- **`diagrams.rs` & `envelopes.rs`:**
  - Exact piecewise polynomial evaluation of $N(x), Q(x), M(x)$ and local deflection $u_y(x)$ at default stations, load discontinuities, and exact zero-shear roots $Q(x)=0$ (`frame_inspect_member_point`), strictly following the Greek sign convention ($M > 0$ when bottom/local $-y$ fibre is in tension).
  - Pointwise max/min `Envelope` computation across combinations with `governing_combination_id` tracking (`SC-007`).
- **`walkthrough.rs`:**
  - Pedagogical DSM step generator (`frame_generate_dsm_walkthrough`) returning DOF maps, per-element $\mathbf{k}_e, \mathbf{T}_e, \mathbf{f}_{0,e}$, global $\mathbf{K}$ (full dense for $\le 6$ nodes, sparse summary for $> 6$ nodes), partitioned $\mathbf{K}_{ff}\mathbf{u}_f = \mathbf{F}_f - \mathbf{K}_{fs}\mathbf{u}_s$, and bilingual (`el`/`en`) formulas (`SC-009`).
- **Tauri IPC Commands:**
  - Register `frame_solve_linear`, `frame_inspect_member_point`, and `frame_generate_dsm_walkthrough`.

## Acceptance criteria

- [x] `frame_solve_linear` solves frames with end releases, rigid offsets, inclined rollers, settlements, thermal gradients, and partial trapezoidal loads matching closed-form Euler-Bernoulli solutions within $10^{-6}$ relative error (`SC-001`).
- [x] Global equilibrium ($\sum F_x = 0, \sum F_y = 0, \sum M_z = 0$) and zero moment at released ends hold within $10^{-9}$.
- [x] Performance test verifies a 500-node, 1000-member planar frame with 5 load cases and 20 linear combinations solves in `< 50 ms`.
- [x] Pointwise `Envelope` computation accurately tracks `max_m`, `min_m`, `max_q`, `min_q`, `max_n`, `min_n` and `governing_combination_id` across combinations (`SC-007`).
- [x] `frame_generate_dsm_walkthrough` returns full dense matrices for $\le 6$ nodes and compact summaries for $> 6$ nodes with verified $\mathbf{K}_{ff}\mathbf{u}_f = \mathbf{F}_f - \mathbf{K}_{fs}\mathbf{u}_s$ identity (`SC-009`).
- [x] Companion unit/benchmark tests (`*_test.rs`) and `npm run check:limits` pass with 0 errors.

## Artifact sync required

- Application PRD: `none - already covered in docs/prd.md`
- Application architecture summary: `none - matches docs/architecture/application-architecture-summary.md`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-frame-analysis-engine.md`
- Issue registry: `required`; node `issues:` reference: `required: .okf/capabilities/cap-frame-analysis-engine.md`
- Reason/no-impact decision: `Delivers the specified linear DSM solver, diagram evaluator, envelopes, and walkthrough endpoints.`

## Human review gate

`None` (backend solver and IPC boundary slice with no rendered UI/UX changes).

## Blocked by

- Blocked by `docs/agents/issues/pending/20261101-016-frame-model-catalog-kinematic-diagnostics.md`

## Artifact anchors

- PRD: `FR-SOL-01`, `FR-RES-01`, `FR-RES-02`, `FR-RES-03`, `NFR-PERF-01`, `NFR-VER-01`
- Domain model: `ResultSet`, `MemberResult`, `Envelope`, `DsmWalkthroughData`, `INV-05`
- Use cases: `UC-04 SolveLinearStatic`, `UC-05 InspectMemberPoint`, `UC-09 GenerateDsmWalkthrough`
- Contract: `frame_solve_linear`, `frame_inspect_member_point`, `frame_generate_dsm_walkthrough`

## Acceptance scenarios addressed

- `SC-001`: Linear Static Frame with End Releases, Rigid Offsets, Inclined Roller, and Interior Loads
- `SC-007`: Internal Force Envelopes with Pointwise Governing Combination Tracking
- `SC-009`: Pedagogical DSM Walkthrough Data Generation ($\le 6$ Nodes vs. $> 6$ Nodes)

## Verification obligations

- Policy source: `/.okf/project.md`

| Scenario | Backend boundary | Frontend integration | End-to-end journey |
|---|---|---|---|
| `SC-001` | `planned` | `deferred: owned by cap-sketcher-frame-modeler UI slice` | `not-applicable` |
| `SC-007` | `planned` | `deferred: owned by cap-sketcher-frame-modeler UI slice` | `not-applicable` |
| `SC-009` | `planned` | `deferred: owned by cap-structural-mechanics-teaching UI slice` | `not-applicable` |
