# Canonical Use-Case & Application-Service Model: 2D Frame Analysis Engine (`cap-frame-analysis-engine`)

- **Capability Owner:** `cap-frame-analysis-engine`
- **Upstream Artifacts:** [domain-glossary.md](domain-glossary.md), [prd.md](prd.md), [canonical-domain-model.md](canonical-domain-model.md)
- **Cross-Capability Baseline:** [cross-capability-conventions.md](../cross-capability-conventions.md)

---

## 1. Application-Layer Goals & Principles

1. **Dual-Speed Execution Architecture:**
   - **Immediate Synchronous Path (< 50 ms):** Every model edit or linear static query runs synchronously (`SolveLinearStatic`, `EvaluateKinematicDiagnostics`), returning immediate feedback for Mode 2 live editing. Even when a model contains `Cable` or unilateral elements, an initial linearized trial estimate (cables treated as secant truss bars at $T_0$ or chord stiffness, unilateral supports treated as bidirectional) can be returned synchronously while the exact nonlinear job starts in the background.
   - **Cancellable Asynchronous Job Path (< 2 s / combination):** Nonlinear (`SecondOrderPDelta`, `LinearBuckling`, exact `Cable` Newton-Raphson, unilateral active-set) and dynamic (`Modal`, `ResponseSpectrum`, `LateralForce`) analyses run in a background worker pool with cooperative cancellation and `analysis://progress` telemetry.
2. **Strict Revision Fencing:** Every command and query operates on an explicit `StructuralModel` snapshot at `revision`. Stale background jobs (`job.model_revision < current_revision`) are automatically marked `Superseded` and never overwrite newer results.
3. **Architecture Neutrality:** Use cases are defined as pure application intents independent of Tauri IPC or CLI flags.

---

## 2. Application Service Boundaries

| Application Service | Responsibility |
| --- | --- |
| **`StructuralModelCatalogService`** | Manages the active `StructuralModel` state, atomic model mutations (`revision` bumping), and built-in/custom `Section` and `Material` libraries. |
| **`KinematicDiagnosticsAppService`** | Evaluates DOF connectivity, static indeterminacy $n_s$, kinematic instability $n_k$, and extracts normalized `MechanismMode` vectors for canvas animation. |
| **`LinearFrameAnalysisService`** | Executes synchronous linear static DSM solves across `LoadCase`s, superposes linear `Combination`s, computes analytical $M/Q/N$ diagrams, and builds linear `Envelope`s. |
| **`NonlinearAndDynamicJobService`** | Orchestrates background `AnalysisJob`s for unilateral contact, $P\text{-}\Delta$, linear buckling, exact Irvine catenary cables, modal analysis, lateral force, and response spectrum (SRSS/CQC). |
| **`DsmWalkthroughAppService`** | Generates pedagogical step-by-step DSM trace data (`DsmWalkthroughData`) with bilingual explanations (`el`/`en`) and full matrix output for small models ($\le 6$ nodes). |

---

## 3. Canonical Commands & Queries

### 3.1 Model & Library Commands/Queries (`StructuralModelCatalogService`)

#### `UpsertStructuralModel` (Command)
- **Intent:** Replace or apply a batch mutation to the active `StructuralModel` (used by Mode 2 modeler store on ~150 ms debounce or project load), incrementing `revision`.
- **Inputs:** `model_payload: StructuralModelInput`, `expected_previous_revision: Option<u64>`.
- **Preconditions:** All element node references, section IDs, material IDs, and support node IDs exist within `model_payload`; element clear lengths $L_{\text{eff}} \ge 10^{-4}\text{ m}$.
- **Application Responsibilities:**
  1. Validate structural referential integrity and physical bounds ($E > 0, A > 0, I > 0$).
  2. Assign `new_revision = current_revision + 1` (or initialize to `1`).
  3. Signal `NonlinearAndDynamicJobService` to mark any running `AnalysisJob` with `model_revision < new_revision` as `Superseded`.
- **Outputs:** `StructuralModelSnapshot { model, revision, diagnostics_summary }`.

