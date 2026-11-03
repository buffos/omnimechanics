# Canonical Domain Model: Mode 2 Structural Frame Modeler (`cap-sketcher-frame-modeler`)

- **Capability:** [`cap-sketcher-frame-modeler`](../../../.okf/capabilities/cap-sketcher-frame-modeler.md)
- **Glossary:** [`domain-glossary.md`](domain-glossary.md)
- **PRD:** [`prd.md`](prd.md)
- **Cross-Capability Conventions:** [`cross-capability-conventions.md`](../cross-capability-conventions.md)
- **Status:** Canonical Reference (2026-11-01)

## 1. Purpose & Modeling Principles

This document defines the canonical domain model for `cap-sketcher-frame-modeler`. Because `cap-frame-analysis-engine` is the single owner of `StructuralModel` (`Node`, `Element`, `Section`, `Material`, `Support`, `Load`, `LoadCase`) and `cap-eurocode-actions-combinations` owns `Combination` and `ActionType`, the Mode 2 domain model focuses on:
1. **Workspace & Editing State (`ModelerWorkspace` aggregate):** How the user views, edits, selects, transforms, and undoes/redoes mutations on `StructuralModel`.
2. **Geometric & Topological Drafting Services:** Priority snapping, colinearity/intersection math, automatic `TopologicalMemberSplit`, and node-level `InternalHingeGesture` expansion.
3. **Live Analysis & Teaching Handoff Coordination:** Managing revision-tagged linear results, background `AnalysisJob` lifecycles, `Envelope` inspection, and `MethodEligibility` synchronization.

## 2. Subdomains & Bounded Context Seam

| Subdomain | Role | Owner Capability |
| --- | --- | --- |
| **Interactive 2D Drafting & Geometric Snapping** | Core (Mode 2) | `cap-sketcher-frame-modeler` |
| **Model Mutation, Undo/Redo & CAD Transformations** | Core (Mode 2) | `cap-sketcher-frame-modeler` |
| **Live Analysis & Result Visualization State** | Core (Mode 2) | `cap-sketcher-frame-modeler` |
| **Structural Mechanics & DSM Solving** | Upstream Shared Core | `cap-frame-analysis-engine` |
| **Normative Actions, Combinations & Spectra** | Upstream Shared Core | `cap-eurocode-actions-combinations` |
| **Pedagogical Method Classification & Step-by-Step** | Peer / Downstream Handoff | `cap-structural-mechanics-teaching` |
| **Workspace Serialization (`.bem` JSON)** | Infrastructure / Peer | `cap-project-persistence` |

## 3. Aggregate Design

### 3.1 `ModelerWorkspace` (Aggregate Root — Owned by `cap-sketcher-frame-modeler`)

`ModelerWorkspace` encapsulates the complete interactive state of Mode 2 and holds the active `StructuralModel` snapshot along with its undo/redo history, active tool state, selection set, and live analysis projections.

```
ModelerWorkspace (Aggregate Root)
├── structural_model: StructuralModel [from cap-frame-analysis-engine, carries revision: u64]
├── combinations: Vec<Combination> [from cap-eurocode-actions-combinations]
├── national_annex: NationalAnnexProfile [default GR]
├── seismic_settings: Option<SeismicSettings>
├── viewport: DraftingViewport
├── active_tool: ActiveTool
├── snap_settings: SnapSettings
├── draft_gesture: Option<DraftingGestureState>
├── selection: SelectionSet
├── undo_redo: UndoRedoStack
├── analysis_context: LiveAnalysisContext
├── result_overlay: ResultOverlayState
└── teaching_eligibility: Option<MethodEligibility>
```

#### Why `ModelerWorkspace` is One Aggregate Boundary
Every mutation to `structural_model` must atomically:
1. Increment `structural_model.revision` by 1.
2. Push a reversible `ModelMutationCommand` onto `undo_redo`.
3. Prune dangling IDs from `selection` and `result_overlay`.
4. Mark `analysis_context` and `teaching_eligibility` as requiring refresh for the new `revision`, immediately superseding any in-flight `AnalysisJob` for an older revision.

## 4. Entities & Value Objects

