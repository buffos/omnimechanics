# Domain Glossary: 2D Frame Analysis Engine (`cap-frame-analysis-engine`)

- **Capability Owner:** `cap-frame-analysis-engine`
- **Status:** Specified (Canonical Vocabulary Baseline)
- **Cross-Capability Baseline:** [cross-capability-conventions.md](../cross-capability-conventions.md)
- **Shared Concerns:** [`cap-eurocode-actions-combinations`](../../../.okf/capabilities/cap-eurocode-actions-combinations.md)

---

## 1. Core Domain Objects (Owned by `cap-frame-analysis-engine`)

| Canonical Term | Category | Concise Definition | Discouraged Aliases / Synonyms | Critical Distinctions |
| --- | --- | --- | --- | --- |
| **StructuralModel** | Aggregate Root | Complete 2D structural definition containing `Node`s, `Element`s, `Section`s, `Material`s, `Support`s, `LoadCase`s, `Load`s, and `LumpedMass`es, tracked by a monotonically increasing integer `revision`. | `SketchModel`, `FrameGraph`, `SystemTopology` | Replaces the retired `SketchModel` from issue 015. Distinct from `ContinuumDomain` (2D/3D BEM/FEM polygons/meshes). |
| **Node** | Entity | A 2D point $(x, y)$ in metres with a stable string identifier (`N1`, `N2`, …) and up to 3 global DOFs $(u_x, u_y, \theta_z)$ depending on connected elements. | `Joint`, `Vertex`, `Point` | A `Node` is never an internal hinge by itself; a UI "node hinge" gesture expands into `EndRelease` flags on incident `FrameMember` ends. |
| **Element** | Entity | A structural connector between two distinct `Node`s (`FrameMember`, `TrussBar`, `Cable`) or a nodal/two-node spring (`Spring`), identified by a stable string ID (`E1`, `E2`, …). | `Member`, `Segment`, `Edge`, `Link` | `Element` is the polymorphic union; `FrameMember` is specifically the 6-DOF Euler-Bernoulli beam-column variant. |
| **FrameMember** | Entity Variant | A 2D Euler-Bernoulli beam-column element with 3 DOFs per end $(u_{x}, u_{y}, \theta_{z})$, axial stiffness $EA/L_{eff}$, flexural stiffness $EI/L_{eff}^3$, optional `EndRelease`s, and optional `RigidOffset`s. | `Beam`, `Column`, `Bar` | Unlike `TrussBar`, carries bending moment $M(x)$ and shear $Q(x)$ and accepts transverse interior loads. |
| **TrussBar** | Entity Variant | A pin-ended 2D axial element transmitting only axial force $N$, with optional unilateral behavior (`Both`, `TensionOnly`, `CompressionOnly`). | `Strut`, `Tie`, `Rod` | Has zero flexural stiffness and no rotational DOFs at its ends; cannot carry transverse member loads directly (except self-weight lumped or projected axially/transversely to end nodes). |
| **Spring** | Entity Variant | A linear elastic discrete spring element acting either between two nodes or between a node and ground, with translational stiffnesses $(k_x, k_y)$ and/or rotational stiffness $k_\theta$ in global or oriented coordinates. | `LinkElement`, `Dashpot` | Distinct from `Support::Elastic` when connecting two structural nodes, though grounded nodal springs assemble identically to elastic supports. |
| **Cable** | Entity Variant | An exact 2D elastic catenary element (Irvine formulation) between two nodes, defined by `Section`, `Material`, self-weight per unit unstressed length $w_0 = \rho g A$, and one state parameter (`unstressed_length` $L_0$, `sag_ratio` $f/L_c$, or `pretension` $T_0$). | `Wire`, `Tendon`, `CatenaryMember` | Inherently nonlinear and tension-only; never superposable and automatically disqualifies the model from classical hand-calculation step-by-step methods. |
| **EndRelease** | Value Object | Per-end boolean release flags `{ axial: bool, shear: bool, moment: bool }` at the start ($i$) or end ($j$) of a `FrameMember`, eliminated via static condensation prior to global assembly. | `MemberHinge`, `PinEnd`, `SliderEnd` | Belongs to a `FrameMember` end, not to a `Node`. Releasing all moments at a node without rotational restraint causes a local rotational mechanism. |
| **RigidOffset** | Value Object | Rigid end zones $(\Delta x_i, \Delta y_i)$ and $(\Delta x_j, \Delta y_j)$ in global or local coordinates at ends $i$ and $j$ of a `FrameMember`, separating the node center from the flexible clear span $L_{eff}$. | `EndEccentricity`, `RigidArm`, `StubZone` | Transforms end DOFs kinematically via rigid-body offset matrices before transforming from local to global axes. |
| **Section** | Entity / Value | Cross-sectional geometric properties referenced by ID (`S1`, …): area $A$, second moment of area $I$, optional shear area $A_v$, depth $h$, and profile metadata (library preset IPE/HEA/HEB/hollow/rectangular/circular or custom). | `Profile`, `CrossSection` | Depth $h$ is required whenever a `ThermalLoad` with $\Delta T_{\text{gradient}} \neq 0$ is applied to an element using that section. |
| **Material** | Entity / Value | Mechanical properties referenced by ID (`M1`, …): Young's modulus $E$ (entered in GPa, stored/solved in $\text{kN/m}^2$), shear modulus $G$, weight density $\gamma = \rho g$ ($\text{kN/m}^3$) or mass density $\rho$ ($\text{t/m}^3$), thermal expansion $\alpha_T$ ($1/^\circ\text{C}$), and material class (`Concrete`, `Steel`, `Timber`, `CableSteel`, `Custom`). | `Grade`, `ConstitutiveLaw` | Elastic isotropic properties only in Mode 2; plastic hinges and nonlinear material stress-strain curves are out of scope. |
| **Support** | Entity | Nodal boundary restraint (`SU1`, …) assigned to a `Node`: `Fixed`, `Pinned`, `Roller{angle}`, `Guided{angle}`, or `Elastic{kx, ky, ktheta}`, plus a `unilateral: bool` flag for compression-only bearing. | `BoundaryCondition`, `Constraint`, `Anchor` | `angle` (in degrees, $0^\circ$ = horizontal rolling / vertical reaction along $+y$) defines the inclined local support frame. `unilateral: true` triggers active-set contact iteration when uplift occurs. |
| **LoadCase** | Entity | Named group of `Load`s (`LC1`, …) tagged with an `ActionType` and a `self_weight: bool` flag indicating whether automatic gravity loads ($\gamma \cdot A$ along $-y$) are generated for all elements. | `Loading`, `LoadGroup`, `ActionCase` | Owned by `cap-frame-analysis-engine`, whereas the `ActionType` classification vocabulary and combination factors belong to `cap-eurocode-actions-combinations`. |
| **Load** | Value Object | Individual action inside a `LoadCase`: `NodalLoad`, `MemberPointLoad`, `MemberDistributedLoad{q1, q2, a, b, frame: Local\|Global\|Projected}`, `SupportSettlement`, or `ThermalLoad{dt_uniform, dt_gradient}`. | `Force`, `Action`, `Excitation` | Member loads at arbitrary positions $(a, b)$ along a `FrameMember` are integrated analytically via fixed-end actions without splitting the member into sub-nodes. |
| **LumpedMass** | Value Object | Additional translational mass $(m_x, m_y)$ in tonnes ($\text{t} = \text{kN}\cdot\text{s}^2/\text{m}$) and optional rotational inertia $I_\theta$ assigned to a `Node` for dynamic/modal analysis, supplementing masses automatically derived from $G + \psi_{2,i} Q_i$. | `NodalWeight`, `PointInertia` | Stored in mass units ($\text{t}$, $\text{t}\cdot\text{m}^2$) consistent with $\text{kN}$, $\text{m}$, $\text{s}$. |

