# Acceptance Scenarios: Mode 2 Structural Frame Modeler (`cap-sketcher-frame-modeler`)

- **Capability:** [`cap-sketcher-frame-modeler`](../../../.okf/capabilities/cap-sketcher-frame-modeler.md)
- **Contract:** [`canonical-api-cli-contract.md`](canonical-api-cli-contract.md)
- **Use-Case Model:** [`canonical-use-cases.md`](canonical-use-cases.md)
- **Domain Model:** [`canonical-domain-model.md`](canonical-domain-model.md)
- **PRD:** [`prd.md`](prd.md)
- **Status:** Canonical Reference (2026-11-01)

## 1. Verification Policy & Surface Legend

Per root planning policy ([`docs/prd.md` §8](../../prd.md)):
- **`backend-boundary` (`required`):** Rust `cargo test` suite verifying IPC DTO serialization, solver invocation, classifier routing, and persistence round-trips.
- **`frontend-integration` (`required`):** Vitest unit/integration suite verifying pure TypeScript snapping, topological member splitting, CAD transformations, internal hinge expansion, undo/redo revision monotonicity, and live-analysis coordinator state machine.
- **`end-to-end` (`catalog-only`):** User-visible workspace reachability and visual review gate on every UI slice.

---

## 2. Canonical Scenarios

### SC-001: Workspace Navigation to Mode 2 & Dynamic Viewport Grid
- **Purpose:** Verify Mode 2 is reachable from the top application header switcher and renders an interactive 2D SVG drafting canvas with pan, zoom, cursor readout, and selectable grid steps.
- **Preconditions:** OmniMechanics launches in the default workspace.
- **Action:** User clicks **Mode 2: Structural Modeling & Analysis** («Μοντελοποίηση & Ανάλυση Φορέων») in the top header switcher, switches the grid selector between `0.1 m`, `0.5 m`, and `1.0 m`, pans/zooms the canvas, and moves the cursor.
- **Expected Outcome:** The Mode 2 SVG canvas, tool palettes, property inspector, and status bar render in Greek or English (`el`/`en`); the dynamic grid updates to the chosen step (`0.1 m`, `0.5 m`, or `1.0 m`); and the live cursor readout displays world $(x, y)$ coordinates in metres with global $y$ pointing upward.
- **Covers:** PRD Goal 1 & 2, `DraftingViewport`.
- **Verification Surfaces:**
  - `backend-boundary`: `not-applicable`
  - `frontend-integration`: Vitest verifies world-to-screen / screen-to-world affine transforms and grid-step snapping math (`0.1 m`, `0.5 m`, `1.0 m`).
  - `end-to-end`: User switches between Mode 1, Mode 2, and Mode 3 in the top header and observes the Mode 2 canvas and bilingual controls.

### SC-002: Priority-Ordered Snapping, Ortho/Angle Guides & Exact Numeric Entry
- **Purpose:** Verify deterministic snap priority (`NodeSnap` $< 15\text{ px}$ $>$ `IntersectionSnap` $>$ `MidpointSnap` $>$ `OrthoLock`/`AngularGuideSnap` $>$ `GridSnap`) and keyboard numeric entry ($(x, y)$ or $(L, \theta)$).
- **Preconditions:** Mode 2 has `Node` `N1` at $(0, 0)$ and `FrameMember` `E1` from $(0, 0)$ to $(6, 0)$. `DrawFrameMember` is active.
- **Action:**
  1. User moves cursor within $12\text{ px}$ of `N1` (while a grid point is $4\text{ px}$ away) and clicks to anchor at `N1`.
  2. User moves cursor near the midpoint $(3, 0)$ of `E1` (within $10\text{ px}$) and observes the `MidpointSnap` badge.
  3. User toggles $45^\circ$ angular guide (or `OrthoLock` at $90^\circ$), observes the live $L/\theta$ dimension tooltip, and types `L=5.0, θ=90` (or `0, 5.0`) in the numeric entry bar.
