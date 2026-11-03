---
type: capability
title: Interactive 3D Visualization Canvas & UI
description: Angular 20+ and Three.js visual workbench featuring interactive boundary condition/load picking on mesh faces, material heatmaps, deformed shape animations, 3D clipping planes, and real-time convergence dashboards.
tags: [ui, threejs, angular, tauri, visualization, picking, 3d, signals]
timestamp: 2026-11-01T19:20:00Z
state: specified
state_changed: 2026-11-01T19:20:00Z
project: /project.md
parent: /project.md
artifact_root: docs/architecture/cap-interactive-canvas
orchestration_status: docs/architecture/cap-interactive-canvas/orchestration-status.md
prd: docs/architecture/cap-interactive-canvas/prd.md
glossary: docs/architecture/cap-interactive-canvas/domain-glossary.md
domain_model: docs/architecture/cap-interactive-canvas/canonical-domain-model.md
use_cases: docs/architecture/cap-interactive-canvas/canonical-use-cases.md
contract: docs/architecture/cap-interactive-canvas/canonical-api-cli-contract.md
scenarios: docs/architecture/cap-interactive-canvas/acceptance-scenarios.md
readiness_review: docs/architecture/cap-interactive-canvas/readiness-review.md
issues:
  - docs/agents/issues/done/20261028-001-scaffold-tauri-angular-rust.md
---

# Intent

Provide an intuitive, responsive, and aesthetically modern visual workspace for computational mechanics, transforming abstract numerical arrays into immediate visual insights in 2D and 3D with interactive boundary selection.

# Scope

### Included:
- **3D / 2D Viewport (Three.js):**
  - Smooth orbit controls, perspective / orthographic cameras, lighting, and coordinate axes.
  - Mesh rendering modes: Surface solid, wireframe overlay, volume element edges.
  - 3D Interactive Face / Edge Picking: Click directly on boundary facets to apply fixed displacements ($\bar{u}_i = 0$), prescribed tractions ($\bar{t}_i = P$), or Radiating / Robin Interface ($\mathbf{t} + \mathbf{Z}\mathbf{u} = \mathbf{g}$).
  - 3D Dynamic Clipping Plane Slider: Slice into the 3D volume to inspect internal stress fields and material variation.
- **Physical Field Visualization:**
  - Material stiffness heatmap $\mu(\mathbf{x})$ rendered over undeformed mesh.
  - Deformed mesh representation with interactive displacement exaggeration slider ($\times 1, \times 10, \times 100, \times 1000$).
  - Continuous colormaps (Viridis, Coolwarm, Turbo, Jet) for von Mises stress $\sigma_{\text{vM}}$, principal stresses, and displacement magnitudes $\|\mathbf{u}\|$.
  - Vector glyph arrows depicting applied loads and reaction tractions.
- **Scientific Analytics & Minimalist UI Architecture (Angular 20+ Signals):**
  - Live streaming residual convergence plot (updated live on each iteration via Tauri IPC).
  - Transient wave playback bar with slider scrubbing, play/pause, and frame stepping.
  - **Modular Dialog & Toast System:** Standalone `DialogService` / `DialogHost` and `NotificationService` / `NotificationHost` (adapted from `code.teacher`) to declutter sidebars and present advanced modal configuration (external soil medium, Talbot parameters, time marching, material expressions) without visual noise.
  - Parameter control sidebars with reactive forms and instant validation.

### Excluded:
- VR / AR headsets rendering.

# Relationships

- Parent: [BemApp Platform](/project.md)
- Depends on: [Geometry & Meshing](/capabilities/cap-geometry-meshing.md)
- Depends on: [Solver Core Engine](/capabilities/cap-solver-core.md)
- Depends on: [Parametric Benchmarking](/capabilities/cap-parametric-benchmarking.md)
