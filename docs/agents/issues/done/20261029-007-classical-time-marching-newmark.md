## Issue Metadata

- Issue number: `007`
- Owning capability node: `/.okf/capabilities/cap-solver-core.md`
- Artifact root: `docs/architecture/cap-solver-core/`
- Issue file: `docs/agents/issues/done/20261029-007-classical-time-marching-newmark.md`
- Category: `feature`
- Execution type: `AFK`
- Review gate: `visual-review`
- Suggested state: `done`

## Parent Artifacts

- `.okf/project.md`
- `.okf/capabilities/cap-solver-core.md`
- `docs/roadmap.md`
- `papers/Variational_BEM_FEM_Predictor_Corrector_Elastostatics.tex`

## What to build

Implement Classical Direct Time-Marching (Newmark-$\beta$ and HHT-$\alpha$) in the Rust computational engine (`src-tauri/src/solver/`) and integrate into the Angular simulation workbench:

1. **Newmark-$\beta$ Temporal Discretization (`time_marching.rs`):**
   - Formulate the effective dynamic stiffness matrix $\mathbf{K}_{\text{eff}} = \mathbf{K} + \frac{\gamma}{\beta \Delta t} \mathbf{C} + \frac{1}{\beta \Delta t^2} \mathbf{M}$.
   - For constant $\Delta t$ and linear material response, pre-factorize $\mathbf{K}_{\text{eff}}$ via sparse Cholesky / LU once prior to time stepping.
   - For each time step $t_{n+1} = (n+1)\Delta t$:
     - Formulate effective RHS: $\mathbf{F}_{\text{eff}} = \mathbf{F}_{n+1} + \mathbf{M}(a_0 \mathbf{u}_n + a_2 \dot{\mathbf{u}}_n + a_3 \ddot{\mathbf{u}}_n) + \mathbf{C}(a_1 \mathbf{u}_n + a_4 \dot{\mathbf{u}}_n + a_5 \ddot{\mathbf{u}}_n)$.
     - Solve for $\mathbf{u}_{n+1}$ via fast forward-backward backsubstitution.
     - Update kinematics:
       $\ddot{\mathbf{u}}_{n+1} = a_0 (\mathbf{u}_{n+1} - \mathbf{u}_n) - a_2 \dot{\mathbf{u}}_n - a_3 \ddot{\mathbf{u}}_n$
       $\dot{\mathbf{u}}_{n+1} = \dot{\mathbf{u}}_n + a_6 \ddot{\mathbf{u}}_n + a_7 \ddot{\mathbf{u}}_{n+1}$.
   - Default integration parameters: Average acceleration ($\beta = 1/4, \gamma = 1/2$) guaranteeing unconditional stability.

2. **Hilber-Hughes-Taylor ($\text{HHT}-\alpha$) Numerical Dissipation:**
   - Parameterize $\alpha \in [-1/3, 0]$ with $\gamma = \frac{1 - 2\alpha}{2}$ and $\beta = \frac{(1 - \alpha)^2}{4}$ to filter spurious high-frequency spatial mesh noise without numerical damping of fundamental modes.

3. **Backend Multi-Domain Solver Dispatch:**
   - Extend `TransientSimulationRequest` with `integration_method: Option<String>` (`"Talbot"` vs `"Newmark"`).
   - In `run_transient_simulation`, dispatch to `time_marching::solve_transient_newmark` when requested.

4. **Monograph Chapter Addition:**
   - Author Section 10 in `Variational_BEM_FEM_Predictor_Corrector_Elastostatics.tex`: Complete mathematical derivation of Newmark-$\beta$ and HHT-$\alpha$, step-by-step algorithmic comparison table against Talbot contour, dispersion and stability analysis, and academic citations (Newmark 1959, Hilber-Hughes-Taylor 1977, Bathe 1996, Hughes 2000, Chopra 2012).

5. **Frontend UI Integration:**
   - Add segmented control / select under Transient analysis domain: `[ Deformed Talbot (Laplace) | Newmark-β Direct ]`.
   - Support seamless wave playback in Three.js viewport for both methods.

## Acceptance criteria

- [x] Mathematical formulation of Newmark-$\beta$ and HHT-$\alpha$ added to monograph with all citations.
- [x] Rust implementation of `solve_transient_newmark` with pre-factorized $\mathbf{K}_{\text{eff}}$ in `time_marching.rs`.
- [x] Exact kinematic updates and initial acceleration recovery ($\mathbf{M}\ddot{\mathbf{u}}_0 = \mathbf{F}_0 - \mathbf{K}\mathbf{u}_0$).
- [x] Direct benchmarking test comparing Newmark-$\beta$ vs Talbot contour on 2D cantilever beam.
- [x] Tauri IPC command and Angular UI selection for transient solver method.
- [x] All Rust unit tests passing cleanly (`cargo test --lib`).

## Artifact sync required

- Application PRD: `none - solver core implementation`
- Application architecture summary: `none - internal solver capability`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-solver-core.md`
- Issue registry: `required`; node `issues:` reference: `required`
- Reason/no-impact decision: `Implements classical direct time-marching for elastodynamic transient benchmarking.`
