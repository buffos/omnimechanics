# Canonical Use-Case & Application Model: Mode 2 Structural Frame Modeler (`cap-sketcher-frame-modeler`)

- **Capability:** [`cap-sketcher-frame-modeler`](../../../.okf/capabilities/cap-sketcher-frame-modeler.md)
- **Domain Model:** [`canonical-domain-model.md`](canonical-domain-model.md)
- **PRD:** [`prd.md`](prd.md)
- **Cross-Capability Conventions:** [`cross-capability-conventions.md`](../cross-capability-conventions.md)
- **Status:** Canonical Reference (2026-11-01)

## 1. Purpose & Application-Layer Goals

This document defines the application-layer use cases (commands, queries, reactive coordinators, and cross-capability orchestration) for `cap-sketcher-frame-modeler`. It separates:
1. **Pure Frontend Workspace Use Cases:** High-frequency interactive drafting, snapping, selection, CAD transformations, and undo/redo executed synchronously in the Mode 2 Signal Store and verified by Vitest.
2. **Cross-Capability Orchestration Use Cases:** Debounced live linear DSM solving, background nonlinear/dynamic job management (`cap-frame-analysis-engine`), Eurocode combination synchronization (`cap-eurocode-actions-combinations`), Mode 3 eligibility classification and bidirectional handoff (`cap-structural-mechanics-teaching`), and `.bem` project persistence (`cap-project-persistence`).

## 2. Application Service Groupings

| Application Service | Responsibility | Execution Boundary |
| --- | --- | --- |
| **`DraftingAndSnappingService`** | Evaluates cursor snap candidates, numeric coordinate/length-angle input, member drawing gestures, and automatic `TopologicalMemberSplit`. | Frontend Pure TS (`frontend-integration`) |
| **`ModelEditingAndTransformService`** | Applies property edits (sections, materials, releases, internal hinges, rigid offsets, cable specs), support/load placement, CAD transforms (`Move`, `Copy`, `Mirror`, `LinearArray`, `Delete`), and `UndoRedoStack` management. | Frontend Pure TS (`frontend-integration`) |
| **`LiveAnalysisCoordinatorService`** | Debounces `StructuralModel.revision` changes (~$150\text{ ms}$), triggers synchronous linear DSM solves, dispatches/cancels background `AnalysisJob`s, and updates `ResultOverlayState`. | Frontend $\leftrightarrow$ Rust IPC (`frame_*`, `actions_*`) |
| **`TeachingHandoffCoordinatorService`** | Evaluates `MethodEligibility` on model changes, transfers eligible models or DSM walkthroughs to Mode 3, and imports Mode 3 presets into Mode 2 as editable sketches. | Frontend $\leftrightarrow$ Rust IPC (`teaching_*`) |
| **`WorkspaceExportAndPersistenceService`** | Coordinates saving/loading `StructuralModel` and `Combination`s via `ProjectDocument` and exports publication-grade SVG/PNG diagrams. | Frontend $\leftrightarrow$ Rust IPC (`project_*`) |

## 3. Canonical Commands

### UC-C01: `SketchElementWithSnapping`
- **Intent:** Place a new `FrameMember`, `TrussBar`, `Spring`, or `Cable` between two snapped or numerically entered points, automatically splitting any intersected or interior-snapped host members.
- **Inputs:** `{ element_kind, start_input: PointerOrNumeric, end_input: PointerOrNumeric, default_section_id, default_material_id, cable_spec?: CableInputSpec, snap_settings }`
- **Preconditions:** Distance between resolved start and end points $\ge 10^{-4}\text{ m}$ (except single-node `Spring`).
- **Orchestration:**
  1. Resolve start and end `SnapCandidate`s via `SnappingPolicyService` (or exact `(x,y)` / `(L,θ)` numeric parser).
  2. If start or end snaps to an existing member interior or member-member intersection, invoke `TopologicalSplitService` to split the host member(s) and obtain shared `NodeId`s.
  3. If start or end is a new point, create `Node` (`N..`).
  4. If the new segment crosses existing segments and `enable_intersection_snap` is active, compute all intermediate intersection points, split crossed members, and subdivide the drawn element into connected colinear segments.
  5. Record a single composite `ModelMutationCommand`, increment `StructuralModel.revision`, and trigger `CoordinateLiveAnalysis`.
- **Failure Modes:** `ZERO_LENGTH_ELEMENT`, `DUPLICATE_ELEMENT`, `INVALID_CABLE_SPEC`.

