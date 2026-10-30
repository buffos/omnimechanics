---
type: capability
title: External CAD Geometry Ingestion & Polyline Healing Engine
description: Parser and geometry cleanup engine for standard CAD formats (2D DXF, SVG; 3D STL, STEP-lite), supporting curve discretization, polyline closure healing, manifold verification, and transfer to the conformal mesher.
tags: [cad, dxf, step, stl, svg, import, polyline-healing, geometry-cleanup]
timestamp: 2026-10-30T21:15:00Z
state: specified
project: /project.md
parent: /capabilities/cap-geometry-meshing.md
---

# Intent

Enable engineers and students to import real-world structural drawings, frame layouts, and solid models from mainstream CAD/BIM tools (AutoCAD, Revit, SolidWorks) into OmniMechanics. The engine automatically heals small drawing gaps, removes duplicate entities, checks manifold watertightness, and passes clean boundary representations to the structural mechanics teaching engine and the BEM-FEM conformal mesher.

# Scope

### Included:
1. **2D Vector CAD Ingestion (DXF & SVG):**
   - **DXF Parsing (AutoCAD R12 to 2018):**
     - Parses `LINE`, `LWPOLYLINE`, `POLYLINE`, `CIRCLE`, and `ARC` entities.
     - Approximate circular arcs and spline curves via chordal tolerance discretization.
     - Layer filtering (allows user to select which CAD layer contains structural boundaries vs annotations/dimensions).
   - **SVG Path Parsing:**
     - Extract path polygons, polylines, and lines for structural cross-sections and frame wireframes.
   - **Automated Polyline Healing:**
     - Gap closure: Snaps non-touching segment endpoints within configurable tolerance ($\epsilon \le 1.0\text{ mm}$).
     - Vertex consolidation: Removes duplicate coincident vertices.
     - Loop identification: Classifies closed outer boundaries vs internal voids/holes using winding numbers.

2. **3D Surface & Solid CAD Ingestion (STL & Triangulated STEP):**
   - **STL Ingestion (Binary & ASCII):**
     - Fast chunk parsing of triangular face arrays.
     - Vertex hashing to reconstruct shared topological edges and nodes from triangle soup.
     - Normal vector recomputation and orientation consistency (outward-pointing normals for BEM).
     - Watertightness check: Verifies that every edge is shared by exactly two triangles (2-manifold mesh).

3. **Handoff Pipeline:**
   - Healed 2D frame lines $\rightarrow$ Structural Mechanics Teaching Engine (`cap-structural-mechanics-teaching`).
   - Healed 2D closed contours $\rightarrow$ Constrained Delaunay Triangulator for FEM volume & BEM boundary meshing.
   - Healed 3D watertight STL $\rightarrow$ 3D BEM surface patches & Tet4 interior volume generation.

### Excluded:
- Full parametric CAD feature tree regeneration (e.g. historical extrude/fillet steps).
- Multi-component kinetic assemblies.

---

# Architecture & Domain Model

```rust
// CAD Import Pipeline Contracts
pub struct CadImportResult {
    pub file_name: String,
    pub format: CadFormat, // DXF, SVG, STL
    pub layers: Vec<CadLayerSummary>,
    pub frame_members: Vec<ImportedFrameSegment>,
    pub closed_loops: Vec<HealedContour2D>,
    pub surface_3d: Option<HealedSurface3D>,
    pub warnings: Vec<String>,
}

pub struct HealedContour2D {
    pub loop_id: usize,
    pub is_outer_boundary: bool,
    pub vertices: Vec<[f64; 2]>,
    pub perimeter: f64,
    pub area: f64,
}

pub struct HealedSurface3D {
    pub vertices: Vec<[f64; 3]>,
    pub triangles: Vec<[usize; 3]>,
    pub is_watertight: bool,
    pub volume: f64,
}
```

---

# Acceptance Scenarios

### Scenario 1: Importing 2D DXF Floor Beam Layout
- **Given** an architectural DXF file containing centerlines on layer "BEAMS" and text on layer "DIMENSIONS".
- **When** the user imports the DXF and selects layer "BEAMS".
- **Then**:
  1. The engine filters out annotation text and dimensions.
  2. Parses line entities and chains collinear segments.
  3. Detects joint intersections and exports a clean structural frame directly to Teaching Mode.

### Scenario 2: Polyline Healing with Micro-Gaps
- **Given** a 2D DXF with a rectangle where one corner has a $0.2\text{ mm}$ gap due to loose drafting.
- **When** the healing engine runs with tolerance $\epsilon = 0.5\text{ mm}$.
- **Then**:
  1. The endpoints are snapped to a single vertex.
  2. The contour is marked as `is_outer_boundary: true`.
  3. Passed to Delaunay mesher without boundary leak errors.

### Scenario 3: Watertight 3D STL Ingestion
- **Given** a binary STL model of a cantilever bracket.
- **When** the user imports the STL into OmniMechanics.
- **Then**:
  1. Engine merges shared vertices, verifies 2-manifold topological closure.
  2. Orientates face normals consistently outward.
  3. Displays the bracket on the Three.js canvas ready for 3D BEM/FEM static analysis.

---

# Relationships

- Parent: [Geometry Definition & Conformal Meshing](/capabilities/cap-geometry-meshing.md)
- Feeds into: [Interactive 2D/3D Canvas Drafting](/capabilities/cap-geometry-canvas-sketcher.md)
- Feeds into: [Parametric Geometry Presets & Conformal Meshing](/capabilities/cap-geometry-presets.md)
- Feeds into: [Structural Mechanics Step-by-Step Educational Engine](/capabilities/cap-structural-mechanics-teaching.md)