- **Expected Outcome:** Step 1 selects `NodeSnap` (`N1`) over the closer grid point because `NodeSnap` has higher priority; Step 2 highlights `MidpointSnap` at $(3.00, 0.00)$; Step 3 places `N2` at exact coordinates $(0.00, 5.00)\text{ m}$ and creates `FrameMember` `E2` of length $5.00\text{ m}$ at $\theta = 90.0^\circ$. Attempting to confirm a second click at $(0.00, 0.00)$ is rejected with `ZERO_LENGTH_ELEMENT`.
- **Covers:** UC-Q01, UC-C01, INV-1, `SnappingPolicyService`.
- **Verification Surfaces:**
  - `backend-boundary`: `not-applicable`
  - `frontend-integration`: Vitest exercises `SnappingEngine.evaluateSnap` across all priority collisions (`< 15 px` node threshold, midpoint, $0^\circ/90^\circ/180^\circ/270^\circ$ ortho, $30^\circ/45^\circ/60^\circ$ angle guides, numeric parser, and `ZERO_LENGTH_ELEMENT` rejection).
  - `end-to-end`: User sketches a portal frame using magnetic node snap, ortho lock, live $L/\theta$ tooltips, and keyboard numeric entry.

### SC-003: Member-Member Intersection Snap with Automatic Topological Splitting
- **Purpose:** Verify that snapping to a member interior or drawing a member across an existing member automatically splits the intersected member(s) (`TopologicalMemberSplit`) while preserving properties and loads.
- **Preconditions:** `FrameMember` `E1` connects `N1` $(0, 0)$ to `N2` $(6, 0)$ with section `SEC1`, start moment release `true`, and uniform local load $q_y = -10\text{ kN/m}$ over $[0, 6\text{ m}]$.
- **Action:** User draws `FrameMember` `E_new` from $(3, -2)$ to $(3, 2)$ with intersection snap enabled.
- **Expected Outcome:**
  1. The intersection at $(3.0, 0.0)$ is detected; shared `Node` `N_int` is created at $(3.0, 0.0)$.
  2. `E1` is split into `E1a` (`N1` $\rightarrow$ `N_int`) and `E1b` (`N_int` $\rightarrow$ `N2`), both retaining `SEC1` and $q_y = -10\text{ kN/m}$ over their respective $3.0\text{ m}$ spans; `E1a.start` preserves `moment: true`, while the internal ends at `N_int` have `moment: false` (continuous).
  3. `E_new` is split at `N_int` into lower and upper segments connected at `N_int`.
  4. The entire gesture is recorded as a single undoable command (`revision` increments by 1).
- **Covers:** UC-C01, `TopologicalSplitService`, `MemberTopologicallySplit`.
- **Verification Surfaces:**
  - `backend-boundary`: Rust DSM test confirms the split 4-member cruciform model solves identically to an analytical continuous intersection.
  - `frontend-integration`: Vitest verifies `TopologicalSplitService` node insertion, ID allocation, end-release preservation, load partitioning, and single-step undo restoration.
  - `end-to-end`: User draws intersecting diagonals or T-junctions on the canvas and sees the shared intersection node created automatically.

### SC-004: Box Selection, CAD Transformations (Move, Copy, Mirror, Array, Delete) & Undo/Redo
- **Purpose:** Verify marquee box selection (Window vs. Crossing), geometric transformations, and monotonic `revision` increments across `Undo` and `Redo`.
- **Preconditions:** A single-bay portal frame (`N1`..`N4`, columns `E1`, `E2`, beam `E3`, supports `S1`, `S2`) exists at `revision: 5`.
- **Action:**
  1. User drags a left-to-right enclosed selection box over the portal frame and runs `LinearArray` with `count = 3`, `dx = 6.0 m, dy = 0.0 m` (with node welding enabled).
  2. User mirrors an inclined roof overhang across the symmetry line $x = 9.0\text{ m}$.
  3. User presses `Undo` (`Ctrl+Z`) twice and `Redo` (`Ctrl+Y`) once.
- **Expected Outcome:**
  1. `LinearArray` produces a 3-bay continuous portal frame ($4$ columns, $3$ beams) with coincident intermediate nodes welded at $x = 6.0\text{ m}$ and $x = 12.0\text{ m}$ (`revision: 6`).
  2. `Mirror` reflects the overhang geometry and preserves global load/roller directions (`revision: 7`).
  3. Two `Undo` steps restore the state before `Mirror` (`revision: 8`) and before `LinearArray` (`revision: 9`), and `Redo` re-applies `LinearArray` (`revision: 10`), keeping `revision` strictly increasing.
