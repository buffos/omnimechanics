## Issue Metadata

- Issue number: `022`
- Owning capability node: `/.okf/capabilities/cap-eurocode-actions-combinations.md`
- Artifact root: `docs/architecture/cap-eurocode-actions-combinations/`
- Issue file: `docs/agents/issues/pending/20261102-022-en1998-seismic-spectra-mass-lateral-cqc.md`
- Category: `feature`
- Execution type: `AFK`
- Review gate: `none`
- Suggested state: `done`

## Parent Artifacts

- `docs/prd.md`
- `docs/architecture/application-architecture-summary.md`
- `docs/architecture/cross-capability-conventions.md`
- `.okf/capabilities/cap-eurocode-actions-combinations.md`
- `docs/architecture/cap-eurocode-actions-combinations/prd.md`
- `docs/architecture/cap-eurocode-actions-combinations/canonical-domain-model.md`
- `docs/architecture/cap-eurocode-actions-combinations/canonical-use-cases.md`
- `docs/architecture/cap-eurocode-actions-combinations/canonical-api-cli-contract.md`
- `docs/architecture/cap-eurocode-actions-combinations/acceptance-scenarios.md`
- `docs/architecture/cap-eurocode-actions-combinations/readiness-review.md`

## What to build

Implement **Vertical Slice 2** (final slice) of `cap-eurocode-actions-combinations`: the EN 1998-1 Type 1 horizontal & vertical elastic/design response spectrum engine ($S_e(T), S_d(T)$), seismic mass participation rule ($G + \psi_{E,i} Q$), signed `SEISMIC` load combinations ($G + \psi_2 Q \pm E_x$ and $30\%$ bidirectional rule $\pm 1.00 E_x \pm 0.30 E_z$), equivalent static lateral force distribution ($F_b, F_i$), and Der Kiureghian CQC/SRSS modal correlation matrix generator.

### 1. EN 1998-1 Seismic Action & Spectrum Engine (`src-tauri/src/actions/seismic.rs`):
- **`SeismicSettings` & `DesignSpectrum`:**
  - Greek National Annex seismic hazard zones (`ZoneI` $0.16g$, `ZoneII` $0.24g$, `ZoneIII` $0.36g$, `Custom`), Importance Classes I–IV ($\gamma_I \in \{0.80, 1.00, 1.20, 1.40\}$), Ground Types `A`–`E` ($S, T_B, T_C, T_D$), behavior factor $q$, damping ratio $\zeta$ ($\eta = \max(\sqrt{10/(5+\zeta)}, 0.55)$), and lower-bound factor $\beta = 0.20$.
  - Horizontal elastic $S_e(T)$ and design $S_d(T)$ spectrum evaluator across $0 \le T \le 4.0\text{ s}$ plus exact modal period queries `query_periods_s`, and optional Type 1 vertical spectrum ($a_{vg} = 0.90 a_g, S_v = 1.0, T_B = 0.05\text{ s}, T_C = 0.15\text{ s}, T_D = 1.00\text{ s}, q_v$) (`SC-005`).
- **Seismic Mass Rule & `SEISMIC` Combinations (`combinations.rs` & `seismic.rs`):**
  - Derive `SeismicMassCombinationRule` ($1.00 G_{k,j} + \psi_{E,i} Q_{k,i}$ where $\psi_{E,i} = \varphi \cdot \psi_{2,i}$) (`SC-006`).
  - Expand `GenerateCombinations` for `ActionType::Seismic` load cases into signed `SEISMIC` (`Eq6_12b`) combinations ($\pm 1.00 E_x$ for horizontal-only; 8 signed $\pm 1.00 E_x \pm 0.30 E_z$ and $\pm 0.30 E_x \pm 1.00 E_z$ combinations when vertical seismic action is active) and populate `ENV_SEISMIC` (`SC-006`).
- **Lateral Force Distribution & CQC Correlation Matrix (`seismic.rs`):**
  - Compute EN 1998-1 §4.3.3.2 correction factor $\lambda \in \{0.85, 1.00\}$, base shear $F_b = S_d(T_1) \cdot m \cdot \lambda$, and story/level horizontal forces $F_i = F_b \frac{z_i m_i}{\sum z_j m_j}$ (`SC-007`).
  - Compute $k\times k$ Der Kiureghian CQC correlation matrix $\rho_{ij} = \frac{8 \zeta^2 (1 + r) r^{3/2}}{(1 - r^2)^2 + 4 \zeta^2 r (1 + r)^2}$ (or identity matrix for `SRSS`) (`SC-008`).
  - Enforce invariants `INV-ACT-05` and `INV-ACT-07` (`ACTIONS_INVALID_SEISMIC_PARAMETERS`, `ACTIONS_INVALID_SPECTRUM_PERIOD`, `ACTIONS_INVALID_LATERAL_FORCE_INPUT`) (`SC-009`).
- **Tauri IPC Commands:**
  - Register `actions_compute_design_spectrum`, `actions_compute_seismic_mass_rule`, `actions_compute_lateral_forces`, and `actions_compute_cqc_matrix` in `src-tauri/src/lib.rs`.

## Acceptance criteria

