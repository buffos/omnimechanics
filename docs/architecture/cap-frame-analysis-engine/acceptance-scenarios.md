# Acceptance Scenarios: 2D Frame Analysis Engine (`cap-frame-analysis-engine`)

- **Capability Owner:** `cap-frame-analysis-engine`
- **Upstream Artifacts:** [prd.md](prd.md), [canonical-domain-model.md](canonical-domain-model.md), [canonical-use-cases.md](canonical-use-cases.md), [canonical-api-cli-contract.md](canonical-api-cli-contract.md)
- **Root Verification Policy:** `backend-boundary: required`, `frontend-integration: required`, `end-to-end: catalog-only`

---

## 1. Scenario Catalog

### SC-001: Linear Static Frame with End Releases, Rigid Offsets, Inclined Roller, and Interior Loads
- **Purpose:** Verify exact linear DSM solution, Greek sign convention for $M(x), Q(x), N(x)$, static condensation of end releases, rigid end offsets, inclined roller reaction, thermal gradient, and support settlement.
- **Preconditions:** A 2-bay, 1-storey planar frame (`revision = 1`) with `FrameMember`s, an internal moment release (`release_j.moment = true`) on the first girder, rigid column-face offsets (`dx = 0.20 m`), an inclined roller support (`angle_deg = 30.0`), a partial trapezoidal load (`a_ratio = 0.2`, `b_ratio = 0.8`), a thermal gradient ($\Delta T_{\text{gradient}} = 15^\circ\text{C}$ on section with $h = 0.40\text{ m}$), and a vertical support settlement ($\Delta y = -0.005\text{ m}$).
- **Action:** Execute `SolveLinearStatic` (`frame_solve_linear`) for `LC1` and inspect interior points via `InspectMemberPoint`.
- **Expected Outcome:**
  - Support reactions satisfy global equilibrium ($\sum F_x = 0, \sum F_y = 0, \sum M_z = 0$) within $10^{-9}\text{ kN}$.
  - At the released girder end, $M_j = 0.0\text{ kNm}$ within $10^{-9}\text{ kNm}$.
  - Analytical $M(x), Q(x), N(x)$ match closed-form Euler-Bernoulli solutions and offline OpenSeesPy CSV reference values within relative tolerance $10^{-6}$, with $M(x) > 0$ where the bottom fibre is in tension.
  - Execution completes in $< 50\text{ ms}$ (verified up to 500 nodes / 1000 members).
- **Verification Surfaces:**
  - `backend-boundary`: Required (`cargo test` companion benchmark suite + OpenSeesPy CSV comparison).
  - `frontend-integration`: Required (Vitest model store + IPC response mapper verifies tension-side diagram plotting coordinates).
  - `end-to-end`: Catalog-only (Mode 2 live canvas visual review).

---

### SC-002: Pre-Solve Kinematic Instability Detection and Mechanism Mode Animation Vector
- **Purpose:** Ensure unstable structures are caught before linear/nonlinear force solving and return an animatable mechanism mode shape and static indeterminacy degree.
- **Preconditions:** A single-bay portal frame with pinned bases (`SU1`, `SU2`) and moment releases at both top corners (`N2`, `N3`), forming a 4-hinge sway mechanism ($n_s = -1, n_k = 1$).
- **Action:** Call `SolveLinearStatic` or `EvaluateKinematicDiagnostics`.
- **Expected Outcome:**
  - Solver rejects force calculation with error code `KINEMATIC_INSTABILITY` (`409`).
  - `Diagnostics` reports `static_indeterminacy_degree: -1`, `kinematic_instability_degree: 1`, `is_stable: false`, and returns `mechanism_modes[0]` normalized to $\max(|\Delta u_x|, |\Delta u_y|) = 1.0$ showing horizontal sway of nodes `N2` and `N3`, paired with bilingual (`el`/`en`) diagnostic explanation.