- **Covers:** UC-C02, UC-C05, `CadTransformationService`, `UndoRedoStack`.
- **Verification Surfaces:**
  - `backend-boundary`: `not-applicable`
  - `frontend-integration`: Vitest tests enclosed vs. crossing box selection, `Move`, `Copy`, `Mirror`, `LinearArray`, cascade `Delete`, and monotonic `revision` across undo/redo.
  - `end-to-end`: User selects a bay on the canvas, arrays it into a multi-bay frame, and steps backward/forward with Undo/Redo buttons.

### SC-005: Property Inspector, Internal Hinge Expansion, Rigid Offsets & Cable Input Modes
- **Purpose:** Verify property editing for sections/materials ($E$ in $\text{GPa}$), `RigidOffset`, node-level `InternalHingeGesture` expansion to $k-1$ member moment releases, and `Cable` specification ($L_0$, sag ratio, $T_0$).
- **Preconditions:** Three `FrameMember`s (`E1`, `E2`, `E3`) meet at unrestrained node `N2`. A `Cable` `E4` connects `N1` to `N3`.
- **Action:**
  1. User clicks `N2` with the `PlaceInternalHinge` tool.
  2. User selects `E1` in the Property Inspector, assigns custom material $E = 210\text{ GPa}$, and sets end `RigidOffset` $(d_x = 0.25\text{ m}, d_y = 0.0\text{ m})$.
  3. User selects `Cable` `E4` and switches `CableInputSpec` between `UnstressedLength (L0 = 6.02 m)`, `SagRatio (0.05)`, and `Pretension (T0 = 45 kN)`. Attempting to set `T0 = -10 kN` is rejected.
- **Expected Outcome:**
  1. Clicking `N2` sets `EndRelease.moment = true` on $2$ of the $3$ incident member ends at `N2` (keeping $1$ continuous to prevent a rotational singularity) and renders an internal hinge ring at `N2`. Clicking a cantilever tip ($k = 1$) is rejected with `INVALID_HINGE_TARGET`.
  2. Custom $E = 210\text{ GPa}$ is stored and converted to $2.10 \times 10^8\text{ kN/m}^2$ for the solver; `RigidOffset` is rendered as thickened rigid end stubs on `E1`.
  3. Valid cable modes update `E4`; negative pretension raises `INVALID_CABLE_SPEC`.
- **Covers:** UC-C03, INV-4, INV-6, BR-1, `InternalHingeExpansionPolicy`.
- **Verification Surfaces:**
  - `backend-boundary`: Rust unit tests verify DSM assembly with `EndRelease`, `RigidOffset`, and `CableInputSpec` variants.
  - `frontend-integration`: Vitest verifies $k-1$ moment release expansion on $k$-member joints, `INVALID_HINGE_TARGET` when $k < 2$, $E$ GPa conversion, and `INVALID_CABLE_SPEC` validation.
  - `end-to-end`: User places an internal hinge at a three-hinged arch crown and edits section/cable properties in the inspector.

### SC-006: Full Support & Load Palettes Without Spurious Member Splitting
- **Purpose:** Verify assignment of all support kinds (`Fixed`, `Pinned`, `Roller{angle}`, `Guided{angle}`, `Elastic{kx,ky,ktheta}`, `unilateral`) and all load kinds (`NodalLoad`, `MemberPointLoad`, `MemberDistributedLoad` in `Local`/`Global`/`Projected`, `SupportSettlement`, `ThermalLoad`, `self_weight`).
- **Preconditions:** A $6.0\text{ m}$ inclined `FrameMember` `E1` exists between `N1` and `N2` under `LoadCase` `LC1`.
- **Action:**
  1. User assigns `Fixed` support at `N1` and an inclined `Roller { angle_deg: 30.0, unilateral: true }` at `N2`.
  2. User places a `MemberPointLoad` ($P_y = -25\text{ kN}$) at distance $a = 2.0\text{ m}$ on `E1`, a trapezoidal `MemberDistributedLoad` ($q_1 = -5, q_2 = -15\text{ kN/m}$, `Projected`) from $a = 1.0\text{ m}$ to $b = 5.0\text{ m}$, a `ThermalLoad` ($\Delta T_{\text{uniform}} = +20^\circ\text{C}, \Delta T_{\text{gradient}} = +10^\circ\text{C}$), a `SupportSettlement` ($\Delta_y = -0.01\text{ m}$) at `N2`, and toggles `self_weight = true` on `LC1`.
  3. User attempts to place a `MemberPointLoad` at $a = 7.5\text{ m}$ ($> L$).
