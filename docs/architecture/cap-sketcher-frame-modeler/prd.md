# Product Requirements Document: Mode 2 Structural Frame Modeler (`cap-sketcher-frame-modeler`)

- **Capability:** [`cap-sketcher-frame-modeler`](../../../.okf/capabilities/cap-sketcher-frame-modeler.md)
- **Parent Capability:** [`cap-geometry-canvas-sketcher`](../../../.okf/capabilities/cap-geometry-canvas-sketcher.md) (roll-up)
- **Root Application PRD:** [`docs/prd.md`](../../prd.md)
- **Cross-Capability Conventions:** [`cross-capability-conventions.md`](../cross-capability-conventions.md)
- **Status:** Specified (2026-11-01)

## 1. Purpose & Product Summary

`cap-sketcher-frame-modeler` delivers **Mode 2: Structural Modeling & Analysis**, an interactive 2D structural drafting and live-analysis workspace inside OmniMechanics. Users sketch arbitrary 2D structures (`FrameMember`, `TrussBar`, `Spring`, `Cable`) on an SVG drafting canvas with precision CAD-style snapping, assign sections, materials, supports, and loads across multiple `LoadCase`s and Eurocode `Combination`s, and observe live Direct Stiffness Method (DSM) reactions, tension-side $M/Q/N$ diagrams, scaled deformed shapes, envelopes, and kinematic mechanism animations on the same canvas. When a sketched structure is eligible for classical analytical derivation, Mode 2 enables a one-click **"Open Step-by-Step in Teaching Studio"** handoff to Mode 3; when ineligible, it displays a clear bilingual (`el`/`en`) explanation while always offering the generic DSM step-by-step walkthrough.

## 2. Goals & Non-Goals

### Goals
1. **Seamless Mode 2 Workspace Entry:** Expose Mode 2 ("Structural Modeling & Analysis" / «Μοντελοποίηση & Ανάλυση Φορέων») in the top application header switcher alongside Mode 1 ("Continuum Workbench") and Mode 3 ("Teaching Studio").
2. **Precision 2D Structural Drafting:** Provide an interactive SVG canvas with pan/zoom, selectable dynamic grid ($0.1\text{ m}, 0.5\text{ m}, 1.0\text{ m}$), priority-ordered snapping (magnetic node $< 15\text{ px}$, member-member intersection with automatic splitting, midpoint, ortho $0^\circ/90^\circ/180^\circ/270^\circ$, angular guides $30^\circ/45^\circ/60^\circ$, grid), live $L/\theta$ dimension tooltips, cursor coordinate readout, and exact keyboard numeric entry ($(x, y)$ or $(L, \theta)$).
3. **Rich Structural Element & Property Authoring:** Support placement and property inspection of `FrameMember` (with per-end axial/shear/moment `EndRelease`, node-level internal hinge expansion, and `RigidOffset`), `TrussBar`, nodal translational/rotational `Spring`, and catenary `Cable` (input via $L_0$, sag ratio, or $T_0$), backed by standard material/section libraries and custom $(E, A, I)$ definitions.
4. **Full Support & Load Palettes:** Expose every support kind (`Fixed`, `Pinned`, inclined `Roller{angle}`, `Guided{angle}`, `Elastic{kx,ky,ktheta}`, and `unilateral` compression-only flag) and every load kind (`NodalLoad`, `MemberPointLoad` at any position without splitting the member, `MemberDistributedLoad` uniform/trapezoidal in `Local`/`Global`/`Projected` frames, `SupportSettlement`, `ThermalLoad`, and `LoadCase` self-weight toggle).
5. **Real-Time Live Analysis Feedback:** Trigger debounced (~$150\text{ ms}$) synchronous linear DSM solves (< $50\text{ ms}$) on every model mutation (`revision` increment), and manage cancellable background `AnalysisJob`s with progress indicators (`analysis://progress`) for nonlinear (`SecondOrderPDelta`, catenary `Cable`, `unilateral` supports), buckling, and modal/seismic runs.
6. **Normative Result & Envelope Inspection:** Provide a target selector for `LoadCase`, `Combination`, and `Envelope` ($\max/\min$), rendering tension-side $M/Q/N$ diagrams (Greek engineering convention), scaled deformed shapes, extrema labels, interactive cross-section hover probes (with governing combination readout), member end-force tables, and animated kinematic mechanisms when unstable.
7. **Bidirectional Mode 3 Integration & Persistence:** Continuously evaluate `MethodEligibility` via the teaching classifier, enable "Open Step-by-Step" handoff to Mode 3 (or explain ineligibility in Greek/English), load Mode 3 presets into Mode 2 as editable sketches, persist/restore via `ProjectDocument`, and export clean SVG/PNG diagrams.