- **Verification Surfaces:**
  - `backend-boundary`: Required (`cargo test` testing collinear hinges, unrestrained trusses, and double axial releases).
  - `frontend-integration`: Required (Vitest test verifying error handling transitions Mode 2 store into mechanism animation state).
  - `end-to-end`: Catalog-only (Mode 2 animated mechanism visual review).

---

### SC-003: Unilateral Support Uplift and Tension-Only Cross-Bracing Active-Set Iteration
- **Purpose:** Verify active-set convergence and non-superposition when compression-only supports lift off or tension-only truss diagonals go slack under lateral reversal.
- **Preconditions:** A braced frame with X-bracing (`E5`, `E6` as `TrussBar` with `axial_behavior: TensionOnly`) on a rigid foundation beam resting on 3 `Roller` supports with `unilateral: true`, subjected to two combinations `CO1` (wind right) and `CO2` (wind left).
- **Action:** Execute `StartAnalysisJob` (`LinearStatic` with nonlinear unilateral elements) for `CO1` and `CO2`.
- **Expected Outcome:**
  - `ResultSet.superposition_valid` is `false`; each combination is solved independently.
  - In `CO1`, the compression diagonal carries $N = 0.0\text{ kN}$ while the tension diagonal carries all lateral shear; any uplifted unilateral support has reaction $R_n = 0.0\text{ kN}$ and normal displacement $u_n > 0$.
  - In `CO2`, the active diagonal and bearing supports reverse symmetrically.
  - Matches closed-form/OpenSeesPy reference within $10^{-6}$ after active-set convergence.
- **Verification Surfaces:**
  - `backend-boundary`: Required (`cargo test` active-set unit & benchmark tests).
  - `frontend-integration`: Required (Vitest test confirming UI displays the non-superposition indicator badge).
  - `end-to-end`: Catalog-only (Mode 2 visual inspection of inactive slack brace styling).

---

### SC-004: Second-Order $P\text{-}\Delta / P\text{-}\delta$ Geometric Nonlinearity and Linear Eigenvalue Buckling
- **Purpose:** Verify 2nd-order moment amplification per combination using geometric stiffness $\mathbf{K}_G(N)$, buckling divergence detection, and linear eigenvalue buckling ($\alpha_{cr}$ and mode shape).
- **Preconditions:** A cantilever column and a portal frame under heavy vertical gravity load $P$ plus horizontal sway load $H$ across combinations `CO1`, `CO2`.
- **Action:**
  1. Run `StartAnalysisJob` with `kind: SecondOrderPDelta` for `CO1`, `CO2` and `compute_envelope: true`.
  2. Run `StartAnalysisJob` with `kind: LinearBuckling` for `CO1`.
- **Expected Outcome:**
  - $P\text{-}\Delta$ base moment matches analytical amplification $M_{\max} = H L \frac{\tan(k L)}{k L}$ (where $k = \sqrt{P / EI}$) and OpenSeesPy reference within $10^{-3}$ relative tolerance.
  - `LinearBuckling` returns $\alpha_{cr,1} = \frac{\pi^2 EI}{4 L^2 P}$ within $10^{-4}$ relative tolerance and normalized sway buckling mode vector $\boldsymbol{\phi}_1$.
  - If $P \ge P_{cr}$ during `SecondOrderPDelta`, job fails cleanly with `PDELTA_BUCKLING_DIVERGENCE`.
- **Verification Surfaces:**
  - `backend-boundary`: Required (`cargo test` Euler column & portal frame $P\text{-}\Delta$ / buckling suite).
  - `frontend-integration`: Required (Vitest job result store integration for buckling modes and $P\text{-}\Delta$ envelopes).
  - `end-to-end`: Catalog-only (Mode 2 deformed shape & buckling mode visual review).

---

