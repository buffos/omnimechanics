---
type: capability
title: Eurocode Reinforced Concrete Structural Design & Detailing Engine
description: Educational and professional Eurocode 2 (EN 1992-1-1) structural dimensioning suite for concrete beams and columns, providing step-by-step ULS bending and shear design, interaction diagrams, SLS crack and deflection verification, and interactive 2D rebar detailing.
tags: [eurocode, ec2, reinforced-concrete, beam-reinforcement, column-design, detailing, uls, sls, civil-engineering, interaction-diagram]
timestamp: 2026-10-30T21:15:00Z
state: specified
project: /project.md
parent: /project.md
shared_with:
  - /capabilities/cap-eurocode-actions-combinations.md
---

# Intent

Bridge structural analysis and concrete engineering practice by delivering an interactive, step-by-step Eurocode 2 (EN 1992-1-1) concrete design suite. The engine ingests internal force envelopes ($M_{Ed}, V_{Ed}, N_{Ed}$) directly from the structural analysis teaching engine or user input, evaluates normative limit states (ULS and SLS) with explicit step-by-step mathematical formulas, and renders interactive cross-sectional and longitudinal reinforcement detailing drawings.

# Scope

### Included:
1. **Ultimate Limit State (ULS) Flexural Design of Beams (Κάμψη Δοκών):**
   - **Cross-Section Types:** Rectangular sections and Flanged T-beams / L-beams (effective flange width $b_{\text{eff}}$ per EC2 §5.3.2.1).
   - **Stress-Block Formulation:** Parabolic-rectangular or equivalent rectangular stress block ($\eta = 1.0, \lambda = 0.8$ for $f_{ck} \le 50\text{ MPa}$).
   - **Dimensioning Algorithm:**
     - Normalized bending moment: $\mu_{sd} = \frac{M_{Ed}}{b d^2 f_{cd}}$.
     - Comparison with limit $\mu_{\lim}$ (preventing brittle concrete crushing before steel yielding):
       - If $\mu_{sd} \le \mu_{\lim}$: Singly reinforced section ($\omega = 1 - \sqrt{1 - 2\mu_{sd}}$, $A_{s1} = \omega b d \frac{f_{cd}}{f_{yd}}$).
       - If $\mu_{sd} > \mu_{\lim}$: Doubly reinforced section (calculates required compression steel $A_{s2}$ and augmented tensile steel $A_{s1}$).
     - Ductility and regulatory steel bounds: $A_{s,\min} = \max\left(0.26 \frac{f_{ctm}}{f_{yk}} b_t d, 0.0013 b_t d\right)$ and $A_{s,\max} = 0.04 A_c$.
   - **Automatic Rebar Selection:** Interactive suggestion of practical commercial bar arrangements (e.g. $3\Phi 16$, $4\Phi 20$) satisfying clear spacing $s \ge \max(\Phi, d_g + 5\text{ mm}, 20\text{ mm})$.

2. **Ultimate Limit State (ULS) Shear Design of Beams (Διάτμηση Δοκών):**
   - **Variable Angle Truss Model (EC2 §6.2.3):** Strut inclination angle $1.0 \le \cot\theta \le 2.5$ ($21.8^\circ \le \theta \le 45^\circ$).
   - **Concrete Compressive Strut Check:**
     - $V_{Rd,\max} = \alpha_{cw} b_w z \nu_1 f_{cd} / (\cot\theta + \tan\theta)$, verifying $V_{Ed} \le V_{Rd,\max}$ to prevent web crushing.
   - **Stirrup Calculation:**
     - Required shear reinforcement: $\frac{A_{sw}}{s} = \frac{V_{Ed}}{z f_{ywd} \cot\theta}$.
     - Minimum shear reinforcement ratio: $\rho_w = \frac{A_{sw}}{s b_w \sin\alpha} \ge \rho_{w,\min} = 0.08 \frac{\sqrt{f_{ck}}}{f_{yk}}$.
     - Maximum longitudinal stirrup spacing: $s_{\max} = 0.75 d (1 + \cot\alpha) \le 300\text{ mm}$.
   - **Tensile Force Shift Rule:** Additional longitudinal tensile force $\Delta F_{td} = 0.5 V_{Ed} (\cot\theta - \cot\alpha)$.