- **Expected Outcome:**
  1. `E1` remains a single unsplit element (`elements.length == 1`) with all loads attached to `E1`.
  2. SVG canvas renders distinct symbols for the $30^\circ$ unilateral roller, point load arrow at $2.0\text{ m}$, projected trapezoidal load comb, thermal badge, and settlement arrow.
  3. The out-of-bounds load at $7.5\text{ m}$ is rejected with `LOAD_OUT_OF_MEMBER_BOUNDS`.
- **Covers:** UC-C04, INV-3, Discovery Q8, Q17, Q36, Q38.
- **Verification Surfaces:**
  - `backend-boundary`: Rust DSM test verifies fixed-end force vectors for point load at $a$, trapezoidal local/global/projected loads, thermal loads, settlements, and self-weight.
  - `frontend-integration`: Vitest verifies load attachment preserves single-element topology and enforces `LOAD_OUT_OF_MEMBER_BOUNDS`.
  - `end-to-end`: User assigns supports and loads from the Mode 2 palettes and inspects their visual glyphs on the canvas.

### SC-007: Live Real-Time Linear DSM Feedback, Tension-Side M/Q/N Diagrams & Hover Probe
- **Purpose:** Verify that editing geometry, supports, or loads updates reactions, tension-side $M/Q/N$ diagrams, scaled deformed shape, extrema labels, hover cross-section probe, and end-force table within the live debounce window (~$150\text{ ms}$).
- **Preconditions:** Simply supported horizontal beam `E1` ($L = 6.0\text{ m}$) under uniform load $q_y = -10\text{ kN/m}$.
- **Action:** User increases $q_y$ to $-20\text{ kN/m}$, switches diagram view to $M$, and hovers the cursor at midspan $x = 3.0\text{ m}$ and quarter-span $x = 1.5\text{ m}$.
- **Expected Outcome:**
  1. Within ~$150\text{ ms}$ debounce + $< 50\text{ ms}$ solve time, reactions update to $R_{Ay} = R_{By} = 60.0\text{ kN}$.
  2. The parabolic bending moment diagram $M(x)$ is plotted **below the beam axis (on the bottom tension fibre side)** with positive extrema label $M_{\max} = +90.0\text{ kNm}$ at $x = 3.0\text{ m}$.
  3. Hovering at $x = 1.5\text{ m}$ displays the `CrossSectionHoverProbe` with $M(1.5) = +67.5\text{ kNm}$ (`tensionSide: 'Bottom'`), $Q(1.5) = +30.0\text{ kN}$, $N(1.5) = 0.0\text{ kN}$, and local deflection $u_y$; the Member End-Force Table shows exact start/end $(N, Q, M)$.
- **Covers:** UC-C06, UC-Q02, UC-Q03, BR-2, NFR-2.
- **Verification Surfaces:**
  - `backend-boundary`: Rust test verifies `frame_solve_linear` reactions, end forces, and analytical diagram sampling within $10^{-6}$ tolerance.
  - `frontend-integration`: Vitest verifies $150\text{ ms}$ debounce scheduling, tension-side SVG path orientation (`M > 0` plotted toward local $-y$ bottom fibre), and hover interpolation.
  - `end-to-end`: User drags a load or edits its value and watches the $M/Q/N$ diagram, deformed shape, and hover probe update live on the canvas.

