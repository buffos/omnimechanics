## Issue Metadata

- Issue number: `019`
- Owning capability node: `/.okf/capabilities/cap-frame-analysis-engine.md`
- Artifact root: `docs/architecture/cap-frame-analysis-engine/`
- Issue file: `docs/agents/issues/done/20261101-019-irvine-elastic-catenary-cable-solver.md`
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

Implement **Vertical Slice 4** of `cap-frame-analysis-engine`: the exact 2D Irvine elastic catenary `Cable` element solver, supporting `UnstressedLength(L0)`, `SagRatio(ratio)`, and `Pretension(T0)` inputs, coupled into the global Newton-Raphson equilibrium loop with automatic slack detection and catenary profile generation.

### 1. Irvine Elastic Catenary Element & Global Newton-Raphson (`src-tauri/src/frame/catenary.rs`):
- Convert `SagRatio` and `Pretension(T0)` at the initial chord geometry into the canonical unstressed length $L_0$ before solving.
- Implement the exact 2D Irvine elastic catenary compatibility equations for relative chord projections $(\Delta x, \Delta y)$ as a function of end forces $(H, V)$ under self-weight $w_0 = \gamma A$:
  $$\Delta x(H, V) = \frac{H L_0}{E A} + \frac{H}{w_0}\left[\sinh^{-1}\!\left(\frac{V}{H}\right) - \sinh^{-1}\!\left(\frac{V - w_0 L_0}{H}\right)\right]$$
  $$\Delta y(H, V) = \frac{L_0}{E A}\left(V - \frac{w_0 L_0}{2}\right) + \frac{1}{w_0}\left[\sqrt{H^2 + V^2} - \sqrt{H^2 + (V - w_0 L_0)^2}\right]$$
- Solve the inner $2\times 2$ element state via damped Newton-Raphson (`tol = 1e-11`) and invert the analytical $2\times 2$ flexibility Jacobian $\mathbf{F}_c$ to obtain the exact $4\times 4$ tangent stiffness $\mathbf{k}_{T,\text{cable}}$.
- Couple cable internal forces and tangent stiffnesses into the outer global Newton-Raphson loop (`max_iter = 100`, backtracking line search, force residual $< 10^{-5}\text{ kN}$).
- Detect cable slackness ($H < 10^{-4} EA$), apply positive-definite regularization ($10^{-6} EA/L_0$), emit `CABLE_SLACK_DETECTED` warnings in Greek and English (`SC-005`), or return `CABLE_COMPLETE_SLACK_MECHANISM` if an unrestrained node loses all stiffness.
- Sample global catenary coordinates `curve_points_global`, mid-span sag `mid_sag_m`, `horizontal_tension_h_kn`, and `max_tension_kn` in `CableResult`.

## Acceptance criteria

- [x] `Cable` elements defined via `UnstressedLength`, `SagRatio`, and `Pretension` all solve to matching equilibrium states and satisfy the analytical Irvine transcendental equations within $10^{-3}$ relative tolerance (`SC-005`).
- [x] Hybrid frame-cable structures (e.g., guyed mast or cable-stayed cantilever) converge in the outer Newton-Raphson solver per combination (`superposition_valid: false`).
- [x] Slack cables under wind/load reversal set `is_slack: true` and append `CABLE_SLACK_DETECTED` to `Diagnostics.warnings` with bilingual messages without crashing the solver (`SC-005`).
- [x] `CableResult.curve_points_global` provides smooth world-space $(x, y)$ coordinates along the catenary curve for viewport rendering.
- [x] Companion unit tests (`*_test.rs`) and `npm run check:limits` pass with 0 errors.

## Artifact sync required

- Application PRD: `none - already covered in docs/prd.md`
- Application architecture summary: `none - matches docs/architecture/application-architecture-summary.md`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-frame-analysis-engine.md`
- Issue registry: `required`; node `issues:` reference: `required: .okf/capabilities/cap-frame-analysis-engine.md`
- Reason/no-impact decision: `Delivers the specified exact Irvine elastic catenary solver inside src-tauri/src/frame/.`

## Human review gate

`None` (backend numerical solver slice with no rendered UI/UX changes).

## Blocked by

- Blocked by `docs/agents/issues/pending/20261101-018-async-jobs-unilateral-pdelta-buckling.md`

## Artifact anchors

- PRD: `FR-MOD-02` (`Cable`), `FR-SOL-05`, `NFR-PERF-02`, `NFR-VER-01`
- Domain model: `Cable`, `CablePrestress`, `CableResult`, `ConvergencePolicy` (Catenary Inner & Global NR)
- Use cases: `UC-06 StartAnalysisJob`, `UC-08 GetAnalysisJobResult`
- Contract: `CableResult` payload §2.2, `CABLE_SLACK_DETECTED`, `CABLE_COMPLETE_SLACK_MECHANISM`

## Acceptance scenarios addressed

- `SC-005`: Exact Irvine Elastic Catenary Cable Solver and Slack Detection

## Verification obligations

- Policy source: `/.okf/project.md`

| Scenario | Backend boundary | Frontend integration | End-to-end journey |
|---|---|---|---|
| `SC-005` | `planned` | `deferred: owned by cap-sketcher-frame-modeler UI slice` | `not-applicable` |
