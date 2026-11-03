---
type: capability
title: Mode 2 Structural Frame Modeler (Interactive 2D Drafting & Live Analysis Workspace)
description: Mode 2 workspace where users sketch any 2D structure on a snapping canvas, assign sections, supports, and loads, and see live analysis results from the frame analysis engine on the same canvas.
tags: [mode2, sketcher, frame-modeler, drafting, snapping, svg-canvas, live-results, angular]
timestamp: 2026-11-01T19:20:00Z
state: implemented
state_changed: 2026-11-02T20:55:00Z
project: /project.md
parent: /capabilities/cap-geometry-canvas-sketcher.md
shared_with:
  - /capabilities/cap-frame-analysis-engine.md
  - /capabilities/cap-interactive-canvas.md
artifact_root: docs/architecture/cap-sketcher-frame-modeler
discovery_notes: docs/architecture/cap-geometry-canvas-sketcher/discovery-notes.md
gap_analysis: docs/architecture/cap-geometry-canvas-sketcher/requirements-gap-analysis.md
orchestration_status: docs/architecture/cap-sketcher-frame-modeler/orchestration-status.md
prd: docs/architecture/cap-sketcher-frame-modeler/prd.md
glossary: docs/architecture/cap-sketcher-frame-modeler/domain-glossary.md
domain_model: docs/architecture/cap-sketcher-frame-modeler/canonical-domain-model.md
use_cases: docs/architecture/cap-sketcher-frame-modeler/canonical-use-cases.md
contract: docs/architecture/cap-sketcher-frame-modeler/canonical-api-cli-contract.md
scenarios: docs/architecture/cap-sketcher-frame-modeler/acceptance-scenarios.md
readiness_review: docs/architecture/cap-sketcher-frame-modeler/readiness-review.md
issues:
  - docs/agents/issues/done/20261102-023-mode2-drafting-canvas-snapping-splits-transforms.md
  - docs/agents/issues/done/20261102-024-palettes-inspector-live-linear-dsm-diagrams-mechanisms.md
  - docs/agents/issues/done/20261102-025-mode2-loadcases-eurocode-combinations-envelopes-async-jobs.md
  - docs/agents/issues/done/20261102-026-mode3-classifier-handoff-preset-import-diagram-export.md
---

# Intent

Mode 2, "Structural Modeling & Analysis": the user draws any 2D structure and immediately sees
reactions, M/Q/N diagrams, and the deformed shape update live as they edit geometry, supports, or loads.

# Scope

### Included
- Header Mode 2 entry: Mode 1 Continuum Workbench, **Mode 2 Structural Modeling & Analysis**, Mode 3
  Teaching Studio.
- **Drawing tools:** grid snap (selectable step), node snap (< 15 px), midpoint and intersection snap
  with automatic member splitting, ortho lock, angle snap, numeric entry (x,y or L,θ), copy, mirror,
  array, undo/redo, box selection, live L/θ tooltips, and a cursor coordinate readout.
- **Element placement:** frame members, truss bars, springs, and cables. An inspector edits sections,
  materials, releases, rigid offsets, and cable input (L₀, sag ratio, or T₀).
- **Support and load palettes** covering every support and load type owned by the frame analysis engine.
- **Load case / combination / envelope selector** and result display: reactions, M/Q/N on the tension
  side, scaled deformed shape, extrema labels, hover inspection, end-force table, nonlinear progress
  indicator, and the mechanism animation.
- **"Open step-by-step"** handoff to Mode 3. It is enabled by the classifier, and the UI explains when a
  structure is not eligible.
- Opening Mode 3 presets as editable sketches.
- Save/load through project persistence. SVG/PNG export of diagrams.
- Bilingual el/en UI.

### Excluded
- Continuum polygons, holes, and CDT meshing (sibling `cap-sketcher-continuum-domain`).
- DXF import of frames, PDF/LaTeX reports (later).
- Analysis logic (owned by `cap-frame-analysis-engine`).

### Verification
- Pure TS logic (snapping, geometry, model store, undo stack) under a new Vitest harness
  (frontend-integration: required).
- Visual review gate on every UI slice.

# Relationships

- Parent: [Interactive Canvas Drafting & Sketcher (roll-up)](/capabilities/cap-geometry-canvas-sketcher.md)
- Edits model of: [2D Frame Analysis Engine](/capabilities/cap-frame-analysis-engine.md)
- Hands off to: [Structural Mechanics Teaching Engine](/capabilities/cap-structural-mechanics-teaching.md)
- Rendered with: [Interactive Canvas & UI](/capabilities/cap-interactive-canvas.md)
- Persisted by: [Project Persistence](/capabilities/cap-project-persistence.md)

# Notes

Replaces the frame part of superseded issue 015. Delivery slice 1 = this modeler plus the linear DSM core.