### Non-Goals
- **No Solver Mathematics in the UI:** Mode 2 owns zero stiffness matrix assembly, solver iterations, or Eurocode combination factor generation rules; those belong strictly to `cap-frame-analysis-engine` and `cap-eurocode-actions-combinations`.
- **No Continuum Polygon or CDT Meshing:** Closed 2D continuum boundaries, interior holes, and Constrained Delaunay Triangulation belong to sibling capability `cap-sketcher-continuum-domain`.
- **No 3D Frame Drafting:** Mode 2 is strictly a 2D planar $(x, y)$ structural workspace.
- **No DXF Frame Import or PDF/LaTeX Calculation Reports in this Capability:** Scheduled for subsequent capabilities/slices.

## 3. Actors & User Journeys

| Actor | Primary Needs in Mode 2 |
| --- | --- |
| **Student** | Quickly sketch homework beams, frames, or trusses (or open a Mode 3 preset in Mode 2 to modify its geometry/loads), immediately see how changing a support or hinge alters the tension-side $M/Q/N$ diagram and deformed shape, understand kinematic mechanisms visually when a structure is unstable, and jump to Mode 3 for step-by-step analytical equations. |
| **Engineer / Instructor** | Rapidly model multi-story frames, trusses, continuous beams, or cable-stayed structures using CAD transforms (copy, mirror, array, box select, undo/redo); configure load cases and EN 1990 / EN 1998 combinations; inspect governing envelopes and end-force tables; and export publication-grade SVG/PNG diagrams. |

## 4. Required Workflows

### WF-1: Sketching & Topological Snapping
1. User selects Mode 2 in the top header and picks an element tool (`DrawFrameMember`, `DrawTrussBar`, `DrawSpring`, `DrawCable`).
2. As the cursor moves, the snapping engine evaluates candidates in strict priority order: `NodeSnap` ($< 15\text{ px}$) $\rightarrow$ `IntersectionSnap` $\rightarrow$ `MidpointSnap` $\rightarrow$ `AngularGuideSnap` / `OrthoLock` $\rightarrow$ `GridSnap`.
3. Live floating badges show cursor $(x, y)$ in metres, current member length $L$ and angle $\theta$, and the active snap glyph. At any point, typing numbers focuses the coordinate bar for exact `x, y` or `L, θ` entry.
4. Clicking an interior point of an existing member or crossing another member with intersection snap active automatically splits the intersected member(s) (`TopologicalMemberSplit`), inserting a shared `Node` and preserving section, material, outer end releases, and distributed load continuity.

### WF-2: Editing, Transformations & Property Inspection
1. Using `Select` (single click or marquee box selection: left-to-right window Enclosed vs. right-to-left Crossing), the user selects nodes, elements, supports, or loads.
2. User executes transformations (`Move`, `Copy`, `Mirror` across a 2-point axis, `LinearArray` with count and $(\Delta x, \Delta y)$ spacing, or `Delete`) or opens the right-hand Property Inspector.
3. In the Inspector, the user assigns a library or custom `Section` and `Material`, toggles per-end `EndRelease` (`axial`, `shear`, `moment`), configures `RigidOffset`, or—for `Cable` elements—selects the input mode (`UnstressedLength` $L_0$, `SagRatio`, or `Pretension` $T_0$).
4. Selecting the `PlaceInternalHinge` tool and clicking a node shared by $k \ge 2$ frame members expands to moment releases on $k - 1$ incident member ends (with inspector override for which member retains rotational continuity) and draws a crisp nodal hinge ring.
5. Every edit pushes a `ModelMutationCommand` onto the `UndoRedoStack` (`Ctrl+Z` / `Ctrl+Y`) and increments `StructuralModel.revision`.

