# OmniMechanics: Application Product Requirements (Synthesis)

- Status: **draft for user review** (application synthesis gate, 2026-10-31)
- Planning map: [`.okf/project.md`](../.okf/project.md), [`.okf/index.md`](../.okf/index.md)
- Architecture summary: [application-architecture-summary.md](architecture/application-architecture-summary.md)
- This document summarises and links to capability nodes. The nodes and their `docs/architecture/<slug>/`
  artifacts remain the detailed sources of truth.

## 1. Product

OmniMechanics is a desktop (Tauri + Angular + Rust) engineering and teaching environment with three
workspaces:

| Mode | Name | Purpose |
| --- | --- | --- |
| 1 | Continuum Workbench | BEM, FEM, and hybrid BEM-FEM solvers for inhomogeneous 2D/3D elastodynamics (static, harmonic, transient) on parametric presets, with verification plots |
| 2 | Structural Modeling & Analysis | Sketch **any 2D structure** (frame members, truss bars, springs, catenary cables), assign supports and loads, and see live analysis results (linear, P-Δ, buckling, modal/seismic) with Eurocode combinations |
| 3 | Teaching Studio | Step-by-step classical structural analysis (isostatic beams/frames, trusses, Clapeyron/Cross, force method) and a DSM walkthrough, with M/Q/N diagrams and exact fractions |

## 2. Actors

- **Student:** learns classical methods and expects step-by-step derivations in Greek or English.
- **Engineer / instructor:** models arbitrary structures, checks combinations and envelopes, and
  prepares teaching material.
- **Researcher:** runs the continuum solver benchmarks (mini-papers 1-5).

## 3. Primary user journeys

1. **Benchmark a continuum problem (Mode 1):** pick a preset, vary the material field, solve, and
   inspect the response in the bottom dock. *Implemented.*
2. **Learn from a preset (Mode 3):** pick a teaching preset, step through the derivation, and watch the
   M/Q/N diagrams build. *Slice 1 implemented (issue 014).*
3. **Model and analyse any frame (Mode 2):**
   - The user draws members with snapping and assigns sections, supports, and loads.
   - Linear results update live (< 50 ms). Nonlinear results (P-Δ, cables) replace them in the
     background when they converge.
   - The user switches between load case, combination, and envelope views. *Target: new.*
4. **From sketch to step-by-step (Mode 2 → Mode 3):**
   - The classifier enables "Open step-by-step" when a classical method applies, or explains why it does not.
   - The DSM walkthrough is always available. *Target: new.*
5. **Seismic check of a 2D frame (Mode 2):** set the EN 1998 parameters (Greek NA), run modal response
   spectrum or lateral force analysis, and review the envelopes. *Target: new.*
6. **Design from envelopes:** RC design consumes M_Ed, V_Ed, and N_Ed envelopes. *Specified, not
   started.*

## 4. Capability map and status

| Capability | State | Role |
| --- | --- | --- |
| [cap-solver-core](../.okf/capabilities/cap-solver-core.md) | implemented | Mode 1 BEM/FEM/hybrid solvers |
| [cap-material-field](../.okf/capabilities/cap-material-field.md) | implemented | Inhomogeneity engine |
| [cap-parametric-benchmarking](../.okf/capabilities/cap-parametric-benchmarking.md) | implemented | Mini-paper benchmarks |
| [cap-geometry-meshing](../.okf/capabilities/cap-geometry-meshing.md) | roll-up: specified | Geometry umbrella |
| ↳ [cap-geometry-presets](../.okf/capabilities/cap-geometry-presets.md) | implemented | Parametric presets + conformal meshing |
| ↳ [cap-geometry-cad-import](../.okf/capabilities/cap-geometry-cad-import.md) | specified | DXF/SVG/STL ingestion |
| ↳ [cap-geometry-canvas-sketcher](../.okf/capabilities/cap-geometry-canvas-sketcher.md) | roll-up: specified | Sketching umbrella |
| ↳↳ [cap-sketcher-frame-modeler](../.okf/capabilities/cap-sketcher-frame-modeler.md) | specified | **Mode 2 workspace UI** ([PRD](architecture/cap-sketcher-frame-modeler/prd.md)) |
| ↳↳ [cap-sketcher-continuum-domain](../.okf/capabilities/cap-sketcher-continuum-domain.md) | specified | Polygon/hole sketching → CDT mesh ([PRD](architecture/cap-sketcher-continuum-domain/prd.md)) |
| [cap-frame-analysis-engine](../.okf/capabilities/cap-frame-analysis-engine.md) | specified | **2D DSM solver, structural model owner** ([PRD](architecture/cap-frame-analysis-engine/prd.md)) |
| [cap-eurocode-actions-combinations](../.okf/capabilities/cap-eurocode-actions-combinations.md) | specified | **EN 1990 combinations, EN 1998 seismic (shared)** ([PRD](architecture/cap-eurocode-actions-combinations/prd.md)) |
| [cap-structural-mechanics-teaching](../.okf/capabilities/cap-structural-mechanics-teaching.md) | specified | Mode 3 + classifier + DSM walkthrough |
| [cap-eurocode-rc-design](../.okf/capabilities/cap-eurocode-rc-design.md) | specified | EC2 RC design |
| [cap-interactive-canvas](../.okf/capabilities/cap-interactive-canvas.md) | specified | Shared canvas/UI ([PRD](architecture/cap-interactive-canvas/prd.md)) |
| [cap-project-persistence](../.okf/capabilities/cap-project-persistence.md) | specified | JSON project save/load ([PRD](architecture/cap-project-persistence/prd.md)) |