### UC-C02: `ApplyCadTransformation`
- **Intent:** Execute `Move`, `Copy`, `Mirror`, `LinearArray`, or `Delete` on the current `SelectionSet`.
- **Inputs:** `{ selection: SelectionSet, operation: Move{dx,dy} | Copy{dx,dy} | Mirror{p1,p2} | LinearArray{dx,dy,count} | Delete, weld_coincident_nodes: bool }`
- **Orchestration:**
  1. Validate `selection` is non-empty (`EMPTY_SELECTION`) and parameters are valid (e.g., `count >= 1`, mirror axis length $\ge 10^{-4}\text{ m}$).
  2. Execute `CadTransformationService` to mutate or clone selected nodes, elements, supports, and loads.
  3. When deleting nodes, cascade-delete incident elements, supports, and nodal loads.
  4. Increment `StructuralModel.revision`, push onto `UndoRedoStack`, and trigger `CoordinateLiveAnalysis`.
- **Failure Modes:** `EMPTY_SELECTION`, `DEGENERATE_MIRROR_AXIS`, `INVALID_ARRAY_COUNT`.

### UC-C03: `ConfigureElementPropertiesAndHinges`
- **Intent:** Edit `Section`, `Material`, `EndRelease`, `RigidOffset`, `CableInputSpec`, or apply/remove a node-level `InternalHingeGesture`.
- **Inputs:** `{ target: ElementIds(Vec<ElementId>) | NodeHingeTarget(NodeId), patch: PropertyPatch }`
- **Orchestration:**
  1. When `target` is `NodeHingeTarget(node_id)`, invoke `InternalHingeExpansionPolicy` to set `EndRelease.moment = true` on $k - 1$ incident `FrameMember` ends (or clear all when toggling off).
  2. When `target` is `ElementIds`, apply section/material IDs (converting custom $E$ from $\text{GPa}$ to $\text{kN/m}^2$ at the solver boundary), end releases, rigid offsets, or cable specs ($L_0$, sag ratio, $T_0$).
  3. Increment `StructuralModel.revision`, push onto `UndoRedoStack`, and trigger `CoordinateLiveAnalysis`.
- **Failure Modes:** `INVALID_HINGE_TARGET`, `INVALID_SECTION_PROPERTIES`, `INVALID_CABLE_SPEC`.

### UC-C04: `AssignSupportOrLoad`
- **Intent:** Assign or update a `Support` on node(s) or a `Load` (`NodalLoad`, `MemberPointLoad`, `MemberDistributedLoad`, `SupportSettlement`, `ThermalLoad`) within an active `LoadCase`, or toggle `LoadCase.self_weight`.
- **Inputs:** `{ load_case_id?: LoadCaseId, payload: SupportAssignment | LoadAssignment | SelfWeightToggle }`
- **Orchestration:**
  1. Validate member load station bounds ($0 \le a \le L$ or $0 \le a < b \le L$) without splitting the target member.
  2. Mutate `StructuralModel.supports` or `StructuralModel.loads`, increment `revision`, push command to `UndoRedoStack`, and trigger `CoordinateLiveAnalysis`.
- **Failure Modes:** `LOAD_OUT_OF_MEMBER_BOUNDS`, `UNKNOWN_LOAD_CASE`, `INVALID_SUPPORT_STIFFNESS`.

### UC-C05: `UndoOrRedoModelMutation`
- **Intent:** Step backward (`Undo`) or forward (`Redo`) in `UndoRedoStack`.
- **Inputs:** `{ direction: Undo | Redo }`
- **Orchestration:**
  1. Pop command from undo/redo stack, apply inverse/forward patch, and **increment `StructuralModel.revision`** (monotonic revision rule: undoing does not revert `revision` to a previously used number, preventing stale async job collisions).
  2. Trigger `CoordinateLiveAnalysis`.

### UC-C06: `CoordinateLiveAnalysis`
- **Intent:** Synchronize live DSM results and background nonlinear/dynamic jobs whenever `StructuralModel.revision`, `active_analysis_kind`, or `active_target` changes.
- **Inputs:** `{ model: StructuralModel, combinations: Vec<Combination>, analysis_kind: AnalysisKind, target: ResultTarget }`
- **Idempotency & Concurrency:** Debounced by $150\text{ ms}$. Any in-flight `AnalysisJob` for an older `model_revision` is immediately cancelled via `frame_cancel_analysis_job` and marked `Superseded`.
- **Orchestration:**
  1. Invoke synchronous `frame_solve_linear` for `model.revision`.
  2. If `Diagnostics` reports a kinematic mechanism (`is_mechanism == true`), set `ResultOverlayState.animate_mechanism = true`, display bilingual stability diagnostics, and stop.
  3. Store the synchronous result in `linear_preview` and immediately render reactions, tension-side $M/Q/N$ diagrams, extrema, and deformed shape.
  4. Concurrently invoke `teaching_classify_model` to update `teaching_eligibility`.
  5. If `analysis_kind != LinearStatic` or `model` contains `Cable` or `unilateral` elements, invoke `frame_start_analysis_job`, subscribe to `analysis://progress`, display the progress badge (`iteration`, `progress_pct`), and upon `Converged` (if `job.model_revision == current.revision`), replace `converged_result` on the canvas.