3. **Ultimate Limit State (ULS) Column Design (Υποστυλώματα):**
   - **Uniaxial & Biaxial Bending with Axial Force ($N_{Ed} - M_{Ed}$):**
   - **Second-Order Effects (EC2 §5.8):** Nominal Curvature Method or Nominal Stiffness Method.
     - Slenderness criterion $\lambda = l_0 / i$. If $\lambda > \lambda_{\lim} = \frac{20 \cdot A \cdot B \cdot C}{\sqrt{n}}$, evaluate second-order moment $M_2 = N_{Ed} e_2$.
     - Total design moment: $M_{Ed} = M_{0Ed} + M_2$ with initial eccentricity $e_0 = \max(h/30, 20\text{ mm})$.
   - **Interactive $N-M$ Interaction Envelopes:**
     - Full analytical generation of the axial load - moment interaction capacity curve.
     - Dynamic plotting of the design point $(N_{Ed}, M_{Ed})$ with safety margin indicator.

4. **Serviceability Limit State (SLS) Verifications (Οριακές Καταστάσεις Λειτουργικότητας):**
   - **Stress Limitation (EC2 §7.2):** Concrete stress $\sigma_c \le 0.60 f_{ck}$ (preventing longitudinal microcracking) and steel stress $\sigma_s \le 0.80 f_{yk}$ under characteristic combinations.
   - **Crack Width Verification (EC2 §7.3):**
     - Direct calculation: $w_k = s_{r,\max} (\epsilon_{sm} - \epsilon_{cm}) \le w_{\max} = 0.3\text{ mm}$ (for exposure classes XC1-XC4).
     - Maximum crack spacing: $s_{r,\max} = 3.4 c + 0.425 k_1 k_2 \Phi / \rho_{p,\text{eff}}$.
   - **Deflection Control (EC2 §7.4):** Basic span-to-effective-depth ratio check $(L/d)$ per Table 7.4N or numerical uncracked/cracked curvature integration.

5. **Interactive 2D Rebar Detailing & Drawing:**
   - **Cross-Section View:** High-fidelity SVG/Canvas rendering showing concrete boundary, nominal concrete cover $c_{\text{nom}}$, closed stirrups with standard $135^\circ$ hooks, and longitudinal rebar circles.
   - **Longitudinal Elevation View:** Displays bar curtailment points, tension lap splices, and anchorage lengths $l_{bd} = \alpha_1 \alpha_2 \alpha_3 \alpha_4 \alpha_5 l_{b,\text{rqd}}$.
   - **Rebar Schedule (Πίνακας Οπλισμού):** Bar marks, diameters, cut lengths, total quantities, and steel weight in kg.

### Excluded:
- Prestressed concrete with post-tensioned tendons (deferred to specialized bridge module).
- Full 3D building seismic design according to Eurocode 8 (EC8 ductility classes DCL/DCM/DCH).

---

# Architecture & Domain Model

```rust
// Eurocode Concrete Cross-Section Specification
pub struct ConcreteCrossSection {
    pub shape: SectionShape, // Rectangular, TBeam, Circular
    pub width: f64,          // b (m)
    pub height: f64,         // h (m)
    pub flange_width: Option<f64>,  // b_eff for T-beams
    pub flange_height: Option<f64>, // h_f for T-beams
    pub concrete_grade: ConcreteClass, // C20/25 through C50/60
    pub steel_grade: SteelClass,       // B500C, B500A
    pub nominal_cover: f64,            // c_nom (m)
}

pub struct UlsBendingResult {
    pub mu_sd: f64,
    pub omega: f64,
    pub compression_zone_x: f64,
    pub is_doubly_reinforced: bool,
    pub required_as1: f64,      // m^2
    pub required_as2: f64,      // m^2
    pub min_as: f64,
    pub max_as: f64,
    pub suggested_rebar: RebarSelection,
}

pub struct UlsShearResult {
    pub v_ed: f64,
    pub v_rd_max: f64,
    pub cot_theta: f64,
    pub required_asw_per_meter: f64,
    pub min_asw_per_meter: f64,
    pub max_stirrup_spacing: f64,
    pub suggested_stirrups: StirrupSelection, // e.g. "Φ8/15 cm"
}

pub struct SlsVerificationResult {
    pub concrete_stress: f64,
    pub steel_stress: f64,
    pub crack_width_mm: f64,
    pub crack_width_limit_mm: f64,
    pub deflection_check_passed: bool,
}
```

