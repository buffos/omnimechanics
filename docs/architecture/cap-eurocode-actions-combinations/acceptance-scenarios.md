# Acceptance Scenarios: Eurocode Actions, Load Combinations & Seismic Loading (`cap-eurocode-actions-combinations`)

- **Owning capability:** [`cap-eurocode-actions-combinations`](../../../.okf/capabilities/cap-eurocode-actions-combinations.md)
- **Cross-capability conventions:** [`cross-capability-conventions.md`](../cross-capability-conventions.md)
- **PRD:** [`prd.md`](prd.md)
- **Domain Model:** [`canonical-domain-model.md`](canonical-domain-model.md)
- **Use-Case Model:** [`canonical-use-cases.md`](canonical-use-cases.md)
- **Contract:** [`canonical-api-cli-contract.md`](canonical-api-cli-contract.md)
- **Status:** Canonical Acceptance Scenarios (2026-11-01)

---

## 1. Scenario Summary & Verification Surface Matrix

Per the root verification policy ([`docs/prd.md`](../../prd.md) §8):
- **`backend-boundary` (required):** Verified via `cargo test` in companion `src-tauri/src/actions/*_test.rs` suites against exact analytical Eurocode values.
- **`frontend-integration` (required):** Verified via Vitest tests on the Angular actions/combinations store and IPC adapter.
- **`end-to-end` (catalog-only):** Visual review gate in Mode 2 (Combinations & Seismic dialogs, spectrum chart, and envelope selector).

| ID | Title | Primary Rule / Branch | `backend-boundary` | `frontend-integration` | `end-to-end` |
| --- | --- | --- | :---: | :---: | :---: |
| **SC-001** | Greek National Annex (`GR`) Default Factor & Guidance Catalog Retrieval | §4.1 `GR` defaults ($\gamma, \xi, \psi$) | required | required | catalog-only |
| **SC-002** | ULS STR Eq. 6.10 & SLS Combination Generation with Leading-Variable Permutations | §4.2 Eq. 6.10, 6.14b, 6.15b, 6.16b | required | required | catalog-only |
| **SC-003** | Twin ULS Equations (Eq. 6.10a & 6.10b) with Reduction Factor $\xi = 0.85$ | §4.2 `Twin6_10a_6_10b` policy | required | required | catalog-only |
| **SC-004** | User-Edited (`UserEdited`) and Custom (`Manual`) Combinations Survive Regeneration | §6.2 Provenance lifecycle | required | required | catalog-only |
| **SC-005** | EN 1998-1 Type 1 Horizontal & Vertical Design Spectrum ($S_e(T), S_d(T)$) Evaluation | §4.3 Spectrum equations & lower bound $\beta$ | required | required | catalog-only |
| **SC-006** | Seismic Mass Participation ($G + \psi_{E,i} Q$) and Signed Seismic Combinations ($G + \psi_2 Q \pm E$) | §4.2 Eq. 6.12b & §4.3 Mass rule | required | required | catalog-only |
| **SC-007** | EN 1998-1 Lateral Force Base Shear ($F_b$) and Story Force Distribution ($F_i$) | §4.3 Lateral force method & $\lambda$ factor | required | not-applicable | catalog-only |
| **SC-008** | CQC Modal Correlation Matrix ($\rho_{ij}$) for Closely Spaced vs Well-Separated Modes | §4.3 CQC vs SRSS rule | required | not-applicable | not-applicable |
| **SC-009** | Normative Validation Rejections (Invalid $\psi$ Hierarchy, Missing $G$, Reserved `Accidental`) | §6.1 Invariants INV-ACT-01..07 | required | required | catalog-only |

---

## 2. Detailed Acceptance Scenarios