### WF-3: Supports, Load Cases & Load Assignment
1. User creates or selects an active `LoadCase` (tagged with an `ActionType` such as `Permanent` $G$, `Imposed` $Q$, `Wind`, `Snow`, `Temperature`, `Seismic`) and toggles automatic self-weight if desired.
2. From the Support Palette, user assigns `Fixed`, `Pinned`, `Roller{angle}`, `Guided{angle}`, or `Elastic{kx,ky,ktheta}` (with optional `unilateral` compression-only toggle) to one or more nodes.
3. From the Load Palette, user places `NodalLoad` $(F_x, F_y, M_z)$, `MemberPointLoad` at arbitrary fractional or metric distance $a \in [0, L]$ along a member without splitting it, `MemberDistributedLoad` $(q_1, q_2, a, b)$ in `Local`, `Global`, or `Projected` coordinates, `SupportSettlement` $(\Delta_x, \Delta_y, \Delta_\theta)$, or `ThermalLoad` $(\Delta T_{\text{uniform}}, \Delta T_{\text{gradient}})$.

### WF-4: Live Analysis, Nonlinear Replacement & Result Exploration
1. Within ~$150\text{ ms}$ of any mutation, `LiveAnalysisCoordinator` calls `frame_solve_linear` and renders support reaction vectors, tension-side $M/Q/N$ diagrams, extrema tags, and the scaled deformed shape.
2. If the model is kinematically unstable (singular stiffness matrix / mechanism detected in `Diagnostics`), the canvas switches to a continuous harmonic animation of the mechanism mode shape, displays the static indeterminacy / mechanism diagnostic banner in Greek/English, and suppresses invalid force diagrams.
3. If the user selects `SecondOrderPDelta`, `LinearBuckling`, `Modal`, `ResponseSpectrum`, `LateralForce`, or if the model contains nonlinear elements (`Cable`, `unilateral` supports), the immediate linear solution is shown as a preview while `frame_start_analysis_job` runs asynchronously. A non-blocking progress pill displays iteration progress from `analysis://progress` events and swaps in the converged `ResultSet` or `Envelope` once complete. Any further user edit before convergence cancels/supersedes the in-flight job.
4. Moving the mouse along any member displays the `CrossSectionHoverProbe` with exact $N(x), Q(x), M(x)$, displacements, and—when viewing an `Envelope`—the governing `Combination` name at that station.

### WF-5: Mode 3 Teaching Handoff & Preset Import
1. Alongside live analysis, Mode 2 queries `teaching_classify_model` for the current `StructuralModel`.
2. The top toolbar displays a live eligibility badge:
   - When at least one classical method (`IsostaticBeamFrame`, `TrussJointsRitter`, `ContinuousBeamClapeyronCross`, `ForceMethod`) is eligible, clicking **"Open Step-by-Step"** transitions to Mode 3 with the selected classical method pre-loaded.
   - When no classical method applies (e.g., models containing `Cable` elements or degree of indeterminacy $> 3$ for force method), a tooltip and info popover explain the exact bilingual reason (`message_el` / `message_en`), while offering **"Open DSM Walkthrough in Mode 3"** (which is always available for any stable structure).
3. Conversely, in Mode 3, clicking **"Edit in Mode 2 Sketcher"** on any teaching preset converts and loads that preset into Mode 2's `StructuralModel` store at `revision: 1` for freeform modification.

## 5. Business Rules & Constraints