### 4.1 Referenced Upstream Entities (in `StructuralModel`)
- **`Node`:** `{ id: NodeId ("N1"...), x: f64, y: f64 }` (metres, $y$ up).
- **`Element`:** `{ id: ElementId ("E1"...), kind: ElementKind }` where `ElementKind` is:
  - `FrameMember { start_node: NodeId, end_node: NodeId, section_id: SectionId, material_id: MaterialId, releases: MemberEndReleases { start: EndRelease, end: EndRelease }, rigid_offsets: MemberRigidOffsets { start: RigidOffset, end: RigidOffset } }`
  - `TrussBar { start_node: NodeId, end_node: NodeId, section_id: SectionId, material_id: MaterialId, tension_only: bool, compression_only: bool }`
  - `Spring { node_a: NodeId, node_b: Option<NodeId>, kx: f64, ky: f64, ktheta: f64 }`
  - `Cable { start_node: NodeId, end_node: NodeId, section_id: SectionId, material_id: MaterialId, input_spec: CableInputSpec }`
- **`CableInputSpec` (Value Object):** `UnstressedLength { l0: f64 } | SagRatio { ratio: f64 } | Pretension { t0: f64 }`.
- **`Support`:** `{ id: SupportId ("S1"...), node_id: NodeId, kind: SupportKind, unilateral: bool }` where `SupportKind` is `Fixed | Pinned | Roller { angle_deg: f64 } | Guided { angle_deg: f64 } | Elastic { kx: f64, ky: f64, ktheta: f64 }`.
- **`LoadCase`:** `{ id: LoadCaseId ("LC1"...), name: String, action_type: ActionType, self_weight: bool }`.
- **`Load`:** `{ id: LoadId ("L1"...), load_case_id: LoadCaseId, kind: LoadKind }` where `LoadKind` is:
  - `NodalLoad { node_id: NodeId, fx: f64, fy: f64, mz: f64 }`
  - `MemberPointLoad { element_id: ElementId, distance_a: f64, frame: LoadFrame (Local | Global), px: f64, py: f64, m: f64 }`
  - `MemberDistributedLoad { element_id: ElementId, q1x: f64, q1y: f64, q2x: f64, q2y: f64, start_a: f64, end_b: f64, frame: DistributedLoadFrame (Local | Global | Projected) }`
  - `SupportSettlement { node_id: NodeId, dx: f64, dy: f64, dtheta: f64 }`
  - `ThermalLoad { element_id: ElementId, delta_t_uniform: f64, delta_t_gradient: f64 }`

### 4.2 Mode 2 Drafting & Viewport Value Objects
- **`DraftingViewport`:** `{ pan_x: f64, pan_y: f64, zoom_px_per_m: f64, grid_step_m: GridStep (0.1 | 0.5 | 1.0), show_grid: bool, show_axes: bool }`.
- **`SnapSettings`:** `{ node_snap_px: f64 (default 15.0), enable_intersection_snap: bool, enable_midpoint_snap: bool, ortho_lock: bool, angle_snap_deg: Option<f64> (30.0 | 45.0 | 60.0), enable_grid_snap: bool }`.
- **`SnapCandidate`:** `{ world_pt: (f64, f64), screen_dist_px: f64, kind: SnapKind, target_node_id: Option<NodeId>, target_element_ids: Vec<ElementId>, guide_angle_deg: Option<f64> }`.
- **`DimensionTooltip`:** `{ start_pt: (f64, f64), current_pt: (f64, f64), length_m: f64, angle_deg: f64, cursor_readout: String }`.
- **`SelectionSet`:** `{ node_ids: BTreeSet<NodeId>, element_ids: BTreeSet<ElementId>, support_ids: BTreeSet<SupportId>, load_ids: BTreeSet<LoadId>, selection_box: Option<MarqueeBox> }` where `MarqueeBox` has `mode: WindowEnclosed | CrossingIntersect`.
- **`ModelMutationCommand`:** `{ id: u64, label_el: String, label_en: String, before_revision: u64, after_revision: u64, inverse_patch: ModelPatch, forward_patch: ModelPatch }`.

### 4.3 Live Analysis & Result Overlay Value Objects
- **`LiveAnalysisContext`:**
  - `active_analysis_kind: AnalysisKind` (`LinearStatic`, `SecondOrderPDelta`, `LinearBuckling`, `Modal`, `ResponseSpectrum`, `LateralForce`).
  - `linear_preview: Option<RevisionedResult>` (carries `model_revision: u64`, `ResultSet` or `Envelope`, and `Diagnostics`).
  - `active_background_job: Option<AnalysisJobState>` (`{ job_id: String, model_revision: u64, kind: AnalysisKind, status: JobStatus, progress_pct: f64, iteration: u32, residual: Option<f64> }`).
  - `converged_result: Option<RevisionedResult>`.
  - `superposition_valid: bool` (`true` only when `active_analysis_kind == LinearStatic` and model has 0 `Cable`s and 0 `unilateral` supports/bars).