### SC-001: Greek National Annex (`GR`) Default Factor & Guidance Catalog Retrieval
- **Purpose:** Verify that the default `GR` profile provides exact normative $\gamma, \xi, \psi_0, \psi_1, \psi_2$ tables and structural system $q$-factor guidance in Greek and English.
- **Preconditions:** Application initialized with default National Annex code `"GR"`.
- **Action:** Query `GetDefaultNationalAnnexProfile({ code: "GR" })`.
- **Expected Outcome:**
  1. $\gamma_{G,\text{sup}} = 1.35$, $\gamma_{G,\text{inf}} = 1.00$, $\gamma_{Q,\text{sup}} = 1.50$, $\gamma_{Q,\text{inf}} = 0.00$, $\xi = 0.85$, `uls_equation_policy = "Single6_10"`, `permanent_favorability_policy = "BothBranches"`.
  2. $\psi$ factors match EN 1990 Table A1.1 / Greek NA: Category A & B $(0.7, 0.5, 0.3)$, Category C & D $(0.7, 0.7, 0.6)$, Category E $(1.0, 0.9, 0.8)$, Category F $(0.7, 0.7, 0.6)$, Category G $(0.7, 0.5, 0.3)$, Category H $(0.0, 0.0, 0.0)$, Snow $(0.5, 0.2, 0.0)$, Wind $(0.6, 0.2, 0.0)$, Temperature $(0.6, 0.5, 0.0)$.
  3. Structural system guidance catalog includes RC Moment Frame DCM ($q = 3.90$), RC Dual System DCM ($q = 3.60$), RC Ductile Wall DCM ($q = 3.00$), and Low Ductility DCL ($q = 1.50$) with bilingual descriptions.
- **Verification Surfaces:** `backend-boundary: required`, `frontend-integration: required`, `end-to-end: catalog-only`.

---

### SC-002: ULS STR Eq. 6.10 & SLS Combination Generation with Leading-Variable Permutations
- **Purpose:** Verify full leading-variable permutation expansion and favorable/unfavorable permanent branches across ULS and SLS limit states.
- **Preconditions:**
  - Default `GR` profile (`Single6_10`, `BothBranches`).
  - Three load cases: `LC1` (`G1`, `Permanent`), `LC2` (`Q1`, `Imposed(A)` where $\psi_0=0.7, \psi_1=0.5, \psi_2=0.3$), and `LC3` (`W1`, `Wind` where $\psi_0=0.6, \psi_1=0.2, \psi_2=0.0$).
- **Action:** Execute `GenerateCombinations` with `[LC1, LC2, LC3]`.
- **Expected Outcome:**
  1. **ULS STR (`ULS_STR`):** Generates 4 leading-variable combinations:
     - Leading `Q1`, unfavorable `G1`: $1.35 G_1 + 1.50 Q_1 + (1.50 \times 0.60 = 0.90) W_1$.
     - Leading `Q1`, favorable `G1`: $1.00 G_1 + 1.50 Q_1 + 0.90 W_1$.
     - Leading `W1`, unfavorable `G1`: $1.35 G_1 + 1.50 W_1 + (1.50 \times 0.70 = 1.05) Q_1$.
     - Leading `W1`, favorable `G1`: $1.00 G_1 + 1.50 W_1 + 1.05 Q_1$.
     *(Plus pure permanent $1.35 G_1$ and $1.00 G_1$ baseline combinations.)*
  2. **SLS Characteristic (`SLS_CHAR`):** Generates 2 combinations:
     - Leading `Q1`: $1.00 G_1 + 1.00 Q_1 + 0.60 W_1$.
     - Leading `W1`: $1.00 G_1 + 1.00 W_1 + 0.70 Q_1$.
  3. **SLS Frequent (`SLS_FREQ`):** Generates 2 combinations:
     - Leading `Q1`: $1.00 G_1 + 0.50 Q_1$ (since $\psi_2(\text{Wind}) = 0.0$).
     - Leading `W1`: $1.00 G_1 + 0.20 W_1 + 0.30 Q_1$.
  4. **SLS Quasi-Permanent (`SLS_QP`):** Generates 1 combination:
     - $1.00 G_1 + 0.30 Q_1$ (since $\psi_2(\text{Wind}) = 0.0$).
  5. **Envelope Specifications:** `ENV_ULS_STR`, `ENV_SLS_CHAR`, `ENV_SLS_FREQ`, and `ENV_SLS_QP` list the corresponding `CO#` IDs.
