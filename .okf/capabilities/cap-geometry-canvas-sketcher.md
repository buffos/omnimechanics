---
type: capability
title: Interactive 2D Canvas Drafting & Custom Geometry Sketcher
description: Roll-up for user-driven sketching inside the application viewport, split into the Mode 2 structural frame modeler and the continuum domain sketcher.
tags: [sketcher, cad, drafting, drawing, interactive, 2d, rollup]
timestamp: 2026-11-01T19:20:00Z
state: specified
state_changed: 2026-11-01T19:20:00Z
state_policy:
  mode: rollup
  source: structural_children
  reducer: min
project: /project.md
parent: /capabilities/cap-geometry-meshing.md
children:
  - /capabilities/cap-sketcher-frame-modeler.md
  - /capabilities/cap-sketcher-continuum-domain.md
artifact_root: docs/architecture/cap-geometry-canvas-sketcher
discovery_notes: docs/architecture/cap-geometry-canvas-sketcher/discovery-notes.md
gap_analysis: docs/architecture/cap-geometry-canvas-sketcher/requirements-gap-analysis.md
issues:
  - docs/agents/issues/done/20261031-015-interactive-canvas-sketcher.md
---

# Intent

Aggregate and navigate user-driven sketching: structural frames (Mode 2) and continuum domains (Mode 1
meshing input). This node owns no delivery scope of its own. Its state is the minimum of its children.

# Scope

### Included (via children)
- [Mode 2 Structural Frame Modeler](/capabilities/cap-sketcher-frame-modeler.md): drafting and live analysis of 2D frames, trusses, springs, and cables.
- [Continuum Domain Sketcher](/capabilities/cap-sketcher-continuum-domain.md): polygons, cutouts, CDT mesh handoff.

### Excluded
- Analysis logic and the structural model (owned by [2D Frame Analysis Engine](/capabilities/cap-frame-analysis-engine.md)).
- Freeform 3D NURBS lofting and parametric constraint solvers.

# Relationships

- Parent: [Geometry Definition & Conformal Meshing](/capabilities/cap-geometry-meshing.md)
- Child: [Mode 2 Structural Frame Modeler](/capabilities/cap-sketcher-frame-modeler.md)
- Child: [Continuum Domain Sketcher](/capabilities/cap-sketcher-continuum-domain.md)

# Notes

- 2026-10-31: Issue 015 failed the pre-implementation gate (no frame solver, divergent domain models,
  missing application synthesis). See the gap analysis. A 40-question fog-clearing run then split this
  node into a roll-up. Issue 015 is archived as superseded.
- The earlier inline domain sketch (`SketchModel` with boundaries) is retired. The structural model is now
  owned by the frame analysis engine, and the continuum model by the continuum sketcher.