- **`ResultOverlayState`:**
  - `active_target: ResultTarget` (`LoadCase(LoadCaseId) | Combination(CombinationId) | Envelope(LimitState)`).
  - `active_diagram: DiagramKind` (`None | BendingMomentM | ShearForceQ | AxialForceN | BucklingMode(usize) | VibrationMode(usize)`).
  - `diagram_scale: f64`, `show_deformed_shape: bool`, `deformed_scale: f64`, `show_reactions: bool`, `show_extrema: bool`, `animate_mechanism: bool`.
  - `hover_probe: Option<CrossSectionHoverProbe>`.

## 5. Domain Policies & Services (Owned by `cap-sketcher-frame-modeler`)

### 5.1 `SnappingPolicyService`
Evaluates pointer world position $p_{\text{raw}}$ (and optional active drawing anchor $p_0$) against `StructuralModel` and `DraftingViewport`:
1. **`NodeSnap`:** If any existing `Node` is within $< 15\text{ px}$ (`node_snap_px`) of $p_{\text{raw}}$ in screen space, snap to the closest node.
2. **`IntersectionSnap`:** If enabled, compute pairwise geometric intersections between existing `FrameMember`/`TrussBar` segments (and between the active rubber-band segment $\overline{p_0 p_{\text{raw}}}$ and existing segments); if an intersection lies within $< 15\text{ px}$, snap to the exact intersection $(x, y)$ and attach both intersected `ElementId`s.
3. **`MidpointSnap`:** If enabled and a member midpoint is within $< 12\text{ px}$, snap to $((x_1+x_2)/2, (y_1+y_2)/2)$ and attach the host `ElementId`.
4. **`AngularGuideSnap` / `OrthoLock`:** When drawing from $p_0$, if `ortho_lock` is active, project $p_{\text{raw}} - p_0$ onto the nearest cardinal angle ($0^\circ, 90^\circ, 180^\circ, 270^\circ$); if `angle_snap_deg` is active and cursor angle is within $\pm 4^\circ$ of a multiple of $30^\circ, 45^\circ,$ or $60^\circ$, lock the ray angle and intersect with grid or distance.
5. **`GridSnap`:** Round $(x, y)$ to nearest multiple of `grid_step_m` ($0.1\text{ m}, 0.5\text{ m}, 1.0\text{ m}$).
6. **`NumericEntry`:** Direct keyboard input `(x, y)` or polar `(L, θ_deg)` bypasses pointer snap and resolves exact coordinates, still checking whether the resulting point coincides (within $10^{-6}\text{ m}$) with an existing `Node` or member interior.

### 5.2 `TopologicalSplitService`
When a node creation or element drawing gesture commits at a point $P$ that lies strictly in the interior of an existing `FrameMember` or `TrussBar` $E = (N_A, N_B)$ with parameter $t \in (\epsilon, 1 - \epsilon)$ ($\epsilon = 10^{-4}$):
1. Creates or reuses `Node` $N_P$ at $P$.
2. Splits $E$ into $E_1 = (N_A, N_P)$ and $E_2 = (N_P, N_B)$ inheriting `section_id`, `material_id`, and `tension_only`/`compression_only`.
3. Transfers $E$'s `start` `EndRelease` and `RigidOffset` to $E_1.\text{start}$, transfers $E$'s `end` `EndRelease` and `RigidOffset` to $E_2.\text{end}$, and sets internal ends ($E_1.\text{end}, E_2.\text{start}$) to unreleased rigid continuity (`EndRelease::none()`, `RigidOffset::zero()`).
4. Re-maps any `MemberPointLoad` and `MemberDistributedLoad` on $E$ proportionally onto $E_1$ and/or $E_2$ by local coordinate $x_P = t \cdot L$.
5. When two crossing members $E_A, E_B$ intersect during drawing, splits both at shared node $N_P$.

### 5.3 `InternalHingeExpansionPolicy`
When the user applies `InternalHingeGesture` at `Node` $N_j$ with $k$ incident `FrameMember`s $(E_1, \dots, E_k)$:
- If $k < 2$, rejects with `INVALID_HINGE_TARGET` (a single cantilever end or truss node does not take an internal frame hinge; support hinges use `Pinned` support).
- If $k \ge 2$, sets `EndRelease.moment = true` on $k - 1$ incident member ends at $N_j$ while keeping exactly 1 member end rotationally continuous with the node DOF $\theta_z(N_j)$ so the global stiffness matrix is never singular due to a decoupled nodal rotation. The canvas renders a single internal hinge circle at $N_j$. Removing the hinge clears `moment` releases on all incident ends at $N_j$.

