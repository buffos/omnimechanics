---
type: capability
title: Structural Mechanics Step-by-Step Educational & Analysis Engine
description: Pedagogical structural mechanics analysis platform for 2D isostatic and hyperstatic frames, trusses, and beams, offering interactive step-by-step analytical derivations, equilibrium cuts, internal force diagrams (M, Q, N), Cross & Kani iterative tables, virtual work deflections, and exportable handout sheets.
tags: [teaching, structural-mechanics, isostatic, hyperstatic, clapeyron, cross, kani, slope-deflection, virtual-work, castigliano, m-q-n-diagrams, step-walker, latex-export]
timestamp: 2026-10-30T21:15:00Z
state: in-progress
project: /project.md
parent: /project.md
issues:
  - docs/agents/issues/done/20261030-014-structural-mechanics-teaching-slice1.md
---

# Intent

Provide a comprehensive, interactive structural mechanics educational platform ("Teaching Mode") that guides civil and mechanical engineering students and practitioners through classical structural analysis methodologies step-by-step. Instead of functioning solely as a black-box numerical solver, it exposes every analytical equilibrium equation, free-body cut, virtual work integration, and iterative cycle table (Cross and Kani) with mathematical rigor, symbolic/rational clarity, and pedagogical visual feedback on the canvas.

# Scope

### Included:
1. **Isostatic Structural Systems (Ισοστατικοί Φορείς):**
   - **Truss Analysis (Δικτυώματα):**
     - *Method of Joints (Μέθοδος των Κόμβων):* Automated search for solvable sequence (joints with $\le 2$ unknown bar forces), explicit isolated joint diagrams, and $\Sigma F_x=0, \Sigma F_y=0$ equations.
     - *Method of Sections (Μέθοδος Ritter / Τομών):* Interactive selection of cuts crossing 3 non-concurrent bars, generation of Free-Body Diagrams (FBD), and moment equilibrium equations around Ritter intersection points.
   - **Beam & Simple Frame Analysis (Δοκοί & Πλαίσια):**
     - Symbolic and exact rational resolution of support reactions ($\Sigma F_x=0, \Sigma F_y=0, \Sigma M=0$).
     - Piecewise analytical internal force equations $N(x), Q(x), M(x)$ across all spans and member intervals.
     - Automated calculation of critical points: shear roots ($Q(x_0)=0$), maximum bending moments ($M_{\max} = M(x_0)$), and moment inflection points ($M(x)=0$).
     - Live interactive diagram overlays ($N, Q, M$) with cross-section mouse inspection.

2. **Hyperstatic Structural Systems (Υπερστατικοί Φορείς):**
   - **Force Method (Μέθοδος των Δυνάμεων / Flexibility Method):**
     - Automated degree of static indeterminacy calculation ($n = r + b - 2j$).
     - Selection and display of the primary isostatic system ($M_0, Q_0, N_0$).
     - Generation of unit virtual systems ($M_1, M_2, \dots, M_n$) for redundant forces $X_i$.
     - Analytical evaluation of flexibility coefficients $\delta_{ik} = \int \frac{M_i M_k}{EI} dx$ via Vereshchagin / Simpson geometric rules.
     - Formulation and solution of the compatibility matrix system $[\delta]\{X\} + \{\delta_0\} = 0$.
     - Superposition of final internal force distributions ($M = M_0 + \sum X_i M_i$).
   - **Clapeyron Three-Moment Equation (Εξίσωση των Τριών Ροπών):**
     - Multi-span continuous beams with arbitrary distributed loads, point loads, and support settlements.
     - Explicit step-by-step equations connecting support moments $M_{i-1}, M_i, M_{i+1}$.
   - **Moment Distribution Method (Μέθοδος Cross):**
     - Step 1: Member stiffness calculation ($k = 4EI/L$ or $3EI/L$ for hinged ends).
     - Step 2: Distribution Factors ($DF$) and Fixed-End Moments ($FEM$).
     - Step 3: Interactive live balancing table with cycles of distribution and carry-over ($COF = 0.5$) until convergence ($< 0.01 \text{ kNm}$).
   - **Kani Iteration Method (Μέθοδος Kani):**
     - Rotation and translation (sway) balance for multi-story, multi-bay unbraced frames.
     - Step-by-step iteration table tracking joint rotation contributions $M'_{ij}$ and story sway displacements $M''_{ij}$.
   - **Slope-Deflection Method (Μέθοδος Στροφών):**
     - Formulation with joint rotation angles $\theta_i$ and chord drift angles $\psi_{ij}$.

