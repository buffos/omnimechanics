---
type: capability
title: 2D Frame Analysis Engine (Direct Stiffness Method)
description: General 2D structural analysis engine that owns the structural model (nodes, members, sections, supports, loads, load cases) and solves arbitrary beams, frames, trusses, springs, and catenary cables with linear, P-Δ, buckling, and modal/seismic analyses.
tags: [frame-analysis, direct-stiffness, dsm, beams, trusses, springs, cables, catenary, p-delta, buckling, modal, response-spectrum, rust]
timestamp: 2026-11-01T19:20:00Z
state: implemented
state_changed: 2026-11-02T18:00:00Z
project: /project.md
parent: /project.md
shared_with:
  - /capabilities/cap-eurocode-actions-combinations.md
artifact_root: docs/architecture/cap-frame-analysis-engine
discovery_notes: docs/architecture/cap-geometry-canvas-sketcher/discovery-notes.md
orchestration_status: docs/architecture/cap-frame-analysis-engine/orchestration-status.md
prd: docs/architecture/cap-frame-analysis-engine/prd.md
glossary: docs/architecture/cap-frame-analysis-engine/domain-glossary.md
domain_model: docs/architecture/cap-frame-analysis-engine/canonical-domain-model.md
use_cases: docs/architecture/cap-frame-analysis-engine/canonical-use-cases.md
contract: docs/architecture/cap-frame-analysis-engine/canonical-api-cli-contract.md
scenarios: docs/architecture/cap-frame-analysis-engine/acceptance-scenarios.md
readiness_review: docs/architecture/cap-frame-analysis-engine/readiness-review.md
issues:
  - docs/agents/issues/done/20261101-016-frame-model-catalog-kinematic-diagnostics.md
  - docs/agents/issues/done/20261101-017-linear-static-dsm-analytical-diagrams-envelopes.md
  - docs/agents/issues/done/20261101-018-async-jobs-unilateral-pdelta-buckling.md
  - docs/agents/issues/done/20261101-019-irvine-elastic-catenary-cable-solver.md
  - docs/agents/issues/done/20261101-020-modal-rsa-lateral-force-opensees-benchmarks.md
---

# Intent

Let a user solve **any** 2D structure (beams, frames, trusses, springs, cables) and see the results
live while editing. The engine is the shared analysis core for Mode 2 (modeling & analysis), Mode 3
(teaching: classifier and DSM walkthrough), and Eurocode design (it consumes the results).

# Scope

### Included
- **Structural model ownership:** nodes, members (Euler-Bernoulli frame with axial/shear/moment end
  releases and rigid end offsets), truss bars, linear translational/rotational springs, elastic catenary
  cables, sections and materials (library plus custom E, A, I), supports (fixed, pinned, inclined
  roller, guided, elastic kx/ky/kθ, compression-only), loads, load cases, and masses.
- **Loads:** nodal forces/moments, member point loads, uniform/trapezoidal loads (local, global,
  projected), automatic self-weight, support settlements, thermal loads (uniform ΔT and gradient).
- **Analyses:**
  - Linear static DSM (live, < 50 ms up to ~500 nodes / 1000 members), with superposition of load cases.
  - Tension-only and compression-only iterations (cables, unilateral supports).
  - 2nd-order P-Δ/P-δ via geometric stiffness per combination, plus linear buckling (α_cr, modes).
  - Exact elastic catenary cables (Irvine) with Newton-Raphson and slack detection.
  - Modal analysis (lumped masses from G + ψ₂Q, consistent optional; modes up to ≥ 90% effective mass).
- **Stability diagnostics:** mechanism detection before solving, animated mechanism mode, degree of
  indeterminacy.
- **Results:** reactions, M/Q/N along members, deformed shape, nodal displacements, member end forces,
  extrema, per-point inspection, envelopes and governing combinations (from combination input).
- **Execution model:** linear results are immediate. Nonlinear jobs run in the background with progress
  and cancellation, and replace the linear results when they converge. Nonlinear models solve each
  combination separately (no superposition).
- **DSM walkthrough data:** step data (local k, T, assembly, boundary conditions, solution, end forces)
  for the Mode 3 walkthrough. Full matrices are shown for small models.

### Excluded
- Actions classification, EN 1990 combination generation, and EN 1998 spectra (owned by
  `cap-eurocode-actions-combinations`; this engine evaluates the combinations it receives).
- Sketching UI (owned by `cap-sketcher-frame-modeler`).
- 3D frames, Timoshenko members, corotational large-displacement analysis, moving loads / influence lines.

### Candidate structural children (not yet split)
- Linear DSM core and model
- Nonlinear: P-Δ, buckling, catenary cables, unilateral elements
- Dynamics: modal analysis and response-spectrum execution

### Verification baseline
- A closed-form reference suite (1e-6 linear, 1e-3 nonlinear) plus offline OpenSeesPy reference CSVs in
  `docs/benchmarks/frame/`, compared by Rust tests.

# Relationships

- Parent: [OmniMechanics Platform](/project.md)
- Shared concern: [Eurocode Actions & Combinations](/capabilities/cap-eurocode-actions-combinations.md)
- Edited by: [Frame Modeler (Mode 2)](/capabilities/cap-sketcher-frame-modeler.md)
- Feeds into: [Structural Mechanics Teaching Engine](/capabilities/cap-structural-mechanics-teaching.md) (classifier, DSM walkthrough)
- Feeds into: [Eurocode RC Design](/capabilities/cap-eurocode-rc-design.md) (internal-force envelopes)
- Persisted by: [Project Persistence](/capabilities/cap-project-persistence.md)

# Notes

Decisions are recorded in the discovery notes (batches 1-4, 2026-10-31). Exact artifacts will live under
`docs/architecture/cap-frame-analysis-engine/` once the spec pipeline runs.
