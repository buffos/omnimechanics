# Product Requirements Document: 2D Frame Analysis Engine (`cap-frame-analysis-engine`)

- **Capability Owner:** `cap-frame-analysis-engine`
- **State Target:** `specified`
- **Parent Product PRD:** [docs/prd.md](../../prd.md)
- **Cross-Capability Conventions:** [cross-capability-conventions.md](../cross-capability-conventions.md)
- **Discovery Source:** [discovery-notes.md](../cap-geometry-canvas-sketcher/discovery-notes.md)

---

## 1. Purpose & Product Summary

`cap-frame-analysis-engine` is the canonical 2D structural modeling and Direct Stiffness Method (DSM) solver core of OmniMechanics. It is the **single owner** of the 2D `StructuralModel` (nodes, frame members, truss bars, springs, elastic catenary cables, sections, materials, supports, load cases, loads, and masses) and executes all static, nonlinear, stability, and dynamic/seismic structural analyses.

It serves three product surfaces simultaneously:
1. **Mode 2 (Structural Modeling & Analysis):** delivers immediate (< 50 ms) linear static feedback while the user sketches and edits a structure, and seamlessly upgrades to background-computed nonlinear ($P\text{-}\Delta$, catenary cables, unilateral contact) or dynamic (modal, response spectrum, lateral force) results upon convergence.
2. **Mode 3 (Teaching Studio):** provides the authoritative structural model and pre-solve diagnostics to the classical method classifier, and generates the step-by-step **DSM Walkthrough** matrices ($\mathbf{k}_e, \mathbf{T}_e, \mathbf{K}, \mathbf{K}_{ff}$) for any 2D structure.
3. **Eurocode Design (`cap-eurocode-rc-design`):** evaluates factored combinations received from `cap-eurocode-actions-combinations` and produces governing $M(x), Q(x), N(x)$ envelopes for cross-sectional design.

---

## 2. Goals & Non-Goals

### 2.1 Goals
- **G1 — Universal 2D Structural Ownership:** Represent any planar structure built from Euler-Bernoulli frame members (with end releases and rigid offsets), axial truss bars (with tension/compression-only flags), translational/rotational springs, and exact elastic catenary cables, tracked by a monotonic `revision`.
- **G2 — Real-Time Linear Static Solving:** Solve models up to 500 nodes and 1000 elements in `< 50 ms` on desktop hardware so live canvas editing (150 ms debounce) feels instantaneous.
- **G3 — Rigorous Nonlinear & Stability Capabilities:** Support 2nd-order $P\text{-}\Delta / P\text{-}\delta$ geometric stiffness iteration per combination, linear eigenvalue buckling ($\alpha_{cr}$ and mode shapes), active-set tension/compression-only iterations, and exact Irvine elastic catenary cables via Newton-Raphson with slack detection.
- **G4 — Modal & Seismic Execution:** Compute structural eigenmodes (lumped or consistent mass derived from $G + \psi_2 Q$ plus nodal masses) up to $\ge 90\%$ effective modal mass participation, and execute EN 1998-1 Lateral Force and Modal Response Spectrum (SRSS/CQC) analyses.
- **G5 — Pre-Solve Kinematic Diagnostics:** Detect mechanisms and rank deficiencies before attempting a full solve, report static indeterminacy $n_s$ and kinematic instability $n_k$, and extract a normalized mechanism displacement vector $\boldsymbol{\phi}_m$ for canvas animation.
- **G6 — Exact Analytical Member Diagrams & Envelopes:** Evaluate $M(x), Q(x), N(x)$ analytically at any interior station $x \in [0, L]$ (incorporating interior point, trapezoidal, and thermal loads without requiring internal nodes) following Greek engineering sign conventions, and compute pointwise max/min envelopes with governing combination tracking.
- **G7 — Pedagogical Transparency:** Export complete DSM walkthrough intermediate matrices and vectors in bilingual (`el`/`en`) step format, with full dense matrix display for models with $\le 6$ nodes.

### 2.2 Non-Goals (Current Horizon)
- **NG1:** 3D space frames, grillages, or out-of-plane torsion/warping DOFs.
- **NG2:** Timoshenko shear-deformable beam elements (classical Euler-Bernoulli is standard for Mode 2/3 consistency; shear area $A_v$ is stored on `Section` only for future extension/EC2).
- **NG3:** Corotational large-rotation continuum frame elements or material plasticity / pushover plastic-hinge analysis.
- **NG4:** Moving traffic loads, influence lines, or time-history transient integration (Newmark-$\beta$).
- **NG5:** Generating EN 1990 combination factors or EN 1998 design spectra (strictly owned by `cap-eurocode-actions-combinations`; this engine consumes combinations and spectra passed to it).
- **NG6:** Interactive canvas rendering, snapping, or UI state management (owned by `cap-sketcher-frame-modeler`).