- **Verification Surfaces:** `backend-boundary: required`, `frontend-integration: required`, `end-to-end: catalog-only`.

---

### SC-003: Twin ULS Equations (Eq. 6.10a & 6.10b) with Reduction Factor $\xi = 0.85$
- **Purpose:** Verify ULS generation when the engineer switches `uls_equation_policy` to `Twin6_10a_6_10b`.
- **Preconditions:** `LC1` (`G1`, `Permanent`) and `LC2` (`Q1`, `Imposed(B)`, $\psi_0 = 0.70$), `uls_equation_policy = "Twin6_10a_6_10b"`, $\xi = 0.85$.
- **Action:** Execute `GenerateCombinations`.
- **Expected Outcome:**
  1. Emits Eq. 6.10a unfavorable combination: $1.35 G_1 + (1.50 \times 0.70 = 1.05) Q_1$ tagged `equation = "Eq6_10a"`.
  2. Emits Eq. 6.10b unfavorable combination: $(0.85 \times 1.35 = 1.1475) G_1 + 1.50 Q_1$ tagged `equation = "Eq6_10b"`.
  3. Both are included in `ENV_ULS_STR`.
- **Verification Surfaces:** `backend-boundary: required`, `frontend-integration: required`, `end-to-end: catalog-only`.

---

### SC-004: User-Edited (`UserEdited`) and Custom (`Manual`) Combinations Survive Regeneration
- **Purpose:** Verify provenance protection across combination edits, manual additions, regenerations, and resets.
- **Preconditions:** `CombinationSet` generated from `[LC1 (G1), LC2 (Q1)]` containing `CO1` (`Generated`, $1.35 G_1 + 1.50 Q_1$).
- **Action:**
  1. Call `SaveCustomCombination` on `CO1` changing $Q_1$ factor from $1.50$ to $1.65$.
  2. Call `SaveCustomCombination` with `id: None` to add a custom combination $1.20 G_1 + 1.20 Q_1$.
  3. Add a new load case `LC3` (`S1`, `Snow`) and call `GenerateCombinations(..., reset_user_edited: false)`.
  4. Call `DeleteOrResetCombination` on `CO1` with `action: "ResetToGenerated"`.
- **Expected Outcome:**
  1. After step 1, `CO1.origin == "UserEdited"` and `CO1` has factor $1.65$ on `LC2`.
  2. After step 2, the new combination has `origin == "Manual"`.
  3. After step 3, `CO1` retains its `UserEdited` factor ($1.65$) and `Manual` combination is preserved, while new snow permutations are added as `Generated`.
  4. After step 4, `CO1.origin` reverts to `"Generated"` with normative factors ($1.35 G_1 + 1.50 Q_1 + 0.75 S_1$).
- **Verification Surfaces:** `backend-boundary: required`, `frontend-integration: required`, `end-to-end: catalog-only`.

---

### SC-005: EN 1998-1 Type 1 Horizontal & Vertical Design Spectrum ($S_e(T), S_d(T)$) Evaluation
- **Purpose:** Verify exact analytical ordinates of the EN 1998-1 Type 1 horizontal and vertical elastic ($S_e$) and design ($S_d$) spectra, including the lower-bound plateau $\beta a_g$.
- **Preconditions:**
  - `SeismicSettings`: `ZoneII` ($a_{gR} = 0.24g$), `GroundType::B` ($S = 1.20, T_B = 0.15\text{ s}, T_C = 0.50\text{ s}, T_D = 2.00\text{ s}$), `ImportanceClass::ClassIII` ($\gamma_I = 1.20 \Rightarrow a_g = 0.288g = 2.82528\text{ m/s}^2$), $q = 3.90$, $\zeta = 5\%$, $\beta = 0.20$, `include_vertical_component: true`, $q_v = 1.50$.
