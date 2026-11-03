## Issue Metadata

- Issue number: `015`
- Owning capability node: `/.okf/capabilities/cap-geometry-canvas-sketcher.md` (now a roll-up)
- Artifact root: `docs/architecture/cap-geometry-canvas-sketcher/`
- Issue file: `docs/agents/issues/done/20261031-015-interactive-canvas-sketcher.md`
- Category: `feature`
- Execution type: `AFK`
- Review gate: `visual-review`
- State: `superseded` (not implemented)

## Superseded Disposition (2026-10-31)

This issue was **not implemented**. The `$process-reference-issue` pre-implementation gate found
material gaps (recorded in `docs/architecture/cap-geometry-canvas-sketcher/requirements-gap-analysis.md`):

- The application PRD and architecture summary were missing.
- The teaching engine has no general frame solver, so the issue's own Portal Frame preset could not be analyzed.
- The OKF node and this issue had divergent sketch domain models.
- There were no scenarios for invalid topology and handoff failures, and no frontend test harness under the `required` policy.

The user then ran fog-of-war planning (40 decisions in
`docs/architecture/cap-geometry-canvas-sketcher/discovery-notes.md`), which set the target as "any 2D
structure (beams, trusses, springs, cables) with live solutions, and step-by-step when possible".
Replacement work:

- `cap-frame-analysis-engine` (new): DSM solver and structural model owner.
- `cap-eurocode-actions-combinations` (new): EN 1990/1998.
- `cap-sketcher-frame-modeler` (new child): Mode 2 workspace.

Replacement issues will be sliced after these nodes pass the architecture spec pipeline. None of the
acceptance criteria below were delivered.

## Original Parent Artifacts

- `.okf/project.md`
- `.okf/capabilities/cap-geometry-meshing.md`
- `.okf/capabilities/cap-geometry-canvas-sketcher.md`
- `.okf/capabilities/cap-structural-mechanics-teaching.md`
- `docs/roadmap.md`

## What to build

Implement **Vertical Slice 1** of Mode 2 (Interactive 2D Canvas Drafting & Custom Geometry Sketcher), enabling engineers and students to sketch custom 2D frames, trusses, and beams directly on an interactive canvas, assign supports and loads visually, and immediately analyze them in Mode 3 (Teaching Studio) or mesh them in Mode 1:

### 1. Unified Sketch Domain Model (`src-tauri/src/sketcher/`):
- **Data Structures (`types.rs`):**
  - `SketchNode`: unique ID, coordinates $(x, y)$, support condition (`Free`, `Pinned`, `RollerX`, `RollerY`, `Fixed`).
  - `SketchMember`: unique ID, `start_node_id`, `end_node_id`, releases (`start_hinge`, `end_hinge`), cross-section properties $(E, A, I)$.
  - `SketchLoad`:
    - `PointLoad`: target node or member with distance $x$, vector magnitude $(P_x, P_y)$.
    - `DistributedLoad`: target member, start and end intensities $(q_1, q_2)$, direction.
    - `PointMoment`: target node, torque $M_z$.
  - `SketchModel`: list of nodes, members, supports, and loads with JSON serialization.
- **Conversion & Handoff Engine (`handoff.rs`):**
  - Converter from `SketchModel` to 1D structural system compatible with `src-tauri/src/teaching/`:
    - Topological connectivity validation (detect orphan nodes, isolated members).
    - Degree of indeterminacy evaluation.
    - Dispatch to analytical reaction and internal force solvers.
- **Tauri IPC Command:**
  - `solve_custom_sketch(sketch: SketchModel, lang: Option<String>) -> Result<TeachingSolutionDto, String>`.

### 2. Angular 22 Interactive Drafting Studio (`src/app/features/sketcher/`):
- **Header Workspace Mode Integration:**
  - Add Mode 2 to top navigation:
    - Mode 1: *Continuum Workbench*
    - Mode 2: *Custom CAD & Sketcher Studio*
    - Mode 3: *Educational Studio*
- **Interactive 2D Canvas Component (`sketch-canvas.component`):**
  - SVG/Canvas viewport with pan, zoom, coordinate origin marker, and dynamic grid ($0.1\text{m}, 0.5\text{m}, 1.0\text{m}$).
  - Live cursor coordinate readout $(x, y\text{ in meters})$.
  - **Snapping Engine:**
    - Magnetic node-to-node snapping ($< 15\text{px}$ radius).
    - Grid snapping lock.
    - Orthogonal mode lock (`Ortho` toggle for exact $0^\circ, 90^\circ, 180^\circ, 270^\circ$).
  - **Interactive Drawing Gestures:**
    - Click-and-drag or click-click member placement with real-time length $L$ and angle $\theta$ tooltips.
    - Visual member lines with start/end node handles.
- **Tools & Inspection Palettes (`sketch-toolbar.component` & `sketch-inspector.component`):**
  - **Tool Modes:**
    - `Select / Move / Delete`: Pick elements, edit coordinates, delete selected.
    - `Member (Beam / Truss)`: Draw continuous or individual members.
    - `Supports`: Click node to cycle or assign Fixed (Πάκτωση), Pinned (Άρθρωση), Roller (Κύλιση), or Internal Hinge.
    - `Loads`: Click node/member to apply point load $P$, distributed load $q$, or moment $M$ with numeric prompt dialog.
  - **Action Controls:**
    - `Clear Sketch` / `Reset Canvas`.
    - `Sample Presets` (Quick-load Portal Frame, Overhanging Continuous Beam, or Triangle Truss for instant testing).
    - `⚡ Analyze in Teaching Studio`: Transmits the sketched structure to Mode 3 for step-by-step resolution, formulas, and $M, Q, N$ diagrams.

## Acceptance criteria

- [ ] Rust sketch data model (`SketchModel`, `SketchNode`, `SketchMember`, `SketchLoad`) implemented with unit tests in separate companion `*_test.rs` files.
- [ ] Tauri IPC command `solve_custom_sketch` registered and functional.
- [ ] Mode 2 button added to top header with smooth workspace mode switching.
- [ ] Interactive 2D canvas allows drawing members with orthogonal lock and magnetic snap ($< 15\text{px}$).
- [ ] Support palette allows assigning Fixed, Pinned, Roller, and Hinge conditions to nodes.
- [ ] Load palette allows placing Point Loads $P$, Distributed Loads $q$, and Moments $M$.
- [ ] `Analyze in Teaching Studio` transfers the custom structure into Mode 3 with instant analytical solution and step-by-step derivations.
- [ ] All code files obey limits: Rust/TS $\le 1000$ lines, HTML $\le 500$ lines.
- [ ] `npm run check:limits`, `cargo test`, and `npx ng build` pass with 0 errors.

## Artifact sync required

- Application PRD: `none - initial Mode 2 custom sketcher slice`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-geometry-canvas-sketcher.md`
- Issue registry: `required: docs/agents/issues/issues.md`
- Reason/no-impact decision: `Delivers Vertical Slice 1 of Mode 2 Interactive Canvas Sketcher & Custom Frame Input.`