---

## 3. Actors & Downstream Consumers

| Actor / Consumer | Role & Interaction with `cap-frame-analysis-engine` |
| --- | --- |
| **Engineer / Instructor (via Mode 2)** | Mutates `StructuralModel`, inspects live reactions, $M/Q/N$ diagrams, deformed shape, $P\text{-}\Delta$ amplification, buckling $\alpha_{cr}$, cable sag/tensions, seismic modes, and combination envelopes. |
| **Student (via Mode 2 & Mode 3)** | Observes animated mechanism modes when a structure is unstable; opens the DSM Walkthrough to inspect local stiffness matrices $\mathbf{k}_e$, coordinate transformations $\mathbf{T}_e$, global assembly, and boundary condition partitioning. |
| **Teaching Classifier (`cap-structural-mechanics-teaching`)** | Reads `StructuralModel` and `Diagnostics` to determine whether classical methods (isostatic, Joints/Ritter, Clapeyron, Cross, Force Method) apply. |
| **RC Design Engine (`cap-eurocode-rc-design`)** | Queries internal force `Envelope`s ($M_{Ed}, V_{Ed}, N_{Ed}$) along frame members for ULS/SLS reinforcement design and code checks. |

---

## 4. Functional Requirements

### 4.1 Structural Model & Library Management
- **FR-MOD-01 (Revisioned Aggregate):** Every mutation to `StructuralModel` increments `revision: u64` by 1. Every analysis request, job, and result set carries the exact `revision` it corresponds to.
- **FR-MOD-02 (Elements):** Support four element variants:
  - `FrameMember`: 2-node Euler-Bernoulli member with `section_id`, `material_id`, optional per-end `EndRelease { axial, shear, moment }`, and optional per-end `RigidOffset { dx_i, dy_i, dx_j, dy_j }`.
  - `TrussBar`: 2-node pin-ended axial member with `section_id`, `material_id`, and `axial_behavior: Both | TensionOnly | CompressionOnly`.
  - `Spring`: 1-node (grounded) or 2-node discrete spring with $(k_x, k_y, k_\theta)$ and orientation `angle`.
  - `Cable`: 2-node exact elastic catenary member with `section_id`, `material_id`, and `CablePrestress` defined by exactly one of: `UnstressedLength(L0)`, `SagRatio(ratio)`, or `Pretension(T0)`.
- **FR-MOD-03 (Material & Section Libraries):** Provide built-in read-only libraries for Materials (Concrete C20/25–C50/60, Steel S235/S275/S355, Timber C24/GL24h, CableSteel Y1770/Y1860) and Sections (IPE 80–600, HEA 100–600, HEB 100–600, CHS/RHS hollow sections, parametric Rectangular $b \times h$, Circular $D$), plus user-defined custom entries $(E, G, \gamma, \alpha_T)$ and $(A, I, h)$.
- **FR-MOD-04 (Supports):** Support nodal restraints `Fixed`, `Pinned`, `Roller { angle_deg }`, `Guided { angle_deg }`, and `Elastic { kx, ky, ktheta }`, each with `unilateral: bool` (when `true`, bearing resists compression toward the support normal and releases under tension/uplift).
- **FR-MOD-05 (Load Cases & Loads):** Group loads into named `LoadCase`s (`ActionType` tag + `self_weight: bool`). Support `NodalLoad { fx, fy, mz }`, `MemberPointLoad { px, py, m, x_ratio, frame }`, `MemberDistributedLoad { qx1, qy1, qx2, qy2, a_ratio, b_ratio, frame: Local | Global | Projected }`, `SupportSettlement { dx, dy, dtheta }`, and `ThermalLoad { dt_uniform, dt_gradient }`.

### 4.2 Pre-Solve Validation & Kinematic Diagnostics
- **FR-DIAG-01 (Static Indeterminacy):** Compute and report static indeterminacy degree $n_s = (3 m_f + m_t + m_c + r_s - c_{\text{rel}}) - (3 n_f + 2 n_p)$ alongside active free DOFs $N_{\text{dof}}$.
- **FR-DIAG-02 (Mechanism Detection):** Before solving for displacements, factorize or perform rank-revealing QR / SVD / pivoted Cholesky on $\mathbf{K}_{ff}$. If rank deficiency $n_k > 0$ or pivot ratio $< 10^{-11}$, abort the force solve with `KINEMATIC_INSTABILITY`, extract the nullspace basis vector(s) $\boldsymbol{\phi}_m$ ($\|\boldsymbol{\phi}_m\|_\infty = 1$), classify the mechanism (global vs. local member/node hinge mechanism), and return bilingual explanations (`el`/`en`).