## 5. MVP for the current frontier (Mode 2)

The delivery order is user-confirmed (discovery Q30):

1. **Slice 1:** Mode 2 modeler with live linear DSM (frame members, truss bars, springs, all support
   types, a single load case), plus M/Q/N, deformed shape, reactions, and mechanism diagnostics.
2. **Slice 2:** load cases, EN 1990 combinations, envelopes, self-weight.
3. **Slice 3:** classifier, "Open step-by-step", DSM walkthrough.
4. **Slice 4:** P-Δ and linear buckling.
5. **Slice 5:** elastic catenary cables, tension/compression-only elements.
6. **Slice 6:** modal analysis, EN 1998 lateral force method and response spectrum.

Then: continuum domain sketcher, RC design, persistence, CAD import.

## 6. Shared business rules

- **Units and signs:** SI (kN, m, kNm, kN/m, E in GPa), y axis up. M is positive when the bottom fibre
  is in tension, and diagrams are drawn on the tension side.
- **Language:** bilingual el/en in UI, messages, and step texts.
- **Superposition:** valid only for linear models. Nonlinear models solve each combination separately,
  and the UI states this.
- **Eligibility for classical step-by-step:** decided by the classifier. Models with cables are never
  eligible. The DSM walkthrough is always offered.
- **Normative defaults:** Greek National Annex, all values editable.

## 7. Non-goals (current planning horizon)

- 3D frame analysis, Timoshenko members, corotational large-displacement analysis.
- Moving loads / influence lines, accidental combinations.
- Plastic collapse analysis, full 3D building EC8 design.
- CSG solid modeling, NURBS, parametric constraint solvers.
- PDF/LaTeX reports and DXF frame import (later slices).

## 8. Verification strategy

The root policy is `required` (see [project.md](../.okf/project.md)):

- **Backend-boundary (required):** Rust unit tests in companion `*_test.rs` files, a closed-form
  reference suite, and offline OpenSeesPy reference CSVs in `docs/benchmarks/frame/`.
- **Frontend-integration (required):** a new Vitest harness for pure TS logic (snapping, geometry, model
  store, undo). This harness must exist before the first Mode 2 slice closes.
- **End-to-end (catalog-only):** visual review gate on every UI slice.
- Code limits: Rust/TS ≤ 1000 lines, HTML ≤ 500 lines (`npm run check:limits`).

## 9. Open items

| Severity | Item | Owner / route |
| --- | --- | --- |
| Low | `cap-structural-mechanics-teaching` classifier extension can be extracted to a standalone node PRD before Slice 3 | Refresh before Slice 3 |
| Low | Vitest harness not yet installed | Part of Mode 2 Slice 1 implementation scope |
| Low | OpenSeesPy not installed for reference generation | Part of Mode 2 Slice 1 benchmark generation |

All High and Medium blockers are resolved. All 15 capability nodes in `.okf/` are now `specified` (11) or `implemented` (4), and the project is ready for issue slicing.