#### `ListMaterialAndSectionCatalog` (Query)
- **Intent:** Retrieve built-in standard libraries (Concrete C20/25–C50/60, Steel S235/275/355, Timber, CableSteel; IPE, HEA, HEB, Hollow, Rectangular, Circular) plus custom items in the current model.
- **Outputs:** `CatalogSnapshot { materials: Vec<Material>, sections: Vec<Section> }`.

---

### 3.2 Diagnostics & Immediate Linear Queries (`KinematicDiagnosticsAppService` & `LinearFrameAnalysisService`)

#### `EvaluateKinematicDiagnostics` (Query)
- **Intent:** Inspect stability, static indeterminacy $n_s$, and mechanism modes without requiring applied loads.
- **Inputs:** `model_revision: u64` (or inline `StructuralModel`).
- **Outputs:** `Diagnostics` containing `static_indeterminacy_degree`, `kinematic_instability_degree`, `is_stable`, `mechanism_modes: Vec<MechanismMode>`, and bilingual warnings.

#### `SolveLinearStatic` (Query / Synchronous Solve)
- **Intent:** Compute immediate linear static DSM results for all (or selected) `LoadCase`s and linear `Combination`s, plus optional `Envelope`s.
- **Inputs:**
  - `model_revision: u64`
  - `load_case_ids: Vec<String>` (empty = all)
  - `combinations: Vec<Combination>` (passed from `cap-eurocode-actions-combinations`)
  - `compute_envelope: bool`
  - `station_count: usize` (default `21` per member, augmented by load discontinuities and local extrema)
- **Preconditions:** `model.revision == model_revision`; model has at least one element and one support restraint.
- **Application Responsibilities:**
  1. Run `EvaluateKinematicDiagnostics`. If `kinematic_instability_degree > 0`, return `KINEMATIC_INSTABILITY` alongside `Diagnostics` (with `mechanism_modes` populated).
  2. Assemble global elastic stiffness $\mathbf{K}_{ff}$ and factorize once via `faer` Cholesky/LDLT.
  3. Solve all unit `LoadCase` right-hand-side vectors $\mathbf{F}_f^{(lc)}$ in batch.
  4. If the model is strictly linear (`superposition_valid == true`), compute `Combination` `ResultSet`s by linear superposition of `LoadCase` displacements and internal force polynomials, and compute `Envelope`s.
  5. If the model contains nonlinear features (`Cable`, `unilateral` support, `TensionOnly`/`CompressionOnly` bar), mark `linear_approximation_only: true` on the synchronous response so the UI knows a background nonlinear job is required for exact combination results.
- **Outputs:** `LinearSolveResponse { revision, diagnostics, load_case_results: Vec<ResultSet>, combination_results: Vec<ResultSet>, envelope: Option<Envelope>, linear_approximation_only: bool }`.

#### `InspectMemberPoint` (Query)
- **Intent:** Evaluate exact analytical $N(x), Q(x), M(x), u_x(x), u_y(x), \theta_z(x)$ and governing combination at an arbitrary coordinate $x \in [0, L]$ along an element for canvas hover tooltips.
- **Inputs:** `model_revision: u64`, `target_id: String` (LoadCase, Combination, or `"ENVELOPE"`), `element_id: String`, `x_local_m: f64`.
- **Outputs:** `MemberPointInspection { x_local_m, n_kn, q_kn, m_knm, ux_m, uy_m, theta_rad, governing_combination_id: Option<String> }`.

---

### 3.3 Asynchronous Nonlinear, Stability & Dynamic Use Cases (`NonlinearAndDynamicJobService`)

#### `StartAnalysisJob` (Command)
- **Intent:** Launch a background cancellable solver job for `SecondOrderPDelta`, `LinearBuckling`, nonlinear `LinearStatic` (with `Cable` or unilateral elements), `Modal`, `LateralForce`, or `ResponseSpectrum`.
- **Inputs:**
  - `model_revision: u64`
  - `kind: AnalysisKind`
  - `target_load_case_ids: Vec<String>`
  - `target_combinations: Vec<Combination>`
  - `compute_envelope: bool`
  - `options: AnalysisOptions` (e.g. `max_iterations`, `tolerance`, `buckling_modes_count`, `modal_mass_source` $G + \psi_2 Q$, `consistent_mass: bool`, `min_effective_mass_ratio: f64` default `0.90`, `design_spectrum: Option<DesignSpectrum>`, `modal_combination_rule: Srss | Cqc`, `damping_ratio: f64` default `0.05`, `include_vertical_spectrum: bool`).
