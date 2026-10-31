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
| ↳ `cap-geometry-cad-import` | Complete | Pending Implementation | Specified | `specified` |
| ↳ `cap-geometry-canvas-sketcher` | Complete | Pending Implementation | Specified | `specified` |
| `cap-material-field` | Complete | Verified (5 Open-Closed FGM strategies) | Approved | `implemented` |
| `cap-solver-core` | Complete | Verified (Static, Dynamic GMRES, Talbot, Newmark) | Approved | `implemented` |
| `cap-interactive-canvas` | Ready | In Progress (Viewport, Modal Dialogs active) | Visual Review | `bounded` |
| `cap-parametric-benchmarking` | Complete | Verified (Mini-Papers 1-5, Issues 008-013) | Approved | `implemented` |
| `cap-project-persistence` | Bounded | Blocked on CAD / canvas state | Pending | `bounded` |
| `cap-structural-mechanics-teaching` | Complete | Slice 1 Verified (Issue 014 closed) | Approved | `in-progress` |
| `cap-eurocode-rc-design` | Complete | Ready for Slicing | Specified | `specified` |

---

## Active Delivery Frontier & Next Transitions

### 1. Mode 2 Frontier: Interactive Canvas Sketcher (`cap-geometry-canvas-sketcher`) [RECOMMENDED NEXT STEP]
- **Capability Owner:** `cap-geometry-canvas-sketcher` (Parent: `cap-geometry-meshing`)
- **Status:** Specified and ready for issue generation / implementation.
- **Scope:** Interactive 2D drawing studio directly in the viewport:
  - Click-and-drag drawing of 1D members (beams, columns, truss bars) with orthogonal lock and magnetic snapping.
  - Support palette (fixed, pinned, roller, internal hinges).
  - Load palette (point loads, distributed loads, concentrated moments).
  - Seamless handoff to Mode 3 Teaching Engine (solving user-drawn custom frames) and Mode 1 continuum mesher.
- **Why Now:** Solves the key user requirement: *"πώς θα γίνεται η εισαγωγή φορέα; Δεν έχουμε κάνει ακόμη τον Interactive Canvas Sketcher... Θα γίνει μετά λογικά."*

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
