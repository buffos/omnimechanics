---
type: capability
title: Interactive 2D/3D Canvas Drafting & Custom Geometry Sketcher
description: Interactive visual drafting studio inside the application viewport enabling engineers and students to sketch custom structural frames, draw arbitrary polygonal continuum domains, insert cutouts, and assign boundary conditions directly on custom geometry.
tags: [sketcher, cad, drafting, drawing, interactive, 2d, 3d, viewport, boundary-tagging]
timestamp: 2026-10-30T21:15:00Z
state: specified
project: /project.md
parent: /capabilities/cap-geometry-meshing.md
---

# Intent

Provide a fluid, interactive sketching environment directly within the application viewport (Mode 2), allowing users to draw custom 2D/3D structural frames, trusses, and continuous domains with real-time grid snapping, orthogonal constraints, and interactive boundary/load assignment. The sketched entities feed seamlessly into both the step-by-step structural mechanics engine (Mode 3) and the continuum BEM-FEM mesher.

# Scope

### Included:
1. **Interactive Drafting Tools:**
   - **Segment & Member Sketcher:** Click-and-drag drawing of 1D beam and truss members with live dynamic dimensions ($L$, $\theta$).
   - **Polygon Boundary Tool:** Closed boundary sketching with automatic loop detection and self-intersection prevention.
   - **Hole & Cutout Placement:** Circular, rectangular, and polygonal void insertion within existing continuous domains.
   - **Snapping Engine:**
     - Grid snapping (configurable $0.1\text{ m}, 0.5\text{ m}, 1.0\text{ m}$).
     - Magnetic node-to-node snapping ($< 15\text{ px}$ radius).
     - Midpoint snapping on existing members.
     - Orthogonal (Ortho) lock mode ($0^\circ, 90^\circ, 180^\circ, 270^\circ$).
     - Angular guide snapping ($30^\circ, 45^\circ, 60^\circ$).

2. **Interactive Boundary & Load Assignment:**
   - **Support Palette:** Direct click-to-apply support icons onto nodes:
     - Fixed Support (Πάκτωση) - 3 DOFs constrained ($u_x = 0, u_y = 0, \theta_z = 0$).
     - Pinned Support (Άρθρωση) - 2 DOFs constrained ($u_x = 0, u_y = 0$).
     - Roller Support (Κύλιση) - 1 DOF constrained ($u_y = 0$ or along inclined normal).
     - Internal Hinge (Εσωτερική Άρθρωση) - Moment release $M = 0$.
   - **Load Palette:** Direct click-and-drag load glyphs:
     - Point loads: Vector magnitude ($F_x, F_y$) and angle.
     - Concentrated moments: Point torque ($M_z$).
     - Distributed line loads: Uniform or trapezoidal ($q_1, q_2$).

3. **Discretization & Solver Handoff:**
   - **1D Structural Frame Handoff:** Direct export of nodes, members, loads, and releases into the Step-by-Step Educational Engine (`cap-structural-mechanics-teaching`).
   - **2D/3D Continuum Mesh Handoff:** Constrained Delaunay Triangulation (CDT) converting closed sketch boundaries into conformal Tri3/Quad4 interior elements and Line2/Line3 BEM boundary patches.

### Excluded:
- Freeform 3D NURBS surface lofting or parametric constraint graph solvers (e.g. geometric dimensioning & tolerancing).

---

# Architecture & Domain Model

```rust
// Unified Sketch Data Structure
pub struct SketchModel {
    pub nodes: Vec<SketchNode>,
    pub members: Vec<SketchMember>,
    pub boundaries: Vec<SketchBoundaryLoop>,
    pub loads: Vec<SketchLoad>,
}

pub struct SketchNode {
    pub id: String,
    pub point: [f64; 3], // x, y, z
    pub support: SupportType, // Free, Pinned, RollerX, RollerY, Fixed, Spring(f64)
}

pub struct SketchMember {
    pub id: String,
    pub start_node_id: String,
    pub end_node_id: String,
    pub start_release: bool, // Internal hinge
    pub end_release: bool,
    pub section: MemberSectionProperties, // E, A, I
}

pub struct SketchBoundaryLoop {
    pub id: String,
    pub vertices: Vec<[f64; 2]>,
    pub is_hole: bool,
    pub target_element_size: f64,
}

pub enum SketchLoad {
    NodalForce { node_id: String, fx: f64, fy: f64 },
    NodalMoment { node_id: String, mz: f64 },
    DistributedLineLoad { member_id: String, q_start: f64, q_end: f64, perpendicular: bool },
}
```

---

# Acceptance Scenarios

### Scenario 1: Sketching a Portal Frame with Ortho Snapping
- **Given** the user selects the "Frame Member" sketch tool.
- **When** clicking $(0, 0)$, dragging upward $4\text{ m}$ with Ortho locked, then rightward $6\text{ m}$, then downward $4\text{ m}$.
- **Then**:
  1. The viewport draws 3 connected members and 4 nodes with live dimension callouts.
  2. Magnetic snapping closes corners cleanly without duplicate nodes.

### Scenario 2: Assigning Supports and Concentrated Loads
- **Given** the 3-member portal frame.
- **When** the user clicks "Fixed Support" and selects the two bottom nodes, then clicks "Distributed Load" and drags across the top beam ($q = 15\text{ kN/m}$).
- **Then**:
  1. Clamped support glyphs are rendered at the column bases.
  2. A distributed load array with down-arrows and "$15\text{ kN/m}$" text is rendered along the top beam.
  3. The model is immediately valid for handoff to Teaching Mode.

### Scenario 3: Closed Polygonal Plate with Hole
- **Given** the user selects the "Continuum Boundary" tool.
- **When** drawing a $10 \times 10\text{ m}$ outer square and inserting a circle cutout ($R=1.5\text{ m}$) at center.
- **Then**:
  1. Automatic loop closure validates the closed polygon.
  2. Clicking "Generate Conformal Mesh" produces a graded Tri3 Delaunay mesh conforming to BEM line boundary elements.

---

# Relationships

- Parent: [Geometry Definition & Conformal Meshing](/capabilities/cap-geometry-meshing.md)
- Feeds directly into: [Structural Mechanics Step-by-Step Educational Engine](/capabilities/cap-structural-mechanics-teaching.md)
- Feeds into: [Parametric Geometry Presets & Conformal Meshing Engine](/capabilities/cap-geometry-presets.md)
- Rendered on: [Interactive 3D Canvas & UI](/capabilities/cap-interactive-canvas.md)