### UC-C07: `GenerateOrUpdateCombinations`
- **Intent:** Generate EN 1990 / EN 1998 combinations from tagged `LoadCase`s and `NationalAnnexProfile` (default `GR`), or save user-edited/manual combinations.
- **Inputs:** `{ load_cases: Vec<LoadCase>, annex: NationalAnnexProfile, uls_equation: Eq6_10 | Eq6_10ab, seismic_settings?: SeismicSettings, manual_overrides?: Vec<Combination> }`
- **Orchestration:** Calls `actions_generate_combinations` (and `actions_compute_design_spectrum` when seismic is active), stores the resulting `Vec<Combination>` in `ModelerWorkspace`, and refreshes `CoordinateLiveAnalysis` if a `Combination` or `Envelope` target is active.

### UC-C08: `HandoffToMode3TeachingStudio`
- **Intent:** Transition from Mode 2 to Mode 3 ("Teaching Studio") with the current `StructuralModel` and a selected pedagogical method (`IsostaticBeamFrame`, `TrussJointsRitter`, `ContinuousBeamClapeyronCross`, `ForceMethod`, or `DsmWalkthrough`).
- **Inputs:** `{ model: StructuralModel, active_load_case_id: LoadCaseId, method_id: String }`
- **Preconditions:** `method_id` is marked `eligible: true` in `MethodEligibility` (or `method_id == "DsmWalkthrough"` and the model is kinematically stable).
- **Orchestration:** Calls `teaching_open_step_by_step` with the shared `StructuralModel` and switches the application header workspace to Mode 3.

### UC-C09: `OpenMode3PresetInMode2`
- **Intent:** Load a Mode 3 teaching preset into Mode 2 as a fully editable `StructuralModel`.
- **Inputs:** `{ preset_id: String }`
- **Orchestration:** Calls `teaching_preset_to_structural_model(preset_id)`, initializes `ModelerWorkspace` with `revision: 1`, clears `UndoRedoStack`, switches header workspace to Mode 2, and runs `CoordinateLiveAnalysis`.

### UC-C10: `ExportDiagramOrPersistProject`
- **Intent:** Export the current canvas view (structure + active $M/Q/N$ diagram + legend + title block) to SVG or PNG, or save/load the workspace via `ProjectDocument`.
- **Inputs:** `ExportSvgPng { format: Svg | Png, include_grid: bool, dpi_scale: f64 } | SaveProject { path } | LoadProject { path }`

## 4. Canonical Queries

| Query ID | Name | Inputs | Returns |
| --- | --- | --- | --- |
| **UC-Q01** | `EvaluateCursorSnap` | `{ raw_world_pt, anchor_pt?, viewport, snap_settings }` | `SnapCandidate` + `DimensionTooltip` |
| **UC-Q02** | `ProbeMemberCrossSection` | `{ element_id, local_x_m, active_target }` | `CrossSectionHoverProbe` ($N, Q, M$, $u_x, u_y, \theta_z$, governing `CombinationId` for max/min) |
| **UC-Q03** | `GetMemberEndForceTable` | `{ active_target, filter_element_ids? }` | Tabular rows of start/end $(N, Q, M)$ and governing combinations per element |
| **UC-Q04** | `GetMethodEligibilitySummary` | `{ model_revision }` | `MethodEligibility` (per-method status, bilingual reasons, DSM walkthrough status, indeterminacy degree) |

## 5. Failure Categories & Recovery

| Failure Code | Category | Bilingual User-Visible Outcome |
| --- | --- | --- |
| `ZERO_LENGTH_ELEMENT` | Domain Validation | Rejects drawing gesture; shows toast in `el`/`en` that start and end nodes cannot coincide. |
| `LOAD_OUT_OF_MEMBER_BOUNDS` | Domain Validation | Highlights distance input field in Inspector; prevents out-of-span load placement. |
| `INVALID_HINGE_TARGET` | Domain Policy | Explains that internal hinges require a node shared by $\ge 2$ frame members (use Pinned Support for support hinges). |
| `KINEMATIC_MECHANISM_DETECTED` | Solver Diagnostic | Animates unstable mechanism mode on canvas; displays indeterminacy/mechanism banner in `el`/`en`. |
| `NONLINEAR_NON_CONVERGENCE` | Solver Diagnostic | Retains `linear_preview` with warning badge; shows residual and slack-cable/plasticity diagnostic from `Diagnostics`. |
| `INELIGIBLE_TEACHING_METHOD` | Handoff Policy | Disables unsupported classical method button; popover explains exact reason (e.g., cable presence or hyperstatic degree $> 3$) and offers DSM Walkthrough. |
