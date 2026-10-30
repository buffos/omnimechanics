---
type: capability
title: Scientific Parametric Benchmarking & Verification Engine
description: Automated test runner and comparative analysis engine evaluating convergence rate, L2 accuracy, CPU/GPU runtime, and speedup ratios of BEM-FEM vs pure FEM across varying inhomogeneity and mesh sizes.
tags: [benchmarking, parametric-study, verification, convergence, profiling, error-norm]
timestamp: 2026-10-28T05:18:00Z
state: in-progress
project: /project.md
parent: /project.md
issues:
  - docs/agents/issues/done/20261029-008-bottom-dock-response-chart-ui-tabs.md
  - docs/agents/issues/done/20261029-009-mini-paper-1-static-contrast-limits.md
  - docs/agents/issues/done/20261029-010-mini-paper-2-complexity-scaling.md
---

# Intent

Provide scientific rigor and empirical evidence answering: *Under what inhomogeneity contrast $\alpha$ and mesh density $h$ is the BEM-FEM predictor-corrector superior to pure FEM?* Automate multi-parameter sweeps and generate publication-ready tables and graphs.

# Scope

### Included:
- **Automated Parameter Sweeps:**
  - Inhomogeneity contrast sweep: $\alpha \in [0.0, 0.60]$ in configurable step sizes.
  - Mesh refinement sweep ($h$-refinement): Evaluation of convergence order and discretization error.
  - Frequency / time sweep: Verification across quasi-static, low-frequency, and transient wave regimes.
- **Metric Extraction & Comparison:**
  - Relative displacement and stress error norms: $L_2 = \|\mathbf{u}_{\text{BEM-FEM}} - \mathbf{u}_{\text{exact/FEM}}\| / \|\mathbf{u}_{\text{exact/FEM}}\|$.
  - Computational timing breakdown: Matrix assembly time, LU factorization time, per-iteration evaluation time, total wall-clock time.
  - Number of iterations to reach specified tolerance $\epsilon = 10^{-3}, 10^{-5}$.
  - Speedup factor: $S = T_{\text{pure FEM}} / T_{\text{BEM-FEM}}$.
- **Export & Reporting:**
  - Export data to CSV, JSON, and LaTeX formatted tables.
  - Graphical convergence plots (iterations vs residual, $\alpha$ vs CPU time).

### Excluded:
- Distributed cluster multi-node MPI execution (focus is single-workstation multi-core CPU & GPU).

# Relationships

- Parent: [BemApp Platform](/project.md)
- Depends on: [Solver Core Engine](/capabilities/cap-solver-core.md)
- Surfaces to: [Interactive 3D Canvas](/capabilities/cap-interactive-canvas.md)