- **BR-1 (SI Units & E Conversion):** All canvas coordinates and lengths are in $\text{m}$, forces in $\text{kN}$, moments in $\text{kNm}$, distributed loads in $\text{kN/m}$, temperatures in $^\circ\text{C}$, and translational/rotational springs in $\text{kN/m}$ and $\text{kNm/rad}$. Elastic modulus $E$ is displayed and entered in $\text{GPa}$ in the UI and converted to $\text{kN/m}^2$ ($1\text{ GPa} = 10^6\text{ kN/m}^2$) at the boundary.
- **BR-2 (Sign & Diagram Convention):** Global $x$ is right, global $y$ is up, counter-clockwise rotation is positive. Axial force $N > 0$ in tension. Bending moment $M > 0$ when the local bottom ($-y$) fibre is in tension, and **bending moment diagrams $M(x)$ are always plotted on the tension side of the member** (standard Greek technical university & engineering practice).
- **BR-3 (Revision Monotonicity & Stale Result Rejection):** Every structural mutation increments `StructuralModel.revision`. Any `ResultSet`, `Envelope`, or `AnalysisJob` progress payload whose `model_revision < current_revision` is discarded and never rendered as current.
- **BR-4 (Superposition Policy):** Superposition of `LoadCase` results is permitted only for linear models (`LinearStatic` without `Cable` or `unilateral` supports). When nonlinearity is active, Mode 2 requests per-combination solves and displays a bilingual badge: *"Nonlinear model — combinations solved directly without superposition / Μη γραμμικό μοντέλο — οι συνδυασμοί επιλύονται απευθείας χωρίς επαλληλία"*.
- **BR-5 (Cable Eligibility Rule):** Any `StructuralModel` containing one or more `Cable` elements is unconditionally ineligible for classical hand-calculation step-by-step methods in Mode 3; only the numerical solver results and DSM walkthrough are offered.

## 6. Non-Functional Requirements

- **NFR-1 (Interactive Responsiveness):** Pan, zoom, cursor coordinate readout, and snap candidate resolution must run at $60\text{ fps}$ ($< 8\text{ ms}$ per pointer move in pure TypeScript) for models up to $500$ nodes and $1000$ elements.
- **NFR-2 (Live Solve Latency):** Input mutation debounce is $150\text{ ms}$ (configurable). Linear static DSM round-trip and SVG diagram update must complete in $< 50\text{ ms}$ for models up to $500$ nodes / $1000$ members.
- **NFR-3 (Bilingual Completeness):** Every toolbar label, inspector field, snap badge, diagnostic banner, classifier reason, and error message must provide both Greek (`el`) and English (`en`) translations.
- **NFR-4 (Code Size & Quality):** All TypeScript and Rust files must remain $\le 1000$ lines and HTML templates $\le 500$ lines (`npm run check:limits`). Pure TypeScript geometry, snapping, transformation, and undo/redo store modules must be verified under the Vitest `frontend-integration` harness.

## 7. Acceptance Scenarios Summary

Detailed canonical scenarios (`SC-001` through `SC-014`) with explicit verification surfaces (`backend-boundary`, `frontend-integration`, `end-to-end`) are specified in [`acceptance-scenarios.md`](acceptance-scenarios.md).

## 8. Explicit Assumptions & Artifact Impact

- **Assumptions:** `cap-frame-analysis-engine` exposes synchronous linear DSM solving and event-driven background job execution (`analysis://progress`), and `cap-eurocode-actions-combinations` exposes EN 1990 combination generation and EN 1998 spectrum parameters as defined in [`cross-capability-conventions.md`](../cross-capability-conventions.md).
- **Application Synthesis Impact:** `no impact` on [`docs/prd.md`](../../prd.md) and [`docs/architecture/application-architecture-summary.md`](../application-architecture-summary.md) — both synthesis documents were authored on 2026-10-31 to reflect these exact Mode 2 boundaries, slices, and shared rules.