---

## 2. Analysis, Diagnostics & Results Vocabulary

| Canonical Term | Category | Concise Definition | Discouraged Aliases / Synonyms | Critical Distinctions |
| --- | --- | --- | --- | --- |
| **AnalysisKind** | Enumeration | The mathematical solver formulation requested: `LinearStatic`, `SecondOrderPDelta`, `LinearBuckling`, `Modal`, `ResponseSpectrum`, or `LateralForce`. | `SolverType`, `RunMode`, `CalculationKind` | Note that `Cable` and unilateral elements invoked under `LinearStatic` or `SecondOrderPDelta` automatically promote execution to the iterative nonlinear solver path per target. |
| **AnalysisJob** | Entity / State | Execution handle tracking a solver run for a specific `(revision, kind, target_set)`. Statuses: `Queued`, `Running`, `Converged`, `Failed`, `Cancelled`, `Superseded`. | `SolverTask`, `WorkerThread`, `Computation` | Carries the exact model `revision` it started with; if the model mutates to `revision + 1`, any in-flight job transitions to `Superseded` and is discarded. |
| **Diagnostics** | Value Object | Pre- and post-solve structural health report containing static indeterminacy degree $n_s$, kinematic instability degree $n_k$, mechanism mode vectors, conditioning warnings, slack-cable flags, and iteration convergence metrics. | `ErrorLog`, `HealthCheck`, `SolverWarnings` | Always returned—even when `KINEMATIC_INSTABILITY` prevents a static force solution—so the UI can animate the mechanism mode shape. |
| **MechanismMode** | Value Object | A normalized nullspace eigenvector $\boldsymbol{\phi}_m$ of the constrained stiffness matrix $\mathbf{K}_{ff}$ representing a zero-energy rigid-body displacement mode of an unstable structure. | `RigidBodyMode`, `InstabilityShape`, `ZeroEigenmode` | Distinct from a `BucklingMode` (which requires reference axial forces $\mathbf{K}_G$) and a `ModalShape` (which solves with mass matrix $\mathbf{M}$). |
| **ResultSet** | Value Object | Complete structural response for one target (`LoadCase` ID or `Combination` ID) at a specific model `revision`: support reactions, nodal displacements, member end forces, analytical internal force functions $M(x), Q(x), N(x)$, deformed curve samples, cable state, and extrema. | `Solution`, `OutputData`, `CaseResult` | Never mixed across revisions. For `LinearBuckling` and `Modal`, accompanied by eigenvalue/mode payloads (`BucklingModeResult`, `ModalResult`). |
| **Envelope** | Value Object | Pointwise maximum and minimum bounds of $M(x)$, $Q(x)$, $N(x)$, nodal displacements, and support reactions across a set of `Combination` `ResultSet`s, recording the governing `Combination` ID at every station. | `MinMaxSummary`, `DesignBounds` | Computed over `Combination` results (not raw unfactored `LoadCase`s) and consumed directly by `cap-eurocode-rc-design`. |
| **DsmWalkthroughData** | Value Object | Step-by-step pedagogical snapshot of the Direct Stiffness Method for a target load case: DOF numbering map, per-element local $\mathbf{k}_e$, transformation $\mathbf{T}_e$, fixed-end vector $\mathbf{f}_{0,e}$, global $\mathbf{K}$, partitioned $\mathbf{K}_{ff}, \mathbf{K}_{fs}, \mathbf{K}_{ss}$, solved $\mathbf{u}_f$, reactions $\mathbf{R}_s$, and member back-substitution. | `MatrixDump`, `EducationalTrace` | Full dense matrices are populated inline when `nodes.len() <= 6`; larger models provide per-element matrices plus sparse summary metadata. |