### SC-008: Background Nonlinear Solve Progress, Replacement & Cancellation on Rapid Edit
- **Purpose:** Verify that nonlinear analyses (`SecondOrderPDelta`, catenary `Cable`, or `unilateral` supports) show an immediate linear preview, run in the background with an `analysis://progress` indicator, replace the canvas results upon convergence, and cancel/supersede stale jobs if the user edits the model mid-solve.
- **Preconditions:** A portal frame with `SecondOrderPDelta` active (or a structure containing a catenary `Cable`) is at `revision: 10`.
- **Action:**
  1. User edits a column load (`revision: 11`).
  2. While background job `J_11` is `Running` (at $45\%$ progress), user moves a node (`revision: 12`), triggering job `J_12`.
  3. Job `J_12` emits `analysis://progress` events and completes with `status: "Converged"` for `model_revision: 12`.
- **Expected Outcome:**
  1. At `revision: 11`, the linear solution for `revision: 11` renders immediately with a *"Linear Preview — Running P-Δ (45%)"* badge.
  2. When the user mutates to `revision: 12`, `frame_cancel_analysis_job` is called for `J_11`; any late event from `J_11` (`model_revision: 11 < 12`) is discarded (`STALE_REVISION_SUPERSEDED`).
  3. When `J_12` converges for `model_revision: 12`, the canvas seamlessly replaces the linear preview with the converged second-order / catenary `ResultSet` and displays the bilingual non-superposition notice.
- **Covers:** UC-C06, BR-3, BR-4, INV-5, Discovery Q12, Q14.
- **Verification Surfaces:**
  - `backend-boundary`: Rust test verifies background job cancellation token and `analysis://progress` payload revision tagging.
  - `frontend-integration`: Vitest tests `LiveAnalysisCoordinatorService` with fake timers and mocked IPC streams, asserting stale `model_revision` rejection and job cancellation on edit.
  - `end-to-end`: User enables P-Δ or sketches a cable, sees the progress indicator, and observes the nonlinear result replace the preview.

### SC-009: Kinematic Mechanism Detection & Animated Instability Mode
- **Purpose:** Verify that kinematically unstable structures are detected before solving, explained in Greek and English, and animated as a mechanism on the canvas without plotting bogus internal forces.
- **Preconditions:** User sketches a four-bar pin-jointed rectangular portal (four hinges, zero diagonal bracing) or a beam on a single roller support.
- **Action:** Live analysis runs for the current `StructuralModel`.
- **Expected Outcome:**
  1. `frame_solve_linear` returns `Diagnostics` with `is_mechanism: true`, the normalized `mechanism_mode` eigenvector, and bilingual explanation (`message_el` / `message_en`, plus static indeterminacy degree).
  2. Mode 2 hides $M/Q/N$ force diagrams, displays the prominent bilingual Mechanism Diagnostic Banner, and smoothly animates the structure oscillating along `mechanism_mode` on the SVG canvas.
- **Covers:** UC-C06, Discovery Q19, `KINEMATIC_MECHANISM_DETECTED`.
- **Verification Surfaces:**
  - `backend-boundary`: Rust test verifies singular stiffness / nullspace mechanism detection and eigenvector extraction on unstable frames.
  - `frontend-integration`: Vitest verifies `ResultOverlayState` disables force diagrams and activates mechanism animation state when `diagnostics.is_mechanism == true`.
  - `end-to-end`: User removes a required support or adds an excess hinge and watches the mechanism animation and bilingual explanation on the canvas.

### SC-010: LoadCase, EN 1990/1998 Combination & Governing Envelope Inspection
- **Purpose:** Verify switching between individual `LoadCase`s, generated/manual Eurocode `Combination`s, and `Envelope` ($\max/\min$) views with per-point governing combination identification.
- **Preconditions:** Model has `LC1` (`Permanent` $G$), `LC2` (`Imposed` Category A $Q$), and `LC3` (`Wind` $W$), with Greek National Annex (`GR`) ULS/SLS combinations generated via `actions_generate_combinations`.
- **Action:** User selects `Envelope (ULS_STR)` in the Result Target Selector, turns on the $M$ envelope diagram, and hovers over column base `E1` ($x = 0$) and beam midspan `E2` ($x = 3.0\text{ m}$).
- **Expected Outcome:**
  1. The canvas renders dual upper/lower ($\max/\min$) tension-side moment envelope curves shaded between the bounding values.
  2. The `CrossSectionHoverProbe` and Member End-Force Table display both $M_{\max}$ and $M_{\min}$ at the hovered station together with the exact governing combination ID and formula (e.g., `CO2: 1.35·LC1 + 1.50·LC2 + 0.90·LC3` for $M_{\max}$ and `CO5: 1.00·LC1 + 1.50·LC3` for $M_{\min}$).
