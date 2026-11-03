# Canonical API, IPC & Store Contract: Mode 2 Structural Frame Modeler (`cap-sketcher-frame-modeler`)

- **Capability:** [`cap-sketcher-frame-modeler`](../../../.okf/capabilities/cap-sketcher-frame-modeler.md)
- **Use-Case Model:** [`canonical-use-cases.md`](canonical-use-cases.md)
- **Domain Model:** [`canonical-domain-model.md`](canonical-domain-model.md)
- **Cross-Capability Conventions:** [`cross-capability-conventions.md`](../cross-capability-conventions.md)
- **Status:** Canonical Reference (2026-11-01)

## 1. Purpose & Contract Goals

This document specifies the canonical external contract for `cap-sketcher-frame-modeler`. Because OmniMechanics is a Tauri + Angular + Rust desktop application with required `frontend-integration` (Vitest) and `backend-boundary` (`cargo test`) verification surfaces, this contract defines:
1. **Transport-Neutral Conventions, Identifiers & Error Envelope** (governed by [`cross-capability-conventions.md`](../cross-capability-conventions.md)).
2. **Frontend Pure-TS Store & Geometry API (`ModelerWorkspaceContract`):** The deterministic TypeScript interface exercised by the UI and verified under Vitest.
3. **Tauri IPC Command & Event Mapping (`frame_*`, `actions_*`, `teaching_*`, `project_*`):** The boundary between Mode 2 and the Rust core.
4. **CLI / Headless Verification Mapping:** Equivalent headless invocation semantics for automated benchmark and contract testing.

## 2. Transport-Neutral Conventions

- **Units:** SI internally (`m`, `kN`, `kNm`, `kN/m`, `kN/m²`, `°C`, `rad`). User inputs for elastic modulus $E$ are entered in $\text{GPa}$ in the UI and converted to $\text{kN/m}^2$ ($\times 10^6$) when populating `Material.e_kpa` or at the solver boundary.
- **Coordinate System & Signs:** Global $x$ right, global $y$ up, counter-clockwise rotation positive. Axial force $N > 0$ in tension; bending moment $M > 0$ when local bottom ($-y$) fibre is in tension; diagrams plotted on the tension side.
- **Stable String IDs:** Per-model monotonic string identifiers: `N1`, `N2` (`Node`), `E1`, `E2` (`Element`), `S1` (`Support`), `LC1` (`LoadCase`), `L1` (`Load`), `CO1` (`Combination`), `SEC1` (`Section`), `MAT1` (`Material`).
- **Revision Monotonicity:** `StructuralModel.revision: u64` starts at `1` and increments by `+1` on every mutation (including `Undo` and `Redo`).
- **Canonical Error Envelope:**
  ```json
  {
    "code": "LOAD_OUT_OF_MEMBER_BOUNDS",
    "message_el": "Η απόσταση φορτίου υπερβαίνει το μήκος του μέλους (L = 6.00 m).",
    "message_en": "Load distance exceeds member length (L = 6.00 m).",
    "details": { "element_id": "E1", "member_length_m": 6.0, "requested_a_m": 7.2 }
  }
  ```

## 3. Canonical Payload Shapes

### 3.1 `StructuralModel` & Modeler Section in `ProjectDocument`
```json
{
  "revision": 14,
  "nodes": [
    { "id": "N1", "x": 0.0, "y": 0.0 },
    { "id": "N2", "x": 0.0, "y": 4.0 },
    { "id": "N3", "x": 6.0, "y": 4.0 }
  ],
  "materials": [
    { "id": "MAT1", "name": "S355", "class": "Steel", "e_gpa": 210.0, "g_gpa": 81.0, "density_kn_m3": 78.5, "alpha_t": 1.2e-5 }
  ],
  "sections": [
    { "id": "SEC1", "name": "IPE 300", "shape": "ISection", "area_m2": 0.00538, "inertia_m4": 0.00008356, "depth_m": 0.30 }
  ],
  "elements": [
    {
      "id": "E1",
      "kind": {
        "type": "FrameMember",
        "start_node": "N1",
        "end_node": "N2",
        "section_id": "SEC1",
        "material_id": "MAT1",
        "releases": {
          "start": { "axial": false, "shear": false, "moment": false },
          "end": { "axial": false, "shear": false, "moment": true }
        },
        "rigid_offsets": {
          "start": { "dx": 0.0, "dy": 0.0 },
          "end": { "dx": 0.0, "dy": -0.15 }
        }
      }
    },
    {
      "id": "E2",
      "kind": {
        "type": "Cable",
        "start_node": "N1",
        "end_node": "N3",
        "section_id": "SEC_CABLE",
        "material_id": "MAT_CABLE",
        "input_spec": { "mode": "Pretension", "t0_kn": 50.0 }
      }
    }
  ],
  "supports": [
    { "id": "S1", "node_id": "N1", "kind": { "type": "Fixed" }, "unilateral": false },
    { "id": "S2", "node_id": "N3", "kind": { "type": "Roller", "angle_deg": 30.0 }, "unilateral": true }
  ],
  "load_cases": [
    { "id": "LC1", "name": "Dead Load G", "action_type": { "type": "Permanent" }, "self_weight": true }
  ],
  "loads": [
    {
      "id": "L1",
      "load_case_id": "LC1",
      "kind": {
        "type": "MemberDistributedLoad",
        "element_id": "E1",
        "q1x": 0.0, "q1y": -12.0,
        "q2x": 0.0, "q2y": -12.0,
        "start_a": 0.0, "end_b": 4.0,
        "frame": "Local"
      }
    }
  ],
  "masses": []
}
```

