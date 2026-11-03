# Domain Glossary: Mode 2 Structural Frame Modeler (`cap-sketcher-frame-modeler`)

- **Capability:** [`cap-sketcher-frame-modeler`](../../../.okf/capabilities/cap-sketcher-frame-modeler.md)
- **Cross-Capability Conventions:** [`cross-capability-conventions.md`](../cross-capability-conventions.md)
- **Discovery Source:** [`discovery-notes.md`](../cap-geometry-canvas-sketcher/discovery-notes.md)
- **Status:** Canonical Reference (2026-11-01)

## 1. Purpose & Scope

This glossary stabilizes the ubiquitous language for the **Mode 2 Structural Modeling & Analysis** workspace (`cap-sketcher-frame-modeler`). It defines the drafting, snapping, selection, transformation, live-analysis coordination, and teaching-handoff terms owned by Mode 2, while preserving the exact canonical vocabulary owned by `cap-frame-analysis-engine`, `cap-eurocode-actions-combinations`, `cap-structural-mechanics-teaching`, and `cap-project-persistence` in [`cross-capability-conventions.md`](../cross-capability-conventions.md).

## 2. Actors

| Canonical Term | Category | Definition | Discouraged Synonyms |
| --- | --- | --- | --- |
| **Engineer / Instructor** | Actor | Primary practitioner or educator who sketches arbitrary 2D structures in Mode 2, configures Eurocode combinations or nonlinear analyses, inspects envelopes, exports diagrams, and prepares teaching models. | Designer, Admin, Power User |
| **Student** | Actor | Learner who sketches 2D structures in Mode 2 or opens a Mode 3 preset as an editable sketch, observes live M/Q/N diagrams and deformed shapes, and transitions to Mode 3 via **Open Step-by-Step**. | Pupil, Novice |

## 3. Core Cross-Capability Terms (Referenced without Renaming)

These terms are governed by [`cross-capability-conventions.md`](../cross-capability-conventions.md) and used verbatim by the Mode 2 modeler:

| Canonical Term | Owner | Definition in Mode 2 Context | Discouraged Synonyms |
| --- | --- | --- | --- |
| **`StructuralModel`** | `[frame]` | Complete 2D structural model (`nodes`, `elements`, `sections`, `materials`, `supports`, `loads`, `load_cases`, `masses`) carrying a monotonically increasing integer `revision`. Single source of structural truth. | `SketchModel`, `CanvasModel`, `FrameGraph` |
| **`Node`** | `[frame]` | Point $(x, y)$ in metres ($\text{m}$, global $y$ up) with a stable string ID (`N1`, `N2`, ...). | Vertex, JointPoint, CanvasPoint |
| **`Element`** | `[frame]` | Structural connector between nodes. Canonical kinds: `FrameMember`, `TrussBar`, `Spring`, `Cable`. | Edge, LineSegment, Rod |
| **`EndRelease`** | `[frame]` | Per-end boolean release flags (`axial`, `shear`, `moment`) on a `FrameMember`. | MemberHinge, PinFlag |
| **`RigidOffset`** | `[frame]` | Per-end rigid end offset vector $(d_x, d_y)$ in metres on a `FrameMember`. | StubOffset, Eccentricity |
| **`Section` / `Material`** | `[frame]` | Cross-section ($A, I$, optional depth $h$ and shape) and material ($E$ entered in $\text{GPa}$, $G$, $\rho$, $\alpha_T$, class) assigned to elements by library ID or custom values. | Profile, CrossSectionProps |
| **`Support`** | `[frame]` | Nodal restraint (`Fixed`, `Pinned`, `Roller{angle}`, `Guided{angle}`, `Elastic{kx,ky,ktheta}`) with a `unilateral` boolean flag for compression-only behavior. | BoundaryCondition, Fixity |
| **`Load`** | `[frame]` | Action belonging to one `LoadCase`: `NodalLoad`, `MemberPointLoad`, `MemberDistributedLoad{q1,q2,a,b,frame: Local/Global/Projected}`, `SupportSettlement`, `ThermalLoad{delta_t_uniform, delta_t_gradient}`. Self-weight is a `LoadCase` boolean flag. | ForceVector, AppliedAction |
| **`LoadCase`** | `[frame]` | Named group of loads (`LC1`, `LC2`, ...) tagged with an `ActionType` and a `self_weight` toggle. | LoadGroup, CaseItem |
| **`ActionType`** | `[actions]` | Normative action category (`Permanent`, `Imposed{category A-H}`, `Snow`, `Wind`, `Temperature`, `Seismic`, `Accidental`). | LoadTypeTag |
| **`Combination`** | `[actions]` | Named set (`CO1`, ...) of $\gamma \cdot \psi \times \text{LoadCase}$ terms with a `LimitState` (`ULS_STR`, `ULS_GEO`, `SLS_CHAR`, `SLS_FREQ`, `SLS_QP`, `SEISMIC`) and `origin` (`Generated`, `UserEdited`, `Manual`). | LoadCombo, FactoredCase |
| **`AnalysisKind`** | `[frame]` | Solver mode selected in Mode 2: `LinearStatic`, `SecondOrderPDelta`, `LinearBuckling`, `Modal`, `ResponseSpectrum`, `LateralForce`. | SolverMode, RunType |
| **`AnalysisJob`** | `[frame]` | Background execution unit for `(model_revision, analysis_kind, target_set)` with status `Queued`, `Running`, `Converged`, `Failed`, `Cancelled`, or `Superseded`. | SolverTask, WorkerThread |
| **`ResultSet`** | `[frame]` | Computed response for one target (`LoadCase` or `Combination`): reactions, nodal displacements, analytically sampled member diagrams ($M, Q, N$), member end forces, and extrema. | SolutionData, OutputSet |
| **`Envelope`** | `[frame]` | Pointwise $\max/\min$ envelope over a set of `ResultSet`s, recording the governing `Combination` ID at every sample station. | MaxMinCurve, GoverningSet |
| **`Diagnostics`** | `[frame]` | Pre-solve and post-solve structural health metadata: kinematic mechanism mode shape, static indeterminacy degree, and solver warnings (slack cable, non-convergence). | SolverWarnings, StabilityCheck |
| **`MethodEligibility`** | `[teaching]` | Classifier response listing classical pedagogical methods with `eligible`/`ineligible` status, bilingual reasons (`el`/`en`), and DSM walkthrough availability. | HandoffCheck, SolverRouting |
| **`ProjectDocument`** | `[persistence]` | Versioned JSON project container holding `structural`, `continuum`, and `ui` workspace sections. | SaveFile, BemArchive |

