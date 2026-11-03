# BemApp Delivery Roadmap (Fog-of-War)

Derived navigation roadmap based on Open Knowledge Format (`.okf/`) capability graph and verified repository delivery evidence.

## Tri-Mode Application Architecture

1. **Benchmark & Verification Workbench Mode (Ready-made Structures):**
   - Parametric presets (Cantilevers, plates with holes, 3D beams).
   - Fast comparison of continuum formulations (Pure FEM, Pure BEM).
   - Time-domain, frequency-domain, and static solutions with material field variations.
2. **Custom Modeling & CAD Drafting Studio Mode:**
   - User-drawn 2D/3D frames, custom cross-sections, and boundaries.
   - External CAD file import (DXF, SVG, STEP, STL).
   - Interactive boundary condition and load tagging directly on custom geometries.
3. **Structural Mechanics & Eurocode Teaching Studio Mode:**
   - Step-by-Step educational analysis for 2D isostatic and hyperstatic frames, continuous beams, and trusses.
   - Complete analytical derivations, equilibrium cuts, internal force diagrams ($M, Q, N$).
   - Classical hand-calculation algorithms (Ritter, Joints, Force Method, Clapeyron 3-Moment, Slope-Deflection, Cross, Kani, Virtual Work, Castigliano).
   - Eurocode 2 reinforced concrete design (beams flexure/shear, columns $N-M$, SLS checks, rebar detailing).

---

## Capability Matrix

| Capability Node | Specification | Implementation | Review Gate | Overall State |
|---|---|---|---|---|
| `cap-geometry-meshing` | Complete (Rollup) | Partial (Children) | In Progress | `specified` (rollup) |
| ↳ `cap-geometry-presets` | Complete | Verified (2D/3D conformal tessellation) | Approved | `implemented` |
| ↳ `cap-geometry-cad-import` | Complete | Pending Implementation (Issues 036-037) | Specified | `specified` |
| ↳ `cap-geometry-canvas-sketcher` | Complete (Rollup) | Verified (Children 023-029 complete) | Approved | `implemented` (rollup) |
| ↳↳ `cap-sketcher-frame-modeler` | Complete (8-file spec set) | Verified (Issues 023-026) | Approved | `implemented` |
| ↳↳ `cap-sketcher-continuum-domain` | Complete (8-file spec set) | Verified (Issues 027-029) | Approved | `implemented` |
| `cap-material-field` | Complete | Verified (5 Open-Closed FGM strategies) | Approved | `implemented` |
| `cap-solver-core` | Complete | Verified (Static, Dynamic GMRES, Talbot, Newmark) | Approved | `implemented` |
| `cap-interactive-canvas` | Complete (8-file spec set) | Verified (Issues 001, 030-031) | Approved | `implemented` |
| `cap-parametric-benchmarking` | Complete | Verified (Mini-Papers 1-5, Issues 008-013) | Approved | `implemented` |
| `cap-project-persistence` | Complete (8-file spec set) | Pending Implementation (Issues 032-033) | Specified | `specified` |
| `cap-structural-mechanics-teaching` | Complete | Slice 1 Verified (014), Pending (034-035) | Approved | `specified` |
| `cap-eurocode-rc-design` | Complete | Pending Implementation (Issues 038-039) | Specified | `specified` |
| `cap-frame-analysis-engine` | Complete (8-file spec set) | Verified (Issues 016-020) | Approved | `implemented` |
| `cap-eurocode-actions-combinations` | Complete (8-file spec set) | Verified (Issues 021-022) | Approved | `implemented` |

---

## Active Delivery Frontier & Next Transitions

### 1. Mode 2 Frontier: Structural Modeling & Analysis [RECOMMENDED NEXT STEP]
- **Owners:** `cap-frame-analysis-engine` (DSM solver, structural model) and `cap-sketcher-frame-modeler` (Mode 2 UI).
- **Status:** `specified` (all 8 canonical architecture artifacts and readiness reviews completed with 0 High/Medium blockers). Ready for issue slicing (`$reference-docs-to-issues`).
- **Decisions:** `docs/architecture/cap-geometry-canvas-sketcher/discovery-notes.md`.
- **Slices:**
  1. Live linear DSM modeler
  2. EN 1990 cases/combinations/envelopes
  3. Classifier + step-by-step + DSM walkthrough
  4. P-Δ + buckling
  5. Catenary cables
  6. Modal + EN 1998
- **Why now:** the user needs to enter any 2D structure (beams, trusses, springs, cables) and see
  solutions live, with step-by-step derivations when possible.
- Issue 015 (sketcher → teaching handoff only) was superseded on 2026-10-31.

### 2. Mode 3 Frontier: Advanced Analytical Structural Mechanics & Eurocode
- **Alternative A (`cap-structural-mechanics-teaching` Slice 2):**
  - Force Method (Μέθοδος Δυνάμεων) with primary isostatic system & Vereshchagin/Mohr integration $\delta_{ik}$.
  - Principle of Virtual Work (Αρχή Δυνατών Έργων - ΑΔΕ) for deflection calculations.
  - Clapeyron 3-moment equation.
- **Alternative B (`cap-eurocode-rc-design` Slice 1):**
  - Eurocode 2 (EN 1992-1-1) cross-sectional concrete design:
  - Bending ULS ($M_{Ed} \to A_s$), shear ($V_{Ed} \to \rho_w$, stirrups), interaction diagrams ($N-M$), and detailing drawings.

---

## Excluded & Deferred Scope
- **Global Radiation Damping on Free Surfaces:** Radiation damping $\mathbf{Z}(s)$ is explicitly excluded from natural traction-free boundaries ($\mathbf{t}=\mathbf{0}$) of bodies in air/vacuum. It is deferred strictly to the designated radiating boundary selection inside the soil/medium modal dialog.
- **VR / AR Viewport:** Deferred beyond v1.0.
