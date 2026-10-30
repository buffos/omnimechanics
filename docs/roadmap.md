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
| `cap-geometry-meshing` | Complete (Rollup) | Partial (Children) | In Progress | `foggy` (rollup) |
| ↳ `cap-geometry-presets` | Complete | Verified (2D/3D conformal tessellation) | Approved | `implemented` |
| ↳ `cap-geometry-cad-import` | Draft Intent | Not Started | Needs Discovery | `foggy` |
| ↳ `cap-geometry-canvas-sketcher` | Draft Intent | Not Started | Needs Discovery | `foggy` |
| `cap-material-field` | Complete | Verified (5 Open-Closed FGM strategies) | Approved | `implemented` |
| `cap-solver-core` | Complete | Verified (Static, Dynamic GMRES, Talbot, Newmark) | Approved | `implemented` |
| `cap-interactive-canvas` | Ready | In Progress (Viewport, Modal Dialogs active) | Visual Review | `bounded` |
| `cap-parametric-benchmarking` | Bounded | Ready for implementation | Pending | `bounded` |
| `cap-project-persistence` | Bounded | Blocked on CAD / canvas state | Pending | `bounded` |
| `cap-structural-mechanics-teaching` | Draft Intent | Not Started | Needs Discovery | `foggy` |
| `cap-eurocode-rc-design` | Draft Intent | Not Started | Needs Discovery | `foggy` |

---

## Active Delivery Frontier & Next Transitions

### 1. Mode 1 Frontier: Interactive Response History & Benchmarking Curves (`cap-parametric-benchmarking`)
- **Capability Owner:** `cap-parametric-benchmarking` & `cap-interactive-canvas`
- **Focus:** Collapsible Bottom Engineering Dock with dynamic response history plotting ($u_{\text{tip}}(t)$), comparing Talbot contour vs Newmark-$\beta$.

### 2. Mode 2 Frontier: Canvas Sketcher & CAD Ingestion (`cap-geometry-meshing`)
- **Capability Owner:** `cap-geometry-canvas-sketcher` & `cap-geometry-cad-import`
- **Transition:** Clear fog on interactive 2D drafting canvas (lines, polygons, holes, snapping) and DXF polyline ingestion into conformal meshes.

### 3. Mode 3 Frontier: Structural Mechanics & Eurocode Teaching Engine (`cap-structural-mechanics-teaching`)
- **Capability Owner:** `cap-structural-mechanics-teaching` & `cap-eurocode-rc-design`
- **Transition:** Clear fog on 2D frame analytical step-by-step solver:
  - Isostatic: Method of Sections, Method of Joints, Beam analytical equations ($M, Q, N$).
  - Hyperstatic: Force Method, 3-Moment Clapeyron, Slope-Deflection, Cross, Kani.
  - Energy: Principle of Virtual Work (ΑΔΕ), Castigliano.
  - Module 3B: Eurocode 2 (EN 1992-1-1) reinforced concrete beam/column cross-section dimensioning & rebar detailing.

---

## Excluded & Deferred Scope
- **Global Radiation Damping on Free Surfaces:** Radiation damping $\mathbf{Z}(s)$ is explicitly excluded from natural traction-free boundaries ($\mathbf{t}=\mathbf{0}$) of bodies in air/vacuum. It is deferred strictly to the designated radiating boundary selection inside the soil/medium modal dialog.
- **VR / AR Viewport:** Deferred beyond v1.0.