- **Action:** Call `ComputeDesignSpectrum` with `query_periods_s: [0.0, 0.15, 0.50, 1.00, 3.00]`.
- **Expected Outcome:**
  1. $a_g = 0.288g = 2.82528\text{ m/s}^2$.
  2. At $T = 0.0\text{ s}$: $S_e(0)/g = 0.288 \times 1.20 = 0.3456$; $S_d(0)/g = 0.288 \times 1.20 \times (2/3) = 0.2304$.
  3. On plateau $T \in [0.15, 0.50]\text{ s}$: $S_e(T)/g = 0.288 \times 1.20 \times 2.5 = 0.864$; $S_d(T)/g = 0.864 / 3.90 = 0.221538$.
  4. At $T = 1.00\text{ s} \in (T_C, T_D]$: $S_d(1.0)/g = \max(0.221538 \times (0.50 / 1.00), 0.20 \times 0.288) = \max(0.110769, 0.0576) = 0.110769$.
  5. At $T = 3.00\text{ s} > T_D$: unconstrained $0.221538 \times (0.50 \times 2.00 / 3.0^2) = 0.024615 < \beta (a_g/g) = 0.0576$, so $S_d(3.0)/g$ is clamped to the lower bound $0.0576$ ($0.565056\text{ m/s}^2$).
  6. Vertical spectrum is populated with $a_{vg} = 0.90 a_g = 0.2592g$, $T_B=0.05\text{ s}, T_C=0.15\text{ s}, T_D=1.00\text{ s}$.
- **Verification Surfaces:** `backend-boundary: required`, `frontend-integration: required`, `end-to-end: catalog-only`.

---

### SC-006: Seismic Mass Participation ($G + \psi_{E,i} Q$) and Signed Seismic Combinations ($G + \psi_2 Q \pm E$)
- **Purpose:** Verify seismic mass factors and signed EN 1998-1 seismic load combinations with horizontal and optional vertical directional components.
- **Preconditions:**
  - Load cases: `LC1` (`G1`, `Permanent`), `LC2` (`Q1`, `Imposed(A)`, $\psi_2 = 0.30$), `LC3` (`Q2`, `Imposed(E)`, $\psi_2 = 0.80$), `LC4` (`W1`, `Wind`, $\psi_2 = 0.00$), `LC5` (`Ex`, `Seismic`).
  - `psi_e_phi = 1.00`.
- **Action:** Call `ComputeSeismicMassRule` and `GenerateCombinations`.
- **Expected Outcome:**
  1. `SeismicMassCombinationRule` assigns `1.00` to `LC1`, `0.30` to `LC2`, `0.80` to `LC3`, and omits `LC4` ($\psi_2 = 0$) and `LC5`.
  2. `GenerateCombinations` emits two `SEISMIC` combinations (`Eq6_12b`):
     - $1.00 G_1 + 0.30 Q_1 + 0.80 Q_2 + 1.00 E_x$
     - $1.00 G_1 + 0.30 Q_1 + 0.80 Q_2 - 1.00 E_x$
  3. When a second seismic load case `LC6` (`Ez`, `Seismic`) and `include_vertical_component: true` are active, `GenerateCombinations` emits the 8 signed $30\%$-rule combinations ($\pm 1.00 E_x \pm 0.30 E_z$ and $\pm 0.30 E_x \pm 1.00 E_z$).
- **Verification Surfaces:** `backend-boundary: required`, `frontend-integration: required`, `end-to-end: catalog-only`.

---

