## Issue Metadata

- Issue number: `013`
- Owning capability node: `/.okf/capabilities/cap-parametric-benchmarking.md`
- Artifact root: `docs/architecture/cap-parametric-benchmarking/`
- Issue file: `docs/agents/issues/done/20261029-013-mini-paper-5-half-space-gibson-soil.md`
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
- `papers/mini_papers/04_dynamic_operator_splitting_talbot.tex`

## Proposed Work

Execute and author **Mini-Paper 5 (MP-05)**:
*Dynamic Elastodynamics in an Inhomogeneous Elastic Half-Space: Pure FEM vs Variational BEM-FEM Operator Splitting across Mesh Discretization, Accuracy, and CPU Wall-Clock Time*.

### 1. Problem Definition (Fair Ground: Full-Depth Gibson Soil Half-Space):
- Computational domain representing a 2D half-space with width $W = 40\,\text{m}$ and depth $H = 20\,\text{m}$.
- Traction-free ground surface with a dynamic surface footing load $P(t)$ ($P_0 = 100\,\text{kN}, t_p = 0.05\,\text{s}$).
- Continuous full-depth inhomogeneity (Gibson soil profile):
  $$E(y) = E_0 \left(1 + \alpha \frac{H - y}{H}\right), \quad \rho(y) = \rho_0 \left(1 + \frac{\alpha}{2} \frac{H - y}{H}\right)$$
- Boundary conditions: Fixed bottom bedrock, lateral boundaries far field.
- Both Pure FEM and BEM-FEM operate on the identical spatial grid.

### 2. Multi-Resolution Parametric Protocol:
- **Ground Truth Baseline:** Overkill high-resolution Pure FEM model ($80 \times 40$ Quad4, 6,642 DOFs) to obtain benchmark time-history $u_{\text{baseline}}(t)$ and peak surface displacement.
- **Mesh Discretization Sweep (Coarse to Fine):**
  - Run Pure FEM + Talbot and BEM-FEM Operator Splitting + Talbot for resolutions:
    - $10 \times 5$
    - $20 \times 10$
    - $40 \times 20$
    - $60 \times 30$
  - For each method and resolution, measure:
    - Mesh DOFs
    - Discretization error relative to baseline ($\% L_2$ error norm)
    - CPU Wall-Clock time (ms)
- **Material Contrast Sweep:** Repeat across contrast ratios $\alpha \in \{-0.5, 0.0, 0.5, 1.0, 1.5\}$.
- **Pareto Trade-off Function:** Construct the empirical 3D surface:
  $$\text{Discretization DOFs} \longrightarrow \text{Accuracy (\% Error vs Baseline)} \longrightarrow \text{Runtime (ms)}$$

### 3. LaTeX Technical Report & Deliverables:
- Export benchmark datasets to `docs/benchmarks/mp05_*.csv`.
- Author `papers/mini_papers/05_half_space_gibson_soil.tex`.
- Compile to `papers/mini_papers/05_half_space_gibson_soil.pdf` with LuaLaTeX.

## Acceptance Criteria

- [x] Surface-loaded half-space benchmark suite implemented in `src-tauri/src/benchmarks/half_space_gibson.rs`.
- [x] Baseline overkill FEM solution computed and verified (6,642 DOFs, 386.0 ms).
- [x] Complete empirical sweep of $(\text{DOFs}, \% \text{error}, \text{runtime})$ exported to CSV (`docs/benchmarks/mp05_*.csv`).
- [x] Mini-Paper 5 LaTeX report authored with PGFPlots Pareto curves and compiled with 0 errors (5 pages, 231,306 bytes).
- [x] Unit tests pass cleanly (`cargo test --lib` -> 29 passed, 0 failed).