## 4. Mode 2 Drafting, Snapping & Workspace Vocabulary (Owned by `cap-sketcher-frame-modeler`)

| Canonical Term | Category | Definition | Discouraged Synonyms |
| --- | --- | --- | --- |
| **Mode 2 Workspace** | Business Object | The "Structural Modeling & Analysis" workspace reachable from the top header switcher alongside Mode 1 ("Continuum Workbench") and Mode 3 ("Teaching Studio"). | Sketcher Page, Frame Tab |
| **DraftingViewport** | Value Object | Viewport pan $(t_x, t_y)$, zoom scale $s$, world-to-screen affine transform, and dynamic grid step ($0.1\text{ m}$, $0.5\text{ m}$, or $1.0\text{ m}$). | Camera2D, CanvasTransform |
| **ActiveTool** | State / Enum | Currently active user interaction tool on the canvas: `Select`, `Pan`, `DrawNode`, `DrawFrameMember`, `DrawTrussBar`, `DrawSpring`, `DrawCable`, `PlaceSupport`, `PlaceInternalHinge`, `PlaceLoad`, `Move`, `Copy`, `Mirror`, `LinearArray`, `Delete`. | CursorMode, ToolState |
| **SnapCandidate** | Value Object | Resolved world coordinate $(x, y)$ produced by the snapping engine with its `SnapKind`, target entity reference(s), screen pixel distance, and visual indicator badge. | MagnetPoint, HitPoint |
| **SnapKind** | Enumeration | Priority-ordered snap classification: `NodeSnap` ($< 15\text{ px}$ to existing `Node`), `IntersectionSnap` (member-member intersection), `MidpointSnap` (member midpoint), `AngularGuideSnap` ($30^\circ, 45^\circ, 60^\circ$ or `OrthoLock` at $0^\circ, 90^\circ, 180^\circ, 270^\circ$), `GridSnap` ($0.1\text{ m}, 0.5\text{ m}, 1.0\text{ m}$), `NumericEntry` (exact typed $x, y$ or $L, \theta$). | SnapType, SnapPriority |
| **TopologicalMemberSplit** | Workflow / Rule | Automatic geometric operation triggered when a new node or member endpoint snaps to an interior point or intersection of an existing `FrameMember` or `TrussBar`: splits the host element at the intersection node while preserving section, material, distributed loads, and outer end releases. | LineBreak, AutoSubdivide |
| **InternalHingeGesture** | Policy / Gesture | Node-level user gesture in Mode 2 that places a visual hinge ring at a shared node and expands into `EndRelease{moment: true}` on $k - 1$ of the $k$ incident `FrameMember` ends (or explicit user-selected ends) so the global rotational DOF remains non-singular. | NodalPin, JointHingeEntity |
| **CableInputSpec** | Value Object | Modeler-side parameter choice for defining a `Cable` element: `UnstressedLength{l0}`, `SagRatio{ratio}`, or `Pretension{t0}`, combined with section and cable material. | CableProps, CatenaryConfig |
| **ModelMutationCommand** | Business Object | Reversible atomic command on `StructuralModel` stored in the `UndoRedoStack`. Every executed mutation increments `StructuralModel.revision` by 1. | HistoryEntry, ActionPatch |
| **ResultOverlayState** | Value Object | Active visualization controls for live analysis results: target selector (`LoadCase` ID, `Combination` ID, or `Envelope`), active diagram kind (`None`, `M`, `Q`, `N`), diagram scale factor, deformed-shape toggle and magnification factor, reaction arrows toggle, extrema labels toggle, and mechanism animation state. | PlotConfig, DiagramSettings |
| **CrossSectionHoverProbe** | Value Object | Live cursor readout when hovering over any member at local coordinate $x \in [0, L]$, displaying exact interpolated $N(x), Q(x), M(x)$, displaced coordinates $(u_x, u_y, \theta_z)$, and—in `Envelope` mode—the governing `Combination` ID for both max and min values. | MemberTooltip, DiagramProbe |
| **LiveAnalysisCoordinator** | Domain Service | Reactive orchestrator in Mode 2 that debounces model mutations (~$150\text{ ms}$), invokes immediate synchronous `LinearStatic` DSM evaluation (< $50\text{ ms}$), manages background `AnalysisJob` execution for nonlinear/dynamic analyses, and cancels stale jobs when `revision` advances. | AutoSolver, LiveRunner |

