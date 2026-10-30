---
type: capability
title: Geometry Definition, CAD Modeling & Conformal Meshing Studio
description: Umbrella capability governing Mode 2 (Custom Modeling & CAD Drafting Studio) and Mode 1 geometry generation, integrating parametric benchmark presets, external CAD file ingestion (DXF, STL, SVG), interactive canvas drafting, and automated conformal BEM/FEM meshing.
tags: [geometry, meshing, cad, drafting, tessellation, 2d, 3d, mode2]
timestamp: 2026-10-30T21:15:00Z
state: specified
state_policy:
  mode: rollup
  source: structural_children
  reducer: min
project: /project.md
parent: /project.md
children:
  - /capabilities/cap-geometry-presets.md
  - /capabilities/cap-geometry-cad-import.md
  - /capabilities/cap-geometry-canvas-sketcher.md
---

# Intent

Provide a unified, multi-faceted geometry modeling environment for OmniMechanics. It services both Mode 1 (instant parametric benchmarking) and Mode 2 (user-driven custom modeling, CAD file import, and interactive canvas drafting), ensuring all geometric definitions are seamlessly translated into conformal boundary (BEM) and volume (FEM) discretizations, as well as 1D discrete frame members for the Step-by-Step Educational Studio (Mode 3).

# Scope

### Included:
- **Structural Sub-capabilities:**
  - [Parametric Benchmark Presets & Conformal Meshing](/capabilities/cap-geometry-presets.md) (`implemented`): Pre-configured 2D/3D benchmarks with verified conformal tessellation.
  - [External CAD Ingestion & Polyline Healing](/capabilities/cap-geometry-cad-import.md) (`specified`): 2D DXF/SVG and 3D STL geometry parsers with topological gap healing.
  - [Interactive Canvas Drafting & Geometry Sketcher](/capabilities/cap-geometry-canvas-sketcher.md) (`specified`): Direct viewport member and polygon sketching with snapping and boundary/load tagging.
- **Conformal Meshing Pipeline:**
  - 1D/2D boundary elements for BEM strictly conforming to 2D/3D interior elements for FEM.
  - Global mesh density control ($h$-refinement) and local adaptive refinement around cutouts and stress concentrations.

### Excluded:
- Non-conformal immersed boundary methods.
- Full CSG solid boolean modeling in the initial release.

# Relationships

- Parent: [OmniMechanics Platform](/project.md)
- Child: [Parametric Geometry Presets & Conformal Meshing](/capabilities/cap-geometry-presets.md)
- Child: [External CAD Ingestion & Healing](/capabilities/cap-geometry-cad-import.md)
- Child: [Interactive Canvas Drafting & Sketcher](/capabilities/cap-geometry-canvas-sketcher.md)
- Shared with: [Interactive 3D Canvas & UI](/capabilities/cap-interactive-canvas.md)
- Feeds into: [Solver Core Engine](/capabilities/cap-solver-core.md)
- Feeds into: [Structural Mechanics Step-by-Step Educational Engine](/capabilities/cap-structural-mechanics-teaching.md)