### 3.2 `SnapCandidate` & `CrossSectionHoverProbe` (Pure-TS Frontend Contract)
```typescript
export interface SnapCandidateDto {
  readonly worldX: number;
  readonly worldY: number;
  readonly screenDistancePx: number;
  readonly kind: 'NodeSnap' | 'IntersectionSnap' | 'MidpointSnap' | 'OrthoLock' | 'AngularGuideSnap' | 'GridSnap' | 'NumericEntry';
  readonly targetNodeId?: string;
  readonly targetElementIds: readonly string[];
  readonly guideAngleDeg?: number;
  readonly tooltip: { readonly lengthM: number; readonly angleDeg: number; readonly readout: string };
}

export interface CrossSectionHoverProbeDto {
  readonly elementId: string;
  readonly localXM: number;
  readonly ratioXi: number; // x / L in [0, 1]
  readonly axialNKn: { readonly value: number; readonly min?: number; readonly max?: number; readonly govMinComboId?: string; readonly govMaxComboId?: string };
  readonly shearQKn: { readonly value: number; readonly min?: number; readonly max?: number; readonly govMinComboId?: string; readonly govMaxComboId?: string };
  readonly momentMKnm: { readonly value: number; readonly tensionSide: 'Bottom' | 'Top' | 'Neutral'; readonly min?: number; readonly max?: number; readonly govMinComboId?: string; readonly govMaxComboId?: string };
  readonly displacementLocalM: { readonly ux: number; readonly uy: number; readonly thetaRad: number };
}
```

## 4. Canonical Tauri IPC & Pure-TS Command Mapping

### 4.1 Frontend Pure-TS Commands (Tested by Vitest under `frontend-integration`)
| Use Case | Pure-TS Store / Service Method | Input | Output |
| --- | --- | --- | --- |
| **UC-Q01** | `SnappingEngine.evaluateSnap(cursor, anchor, model, viewport, settings)` | World/screen coordinates + settings | `SnapCandidateDto` |
| **UC-C01** | `ModelerStore.sketchElement(cmd)` | `SketchElementInput` | `{ model: StructuralModel, splitEvents: MemberSplitEvent[] }` |
| **UC-C02** | `ModelerStore.applyTransform(cmd)` | `CadTransformInput` (`Move`, `Copy`, `Mirror`, `LinearArray`, `Delete`) | `StructuralModel` (`revision + 1`) |
| **UC-C03** | `ModelerStore.applyInternalHinge(nodeId, toggle)` | `nodeId: string` | `StructuralModel` (expands to $k-1$ `EndRelease.moment`) |
| **UC-C03** | `ModelerStore.updateElementProperties(cmd)` | Section, material, releases, rigid offsets, cable spec | `StructuralModel` (`revision + 1`) |
| **UC-C04** | `ModelerStore.assignSupportOrLoad(cmd)` | Support or Load DTO | `StructuralModel` (`revision + 1`) |
| **UC-C05** | `ModelerStore.undo()` / `ModelerStore.redo()` | — | `StructuralModel` (`revision + 1`) |

### 4.2 Tauri IPC Commands & Events (Frontend $\leftrightarrow$ Rust Core)
All Tauri commands follow the prefix rule in [`cross-capability-conventions.md`](../cross-capability-conventions.md):