- **Application Responsibilities:**
  1. Verify `model_revision == current_revision`; otherwise reject with `STALE_MODEL_REVISION`.
  2. Supersede any active job for the same capability session.
  3. Spawn background task with cancellation token; emit initial `analysis://progress` event (`status: Running, percent: 0.0`).
  4. **Per-Analysis Execution:**
     - **Nonlinear Static / $P\text{-}\Delta$ / Cables / Unilateral:** Disable superposition (`superposition_valid = false`). Solve each target `LoadCase` and each `Combination` independently from zero or chord trial state. Emit progress after each combination converges. Assemble `Envelope` across converged `Combination` `ResultSet`s.
     - **Linear Buckling:** Solve reference static axial forces $\mathbf{N}_{\text{ref}}$ for each target, assemble $\mathbf{K}_G(\mathbf{N}_{\text{ref}})$, extract lowest positive $\alpha_{cr,m}$ and buckling mode vectors.
     - **Modal:** Assemble $\mathbf{M}$ from $G + \sum \psi_{2,i} Q_i$ + `LumpedMass`, solve eigenmodes iteratively until $\sum M_{\text{eff},x} / M_{\text{tot},x} \ge 0.90$ and $\sum M_{\text{eff},y} / M_{\text{tot},y} \ge 0.90$ (or all modes extracted).
     - **Lateral Force / Response Spectrum:** Run or reuse `Modal`, evaluate $S_d(T_i)$, compute equivalent static forces (`LateralForce`) or combine modal peak responses via SRSS/CQC (`ResponseSpectrum`).
  5. Store final `AnalysisJobResult` keyed by `job_id` and emit completion event.
- **Outputs:** `AnalysisJobHandle { job_id, model_revision, kind, status: Queued }`.

#### `CancelAnalysisJob` (Command)
- **Intent:** Cooperatively abort a running background job.
- **Inputs:** `job_id: String`.
- **Outputs:** `AnalysisJobHandle { job_id, status: Cancelled }`.

#### `GetAnalysisJobResult` (Query)
- **Intent:** Retrieve the converged `ResultSet`s, `Envelope`, and `Diagnostics` of a completed `AnalysisJob`.
- **Inputs:** `job_id: String`.
- **Outputs:** `AnalysisJobResult { job, diagnostics, results: Vec<ResultSet>, envelope: Option<Envelope> }`.

---

### 3.4 Pedagogical Walkthrough Use Cases (`DsmWalkthroughAppService`)

#### `GenerateDsmWalkthrough` (Query)
- **Intent:** Produce the step-by-step Direct Stiffness Method derivation for a selected `LoadCase` or linear `Combination` for Mode 3 inspection.
- **Inputs:** `model_revision: u64`, `target_id: String`.
- **Preconditions:** Model is kinematically stable (`is_stable == true`) and does not rely on nonlinear iterative convergence for the requested walkthrough (or explains that walkthrough shows the initial linear tangent stiffness).
- **Outputs:** `DsmWalkthroughData` containing DOF map, per-element $\mathbf{k}_e, \mathbf{T}_e, \mathbf{f}_{0,e}$, global $\mathbf{K}, \mathbf{F}$ (full dense matrices when `nodes.len() <= 6`, sparse block summary when `> 6`), boundary partitioning $\mathbf{K}_{ff}, \mathbf{K}_{fs}, \mathbf{K}_{ss}$, solved $\mathbf{u}_f, \mathbf{R}_s$, member end-force recovery, and bilingual (`el`/`en`) KaTeX explanations.

---

## 4. Transaction Boundaries, Idempotency & Concurrency

- **Single-Aggregate Consistency:** `UpsertStructuralModel` is an atomic in-memory (and persistable) replacement of `StructuralModel` that increments `revision`.
- **Idempotency:**
  - All read/solve queries (`SolveLinearStatic`, `EvaluateKinematicDiagnostics`, `GenerateDsmWalkthrough`, `InspectMemberPoint`) are pure deterministic functions of `(StructuralModel@revision, inputs)` and are completely idempotent.
  - Calling `StartAnalysisJob` with identical `(model_revision, kind, targets, options)` while an identical job is already `Running` or `Converged` returns the existing `AnalysisJobHandle` without duplicating computation.

