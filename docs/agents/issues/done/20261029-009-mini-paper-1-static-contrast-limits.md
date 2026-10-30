## Issue Metadata

- Issue number: `009`
- Owning capability node: `/.okf/capabilities/cap-parametric-benchmarking.md`
- Artifact root: `docs/architecture/cap-parametric-benchmarking/`
- Issue file: `docs/agents/issues/pending/20261029-009-mini-paper-1-static-contrast-limits.md`
- Category: `experiment`
- Execution type: `AFK`
- Review gate: `document-review`
- Suggested state: `in-progress`

## Parent Artifacts

- `.okf/project.md`
- `.okf/capabilities/cap-parametric-benchmarking.md`
- `papers/Variational_BEM_FEM_Predictor_Corrector_Elastostatics.tex`

## What to build

Implement an automated Rust benchmarking suite and author **Mini-Paper 1 (MP-01)**:
*Static Convergence Limits, Inhomogeneity Contrast Thresholds ($\alpha$), and the "Rock in Soil" Hard/Soft Inclusion Challenge*.

1. **Automated Rust Benchmark Harness (`src-tauri/src/benchmarks/static_contrast.rs`):**
   - Provide standalone, reproducible benchmarking execution for static operator splitting $\mathbf{E}_0 \mathbf{u}^{(k+1)} = \mathbf{F} - \mathbf{K}_{\text{pert}} \mathbf{u}^{(k)}$ vs Pure FEM.
   - **Sweep A (Functional Continuous Gradient):**
     Linear Young's modulus gradient $E(x) = E_0 (1 + \alpha x / L)$ across $\alpha \in [-0.9, -0.5, -0.2, 0.0, 0.2, 0.5, 0.8, 1.0, 1.2, 1.5, 2.0, 3.0, 5.0, 10.0]$.
   - **Sweep B (Hard Inclusion -- "Rock in Soil"):**
     Circular/Square inclusion embedded in homogeneous matrix:
     $E_{\text{matrix}} = 100\text{ MPa}$, $E_{\text{inc}} / E_0 \in [1.0, 1.5, 2.0, 3.0, 5.0, 10.0, 20.0, 50.0, 100.0, 1000.0]$.
   - **Sweep C (Soft Inclusion / Cavity -- "Soft Pocket in Rock"):**
     $E_{\text{matrix}} = 30\text{ GPa}$, $E_{\text{inc}} / E_0 \in [1.0, 0.8, 0.5, 0.2, 0.1, 0.05, 0.01]$.
   - For every sample point, measure and export:
     - Iterations to convergence (tolerance $\epsilon = 10^{-4}$, max iterations 100)
     - Convergence flag (`converged: bool`)
     - Approximate asymptotic spectral radius $\rho_{\text{est}} = \frac{\|r_k\|}{\|r_{k-1}\|}$
     - Relative $L_2$ error norm vs Pure FEM: $\| \mathbf{u}_{\text{BEMFEM}} - \mathbf{u}_{\text{PureFEM}} \|_2 / \| \mathbf{u}_{\text{PureFEM}} \|_2$
     - CPU execution time (ms) for BEMFEM vs Pure FEM
   - Output structured CSV datasets into `docs/benchmarks/`.

2. **Mini-Paper 1 Technical Report (`papers/mini_papers/01_static_contrast_limits.tex`):**
   - Author publication-quality LuaLaTeX technical paper:
     - Mathematical analysis of the Banach fixed-point iteration matrix $\mathbf{G} = \mathbf{I} - \omega \mathbf{K}_0^{-1} \mathbf{K}(\mathbf{x})$ and Rayleigh quotient bounds.
     - Why high-contrast hard inclusions ("rock in soil") cause spectral radius expansion $\rho(\mathbf{G}) > 1$ when $E_0 = E_{\text{soil}}$.
     - Rigorous comparison tables with exact values from the Rust benchmarks.
     - TikZ/PGFPlots convergence curves and spectral radius vs contrast ratio plots.
     - Concrete operational guidelines: When is BEM-FEM recommended, when is under-relaxation required, and when should the dual formulation or Pure FEM be selected.
   - Compile cleanly to PDF using LuaLaTeX.

## Acceptance criteria

- [x] Automated benchmark harness implemented in Rust (`src-tauri/src/benchmarks/static_contrast.rs`).
- [x] Benchmark test suite executed, generating valid CSV data in `docs/benchmarks/` (`mp01_gradient_sweep.csv`, `mp01_rock_inclusion_sweep.csv`, `mp01_soft_cavity_sweep.csv`).
- [x] Mini-Paper 1 LaTeX document written in `papers/mini_papers/01_static_contrast_limits.tex`.
- [x] LuaLaTeX compiles `01_static_contrast_limits.tex` to PDF with 0 errors (`MP01_Static_Convergence_Limits.pdf`, 11 pages).
- [x] All Rust unit and benchmark tests pass (`cargo test`).
- [x] Theoretical theorems proved: $\rho = |\alpha|$ for continuous gradients, $\rho = 1 - 1/\kappa$ for rock-base operator, and unconditional stability of soft inclusions.

## Verification Evidence

1. **Rust Test Execution:**
   - Ran `cargo test --lib test_execute_and_export_all_mp01_benchmarks -- --nocapture` -> Passed in 180s.
   - Ran `cargo test --lib solver::tests` -> All 14 unit tests passed in 1.55s.
2. **Benchmark CSV Data:**
   - `docs/benchmarks/mp01_gradient_sweep.csv` (36 rows): Verified exact linear scaling $\rho = |\alpha|$ for $\omega = 1.00$, with divergence at $\alpha \ge 1.0$, and stability restored up to $\alpha = 2.0$ via $\omega = 0.50$.
   - `docs/benchmarks/mp01_rock_inclusion_sweep.csv` (50 rows): Verified that soil-base operator diverges for $\kappa \ge 3$ ($\omega=1.0$) and $\kappa \ge 20$ ($\omega=0.20$); whereas rock-base operator is unconditionally stable with $\rho = 1 - 1/\kappa$ to 4 decimal places.
   - `docs/benchmarks/mp01_soft_cavity_sweep.csv` (16 rows): Verified that cavities never diverge ($\rho \le 0.88$ for 99% void), matching monolithic FEM to $10^{-6}$ error norm.
3. **LaTeX Monograph Compilation:**
   - `papers/mini_papers/MP01_Static_Convergence_Limits.pdf` compiled cleanly with MiKTeX LuaLaTeX (11 pages, exit code 0).
4. **Angular Frontend Verification:**
   - `npm run build` compiled cleanly in 3.08s with 0 errors.
5. **OKF Conformance:**
   - `node .agents/plugins/planning/skills/okf-validate/scripts/okf-validate.mjs .okf` reported `conformant - no issues`.
