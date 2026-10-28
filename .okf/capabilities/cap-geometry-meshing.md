---
type: capability
title: Geometry Definition & Conformal Auto-Tessellation Engine
description: Parametric geometry modeling, CAD file import (DXF, STEP, STL), and automated conformal meshing generating synchronized 2D boundary elements and 2D/3D interior volume elements.
tags: [geometry, meshing, tessellation, cad, 2d, 3d, gmsh, triangle, tetgen]
timestamp: 2026-10-28T05:18:00Z
state: bounded
project: /project.md
parent: /project.md
---

# Intent

Enable automated, error-free geometry creation and discretization for both 2D and 3D domains without requiring manual nodal coordinates or connectivity tables. Provide conformal meshes where the exterior boundary element faces (BEM) precisely match the outer facets of the interior volume elements (FEM).

# Scope

### Included:
- **Dimension Modes:** Explicit 2D (plane stress/strain) and 3D solid continuum modes chosen at project initialization.
- **Parametric Benchmark Presets:**
  - 2D: Cantilever beam, simple-supported beam, plate with central circular hole, cracked tensile specimen.
  - 3D: Prismatic rectangular beam, cube with spherical cavity, thick cylinder, tunnel cross-section.
- **CAD File Ingestion:**
  - 2D: DXF / SVG polygon boundary import.
  - 3D: STEP (B-Rep) and STL surface mesh import.
- **Automated Conformal Meshing:**
  - 2D: High-quality Constrained Delaunay Triangulation (1D boundary elements for BEM, 2D triangles for FEM).
  - 3D: Surface triangular tessellation (BEM) + interior Delaunay tetrahedralization (FEM volume).
  - Global characteristic length control ($h$-refinement) and adaptive boundary clustering.

### Excluded:
- Full-blown interactive CAD sketching engine (geometry is either imported or driven parametrically).

# Relationships

- Parent: [BemApp Platform](/project.md)
- Shared with: [Interactive 3D Canvas](/capabilities/cap-interactive-canvas.md)
- Shared with: [Solver Core Engine](/capabilities/cap-solver-core.md)