- [x] `actions_compute_design_spectrum` matches analytical EN 1998-1 Type 1 horizontal and vertical $S_e(T)$ and $S_d(T)$ ordinates (including the $\beta a_g$ lower bound at $T = 3.0\text{ s}$) within $10^{-6}$ (`SC-005`).
- [x] `actions_compute_seismic_mass_rule` and `actions_generate_combinations` produce exact $G + \psi_{E,i} Q$ mass multipliers, $\pm E_x$ combinations, and the 8 signed $30\%$-rule combinations when vertical seismic loading is enabled (`SC-006`).
- [x] `actions_compute_lateral_forces` computes $\lambda = 0.85$ (when $T_1 \le 2 T_C$ and $n_{\text{storeys}} > 2$), exact base shear $F_b$, and level forces $F_i$ summing to $F_b$ (`SC-007`).
- [x] `actions_compute_cqc_matrix` returns a symmetric positive-definite correlation matrix with $\rho_{ii} = 1.0$, $\rho_{12} \approx 0.9634$ for closely spaced modes ($0.50\text{ s}, 0.51\text{ s}$), and $\rho_{13} \approx 0.0080$ for separated modes (`SC-008`).
- [x] Invalid seismic parameters ($q < 1.0$, $T_B \ge T_C$, $T < 0$, or non-positive story mass) return `ACTIONS_INVALID_SEISMIC_PARAMETERS`, `ACTIONS_INVALID_SPECTRUM_PERIOD`, or `ACTIONS_INVALID_LATERAL_FORCE_INPUT` (`SC-009`).
- [x] Companion unit tests (`*_test.rs`) and `npm run check:limits` pass with 0 errors.

## Artifact sync required

- Application PRD: `none - already covered in docs/prd.md`
- Application architecture summary: `none - matches docs/architecture/application-architecture-summary.md`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-eurocode-actions-combinations.md` (transitions to `implemented` once both slices close)
- Issue registry: `required`; node `issues:` reference: `required: .okf/capabilities/cap-eurocode-actions-combinations.md`
- Reason/no-impact decision: `Completes all EN 1998-1 seismic spectra, mass rules, lateral force, and CQC normative services of cap-eurocode-actions-combinations.`

## Human review gate

`None` (backend normative calculation and IPC boundary slice with no rendered UI/UX changes).

## Blocked by

- Blocked by `docs/agents/issues/done/20261102-021-eurocode-na-profile-en1990-combination-engine.md`

## Artifact anchors

- PRD: §4.2 item 5 (`SEISMIC` Eq. 6.12b & 30% rule), §4.3 (EN 1998-1 zones, spectra, mass rule, lateral forces, CQC/SRSS), §5 Workflows 4–5
- Domain model: `SeismicSettings`, `DesignSpectrum`, `SeismicMassCombinationRule`, `LateralForceDistributionResult`, `ModalCorrelationMatrix`, `INV-ACT-05`, `INV-ACT-07`
- Use cases: `ComputeDesignSpectrum`, `ComputeSeismicMassRule`, `ComputeLateralForceDistribution`, `ComputeCqcCorrelationMatrix`
- Contract: `actions_compute_design_spectrum`, `actions_compute_seismic_mass_rule`, `actions_compute_lateral_forces`, `actions_compute_cqc_matrix`, error codes §4

## Acceptance scenarios addressed

- `SC-005`: EN 1998-1 Type 1 Horizontal & Vertical Design Spectrum ($S_e(T), S_d(T)$) Evaluation
- `SC-006`: Seismic Mass Participation ($G + \psi_{E,i} Q$) and Signed Seismic Combinations ($G + \psi_2 Q \pm E$)
- `SC-007`: EN 1998-1 Lateral Force Base Shear ($F_b$) and Story Force Distribution ($F_i$)
- `SC-008`: CQC Modal Correlation Matrix ($\rho_{ij}$) for Closely Spaced vs Well-Separated Modes
- `SC-009`: Normative Validation Rejections (Invalid $\psi$ Hierarchy, Missing $G$, Reserved `Accidental`)

## Verification obligations

- Policy source: `/.okf/project.md`

| Scenario | Backend boundary | Frontend integration | End-to-end journey |
|---|---|---|---|
| `SC-005` | `verified: passed in seismic_test::test_sc005_en1998_type1_design_spectrum_evaluation` | `deferred: owned by cap-sketcher-frame-modeler UI slice` | `not-applicable` |
| `SC-006` | `verified: passed in seismic_test::test_sc006_seismic_mass_participation_and_signed_combinations` | `deferred: owned by cap-sketcher-frame-modeler UI slice` | `not-applicable` |
| `SC-007` | `verified: passed in seismic_test::test_sc007_en1998_lateral_force_base_shear_and_story_distribution` | `not-applicable` | `not-applicable` |
| `SC-008` | `verified: passed in seismic_test::test_sc008_cqc_modal_correlation_matrix` | `not-applicable` | `not-applicable` |
| `SC-009` | `verified: passed in seismic_test::test_sc009_seismic_normative_validation_rejections` | `deferred: owned by cap-sketcher-frame-modeler UI slice` | `not-applicable` |
