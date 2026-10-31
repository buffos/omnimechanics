## Issue Metadata

- Issue number: `011`
- Owning capability node: `/.okf/capabilities/cap-parametric-benchmarking.md`
- Artifact root: `docs/architecture/cap-parametric-benchmarking/`
- Issue file: `docs/agents/issues/done/20261029-011-mini-paper-3-transient-elastodynamics-talbot.md`
- Category: `experiment`
- Execution type: `AFK`
- Review gate: `document-review`
- Suggested state: `done`

## Parent Artifacts

- `.okf/project.md`
- `.okf/capabilities/cap-parametric-benchmarking.md`
- `.okf/capabilities/cap-solver-core.md`
- `src-tauri/src/solver/talbot.rs`
- `src-tauri/src/solver/time_marching.rs`
- `src-tauri/src/solver/dynamic_solver.rs`

## Proposed Work

Implement an automated elastodynamic benchmarking suite comparing Deformed Talbot Contour Inversion vs Direct Time-Marching (Newmark-$\beta$ / HHT-$\alpha$) and author **Mini-Paper 3 (MP-03)**:
*Transient Elastodynamics and Numerical Dispersion: Deformed Talbot Contour Integration vs Direct Time-Marching (Newmark-$\beta$ & HHT-$\alpha$)*.

### 1. Benchmark Suite (`src-tauri/src/benchmarks/transient_elastodynamics.rs`):
- **Sweep A: Numerical Dispersion & Phase Error vs Time Step $\Delta t$:**
  - Harmonic excitation (sine wave $f = 3\,\text{Hz}$ and $10\,\text{Hz}$).
  - Measure period elongation $\Delta T / T$ and artificial dissipation across Newmark time step sizes ($\Delta t \in [T/10, T/20, T/50, T/100, T/500]$) vs Talbot with $N \in [16, 24, 32, 48]$ contour nodes.
  - Export to `docs/benchmarks/mp03_dispersion_newmark_vs_talbot.csv`.
- **Sweep B: Transient Pulse / Shock Wave Transmission:**
  - Half-sine pulse impact ($t_{\text{pulse}} = 5\,\text{ms}$).
  - Compare wavefront sharpness, arrival time, and peak displacement between Newmark and Talbot.
  - Export to `docs/benchmarks/mp03_shock_pulse_wave_propagation.csv`.
- **Sweep C: Computational Wall-Clock Time vs Simulation Duration:**
  - Evaluate point-in-time calculation ($t = 1.0\,\text{s}, 5.0\,\text{s}, 10.0\,\text{s}$) showing how Talbot solves directly without stepping through thousands of historical steps.
  - Measure parallel scaling across Rayon threads for Talbot quadrature nodes.
  - Export to `docs/benchmarks/mp03_runtime_talbot_vs_newmark.csv`.

### 2. LaTeX Technical Report (`papers/mini_papers/03_transient_talbot_vs_newmark.tex`):
- Author publication-grade LaTeX report with rigorous mathematical derivations (Talbot contour geometry, Weideman-Trefethen optimal parameters, Newmark spectral stability & dispersion relation, parallel speedup).
- Compile via LuaLaTeX into `papers/mini_papers/03_transient_talbot_vs_newmark.pdf`.

## Acceptance Criteria

- [x] Transient elastodynamics benchmark module implemented in `src-tauri/src/benchmarks/transient_elastodynamics.rs`.
- [x] Automated benchmark execution exports CSV datasets in `docs/benchmarks/`.
- [x] Mini-Paper 3 LaTeX report authored in `papers/mini_papers/03_transient_talbot_vs_newmark.tex`.
- [x] LuaLaTeX compiles `03_transient_talbot_vs_newmark.pdf` with 0 errors.
- [x] All unit and benchmark tests pass (`cargo test --lib`).