- **Covers:** UC-C07, UC-Q02, UC-Q03, Discovery Q15, Q16, Q37.
- **Verification Surfaces:**
  - `backend-boundary`: Rust tests verify `actions_generate_combinations` and `Envelope` governing combination tracking across stations.
  - `frontend-integration`: Vitest verifies target selector switching (`LoadCase` $\leftrightarrow$ `Combination` $\leftrightarrow$ `Envelope`) and hover probe governing-combination binding.
  - `end-to-end`: User switches the toolbar selector to `Envelope (ULS)` and inspects governing combinations along members.

### SC-011: Mode 3 Handoff for Eligible Structure ("Open Step-by-Step")
- **Purpose:** Verify that when a user sketches a structure eligible for a classical method, the classifier enables **"Open Step-by-Step"** and hands the model off to Mode 3 Teaching Studio.
- **Preconditions:** User sketches a three-hinged portal frame (or a 2-span continuous beam, or a pin-jointed Pratt truss) in Mode 2.
- **Action:** `teaching_classify_model` runs automatically; user clicks **"Open Step-by-Step in Teaching Studio"** («Άνοιγμα Βήμα-προς-Βήμα στο Teaching Studio») and selects an eligible method (e.g., `IsostaticBeamFrame` or `ContinuousBeamClapeyronCross`).
- **Expected Outcome:** The classifier badge marks the classical method `eligible: true`; clicking the handoff button invokes `teaching_open_step_by_step` with the shared `StructuralModel`, transitions the application to Mode 3, and displays the full step-by-step analytical derivation and progressive $M/Q/N$ diagrams.
- **Covers:** UC-C08, UC-Q04, Discovery Q4, Q22, Q25.
- **Verification Surfaces:**
  - `backend-boundary`: Rust test verifies `teaching_classify_model` and `teaching_open_step_by_step` accept the `StructuralModel` and produce derivation steps.
  - `frontend-integration`: Vitest verifies `TeachingHandoffCoordinatorService` enables the classical handoff action and passes the active `StructuralModel` and `LoadCaseId`.
  - `end-to-end`: User sketches a continuous beam in Mode 2, clicks "Open Step-by-Step", and steps through the derivation in Mode 3.

### SC-012: Mode 3 Handoff Blocked for Ineligible Structure (Cables / High Hyperstatic Degree) with DSM Walkthrough Fallback
- **Purpose:** Verify that when a structure is ineligible for classical methods (e.g., contains a `Cable` element or hyperstatic degree $> 3$), Mode 2 explains why in Greek and English and offers the generic **DSM Walkthrough**.
- **Preconditions:** User adds a `Cable` element `E_cable` to a frame in Mode 2 (or sketches a 4-story hyperstatic frame of degree $12$).
- **Action:** User inspects the Mode 3 handoff control in the Mode 2 toolbar.
- **Expected Outcome:**
  1. Classical methods are marked `eligible: false` in `MethodEligibility`.
  2. The UI displays a clear bilingual explanation (e.g., *"Classical hand-calculation methods are unavailable because the structure contains nonlinear catenary cables (E_cable). / Οι κλασικές αναλυτικές μέθοδοι δεν είναι διαθέσιμες επειδή ο φορέας περιέχει μη γραμμικά καλώδια αλυσοειδούς."*).
  3. The **"Open DSM Walkthrough"** button remains active and, when clicked, opens Mode 3 with the Direct Stiffness Method step-by-step matrices ($k_{\text{local}}, T, K_{\text{global}}$, boundary conditions, nodal displacements, member end forces).
- **Covers:** UC-C08, UC-Q04, BR-5, Discovery Q4, Q5, Q39.
- **Verification Surfaces:**
  - `backend-boundary`: Rust test verifies `teaching_classify_model` rejects cable-bearing models for classical solvers while keeping `dsm_walkthrough_available: true`.
  - `frontend-integration`: Vitest verifies bilingual ineligibility reason binding and DSM Walkthrough fallback state.
  - `end-to-end`: User adds a cable to a beam, reads the bilingual ineligibility explanation popover, and opens the DSM Walkthrough in Mode 3.