3. **Energy Principles & Deflection Computations:**
   - **Principle of Virtual Work (Αρχή Δυνατών Έργων - ΑΔΕ / Unit Load Method):**
     - Step-by-step computation of nodal deflections $\Delta$ and member rotations $\theta$ under real loads via virtual dummy loads $\bar{P}=1$ or $\bar{M}=1$.
   - **Castigliano's Theorems (Θεωρήματα Castigliano I & II):**
     - Strain energy differentiation $U = \int \frac{M^2}{2EI} dx$ with respect to load parameters.

4. **Pedagogical Step-Walker & UI Experience:**
   - **Interactive Timeline Controls:** Forward/backward stepper (`Next Step`, `Prev Step`, `Auto-Play`, `Jump to Phase`).
   - **Dual-View Educational Interface:**
     - *Canvas Highlight:* Synchronized highlighting of the current active joint, member cut, or support on the 2D/3D viewport, with force vector arrows and sign conventions.
     - *Pedagogical Step Card:* Displays:
       1. Educational title and rationale (e.g. *"Step 2: Joint B Equilibrium - selecting B because only 2 unknown member forces remain"*).
       2. Symbolic equation ($\Sigma F_y = 0 \Rightarrow R_B - N_{AB}\sin\theta - P = 0$).
       3. Exact rational substitution ($\Sigma F_y = 0 \Rightarrow 15 - N_{AB}\cdot \frac{\sqrt{2}}{2} - 10 = 0$).
       4. Result with engineering interpretation ($N_{AB} = +7.07\text{ kN}$ [Tension / Εφελκυσμός]).
       5. Progressive construction of the final $M, Q, N$ diagram curves.
   - **Exportable Handout & Exam Generator:**
     - One-click export of the full pedagogical derivation into a LuaLaTeX / PDF handout formatted for university courses, including TikZ vector schematics, step cards, and final diagrams.

### Excluded:
- Non-linear plastic hinge collapse mechanisms (deferred to plastic limit analysis).
- Dynamic modal superposition (owned by the Dynamic Solver Core).

---

# Architecture & Domain Model

### 1. Data Contracts & Step Representation

```rust
// Rust Domain Representation of a Pedagogical Step
pub struct PedagogicalStep {
    pub step_index: usize,
    pub phase: PedagogicalPhase,
    pub title: String,
    pub rationale: String,
    pub target_entity_id: String, // Joint ID, Member ID, or Cut ID
    pub active_diagram: Option<DiagramType>, // N, Q, or M
    pub symbolic_equation: String,          // e.g. "\Sigma F_y = R_A - q \cdot L = 0"
    pub numeric_equation: String,           // e.g. "R_A - 5.0 \cdot 6.0 = 0"
    pub solved_values: Vec<SolvedQuantity>,  // Variable name, numerical value, unit, status (Tension/Compression)
    pub free_body_cut: Option<FreeBodyCutGeometry>,
    pub explanation_markdown: String,
}

pub enum PedagogicalPhase {
    SupportReactions,
    TrussJointEquilibrium,
    TrussRitterSection,
    BeamPiecewiseInternalForces,
    ForceMethodFlexibilityMatrix,
    ClapeyronEquations,
    CrossMomentDistributionCycle(usize),
    KaniIterationCycle(usize),
    VirtualWorkDeflection,
    FinalDiagramAssembly,
}

pub struct SolvedQuantity {
    pub symbol: String,
    pub exact_fraction: Option<(i64, i64)>, // p / q
    pub decimal_value: f64,
    pub unit: String,
    pub interpretation: Option<String>, // e.g. "Tension", "Compression", "Sagging", "Hogging"
}
```

### 2. Cross and Kani Table State Machine

```rust
pub struct CrossCycle {
    pub cycle_number: usize,
    pub distributed_moments: HashMap<MemberEndId, f64>,
    pub carry_over_moments: HashMap<MemberEndId, f64>,
    pub unbalanced_joint_moments: HashMap<JointId, f64>,
    pub max_unbalance: f64,
}

pub struct KaniCycle {
    pub iteration_number: usize,
    pub rotation_contributions: HashMap<MemberEndId, f64>,
    pub sway_contributions: HashMap<StoryId, f64>,
    pub convergence_residual: f64,
}
```

---

# Acceptance Scenarios

### Scenario 1: Isostatic Cantilever & Simple Beams with Piecewise M-Q-N
- **Given** a 2D simply supported beam of span $L=6\text{ m}$ under a uniform load $q=10\text{ kN/m}$ and a point load $P=20\text{ kN}$ at midspan.
- **When** the student initiates "Solve Step-by-Step".
- **Then**:
  1. Step 1 derives support reactions $R_A = 40\text{ kN}, R_B = 40\text{ kN}$ with moment equilibrium about $A$.
  2. Step 2 formulates interval $[0, 3\text{ m}]$: $Q(x) = 40 - 10x$, $M(x) = 40x - 5x^2$.
  3. Step 3 detects shear root $Q(x)=0 \Rightarrow x=4\text{ m}$ (or at point load discontinuity $x=3\text{ m}$) and calculates exact maximum bending moment $M_{\max} = 75\text{ kNm}$.
  4. The canvas highlights the current slice $x$ and graphs the parabola incrementally.

