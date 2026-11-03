---
type: capability
title: Continuum Domain Sketcher (Polygons, Cutouts & Conformal Mesh Handoff)
description: Sketching of closed 2D continuum boundaries with holes and cutouts, with loop validation and constrained Delaunay handoff into conformal FEM/BEM meshes for Mode 1 solvers.
tags: [sketcher, continuum, polygon, cutout, cdt, delaunay, meshing, bem-fem]
timestamp: 2026-11-03T18:00:00Z
state: implemented
state_changed: 2026-11-03T18:00:00Z
project: /project.md
parent: /capabilities/cap-geometry-canvas-sketcher.md
shared_with:
  - /capabilities/cap-geometry-presets.md
artifact_root: docs/architecture/cap-sketcher-continuum-domain
orchestration_status: docs/architecture/cap-sketcher-continuum-domain/orchestration-status.md
prd: docs/architecture/cap-sketcher-continuum-domain/prd.md
glossary: docs/architecture/cap-sketcher-continuum-domain/domain-glossary.md
domain_model: docs/architecture/cap-sketcher-continuum-domain/canonical-domain-model.md
use_cases: docs/architecture/cap-sketcher-continuum-domain/canonical-use-cases.md
contract: docs/architecture/cap-sketcher-continuum-domain/canonical-api-cli-contract.md
scenarios: docs/architecture/cap-sketcher-continuum-domain/acceptance-scenarios.md
readiness_review: docs/architecture/cap-sketcher-continuum-domain/readiness-review.md
issues:
  - docs/agents/issues/done/20261103-027-continuum-domain-polygon-cutout-validation-engine.md
  - docs/agents/issues/done/20261103-028-conformal-cdt-mesher-boundary-tagging-inclusions.md
  - docs/agents/issues/done/20261103-029-mode1-continuum-sketcher-ui-canvas-solver-handoff.md
---

# Intent

Let users draw arbitrary 2D continuum domains (plates with holes, inclusions) and send them to the
conformal BEM-FEM meshing pipeline.

# Scope

### Included
- Polygon boundary tool with automatic loop closure and self-intersection prevention.
- Circular, rectangular, and polygonal cutouts inside existing domains.
- Boundary-condition and load tagging on boundary segments.
- Constrained Delaunay triangulation into Tri3/Quad4 interior elements and Line2/Line3 BEM patches.

### Excluded
- Frame/truss sketching (sibling `cap-sketcher-frame-modeler`).
- NURBS lofting, parametric constraint solvers.

### Acceptance seed (carried over)
- A 10 × 10 m square with an R = 1.5 m central hole: loop closure validates, and "Generate Conformal
  Mesh" produces a graded Tri3 mesh conforming to the BEM boundary elements.

# Relationships

- Parent: [Interactive Canvas Drafting & Sketcher (roll-up)](/capabilities/cap-geometry-canvas-sketcher.md)
- Feeds into: [Parametric Geometry Presets & Conformal Meshing](/capabilities/cap-geometry-presets.md)

# Notes

- 2026-11-03: All 3 vertical delivery slices (`027` polygon/cutout/clearance validation engine, `028` conformal CDT mesher with `Tri3`/`Quad4`, `Line2`/`Line3`, inclusions & provenance, and `029` Mode 1 2D Continuum Sketcher UI, Inspector & solver handoff) implemented, strict code-review-loop verified, and archived to `docs/agents/issues/done/`.