### 4.3 Analysis Formulations
- **FR-SOL-01 (Linear Static DSM):** Assemble global stiffness $\mathbf{K}$ and equivalent nodal load vectors for all requested `LoadCase`s; solve via sparse/dense Cholesky (`faer`); superpose factored `LoadCase` results for linear `Combination`s.
- **FR-SOL-02 (Unilateral & Tension/Compression-Only Iteration):** When `TrussBar` has `TensionOnly`/`CompressionOnly` or `Support` has `unilateral: true`, solve each target (`LoadCase` or `Combination`) separately via active-set iteration (max 50 iterations, state-stabilization + residual norm $< 10^{-6}$). Detect instability if deactivating elements creates a mechanism.
- **FR-SOL-03 (2nd-Order $P\text{-}\Delta / P\text{-}\delta$):** For each target `Combination`, assemble the consistent Euler-Bernoulli geometric stiffness matrix $\mathbf{K}_G(N)$ capturing both chord rotation ($P\text{-}\Delta$) and member curvature ($P\text{-}\delta$), and iterate $\left[\mathbf{K}_E + \mathbf{K}_G(\mathbf{N}^{(k)})\right]\mathbf{u}^{(k+1)} = \mathbf{F}$ until $\|\Delta\mathbf{u}\|_2 / \|\mathbf{u}\|_2 < 10^{-5}$. Reject or flag if axial compression reaches or exceeds critical buckling ($\det(\mathbf{K}_E + \mathbf{K}_G) \le 0$).
- **FR-SOL-04 (Linear Buckling):** For a reference `LoadCase` or `Combination`, solve the generalized eigenvalue problem $\left(\mathbf{K}_E + \alpha\,\mathbf{K}_G(\mathbf{N}_{\text{ref}})\right)\boldsymbol{\phi} = \mathbf{0}$ for the lowest $k$ positive critical load factors $\alpha_{cr,1} \le \alpha_{cr,2} \le \dots$ and normalized buckling mode shapes.
- **FR-SOL-05 (Elastic Catenary Cables):** Implement the exact 2D Irvine elastic catenary formulation:
  $$\Delta x(H, V) = \frac{H L_0}{E A} + \frac{H}{w_0}\left[\sinh^{-1}\!\left(\frac{V}{H}\right) - \sinh^{-1}\!\left(\frac{V - w_0 L_0}{H}\right)\right]$$
  $$\Delta y(H, V) = \frac{L_0}{E A}\left(V - \frac{w_0 L_0}{2}\right) + \frac{1}{w_0}\left[\sqrt{H^2 + V^2} - \sqrt{H^2 + (V - w_0 L_0)^2}\right]$$
  Compute exact $2\times 2$ end flexibility/stiffness tangent $\mathbf{k}_T = \mathbf{f}_T^{-1}$ via inner Newton-Raphson, and iterate global nodal equilibrium via outer Newton-Raphson with line search and automatic slack/low-tension regularization. Convert input `SagRatio` or `Pretension(T0)` at initial chord geometry into equivalent $L_0$ before solving.
- **FR-SOL-06 (Modal Analysis):** Assemble total mass matrix $\mathbf{M}$ (lumped nodal translational/rotational masses or optional consistent element mass matrix) from permanent cases $G + \sum \psi_{2,i} Q_i$ plus explicit `LumpedMass`es. Solve $(\mathbf{K}_{ff} - \omega^2 \mathbf{M}_{ff})\boldsymbol{\phi} = \mathbf{0}$ for natural frequencies $f_i$, periods $T_i$, participation factors $\Gamma_{x,i}, \Gamma_{y,i}$, and effective modal masses $M_{\text{eff},x,i}, M_{\text{eff},y,i}$, automatically expanding mode count until cumulative effective mass $\ge 90\%$ of total active mass (or all available DOFs are exhausted).
- **FR-SOL-07 (Response Spectrum & Lateral Force):** Given an EN 1998 `DesignSpectrum` $S_d(T)$ from `cap-eurocode-actions-combinations`:
  - `LateralForce`: compute fundamental period $T_1$ (from modal or Rayleigh quotient), base shear $F_b = S_d(T_1)\cdot m \cdot \lambda$, distribute horizontal nodal forces $F_i = F_b \frac{z_i m_i}{\sum z_j m_j}$, and solve static response.
  - `ResponseSpectrum`: scale modal responses by $S_{d,x}(T_i)$ (and optional vertical $S_{d,y}(T_i)$) and combine modal peaks via selectable **SRSS** or **CQC** (Der Kiureghian correlation $\rho_{ij}$ with damping $\xi$, default $5\%$).