### SC-007: EN 1998-1 Lateral Force Base Shear ($F_b$) and Story Force Distribution ($F_i$)
- **Purpose:** Verify EN 1998-1 §4.3.3.2 base shear and height-proportional lateral force distribution, including the $\lambda = 0.85$ vs $1.00$ correction factor.
- **Preconditions:**
  - `SeismicSettings` from SC-005 where $T_C = 0.50\text{ s}$ and at $T_1 = 0.40\text{ s} \le 2 T_C$, $S_d(0.40) = 2.173292\text{ m/s}^2$.
  - 3-storey frame ($n_{\text{storeys}} = 3 > 2 \Rightarrow \lambda = 0.85$) with levels:
    - Level 1: $z_1 = 3.0\text{ m}, m_1 = 100\text{ t}$ ($z_1 m_1 = 300$)
    - Level 2: $z_2 = 6.0\text{ m}, m_2 = 100\text{ t}$ ($z_2 m_2 = 600$)
    - Level 3: $z_3 = 9.0\text{ m}, m_3 = 80\text{ t}$ ($z_3 m_3 = 720$)
    - Total mass $m = 280\text{ t}$, $\sum z_j m_j = 1620\text{ t}\cdot\text{m}$.
- **Action:** Call `ComputeLateralForceDistribution`.
- **Expected Outcome:**
  1. $\lambda = 0.85$.
  2. Base shear $F_b = 2.173292 \times 280 \times 0.85 = 517.2435\text{ kN}$.
  3. Level forces: $F_1 = F_b (300/1620) = 95.7858\text{ kN}$, $F_2 = F_b (600/1620) = 191.5717\text{ kN}$, $F_3 = F_b (720/1620) = 229.8860\text{ kN}$, with $F_1 + F_2 + F_3 = F_b$.
- **Verification Surfaces:** `backend-boundary: required`, `frontend-integration: not-applicable`, `end-to-end: catalog-only`.

---

### SC-008: CQC Modal Correlation Matrix ($\rho_{ij}$) for Closely Spaced vs Well-Separated Modes
- **Purpose:** Verify Der Kiureghian CQC correlation coefficients $\rho_{ij}$ at $\zeta = 5\%$ damping.
- **Preconditions:** Modal periods $T = [0.50\text{ s}, 0.51\text{ s}, 0.15\text{ s}]$, $\zeta = 5\%$.
- **Action:** Call `ComputeCqcCorrelationMatrix` with `rule: "CQC"`.
- **Expected Outcome:**
  1. Diagonal entries $\rho_{ii} = 1.000000$.
  2. Matrix is symmetric ($\rho_{ij} = \rho_{ji}$).
  3. Closely spaced modes 1 and 2 ($r = 0.50/0.51 = 0.980392$) have strong positive correlation $\rho_{12} \approx 0.9634 > 0.90$.
  4. Well-separated modes 1 and 3 ($r = 0.15/0.50 = 0.30$) have small correlation $\rho_{13} \approx 0.0080 < 0.02$.
- **Verification Surfaces:** `backend-boundary: required`, `frontend-integration: not-applicable`, `end-to-end: not-applicable`.

---

### SC-009: Normative Validation Rejections (Invalid $\psi$ Hierarchy, Missing $G$, Reserved `Accidental`)
- **Purpose:** Verify deterministic bilingual error codes when normative invariants are violated.
- **Preconditions:** Default `GR` profile and load-case list.
- **Action & Expected Outcome:**
  1. Submit `NationalAnnexProfile` with $\psi_1 = 0.80 > \psi_0 = 0.70$ $\to$ rejected with `ACTIONS_INVALID_PSI_FACTOR`.
  2. Submit `GenerateCombinations` with only `Imposed` load cases and zero `Permanent` load cases $\to$ rejected with `ACTIONS_NO_PERMANENT_LOAD_CASE`.
  3. Submit `GenerateCombinations` with a load case tagged `ActionType::Accidental` $\to$ rejected with `ACTIONS_ACCIDENTAL_NOT_SUPPORTED`.
  4. Submit `SeismicSettings` with $q = 0.5 < 1.0$ or $T_B = 0.60 > T_C = 0.50$ $\to$ rejected with `ACTIONS_INVALID_SEISMIC_PARAMETERS`.
- **Verification Surfaces:** `backend-boundary: required`, `frontend-integration: required`, `end-to-end: catalog-only`.