## 5. Critical Semantic Distinctions

1. **`StructuralModel` vs. `SketchModel` (Retired):**
   - `SketchModel` from superseded issue 015 is retired. Mode 2 edits `StructuralModel` (owned by `cap-frame-analysis-engine`) directly through a frontend signal store. There is no separate sketch-only structural schema.
2. **`InternalHingeGesture` vs. `EndRelease`:**
   - The solver domain model (`StructuralModel`) has no node-level hinge entity; it only has per-member `EndRelease`. Clicking a node with the Internal Hinge tool in Mode 2 is a UI gesture (`InternalHingeGesture`) that sets `moment: true` on $k - 1$ incident `FrameMember` ends (leaving one member continuous to avoid a zero-stiffness rotational mechanism at the node), while rendering a single hinge symbol on the canvas when all incident members cannot transmit moment across the joint.
3. **`TopologicalMemberSplit` vs. Point Load Placement:**
   - Placing a `MemberPointLoad` or `MemberDistributedLoad` at distance $x$ along a `FrameMember` does **not** split the member or insert an internal `Node` (discovery Q36). Conversely, drawing a new member or node that snaps to an interior point or intersection of an existing member **does** execute `TopologicalMemberSplit`, creating a shared `Node` and two connected sub-elements.
4. **Immediate Linear Preview vs. Background Nonlinear Convergence:**
   - Every edit increments `StructuralModel.revision` and triggers a debounced (~$150\text{ ms}$) `LinearStatic` solve so diagrams update immediately. When a nonlinear analysis (`SecondOrderPDelta`, `Cable` catenary, or unilateral supports) is active, the linear result is badged as *Linear Preview* while the background `AnalysisJob` runs via `analysis://progress`, replacing the canvas overlay only if its `revision` still matches the current model revision upon convergence.
5. **Linear Superposition vs. Nonlinear Per-Combination Solving:**
   - For purely linear models, combinations are superposed from `LoadCase` result sets. Whenever the model contains `Cable` elements, `unilateral` supports, or `SecondOrderPDelta` analysis, superposition is invalid: each `Combination` is solved independently by the engine, and the Mode 2 UI explicitly displays a bilingual non-superposition notice.
6. **`StructuralModel` vs. `ContinuumDomain`:**
   - `StructuralModel` represents 1D/0D skeletal elements (`FrameMember`, `TrussBar`, `Spring`, `Cable`) in Mode 2. Closed 2D polygons, interior holes, and Constrained Delaunay Triangulation (CDT) meshing belong exclusively to `ContinuumDomain` in sibling capability `cap-sketcher-continuum-domain`.