### SC-005: Exact Irvine Elastic Catenary Cable Solver and Slack Detection
- **Purpose:** Verify exact 2D Irvine elastic catenary kinematics under self-weight for `UnstressedLength`, `SagRatio`, and `Pretension` inputs, plus slack warning detection.
- **Preconditions:** A guyed mast where `Cable` element `E2` is specified via `UnstressedLength { l0 }`, `SagRatio { sag_to_span: 0.05 }`, or `Pretension { t0: 50.0 }`, and a second leeward cable `E3` that loses tension under high wind load.
- **Action:** Run `StartAnalysisJob` (`LinearStatic` / `SecondOrderPDelta`) and retrieve `GetAnalysisJobResult`.
- **Expected Outcome:**
  - Horizontal tension $H$, vertical end reactions $(V_i, V_j)$, mid-span sag, and `curve_points_global` match the analytical Irvine transcendental equations within $10^{-3}$ relative error.
  - When leeward cable `E3` unloads below threshold, `CableResult.is_slack` is `true` and `Diagnostics.warnings` includes `CABLE_SLACK_DETECTED` in Greek and English without crashing the global Newton-Raphson loop.
- **Verification Surfaces:**
  - `backend-boundary`: Required (`cargo test` Irvine single-cable & guyed-mast benchmark suite).
  - `frontend-integration`: Required (Vitest test verifying catenary polyline rendering data and slack warning banner).
  - `end-to-end`: Catalog-only (Mode 2 curved cable rendering visual check).

---

### SC-006: Modal Analysis ($\ge 90\%$ Effective Mass), Response Spectrum (SRSS/CQC), and Lateral Force
- **Purpose:** Verify automatic seismic mass derivation from $G + \psi_2 Q$ + `LumpedMass`, eigenmode extraction up to $\ge 90\%$ effective mass, CQC vs. SRSS modal combination, and EN 1998-1 Lateral Force distribution.
- **Preconditions:** A 3-storey shear/moment frame with `Permanent` ($G$) and `Imposed` ($Q$, $\psi_2 = 0.3$) load cases, plus an EN 1998 `DesignSpectrum` passed from `cap-eurocode-actions-combinations`.
- **Action:** Run `StartAnalysisJob` for `Modal`, `LateralForce`, and `ResponseSpectrum` (both `Srss` and `Cqc` with $\xi = 5\%$).
- **Expected Outcome:**
  - `Modal` extracts modes $1\dots k$ such that cumulative effective mass $\sum M_{\text{eff},x} / M_{\text{tot},x} \ge 0.90$, matching OpenSeesPy natural periods $T_i$ and participation factors $\Gamma_i$ within $10^{-4}$.
  - `ResponseSpectrum` combines modal peak base shears and member forces via SRSS and CQC (with CQC capturing closely spaced mode coupling $\rho_{ij}$).
  - `LateralForce` distributes base shear $F_b = S_d(T_1) m \lambda$ proportionally to $z_i m_i$ and returns static internal forces.
- **Verification Surfaces:**
  - `backend-boundary`: Required (`cargo test` 3-storey benchmark frame against closed-form & OpenSeesPy).
  - `frontend-integration`: Required (Vitest modal table & RSA combination selector tests).
  - `end-to-end`: Catalog-only (Mode 2 modal animation & RSA diagram review).

---

### SC-007: Internal Force Envelopes with Pointwise Governing Combination Tracking
- **Purpose:** Verify pointwise max/min envelope calculation across multiple combinations for downstream Eurocode RC Design consumption.
- **Preconditions:** A 3-span continuous beam solved across 12 ULS combinations (`CO1`..`CO12`).
- **Action:** Request `compute_envelope: true` in `SolveLinearStatic` (or `StartAnalysisJob` for $P\text{-}\Delta$).
- **Expected Outcome:**
  - `Envelope.member_envelopes` contains exact `max_m`, `min_m`, `max_q`, `min_q`, `max_n`, `min_n` at every station along each span, each paired with the exact `governing_combination_id` (`"CO4"`, `"CO9"`, etc.) that produced that value.
