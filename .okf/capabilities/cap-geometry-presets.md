---
type: capability
title: Parametric Geometry Presets & Conformal Meshing Engine
description: Parametric generation of canonical 2D/3D benchmark structures (beams, plates with cavities, solids) with synchronized conformal discretization between exterior boundary elements (BEM) and interior volume elements (FEM).
tags: [geometry, parametric, meshing, tessellation, benchmarks, conformal, 2d, 3d]
timestamp: 2026-10-30T15:23:00Z
state: implemented
project: /project.md
parent: /capabilities/cap-geometry-meshing.md
issues:
  - docs/agents/issues/done/20261028-002-geometry-conformal-meshing.md
---

# Intent

Provide automated, deterministic parametric generation and conformal discretization for standard benchmark geometries in 2D and 3D without requiring external CAD tools, serving as the verified foundation for computational solver benchmarking.

# Scope

### Included:
- **Parametric Canonical Geometries:**
  - 2D: Cantilever beam, simply supported beam, plate with central circular hole.
  - 3D: Prismatic rectangular solid beam, solid block with spherical cavity.
- **Conformal Boundary-to-Volume Tessellation:**
  - 2D: 1D boundary elements for BEM coupled to 2D triangular/quadrilateral volume elements for FEM.
  - 3D: Surface triangular/quadrilateral boundary elements for BEM coupled to interior tetrahedral/hexahedral volume elements for FEM.
  - Strict topological conformity: Exterior faces of FEM elements coincide exactly with BEM boundary elements.
- **Boundary Tagging System:**
  - Semantic tagging (`Left`, `Right`, `Top`, `Bottom`, `Hole`, `Front`, `Back`) for automated boundary condition and load mapping.

### Excluded:
- Arbitrary user-sketched geometry (handled by [Interactive Canvas Sketcher](/capabilities/cap-geometry-canvas-sketcher.md)).
- External CAD file formats (handled by [CAD File Ingestion](/capabilities/cap-geometry-cad-import.md)).

# Relationships

- Parent: [Geometry Definition & Conformal Meshing](/capabilities/cap-geometry-meshing.md)
- Feeds into: [Solver Core Engine](/capabilities/cap-solver-core.md)
- Feeds into: [Parametric Benchmarking](/capabilities/cap-parametric-benchmarking.md)
