## Issue Metadata

- Issue number: `039`
- Owning capability node: `/.okf/capabilities/cap-eurocode-rc-design.md`
- Artifact root: `.okf/capabilities/cap-eurocode-rc-design.md`
- Issue file: `docs/agents/issues/pending/20261103-039-ec2-column-nm-interaction-sls-cracks-rebar-detailing-ui.md`
- Category: `feature`
- Execution type: `AFK`
- Review gate: `visual-review`
- Suggested state: `ready-for-agent`

## Parent Artifacts

- `docs/prd.md`
- `docs/architecture/application-architecture-summary.md`
- `docs/architecture/cross-capability-conventions.md`
- `.okf/capabilities/cap-eurocode-rc-design.md`
- `.okf/capabilities/cap-eurocode-actions-combinations.md`
- `.okf/capabilities/cap-sketcher-frame-modeler.md`
- `.okf/capabilities/cap-structural-mechanics-teaching.md`

## What to build

Implement **Vertical Slice 2** of `cap-eurocode-rc-design`: **ULS Column Design with Second-Order Slenderness ($M_2$) and Analytical $N\text{-}M$ Interaction Envelopes** (`Scenario 3`), **SLS Stress, Crack Width ($w_k \le w_{\max}$), and Deflection ($L/d$) Verifications** (`Scenario 4`), and the **Interactive Eurocode 2 RC Design & 2D Rebar Detailing Studio UI** (`src/app/features/rc-design/`), accessible from both Mode 2 (reading member governing $M_{Ed}, V_{Ed}, N_{Ed}$ envelopes) and Mode 3.

### 1. ULS Column $N\text{-}M$ Interaction & 2nd-Order Nominal Curvature (`src-tauri/src/rc_design/column.rs`):
- Compute column slenderness $\lambda = l_0 / i$, limit slenderness $\lambda_{\lim} = \frac{20 \cdot A \cdot B \cdot C}{\sqrt{n}}$ (`EC2 §5.8.3.1`), minimum eccentricity $e_0 = \max(h/30, 20\text{ mm})$, and second-order moment $M_2 = N_{Ed} e_2$ via the Nominal Curvature Method (`Scenario 3`).
- Generate the analytical $(N_{Rd}, M_{Rd})$ interaction capacity envelope across strain regimes (pure axial compression, decompression, balanced failure at $\epsilon_{cu2}=3.5\text{‰}, \epsilon_s = \epsilon_{yd}$, pure bending, and pure tension) and evaluate safety factor / adequacy for operating point $(N_{Ed}, M_{Ed})$ (`Scenario 3`).

### 2. SLS Stress, Crack Width ($w_k$) & Deflection ($L/d$) Engine (`src-tauri/src/rc_design/sls.rs`):
- Compute cracked/uncracked section properties ($x_{II}, I_{II}$), concrete compressive stress $\sigma_c \le 0.60 f_{ck}$, and steel tensile stress $\sigma_s \le 0.80 f_{yk}$ (`EC2 §7.2`).
- Compute direct crack width $w_k = s_{r,\max}(\epsilon_{sm} - \epsilon_{cm}) \le w_{\max}$ (`0.30 mm` for XC2–XC4) with $s_{r,\max} = 3.4 c + 0.425 k_1 k_2 \Phi / \rho_{p,\text{eff}}$ (`EC2 §7.3`, `Scenario 4`) and span-to-effective-depth ratio $L/d$ verification (`EC2 §7.4`).
- Compute anchorage length $l_{bd}$ (`EC2 §8.4`) and rebar schedule quantities (bar marks, cut lengths, total steel weight in $\text{kg}$).
- Register Tauri IPC commands `rc_design_column_interaction`, `rc_design_sls_verification`, and `rc_design_member_from_envelope`.