- **Verification Surfaces:**
  - `backend-boundary`: Required (`cargo test` envelope aggregation & governing ID verification).
  - `frontend-integration`: Required (Vitest hover inspection test verifying governing combination tooltip text).
  - `end-to-end`: Catalog-only.

---

### SC-008: Background Job Cancellation, Progress Telemetry, and Revision Superseding
- **Purpose:** Verify lifecycle interactions when a user edits `StructuralModel` while a multi-combination nonlinear job is running in the background.
- **Preconditions:** `StartAnalysisJob` (`SecondOrderPDelta` across 15 combinations) is `Running` at `model_revision = 10` and emitting `analysis://progress`.
- **Action:** User moves a node via `UpsertStructuralModel`, advancing the model to `revision = 11` before `job-10` finishes.
- **Expected Outcome:**
  - `job-10` is immediately signaled via its cancellation token and transitions to `status: Superseded`.
  - Any subsequent attempt to apply or start a job with `model_revision = 10` is rejected with `STALE_MODEL_REVISION`.
  - Active results for `revision = 11` are never overwritten by late messages from `job-10`.
- **Verification Surfaces:**
  - `backend-boundary`: Required (`cargo test` concurrent revision bump & job cancellation test).
  - `frontend-integration`: Required (Vitest async race test confirming store discards superseded job payloads).
  - `end-to-end`: Not-applicable (concurrency race verified deterministically in backend/frontend tests).

---

### SC-009: Pedagogical DSM Walkthrough Data Generation ($\le 6$ Nodes vs. $> 6$ Nodes)
- **Purpose:** Verify step-by-step DSM matrix extraction (`k_local`, `T`, `f_0`, `K_global`, partitioned `K_ff`, `u_f`, `R_s`) for Mode 3 Teaching Studio.
- **Preconditions:** A 3-node portal frame (`nodes.len() == 3 <= 6`) and a 10-node truss (`nodes.len() == 10 > 6`).
- **Action:** Call `GenerateDsmWalkthrough` (`frame_generate_dsm_walkthrough`) for each model.
- **Expected Outcome:**
  - For the 3-node frame, full dense matrices $\mathbf{k}_e$ ($6\times 6$), $\mathbf{T}_e$ ($6\times 6$), $\mathbf{K}_{\text{global}}$ ($9\times 9$), and partitioned submatrices $\mathbf{K}_{ff}, \mathbf{K}_{fs}, \mathbf{K}_{ss}$ are returned with bilingual (`el`/`en`) step explanations and match the solved displacements and end forces identically.
  - For the 10-node truss, per-element matrices and DOF tables are returned alongside a compact sparse summary of $\mathbf{K}_{\text{global}}$ (`full_global_matrix_omitted: true`).
- **Verification Surfaces:**
  - `backend-boundary`: Required (`cargo test` verifying $\mathbf{K}_{ff}\mathbf{u}_f = \mathbf{F}_f - \mathbf{K}_{fs}\mathbf{u}_s$ identity on walkthrough payloads).
  - `frontend-integration`: Required (Vitest walkthrough step viewer state test).
  - `end-to-end`: Catalog-only (Mode 3 DSM walkthrough visual gate).

---

## 2. Stateful & Lifecycle Interaction Matrix

| Lifecycle / Concurrency Dimension | Coverage Decision | Mapped Scenario |
| --- | --- | --- |
| Model mutated while background nonlinear job is `Running` | Covered | `SC-008` (`Superseded` transition + cooperative cancellation) |
| Duplicate `StartAnalysisJob` request for same `(revision, kind, targets)` | Covered | Idempotent return of active/converged `AnalysisJobHandle` (`canonical-use-cases.md` §4) |
| Unstable geometry submitted to linear or nonlinear solver | Covered | `SC-002` (`KINEMATIC_INSTABILITY` + `mechanism_modes` before solve) |
| Nonlinear combination fails to converge within multi-combination batch | Covered | `SC-004` / `SC-005` (per-combination failure diagnostics reported without corrupting converged combinations) |