| IPC Command / Event | Direction | Owner | Request Payload | Response / Event Payload |
| --- | --- | --- | --- | --- |
| `frame_solve_linear` | Invoke | `[frame]` | `{ model: StructuralModel, target: ResultTarget, combinations: Vec<Combination> }` | `{ model_revision: u64, result_set?: ResultSet, envelope?: Envelope, diagnostics: Diagnostics }` |
| `frame_start_analysis_job` | Invoke | `[frame]` | `{ model: StructuralModel, analysis_kind: AnalysisKind, target: ResultTarget, combinations: Vec<Combination>, seismic_settings?: SeismicSettings }` | `{ job_id: String, model_revision: u64, status: "Queued" \| "Running" }` |
| `frame_cancel_analysis_job` | Invoke | `[frame]` | `{ job_id: String }` | `{ job_id: String, status: "Cancelled" }` |
| `analysis://progress` | Event | `[frame]` | — | `{ job_id: String, model_revision: u64, analysis_kind: AnalysisKind, status: JobStatus, progress_pct: f64, iteration: u32, residual?: f64, result_set?: ResultSet, envelope?: Envelope, diagnostics?: Diagnostics }` |
| `actions_generate_combinations` | Invoke | `[actions]` | `{ load_cases: Vec<LoadCase>, annex: NationalAnnexProfile, uls_rule: String, seismic_settings?: SeismicSettings }` | `{ combinations: Vec<Combination>, spectrum?: DesignSpectrum }` |
| `teaching_classify_model` | Invoke | `[teaching]` | `{ model: StructuralModel, load_case_id: String }` | `MethodEligibility` (`{ model_revision, methods: Vec<MethodStatusDto>, dsm_walkthrough_available: bool, indeterminacy_degree: i32 }`) |
| `teaching_open_step_by_step` | Invoke | `[teaching]` | `{ model: StructuralModel, load_case_id: String, method_id: String }` | `PedagogicalDerivationResponse` (transitions UI to Mode 3) |
| `teaching_preset_to_structural_model` | Invoke | `[teaching]` | `{ preset_id: String }` | `{ model: StructuralModel }` (opens preset in Mode 2) |
| `project_save_workspace` / `project_load_workspace` | Invoke | `[persistence]` | `{ path: String, document?: ProjectDocument }` | `ProjectDocument` |

## 5. Error Codes Owned or Surface-Handled by Mode 2

| Error Code | Owner | Trigger Condition |
| --- | --- | --- |
| `ZERO_LENGTH_ELEMENT` | `[modeler]` | Start and end points of drawn element have distance $< 10^{-4}\text{ m}$. |
| `DUPLICATE_ELEMENT` | `[modeler]` | Identical element kind already connects the same unordered pair of nodes. |
| `INVALID_HINGE_TARGET` | `[modeler]` | `InternalHingeGesture` applied to a node with $< 2$ incident `FrameMember`s. |
| `LOAD_OUT_OF_MEMBER_BOUNDS` | `[modeler]` / `[frame]` | Point/distributed load distance $a < 0$, $a > L$, or $a \ge b$. |
| `INVALID_CABLE_SPEC` | `[modeler]` / `[frame]` | Cable $L_0 \le 0$, sag ratio $\notin (0, 0.5)$, or pretension $T_0 \le 0$. |
| `EMPTY_SELECTION` | `[modeler]` | Transformation (`Move`, `Copy`, `Mirror`, `LinearArray`, `Delete`) invoked with empty selection. |
| `DEGENERATE_MIRROR_AXIS` | `[modeler]` | Mirror axis points $p_1, p_2$ are closer than $10^{-4}\text{ m}$. |
| `KINEMATIC_MECHANISM_DETECTED` | `[frame]` | Structure is kinematically unstable; accompanied by `Diagnostics.mechanism_mode`. |
| `STALE_REVISION_SUPERSEDED` | `[modeler]` / `[frame]` | Background job result ignored because `job.model_revision < model.revision`. |
| `INELIGIBLE_TEACHING_METHOD` | `[teaching]` | User attempts classical Mode 3 handoff on an ineligible model (e.g., model with cables). |

## 6. CLI & Headless Parity Mapping

For automated `backend-boundary` and `frontend-integration` verification without manual GUI interaction:
- **Vitest Headless Runner (`pnpm test:unit` / `vitest run`):** Exercises `SnappingEngine`, `TopologicalSplitService`, `InternalHingeExpansionPolicy`, `CadTransformationService`, `ModelerStore`, and `LiveAnalysisCoordinatorService` against deterministic JSON fixtures.
- **Rust Benchmark CLI (`cargo test --package bem_app_lib frame::`):** Executes the exact `StructuralModel` JSON payloads produced by `ModelerStore` against `frame_solve_linear`, `frame_start_analysis_job`, and `teaching_classify_model`.

## 7. Minimum First-Slice Surface (Mode 2 Slice 1)

In accordance with [`docs/prd.md` §5](../../prd.md), Delivery Slice 1 requires:
1. Header switcher entry for Mode 2 ("Structural Modeling & Analysis").
2. Full 2D SVG drafting canvas, snapping engine (grid, node $< 15\text{ px}$, midpoint, intersection with auto-split, ortho, angle, numeric entry), CAD transforms (`Move`, `Copy`, `Mirror`, `LinearArray`, `Delete`), box selection, and undo/redo stack verified under Vitest.
3. Placement and property inspection of `FrameMember` (with `EndRelease`, `InternalHingeGesture` expansion, and `RigidOffset`), `TrussBar`, and `Spring`, plus all `Support` types and single-`LoadCase` loads.
4. Live debounced `frame_solve_linear` integration showing reactions, tension-side $M/Q/N$ diagrams, scaled deformed shape, extrema, hover inspection, end-force table, and animated mechanism diagnostics.