### 5.4 `CadTransformationService`
Executes geometric transformations on a non-empty `SelectionSet`:
- **`Move(dx, dy)`:** Translates selected nodes by $(\Delta x, \Delta y)$, merging coincident nodes within $10^{-6}\text{ m}$ upon user confirmation or auto-weld policy.
- **`Copy(dx, dy)` / `LinearArray(dx, dy, count)`:** Duplicates selected nodes, elements, supports, and nodal/member loads $n$ times at offsets $i \cdot (\Delta x, \Delta y)$, assigning fresh monotonic stable IDs (`N..`, `E..`).
- **`Mirror(axis_p1, axis_p2)`:** Reflects selected entities across the line $\overline{p_1 p_2}$, reversing local member load orientations or angle-dependent roller/guided support angles so physical world directions are preserved.

## 6. Domain Invariants

- **INV-1 (Zero-Length Element Prohibition):** No `FrameMember`, `TrussBar`, or `Cable` may connect a `Node` to itself or connect two nodes with Euclidean distance $L < 10^{-4}\text{ m}$ (`ZERO_LENGTH_ELEMENT`). Only nodal `Spring` elements may reference a single `Node` (`node_b: None`) or two coincident nodes.
- **INV-2 (Duplicate Collinear Element Prevention):** Two elements of the same kind (`FrameMember`, `TrussBar`, `Cable`) cannot share the exact same unordered node pair $\{N_A, N_B\}$ unless explicitly placed as a parallel spring (`DUPLICATE_ELEMENT`).
- **INV-3 (Member Load Bounds):** Every `MemberPointLoad` must satisfy $0 \le a \le L$, and every `MemberDistributedLoad` must satisfy $0 \le a < b \le L$ for the current length $L$ of its target element (`LOAD_OUT_OF_MEMBER_BOUNDS`).
- **INV-4 (Cable Input Positive Definite):** A `Cable` element's `CableInputSpec` must satisfy $L_0 > 0$, $0 < \text{sag\_ratio} < 0.5$, or $T_0 > 0$ (`INVALID_CABLE_SPEC`).
- **INV-5 (Revision-Strict Result Rendering):** `ResultOverlayState` may only render diagrams or displacements from a `RevisionedResult` whose `model_revision == structural_model.revision`.
- **INV-6 (Internal Hinge Non-Singularity):** At any node $N_j$ without rotational support restraint, at least one incident `FrameMember` must have `moment == false` at $N_j$ unless all incident elements are `TrussBar`/`Cable` (in which case rotational DOF is condensed out by the engine).

## 7. Domain Events

1. `ModelMutated { new_revision: u64, command_label_el: String, command_label_en: String }`
2. `MemberTopologicallySplit { original_element_id: ElementId, intersection_node_id: NodeId, new_element_ids: (ElementId, ElementId) }`
3. `LinearPreviewUpdated { model_revision: u64, is_mechanism: bool, indeterminacy_degree: i32 }`
4. `BackgroundAnalysisProgressed { job_id: String, model_revision: u64, status: JobStatus, progress_pct: f64 }`
5. `BackgroundAnalysisConverged { job_id: String, model_revision: u64, analysis_kind: AnalysisKind }`
6. `MethodEligibilityEvaluated { model_revision: u64, eligible_methods: Vec<String>, dsm_walkthrough_available: bool }`
7. `Mode3HandoffInitiated { model_revision: u64, selected_method: String }`

## 8. Mapping Guidance for Implementation

- **Frontend State Store (`src/app/features/modeler/`):** Model `ModelerWorkspace` as an Angular Signal Store (`ModelerStore`) holding immutable snapshots of `StructuralModel` for O(1) undo/redo and reactive `computed()` signals for SVG rendering layers (grid, elements, supports, loads, tension-side $M/Q/N$ paths, deformed shape paths, and hover probe).
- **Pure TypeScript Core:** Keep `SnappingPolicyService`, `TopologicalSplitService`, `InternalHingeExpansionPolicy`, and `CadTransformationService` in pure, zero-DOM TypeScript modules so they execute deterministically under the Vitest `frontend-integration` suite.
