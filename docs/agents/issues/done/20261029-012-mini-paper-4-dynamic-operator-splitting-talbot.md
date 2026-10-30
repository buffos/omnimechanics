## Issue Metadata

- Issue number: `012`
- Owning capability node: `/.okf/capabilities/cap-parametric-benchmarking.md`
- Artifact root: `docs/architecture/cap-parametric-benchmarking/`
- Issue file: `docs/agents/issues/done/20261029-012-mini-paper-4-dynamic-operator-splitting-talbot.md`
- Category: `experiment`
- Execution type: `AFK`
- Review gate: `document-review`
- Suggested state: `done`

## Parent Artifacts

- `.okf/project.md`
- `.okf/capabilities/cap-parametric-benchmarking.md`
- `.okf/capabilities/cap-solver-core.md`
- `papers/mini_papers/01_static_contrast_limits.tex`
- `papers/mini_papers/02_complexity_scaling_dofs.tex`
- `papers/mini_papers/03_transient_talbot_vs_newmark.tex`

## Proposed Work

Implement the **Dynamic Variational Operator Splitting (BEM Predictor - FEM Corrector)** framework in the complex Laplace domain, integrate it with Deformed Talbot contour synthesis, benchmark directly against **Pure FEM + Talbot**, and author **Mini-Paper 4 (MP-04)**:
*Dynamic Variational Operator Splitting in the Complex Laplace Domain: Pure FEM vs BEM Predictor -- FEM Corrector under Transient Excitation*.

### 1. Dynamic Operator Splitting Implementation:
- Develop `solve_laplace_dynamic_operator_splitting` and `solve_transient_dynamic_operator_splitting_talbot` for complex frequency $s \in \mathbb{C}$:
  - **Predictor:** Reference homogeneous operator $\mathbf{A}_0(s) = \mathbf{K}_0 + s^2 \mathbf{M}_0$.
  - **Corrector:** Internal dynamic resisting forces $\mathbf{F}_{\text{dyn}}(\mathbf{u}; s) = \mathbf{F}_{\text{int}}(\mathbf{u}) + s^2 \mathbf{M}_{\text{hetero}} \mathbf{u}$ evaluated at Gauss quadrature points.
  - **Relaxation:** $\mathbf{u}^{(k+1)} = \mathbf{u}^{(k)} + \omega \Delta \mathbf{u}$ until residual convergence.
- Time synthesis via Deformed Talbot Contour Inversion across $N \approx 24 - 32$ parallel quadrature nodes.

### 2. Parametric Benchmark Suite (`src-tauri/src/benchmarks/dynamic_operator_splitting.rs`):
- **Sweep A: Broadband Transient Shock Wave:** Half-sine impact pulse comparing time-history trajectories $u(t)$ and relative $L_2$ error norm.
- **Sweep B: Complex Dynamic Convergence vs Inhomogeneity Contrast:** Measure iterations and spectral contraction across material contrast $\alpha \in [-0.7, 2.0]$ for complex frequencies $s_k$.
- **Sweep C: Computational Wall-Clock Time & Scaling:** Measure execution time across mesh sizes for Pure FEM + Talbot vs BEM-FEM Predictor-Corrector + Talbot.
- Export CSV data to `docs/benchmarks/mp04_*.csv`.

### 3. LaTeX Technical Report:
- Author `papers/mini_papers/04_dynamic_operator_splitting_talbot.tex`.
- Compile to publication-grade `04_dynamic_operator_splitting_talbot.pdf` via LuaLaTeX.

## Acceptance Criteria

- [x] Dynamic operator splitting solver implemented in Rust (`src-tauri/src/solver/hybrid_solver.rs`).
- [x] Automated benchmark harness exporting CSV datasets in `docs/benchmarks/` (`mp04_shock_pulse_fem_vs_splitting.csv`, `mp04_contrast_convergence_laplace.csv`, `mp04_runtime_scaling_dofs.csv`).
- [x] Publication-grade LaTeX report authored in `papers/mini_papers/04_dynamic_operator_splitting_talbot.tex`.
- [x] LuaLaTeX compiles `04_dynamic_operator_splitting_talbot.pdf` with 0 errors (8 pages, 320,555 bytes).
- [x] All unit and benchmark tests pass (`cargo test --lib` -> 29 passed, 0 failed).