---

## 3. Shared Cross-Capability Vocabulary (Owned by Sibling Capabilities)

| Canonical Term | Owner | Usage in `cap-frame-analysis-engine` |
| --- | --- | --- |
| **ActionType** | `cap-eurocode-actions-combinations` | Stored as a classification tag on each `LoadCase` (`Permanent`, `Imposed{category}`, `Snow`, `Wind`, `Temperature`, `Seismic`, `Accidental`). |
| **Combination** | `cap-eurocode-actions-combinations` | Passed into `cap-frame-analysis-engine` as a factored linear combination of `LoadCase` IDs ($\sum \gamma_i \psi_i \cdot \text{LC}_i$) with a `LimitState` tag (`ULS_STR`, `ULS_GEO`, `SLS_CHAR`, `SLS_FREQ`, `SLS_QP`, `SEISMIC`). |
| **DesignSpectrum** | `cap-eurocode-actions-combinations` | Passed into `ResponseSpectrum` and `LateralForce` analyses as a curve parameters object / evaluator $S_d(T)$ along with seismic mass combination rules ($G + \psi_{2,i} Q_i$). |
| **MethodEligibility** | `cap-structural-mechanics-teaching` | Evaluated by inspecting `StructuralModel` and `Diagnostics` (e.g. `n_s`, element kinds, releases); `DsmWalkthroughData` is always eligible for any valid linear static model. |
| **ProjectDocument** | `cap-project-persistence` | Serializes `StructuralModel` inside the `"structural"` section of the versioned project JSON file. |