### 3. Interactive EC2 Design & 2D Rebar Detailing Studio UI (`src/app/features/rc-design/`):
- Provide the **EC2 RC Design & Detailing Drawer / Modal** launchable directly from any concrete member in Mode 2 ("Design RC Member (EC2)" button in `PropertyInspectorComponent` / `EndForceTableComponent`, pre-populated with governing ULS $M_{Ed}, V_{Ed}, N_{Ed}$ and SLS $M_{Eqp}$ envelopes) and from Mode 3 (`TeachingStudioComponent`).
- Render:
  - **Step-by-step EC2 Formula Cards** (KaTeX equations for ULS bending, ULS shear, column $M_2$, and SLS crack width $w_k$).
  - **2D Cross-Section SVG Drawing:** concrete outline, nominal cover $c_{\text{nom}}$, closed stirrups with $135^\circ$ seismic hooks, and positioned tension/compression rebar circles ($\Phi$).
  - **Longitudinal Elevation & Rebar Schedule Table (`Πίνακας Οπλισμού`):** top/bottom bars, stirrup zones, anchorage lengths $l_{bd}$, and total steel weight ($\text{kg}$).
  - **Interactive $N\text{-}M$ Interaction Diagram SVG:** capacity curve with plotted design point $(N_{Ed}, M_{Ed})$.

## Acceptance criteria

- [ ] For a $400 \times 400\text{ mm}$ column under $N_{Ed} = 1200\text{ kN}$ and $M_{Ed} = 180\text{ kNm}$ (`Scenario 3`), `rc_design_column_interaction` evaluates slenderness $\lambda$, second-order eccentricity $e_2$, generates the full $N\text{-}M$ interaction curve, and plots the operating point $(N_{Ed}, M_{Ed})$.
- [ ] For the `Scenario 1` beam in exposure class `XC3` under quasi-permanent moment $M_{Eqp} = 85\text{ kNm}$ (`Scenario 4`), `rc_design_sls_verification` computes cracked inertia $I_{II}$, $\epsilon_{sm} - \epsilon_{cm}$, $s_{r,\max}$, and verifies $w_k \le 0.30\text{ mm}$ (`PASS`).
- [ ] Users can open the EC2 RC Design Studio from Mode 2 or Mode 3, inspect step-by-step ULS/SLS derivations, view the 2D SVG cross-section and longitudinal rebar detailing drawings, view the $N\text{-}M$ curve, and inspect the rebar schedule (`Πίνακας Οπλισμού`).
- [ ] Rust unit tests (`rc_column_sls_test.rs`), Vitest tests (`rc-design.spec.ts`), `npx ng build`, and `npm run check:limits` pass with 0 errors.

## Artifact sync required

- Application PRD: `none - already covered in docs/prd.md`
- Application architecture summary: `none - matches docs/architecture/application-architecture-summary.md`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-eurocode-rc-design.md (transition state to implemented upon completion of 038-039; transition root project.md once all capabilities are implemented)`
- Issue registry: `required`; node `issues:` reference: `required: .okf/capabilities/cap-eurocode-rc-design.md`
- Reason/no-impact decision: `Completes all 4 acceptance scenarios (Scenarios 1..4) of cap-eurocode-rc-design and exposes interactive 2D rebar detailing in Mode 2 and Mode 3.`

## Human review gate

`visual-review`: User selects a solved beam/column in Mode 2 (or Mode 3), opens the EC2 RC Design & Detailing Studio, inspects ULS bending/shear steps, views the 2D SVG concrete cross-section with stirrups and rebars, checks the column $N\text{-}M$ interaction diagram, and verifies SLS crack width $w_k \le 0.30\text{ mm}$.

## Blocked by

- Blocked by `docs/agents/issues/pending/20261103-038-ec2-uls-beam-flexural-shear-design-engine.md`

## Artifact anchors

- Capability spec (`.okf/capabilities/cap-eurocode-rc-design.md`): Scope §3 (ULS Column Design & $N\text{-}M$ Interaction), §4 (SLS Verifications), §5 (Interactive 2D Rebar Detailing & Rebar Schedule), `SlsVerificationResult`

## Acceptance scenarios addressed

- `Scenario 1`: Singly Reinforced Beam Bending Design
- `Scenario 2`: Shear Design with Variable Strut Inclination
- `Scenario 3`: Column Axial Load-Moment Interaction Diagram
- `Scenario 4`: SLS Crack Width Control

## Verification obligations

- Policy source: `/.okf/project.md`

| Scenario | Backend boundary | Frontend integration | End-to-end journey |
|---|---|---|---|
| `Scenario 1` | `implemented: Issue 038` | `planned` | `catalog-only` |
| `Scenario 2` | `implemented: Issue 038` | `planned` | `catalog-only` |
| `Scenario 3` | `planned` | `planned` | `catalog-only` |
| `Scenario 4` | `planned` | `planned` | `catalog-only` |