### 4.4 Results, Envelopes & DSM Walkthrough
- **FR-RES-01 (Analytical Member Diagrams):** Provide closed-form piecewise polynomial evaluation of $N(x), Q(x), M(x)$ and local deflections $u_y(x)$ at default station density (e.g. 21 stations + all load discontinuity points + exact zero-shear local extrema $Q(x)=0$), adhering strictly to Greek sign convention.
- **FR-RES-02 (Envelopes):** Given a set of `Combination`s, compute max/min envelopes across all member stations, nodal displacements, and support reactions, tagging every extremum with `governing_combination_id`.
- **FR-RES-03 (DSM Walkthrough):** Produce step-by-step pedagogical output for any linear static load case: (1) DOF numbering & indeterminacy, (2) Per-element local $\mathbf{k}_e$, $\mathbf{T}_e$, $\mathbf{K}_e = \mathbf{T}_e^T \mathbf{k}_e \mathbf{T}_e$, fixed-end vector $\mathbf{f}_{0,e}$, (3) Global assembly $\mathbf{K}$ and $\mathbf{F}$, (4) Boundary partitioning $\mathbf{K}_{ff}\mathbf{u}_f = \mathbf{F}_f - \mathbf{K}_{fs}\mathbf{u}_s$, (5) Displacement solution $\mathbf{u}_f$ and support reactions $\mathbf{R}_s$, (6) Member end-force back-substitution.

---

## 5. Non-Functional Requirements

- **NFR-PERF-01 (Linear Latency):** Synchronous linear static solve (`LinearStatic` without nonlinear elements) for a 500-node, 1000-member planar frame with 5 load cases and 20 linear combinations must complete in `< 50 ms` on a standard 4-core desktop CPU.
- **NFR-PERF-02 (Nonlinear & Background Execution):** Nonlinear ($P\text{-}\Delta$, catenary cables, unilateral) and dynamic (Modal, RSA) runs execute asynchronously in a background Rust worker thread, emitting `analysis://progress` events (`job_id`, `revision`, `percent`, `iteration`, `residual`) and completing in `< 2 s` per combination for typical frames ($\le 500$ nodes).
- **NFR-REL-01 (Revision Safety):** Background jobs check a cancellation token at every iteration/mode step. Any mutation to `StructuralModel` increments `revision` and immediately marks in-flight jobs for older revisions as `Superseded`.
- **NFR-VER-01 (Numerical Accuracy):** Closed-form verification benchmarks must match analytical solutions within relative tolerance $10^{-6}$ for linear analyses and $10^{-3}$ for nonlinear/catenary/$P\text{-}\Delta$ analyses, and match offline OpenSeesPy reference CSVs stored in `docs/benchmarks/frame/`.
- **NFR-CODE-01 (Repository Hygiene):** Every Rust source file in `src-tauri/src/frame/` must remain $\le 1000$ lines with companion `*_test.rs` unit test files (`npm run check:limits`).

---

## 6. Explicit Assumptions & Artifact Impact

### Assumptions
1. **Cross-Sectional Shear Deformation:** Frame members use classical Euler-Bernoulli kinematics ($EI$ bending + $EA$ axial) so that Mode 2 DSM solutions match Mode 3 classical methods (Cross, Clapeyron, Force Method) to machine precision ($< 10^{-10}$).
2. **Catenary Cable Loads:** `Cable` elements carry their exact distributed self-weight $w_0 = \gamma A$ along the arc length $s \in [0, L_0]$; external point loads along a cable are applied by placing a `Node` at the load point and connecting two `Cable` segments.

### Application Synthesis Impact Assessment
- **Product Truth (`docs/prd.md`):** `no impact` — all goals, slices (1–6), and non-goals in this node PRD strictly refine the boundaries already established in `docs/prd.md`.
- **Architecture Truth (`docs/architecture/application-architecture-summary.md`):** `no impact` — code location (`src-tauri/src/frame/`), Tauri `frame_` command prefix, and dependency direction (`modeler -> frame -> actions`) match the synthesis baseline.