---

# Acceptance Scenarios

### Scenario 1: Singly Reinforced Beam Bending Design
- **Given** a beam of $b=250\text{ mm}, h=500\text{ mm}$, $d=450\text{ mm}$, Concrete C25/30, Steel B500C, subjected to $M_{Ed} = 120\text{ kNm}$.
- **When** the student evaluates ULS flexural design.
- **Then**:
  1. $f_{cd} = 25/1.5 = 16.67\text{ MPa}$, $f_{yd} = 500/1.15 = 434.78\text{ MPa}$.
  2. $\mu_{sd} = 120 \times 10^6 / (250 \times 450^2 \times 16.67) = 0.142 < \mu_{\lim} = 0.296$.
  3. $\omega = 1 - \sqrt{1 - 2(0.142)} = 0.154$.
  4. Required $A_{s1} = 0.154 \times 250 \times 450 \times (16.67 / 434.78) = 6.64\text{ cm}^2$.
  5. The engine suggests $4\Phi 16$ ($A_s = 8.04\text{ cm}^2$) and displays the cross-section drawing with $c_{\text{nom}}=30\text{ mm}$.

### Scenario 2: Shear Design with Variable Strut Inclination
- **Given** the above beam with design shear force $V_{Ed} = 150\text{ kN}$.
- **When** the student performs shear dimensioning.
- **Then**:
  1. The student selects optimal $\cot\theta = 2.5$ ($\theta = 21.8^\circ$).
  2. The engine calculates $V_{Rd,\max} = 345\text{ kN} > V_{Ed}$ (concrete crushing avoided).
  3. Evaluates $A_{sw}/s = 150 \times 10^3 / (0.9 \times 450 \times 434.78 \times 2.5) = 0.342\text{ mm}^2/\text{mm}$.
  4. Suggests 2-legged $\Phi 8$ stirrups ($A_{sw} = 101\text{ mm}^2$) with spacing $s = 101 / 0.342 = 295\text{ mm} \rightarrow$ rounded to practical $s = 150\text{ mm}$.

### Scenario 3: Column Axial Load-Moment Interaction Diagram
- **Given** a $400 \times 400\text{ mm}$ column with $N_{Ed} = 1200\text{ kN}$ and $M_{Ed} = 180\text{ kNm}$.
- **When** the student queries column capacity.
- **Then**:
  1. The second-order eccentricity $e_2$ is computed per EC2 Nominal Curvature.
  2. The $N-M$ interaction curve is drawn with characteristic failure zones (pure compression, balanced failure, pure tension).
  3. The operating point $(N_{Ed}, M_{Ed})$ is plotted, indicating whether reinforcement is adequate or requires augmentation.

### Scenario 4: SLS Crack Width Control
- **Given** a beam in exposure class XC3 requiring $w_k \le 0.3\text{ mm}$.
- **When** checking serviceability under quasi-permanent moment $M_{Eqp} = 85\text{ kNm}$.
- **Then**:
  1. Engine calculates cracked moment of inertia $I_{II}$ and steel tensile strain $\epsilon_{sm}$.
  2. Computes maximum crack spacing $s_{r,\max}$ and characteristic crack width $w_k = 0.22\text{ mm} \le 0.30\text{ mm}$ (PASS).

---

# Relationships

- Shared concern: [Eurocode Actions & Combinations (EN 1990/1998)](/capabilities/cap-eurocode-actions-combinations.md) (combination factors and envelopes)
- Feeds from: [2D Frame Analysis Engine](/capabilities/cap-frame-analysis-engine.md) (M_Ed, V_Ed, N_Ed envelopes)
- Parent: [OmniMechanics Platform](/project.md)
- Ingests internal force envelopes from: [Structural Mechanics Teaching Engine](/capabilities/cap-structural-mechanics-teaching.md)
- Feeds interactive drawings into: [Interactive 3D Canvas & UI](/capabilities/cap-interactive-canvas.md)
- Verification Baseline: [Solver Core Engine](/capabilities/cap-solver-core.md)