---

## 5. Canonical Failure Model

| Failure Category | Canonical Error Code | Trigger Condition |
| --- | --- | --- |
| **Domain Geometry / Topology** | `INVALID_ELEMENT_GEOMETRY` | Zero/near-zero element length ($L_{\text{eff}} < 10^{-4}\text{ m}$), self-loop (`node_i == node_j`), or rigid offsets exceeding member length. |
| **Referential Integrity** | `MISSING_MODEL_REFERENCE` | Element or load references a non-existent `NodeId`, `SectionId`, `MaterialId`, or `SupportId`. |
| **Boundary / Load Validation** | `DUPLICATE_NODE_SUPPORT` | Two `Support` definitions target the same `NodeId`. |
| **Load / Section Validation** | `INVALID_LOAD_ASSIGNMENT` | Transverse load applied to a `Cable`, or thermal gradient applied to a `Section` lacking `depth_h`. |
| **Kinematic Mechanism** | `KINEMATIC_INSTABILITY` | Global free stiffness $\mathbf{K}_{ff}$ is singular / rank-deficient ($n_k > 0$); returns `Diagnostics` with `mechanism_modes` so the UI can animate the mechanism. |
| **Nonlinear Convergence** | `NONLINEAR_NOT_CONVERGED` | Unilateral active-set, $P\text{-}\Delta$, or catenary Newton-Raphson fails to reach tolerance within `max_iterations`. |
| **Stability Collapse** | `PDELTA_BUCKLING_DIVERGENCE` | Applied compressive loads exceed critical buckling capacity during 2nd-order $P\text{-}\Delta$ iteration ($\det(\mathbf{K}_E + \mathbf{K}_G) \le 0$). |
| **Cable Slack Singularity** | `CABLE_COMPLETE_SLACK_MECHANISM` | A node supported solely by cables loses all geometric stiffness when all attached cables go slack simultaneously without stabilizing self-weight. |
| **Concurrency / Revision** | `STALE_MODEL_REVISION` | Request `model_revision` does not match the active `StructuralModel.revision`. |

---

## 6. End-to-End Use-Case Chains

1. **Chain 1 — Live Mode 2 Linear Editing:** User draws a portal frame and adds a trapezoidal load $\rightarrow$ `UpsertStructuralModel` (`revision = 4`) $\rightarrow$ `SolveLinearStatic` (`revision = 4`, $< 50\text{ ms}$) $\rightarrow$ UI renders reactions, deformed shape, and exact analytical $M/Q/N$ diagrams on the tension side.
2. **Chain 2 — Unstable Structure Mechanism Animation:** User places 4 hinges in a single-bay portal frame $\rightarrow$ `UpsertStructuralModel` (`revision = 5`) $\rightarrow$ `SolveLinearStatic` returns `KINEMATIC_INSTABILITY` with `Diagnostics.mechanism_modes[0]` $\rightarrow$ Mode 2 canvas animates the sway mechanism and displays the bilingual explanation.
3. **Chain 3 — Linear-First to Background Nonlinear ($P\text{-}\Delta$ / Catenary):** User adds an elastic catenary cable to a mast (`revision = 6`) $\rightarrow$ `SolveLinearStatic` immediately returns linearized trial diagrams (`linear_approximation_only: true`) $\rightarrow$ UI invokes `StartAnalysisJob(LinearStatic, revision = 6)` $\rightarrow$ Rust worker runs Irvine catenary Newton-Raphson per combination, emitting `analysis://progress` $\rightarrow$ `GetAnalysisJobResult` replaces linear diagrams with exact catenary sag curves and tensions.
4. **Chain 4 — Mode 2 to Mode 3 DSM Walkthrough:** User clicks "Open DSM Walkthrough" on a 4-node frame $\rightarrow$ `GenerateDsmWalkthrough(revision = 6, "LC1")` $\rightarrow$ Mode 3 displays full $12\times 12$ global stiffness matrix, boundary partitioning, and member end-force recovery steps.