### SC-013: Opening a Mode 3 Teaching Preset as an Editable Sketch in Mode 2
- **Purpose:** Verify bidirectional Mode 3 $\rightarrow$ Mode 2 flow by loading any Mode 3 teaching preset into Mode 2 as an editable `StructuralModel`.
- **Preconditions:** User is viewing a teaching preset (e.g., *"3-Span Continuous Beam"* or *"Pratt Roof Truss"*) in Mode 3.
- **Action:** User clicks **"Edit in Mode 2 Sketcher"** («Επεξεργασία στον Σχεδιαστή Mode 2») and then adds a new point load and moves a support on the Mode 2 canvas.
- **Expected Outcome:** Mode 2 opens with the preset converted into a complete `StructuralModel` (`revision: 1`), live DSM diagrams match the preset's analytical solution, and subsequent edits in Mode 2 immediately re-solve and update the diagrams.
- **Covers:** UC-C09, Discovery Q22.
- **Verification Surfaces:**
  - `backend-boundary`: Rust test verifies `teaching_preset_to_structural_model` produces a valid `StructuralModel` whose `frame_solve_linear` output matches the Mode 3 analytical solution.
  - `frontend-integration`: Vitest verifies `ModelerStore` initialization from a preset `StructuralModel` and subsequent mutation/undo behavior.
  - `end-to-end`: User clicks "Edit in Mode 2 Sketcher" from Mode 3 and modifies the structure live on the Mode 2 canvas.

### SC-014: Project Persistence Round-Trip & Publication SVG/PNG Diagram Export
- **Purpose:** Verify saving and restoring the complete Mode 2 workspace via `ProjectDocument` (`.bem`) and exporting SVG/PNG diagrams.
- **Preconditions:** User has modeled a frame with custom sections, releases, cables, load cases, and Eurocode combinations at `revision: 18` with the tension-side $M$ diagram displayed.
- **Action:**
  1. User invokes **Save Project** (`project_save_workspace`), resets the workspace, and invokes **Open Project** (`project_load_workspace`).
  2. User clicks **Export Diagram (SVG / PNG)**.
- **Expected Outcome:**
  1. Reloading the `.bem` file restores the exact `StructuralModel` (nodes, elements, releases, rigid offsets, cables, supports, load cases), `Combination`s, and `DraftingViewport`, and immediately recomputes matching live results.
  2. SVG and PNG exports generate standalone, self-contained files containing the structural geometry, support symbols, tension-side $M/Q/N$ curves, extrema labels, scale bar, and bilingual legend.
- **Covers:** UC-C10, Discovery Q24.
- **Verification Surfaces:**
  - `backend-boundary`: Rust test verifies lossless serde JSON round-trip of `ProjectDocument.structural`.
  - `frontend-integration`: Vitest verifies deterministic SVG serializer output (including tension-side diagram paths and extrema labels) and store hydration from `ProjectDocument`.
  - `end-to-end`: User saves/reloads a project and exports an SVG diagram from the Mode 2 toolbar.

---

## 3. Stateful & Concurrent Interaction Matrix Coverage

| Stateful Interaction | Coverage Decision | Mapped Scenario |
| --- | --- | --- |
| User mutates model while background nonlinear `AnalysisJob` is `Running` | Covered (cancels in-flight job, discards stale events) | `SC-008` |
| User performs `Undo` / `Redo` while background `AnalysisJob` is `Running` | Covered (`Undo`/`Redo` increments `revision` monotonically, superseding stale jobs) | `SC-004`, `SC-008` |
| Drawing an element across multiple existing members with loads | Covered (atomic topological split + load partitioning in one undoable command) | `SC-003` |
| Model transitions from stable to kinematic mechanism upon support deletion or hinge placement | Covered (suppresses force diagrams, animates mechanism mode) | `SC-005`, `SC-009` |
| Model transitions from linear-eligible to cable-bearing nonlinear | Covered (disables superposition, activates background catenary solve, updates Mode 3 classifier badge) | `SC-008`, `SC-012` |
