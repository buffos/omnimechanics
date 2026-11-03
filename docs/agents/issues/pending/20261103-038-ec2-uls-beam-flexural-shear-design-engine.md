## Issue Metadata

- Issue number: `038`
- Owning capability node: `/.okf/capabilities/cap-eurocode-rc-design.md`
- Artifact root: `.okf/capabilities/cap-eurocode-rc-design.md`
- Issue file: `docs/agents/issues/pending/20261103-038-ec2-uls-beam-flexural-shear-design-engine.md`
- Category: `feature`
- Execution type: `AFK`
- Review gate: `none`
- Suggested state: `ready-for-agent`

## Parent Artifacts

- `docs/prd.md`
- `docs/architecture/application-architecture-summary.md`
- `docs/architecture/cross-capability-conventions.md`
- `.okf/capabilities/cap-eurocode-rc-design.md`
- `.okf/capabilities/cap-eurocode-actions-combinations.md`

## What to build

Implement **Vertical Slice 1** of `cap-eurocode-rc-design`: the Rust Eurocode 2 (`EN 1992-1-1` + Greek National Annex) **ULS Flexural Design (`Κάμψη Δοκών`)** and **ULS Shear Design (`Διάτμηση Δοκών`)** engine (`src-tauri/src/rc_design/`) for rectangular and flanged T/L concrete beams (`Scenario 1`, `Scenario 2`), complete with step-by-step pedagogical formula derivations and commercial rebar/stirrup selection.

### 1. Concrete & Reinforcing Steel Catalog and Cross-Section Model (`src-tauri/src/rc_design/types.rs`):
- Define `ConcreteCrossSection` (`Rectangular`, `TBeam`, `Circular`, dimensions $b, h, b_{\text{eff}}, h_f, c_{\text{nom}}$), `ConcreteClass` (`C20/25` through `C50/60` with $f_{ck}, f_{ctm}, E_{cm}, f_{cd} = \alpha_{cc} f_{ck} / \gamma_c$), and `SteelClass` (`B500C`, `B500A` with $f_{yk} = 500\text{ MPa}, f_{yd} = f_{yk} / 1.15 = 434.78\text{ MPa}, E_s = 200\text{ GPa}$).

### 2. ULS Flexural Dimensioning Engine (`src-tauri/src/rc_design/flexure.rs`):
- Implement rectangular and T-beam stress-block flexural design (`EC2 §6.1`, `Scenario 1`):
  - Compute effective depth $d$, normalized moment $\mu_{sd} = \frac{|M_{Ed}|}{b d^2 f_{cd}}$, and ductility limit $\mu_{\lim}$ (default $0.296$ for $\xi_{\lim} = 0.45$).
  - For $\mu_{sd} \le \mu_{\lim}$ (singly reinforced): compute mechanical reinforcement ratio $\omega = 1 - \sqrt{1 - 2\mu_{sd}}$, neutral axis depth $x = \frac{\omega}{\lambda} d$, and required tension steel $A_{s1} = \omega b d \frac{f_{cd}}{f_{yd}}$.
  - For $\mu_{sd} > \mu_{\lim}$ (doubly reinforced): compute compression steel $A_{s2}$ and total tension steel $A_{s1}$.
  - Enforce regulatory bounds $A_{s,\min} = \max\left(0.26 \frac{f_{ctm}}{f_{yk}} b_t d, 0.0013 b_t d\right)$ and $A_{s,\max} = 0.04 A_c$.
  - Select practical commercial bar arrangements ($\Phi 12 \dots \Phi 28$) satisfying clear bar spacing $s_{\text{clear}} \ge \max(\Phi, d_g + 5\text{ mm}, 20\text{ mm})$ and emit bilingual step-by-step derivations.