---

## 4. Policies, Sign Conventions & Mathematical Rules

| Rule / Policy Name | Canonical Specification |
| --- | --- |
| **Internal SI Unit System** | Forces in $\text{kN}$, lengths in $\text{m}$, moments in $\text{kNm}$, distributed loads in $\text{kN/m}$, pressures/stresses in $\text{kN/m}^2$ ($\text{kPa}$), temperatures in $^\circ\text{C}$, masses in $\text{t}$ ($10^3\text{ kg}$). Young's modulus $E$ and shear modulus $G$ are entered at the UI/API boundary in $\text{GPa}$ and converted to $\text{kN/m}^2$ ($\times 10^6$) internally. |
| **Global Coordinate Frame** | Right-handed 2D Cartesian frame: $+X$ horizontal to the right, $+Y$ vertical upwards, $+\theta_z$ counter-clockwise rotation in radians. |
| **Local Element Coordinate Frame** | Origin at start node $i$, $+x_{\text{local}}$ directed along the chord from node $i$ to node $j$ ($\cos\alpha = (x_j-x_i)/L$, $\sin\alpha = (y_j-y_i)/L$), $+y_{\text{local}}$ obtained by rotating $+x_{\text{local}}$ counter-clockwise by $90^\circ$ ("top" fibre for a horizontal left-to-right beam). |
| **Greek Diagram Sign Convention** | **Axial force $N(x)$:** positive in tension ($+$), negative in compression ($-$). **Shear force $Q(x)$:** positive when the left cross-section tends to move upwards ($+y_{\text{local}}$) relative to the right cross-section (clockwise rotation of differential element). **Bending moment $M(x)$:** positive when the bottom fibre ($-y_{\text{local}}$) is in tension; **bending moment diagrams are always plotted on the tension side of the member**. |
| **Linear vs. Nonlinear Superposition Policy** | Superposition of unit/individual `LoadCase` solutions is strictly restricted to purely linear models (`LinearStatic` without `Cable` elements, without `unilateral` supports, and without tension/compression-only `TrussBar`s). Whenever geometric nonlinearity (`SecondOrderPDelta`), `Cable` elements, or unilateral contact/bars are present, every target `Combination` is assembled and solved from scratch as an independent nonlinear boundary value problem. |

---

## 5. Terminology Stability Assessment

- **Resolved Collisions:**
  - *SketchModel vs. StructuralModel:* Standardized on `StructuralModel`; `SketchModel` is retired.
  - *Internal Hinge vs. EndRelease:* Standardized on `EndRelease` in the domain model; "Internal Hinge at Node" is strictly a UI modeling shortcut in `cap-sketcher-frame-modeler` that sets `moment: true` on $m-1$ of the $m$ frame member ends meeting at that node.
  - *Beam/Column vs. FrameMember:* Standardized on `FrameMember` for 6-DOF bending+axial elements and `TrussBar` for 2-DOF (local) axial-only elements.
- **Open Terminology Issues:** None. All terms align with `docs/architecture/cross-capability-conventions.md`.