### Scenario 2: Method of Joints on Pratt / Howe Truss
- **Given** an isostatic 2D bridge truss with 5 panels under vertical nodal loads.
- **When** the student selects "Method of Joints".
- **Then**:
  1. The solver identifies support joint $A$ with only 2 unknown member forces.
  2. The UI renders the isolated joint with reaction vector and unknown bar arrows.
  3. Steps sequentially solve every joint in topological dependency order.
  4. Tension bars are rendered in blue ($+$) and compression bars in red ($-$) on the truss canvas.

### Scenario 3: Method of Sections (Ritter) on Selected Bay
- **Given** the user selects 3 truss members to cut on the canvas.
- **When** the student clicks "Evaluate Ritter Cut".
- **Then**:
  1. The engine generates a clean Free-Body Diagram of the isolated half.
  2. It identifies the Ritter intersection pivot point (intersection of the other 2 bars) to solve the 3rd bar directly via $\Sigma M_{\text{pivot}} = 0$ in a single equation.
  3. The full derivation is presented in the step card with KaTeX math.

### Scenario 4: Continuous Beam via Clapeyron Three-Moment Equation
- **Given** a 3-span continuous beam ($L_1=4\text{ m}, L_2=6\text{ m}, L_3=4\text{ m}$) with unequal loads.
- **When** the student selects "Clapeyron Method".
- **Then**:
  1. The engine sets up the three-moment equation for each internal support.
  2. Explicitly shows the $6 \cdot A_i a_i / L_i$ load terms.
  3. Solves the tridiagonal system for support moments $M_B, M_C$.
  4. Superposes the isostatic span moments to plot the continuous $M(x)$ envelope.

### Scenario 5: Moment Distribution (Cross) with Interactive Cycle Table
- **Given** a 2-span continuous beam or rigid frame with fixed and pinned ends.
- **When** the student selects "Cross Method".
- **Then**:
  1. The engine computes stiffnesses $k$ and distribution factors $DF$ at all rigid joints.
  2. Fixed-end moments $FEM$ are populated in a formatted interactive spreadsheet.
  3. The student can click "Next Cycle" or "Auto-Balance", watching the moment balance and $0.5$ carry-over iterations until the unbalanced moment is $< 0.01\text{ kNm}$.

### Scenario 6: Rigid Frame with Side Sway via Kani Method
- **Given** an unbraced portal frame subjected to both gravity and lateral wind loads.
- **When** the student selects "Kani Method".
- **Then**:
  1. The engine computes joint rotation factors $\mu_{ij}$ and story sway factors $\nu_r$.
  2. It generates the iterative calculation table showing joint moments $M'_{ij}$ and column sway moments $M''_{r}$.
  3. Steps demonstrate the convergence of sway and rotation contributions.

### Scenario 7: Deflection via Principle of Virtual Work (ΑΔΕ)
- **Given** an isostatic or hyperstatic frame where the student queries the vertical deflection at point $C$.
- **When** the student selects "Virtual Work Deflection (ΑΔΕ)".
- **Then**:
  1. The engine calculates the real moment diagram $M_0(x)$.
  2. It removes all loads and places a unit dummy load $\bar{P}=1$ at point $C$, calculating the virtual moment diagram $\bar{M}(x)$.
  3. It presents the Vereshchagin integration table (shape areas and centroid ordinates) to compute $\Delta_C = \sum \frac{1}{EI} \int M_0 \bar{M} dx$ with exact rational fractions.

### Scenario 8: Automated Handout & Exam Sheet Export
- **Given** any solved structural problem in Teaching Mode.
- **When** the student or instructor clicks "Export Handout".
- **Then**:
  1. A standalone LuaLaTeX/PDF document is compiled containing: Problem statement, schematic TikZ diagram, step-by-step analytical equations, Cross/Kani convergence tables, and full $M, Q, N$ diagrams.

---

# Relationships

- Parent: [OmniMechanics Platform](/project.md)
- Feeds into: [Eurocode Reinforced Concrete Structural Design & Detailing](/capabilities/cap-eurocode-rc-design.md)
- Feeds from: [Interactive 2D/3D Canvas Drafting & Geometry Sketcher](/capabilities/cap-geometry-canvas-sketcher.md)
- Shared with: [Interactive 3D Canvas & UI](/capabilities/cap-interactive-canvas.md)
- Verification Baseline: [Solver Core Engine](/capabilities/cap-solver-core.md)