### 3. ULS Variable-Angle Truss Shear Engine (`src-tauri/src/rc_design/shear.rs`):
- Implement variable strut inclination shear design (`EC2 §6.2.3`, $1.0 \le \cot\theta \le 2.5$, `Scenario 2`):
  - Compute concrete strut crushing capacity $V_{Rd,\max} = \frac{\alpha_{cw} b_w z \nu_1 f_{cd}}{\cot\theta + \tan\theta}$ ($z = 0.9 d$) and check $V_{Ed} \le V_{Rd,\max}$.
  - Compute required stirrup ratio $\frac{A_{sw}}{s} = \frac{V_{Ed}}{z f_{ywd} \cot\theta}$, minimum ratio $\rho_{w,\min} = \frac{0.08 \sqrt{f_{ck}}}{f_{yk}}$, maximum spacing $s_{\max} = \min(0.75 d (1 + \cot\alpha), 300\text{ mm})$, and tensile force shift $\Delta F_{td} = 0.5 V_{Ed}(\cot\theta - \cot\alpha)$.
  - Select practical commercial stirrups (e.g., 2-legged or 4-legged $\Phi 8 / \Phi 10$ at $100..250\text{ mm}$ spacing) and register Tauri IPC commands `rc_design_uls_bending` and `rc_design_uls_shear`.

## Acceptance criteria

- [ ] Given $b=250\text{ mm}, h=500\text{ mm}, d=450\text{ mm}$, `C25/30`, `B500C`, and $M_{Ed} = 120\text{ kNm}$ (`Scenario 1`), `rc_design_uls_bending` computes $f_{cd} = 16.67\text{ MPa}$, $f_{yd} = 434.78\text{ MPa}$, $\mu_{sd} \approx 0.142 < \mu_{\lim}$, $\omega \approx 0.154$, required $A_{s1} \approx 6.64\text{ cm}^2$, and suggests $4\Phi 16$ ($8.04\text{ cm}^2$).
- [ ] Given the same section under $V_{Ed} = 150\text{ kN}$ and $\cot\theta = 2.5$ (`Scenario 2`), `rc_design_uls_shear` verifies $V_{Rd,\max} \approx 345\text{ kN} > V_{Ed}$, computes $A_{sw}/s \approx 0.342\text{ mm}^2/\text{mm}$, and suggests 2-legged $\Phi 8$ stirrups within $s \le s_{\max}$.
- [ ] Doubly reinforced sections ($\mu_{sd} > \mu_{\lim}$) and T-beams ($M_{Ed} > 0$ compression in flange vs. $M_{Ed} < 0$ web in compression) are accurately dimensioned.
- [ ] Rust unit tests in `src-tauri/src/rc_design/rc_uls_test.rs` pass 100% (`cargo test --lib`), and `npm run check:limits` passes.

## Artifact sync required

- Application PRD: `none - already covered in docs/prd.md`
- Application architecture summary: `none - matches docs/architecture/application-architecture-summary.md`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-eurocode-rc-design.md`
- Issue registry: `required`; node `issues:` reference: `required: .okf/capabilities/cap-eurocode-rc-design.md`
- Reason/no-impact decision: `Delivers backend EC2 ULS flexural and shear dimensioning algorithms.`

## Human review gate

`none`: Pure backend Eurocode 2 ULS calculation slice; interactive RC design UI and rebar detailing drawings are delivered in Issue `039`.

## Blocked by

- None - can start immediately

## Artifact anchors

- Capability spec (`.okf/capabilities/cap-eurocode-rc-design.md`): Scope §1 (ULS Flexural Design), §2 (ULS Shear Design), `ConcreteCrossSection`, `UlsBendingResult`, `UlsShearResult`

## Acceptance scenarios addressed

- `Scenario 1`: Singly Reinforced Beam Bending Design
- `Scenario 2`: Shear Design with Variable Strut Inclination

## Verification obligations

- Policy source: `/.okf/project.md`

| Scenario | Backend boundary | Frontend integration | End-to-end journey |
|---|---|---|---|
| `Scenario 1` | `planned` | `deferred: Issue 039` | `catalog-only` |
| `Scenario 2` | `planned` | `deferred: Issue 039` | `catalog-only` |
