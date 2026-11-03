# Product Requirements Document: Eurocode Actions, Load Combinations & Seismic Loading (`cap-eurocode-actions-combinations`)

- **Owning capability:** [`cap-eurocode-actions-combinations`](../../../.okf/capabilities/cap-eurocode-actions-combinations.md)
- **Root synthesis:** [`docs/prd.md`](../../prd.md), [`application-architecture-summary.md`](../application-architecture-summary.md)
- **Cross-capability conventions:** [`cross-capability-conventions.md`](../cross-capability-conventions.md)
- **Glossary:** [`domain-glossary.md`](domain-glossary.md)
- **Status:** Canonical Node PRD (2026-11-01)

---

## 1. Purpose & Product Summary

`cap-eurocode-actions-combinations` is the shared normative engine for Eurocode 0 (EN 1990) action classification and load combinations and Eurocode 8 (EN 1998-1) seismic action definition across OmniMechanics. It provides a single authoritative source for partial safety factors ($\gamma$), combination coefficients ($\psi_0, \psi_1, \psi_2$), automatic leading-variable combination permutations, seismic design spectra ($S_e(T), S_d(T)$), seismic mass combination factors ($G + \psi_{E,i} Q$), lateral force distribution formulas, and envelope grouping specifications, pre-configured with Greek National Annex (`GR`) defaults and fully editable by the user.

It is a pure normative capability: it never assembles stiffness matrices or solves structural equilibrium equations (owned by `cap-frame-analysis-engine`) and never performs cross-sectional reinforcement checks (owned by `cap-eurocode-rc-design`).

---

## 2. Goals & Non-Goals

### 2.1 Goals
1. **Single Normative Source of Truth:** Ensure `cap-frame-analysis-engine` (Mode 2 analysis) and `cap-eurocode-rc-design` (EC2 concrete design) consume identical `ActionType` tags, `NationalAnnexProfile` tables, `Combination` definitions, and `DesignSpectrum` curves.
2. **Complete EN 1990 Combination Generation:** Automatically expand any set of tagged load cases into ULS STR/GEO (selectable Eq. 6.10 or Eq. 6.10a/6.10b), SLS Characteristic, SLS Frequent, SLS Quasi-Permanent, and EN 1998 Seismic combinations over every leading-variable permutation and permanent favorability branch.
3. **Editable Greek National Annex (`GR`) Defaults:** Ship exact Greek National Annex defaults for $\gamma_G, \gamma_Q, \xi, \psi_0, \psi_1, \psi_2$, seismic hazard zones I–III, ground types A–E, and importance classes I–IV, while allowing engineers and instructors to override any factor.
4. **User Control over Combinations:** Support editing any generated combination (`Generated` $\to$ `UserEdited`) and adding custom combinations (`Manual`) that survive automatic regeneration.
5. **EN 1998-1 Seismic Spectra & Lateral Force Normative Rules:** Compute Type 1 horizontal and optional vertical elastic ($S_e(T)$) and design ($S_d(T)$) response spectra, guide behavior factor $q$ selection per structural system, define seismic mass participation ($G + \psi_2 Q$), and evaluate normative lateral force story/nodal distributions ($F_b, F_i$) and CQC modal correlation coefficients ($\rho_{ij}$).
6. **Bilingual Educational Transparency:** Expose every generated combination's symbolic formula (e.g., $1.35 G_1 + 1.50 Q_1 + 1.50 \cdot 0.70 W_1$) and clause citation in Greek (`el`) and English (`en`).

### 2.2 Non-Goals
- **Accidental combinations** (EN 1990 Eq. 6.11a/b): `ActionType::Accidental` is reserved in the enum vocabulary for schema stability, but accidental combination generation is excluded from the current horizon.
- **Structural solving:** Assembling or solving linear, P-$\Delta$, buckling, catenary, or modal eigenvalue systems is strictly owned by `cap-frame-analysis-engine`.
- **Member resistance checks:** Concrete, steel, or timber section capacity checks are owned by `cap-eurocode-rc-design` and future design nodes.
- **EN 1991 wind/snow pressure geometry calculators:** Users enter wind, snow, and thermal loads as forces/temperatures in load cases; geometric pressure-coefficient generation from building envelopes is out of scope.
- **Type 2 seismicity spectra:** Only Type 1 response spectra (mandated by the Greek National Annex) are included.

---

## 3. Actors & Consumers

1. **Engineer / Instructor (Mode 2 UI):** Tags load cases with `ActionType`, configures `NationalAnnexProfile` and `SeismicSettings`, triggers combination generation, customizes combinations, previews $S_d(T)$ curves, and selects envelope views.
2. **Student (Mode 2 & Mode 3):** Inspects how Eurocode formulas and $\gamma/\psi$ tables produce specific load factors and governing design envelopes in Greek or English.
3. **Frame Analysis Engine (`cap-frame-analysis-engine`):** Calls Rust domain functions or receives generated `Combination`, `DesignSpectrum`, `SeismicMassCombinationRule`, and `EnvelopeSpecification` payloads to execute static, P-$\Delta$, modal RSA, and lateral force analyses.
4. **RC Design Engine (`cap-eurocode-rc-design`):** Consumes ULS (`ULS_STR`, `SEISMIC`) and SLS (`SLS_CHAR`, `SLS_FREQ`, `SLS_QP`) envelopes tagged by `LimitState` to verify flexural, shear, column interaction, and serviceability criteria.

---

## 4. Normative Business Rules

### 4.1 Action Classification (`ActionType`) & Greek NA (`GR`) Factor Table
Every `LoadCase` is tagged with one `ActionType`. The default `GR` profile defines:
- **Permanent ($G$):** $\gamma_{G,\text{sup}} = 1.35$ (unfavorable), $\gamma_{G,\text{inf}} = 1.00$ (favorable), $\xi = 0.85$ (for Eq. 6.10b).
- **Variable ($Q, \text{Snow}, \text{Wind}, \text{Temperature}$):** $\gamma_{Q,\text{sup}} = 1.50$ (unfavorable), $\gamma_{Q,\text{inf}} = 0.00$ (favorable).

| ActionType / ImposedCategory | Description (`en` / `el`) | $\psi_0$ | $\psi_1$ | $\psi_2$ |
| --- | --- | :---: | :---: | :---: |
| `Permanent` | Permanent action ($G$) / Μόνιμη δράση | — | — | — |
| `Imposed(A)` | Domestic, residential / Κατοικίες | 0.70 | 0.50 | 0.30 |
| `Imposed(B)` | Office areas / Γραφεία | 0.70 | 0.50 | 0.30 |
| `Imposed(C)` | Congregation areas / Χώροι συνάθροισης | 0.70 | 0.70 | 0.60 |
| `Imposed(D)` | Shopping areas / Εμπορικοί χώροι | 0.70 | 0.70 | 0.60 |
| `Imposed(E)` | Storage areas / Αποθήκες | 1.00 | 0.90 | 0.80 |
| `Imposed(F)` | Traffic area, vehicle weight $\le 30\text{ kN}$ / Κυκλοφορία $\le 30\text{ kN}$ | 0.70 | 0.70 | 0.60 |
| `Imposed(G)` | Traffic area, $30\text{ kN} < \text{weight} \le 160\text{ kN}$ / Κυκλοφορία $\le 160\text{ kN}$ | 0.70 | 0.50 | 0.30 |
| `Imposed(H)` | Roofs (non-accessible except maintenance) / Στέγες | 0.00 | 0.00 | 0.00 |
| `Snow` | Snow load (Greece / altitude $\le 1000\text{ m}$ default) / Χιόνι | 0.50 | 0.20 | 0.00 |
| `Wind` | Wind action / Άνεμος | 0.60 | 0.20 | 0.00 |
| `Temperature` | Thermal action (non-fire) / Θερμοκρασία | 0.60 | 0.50 | 0.00 |
| `Seismic` | Seismic action ($A_{Ed}$) / Σεισμική δράση | — | — | — |
| `Accidental` | Accidental action (reserved, rejected in generation) / Τυχαία δράση | — | — | — |

*(Note: All $\gamma$, $\xi$, and $\psi$ coefficients are editable per project in `NationalAnnexProfile`, including altitude $>1000\text{ m}$ snow overrides $\psi_0=0.70, \psi_1=0.50, \psi_2=0.20$.)*

### 4.2 EN 1990 Combination Equations & Permutation Rules
Given permanent load cases $\mathcal{G} = \{G_1, \dots, G_m\}$, variable load cases $\mathcal{Q} = \{Q_1, \dots, Q_n\}$, and seismic load cases $\mathcal{E} = \{E_x, E_z\}$:

1. **ULS STR / GEO (`ULS_STR`, `ULS_GEO`):**
   - **Policy `Single6_10` (EN 1990 Eq. 6.10, Greek NA default):**
     For each leading variable action $Q_k \in \mathcal{Q}$ (and one permanent-only combination when $\mathcal{Q} = \emptyset$ or for pure permanent check):
     $$\sum_{j \ge 1} \gamma_{G,j} G_{k,j} + \gamma_{Q,1} Q_{k,1} + \sum_{i > 1} \gamma_{Q,i} \psi_{0,i} Q_{k,i}$$
   - **Policy `Twin6_10a_6_10b` (EN 1990 Eq. 6.10a & 6.10b):**
     - Eq. 6.10a: $\sum_{j \ge 1} \gamma_{G,j} G_{k,j} + \sum_{i \ge 1} \gamma_{Q,i} \psi_{0,i} Q_{k,i}$ (selecting each $Q_1$ with $\psi_{0,1} Q_{k,1}$).
     - Eq. 6.10b: $\sum_{j \ge 1} \xi \gamma_{G,\text{sup},j} G_{k,j} \text{ (or } \gamma_{G,\text{inf},j} G_{k,j}\text{)} + \gamma_{Q,1} Q_{k,1} + \sum_{i > 1} \gamma_{Q,i} \psi_{0,i} Q_{k,i}$.
   - **Permanent Favorability Policy (`PermanentActionFavorabilityPolicy`):**
     - `BothBranches` (default): generates both the all-unfavorable permanent branch ($\gamma_{G,\text{sup}} = 1.35$) and the all-favorable permanent branch ($\gamma_{G,\text{inf}} = 1.00$) for each leading variable permutation so uplift/overturning/reversal is captured.
     - `UnfavorableOnly`: generates only the $\gamma_{G,\text{sup}}$ branch.
     - `PerCasePermutation`: optional $2^m$ expansion when individual permanent cases can oppose one another (capped at $m \le 6$).
   - **Accompanying Variable Filtering:** Any accompanying variable term with $\psi_{0,i} = 0$ (e.g., `Imposed(H)`) receives factor $0.0$ and is omitted from nonzero terms.

2. **SLS Characteristic (`SLS_CHAR`, EN 1990 Eq. 6.14b):**
   For each leading variable action $Q_1 \in \mathcal{Q}$:
   $$\sum_{j \ge 1} 1.00 G_{k,j} + 1.00 Q_{k,1} + \sum_{i > 1} \psi_{0,i} Q_{k,i}$$

3. **SLS Frequent (`SLS_FREQ`, EN 1990 Eq. 6.15b):**
   For each leading variable action $Q_1 \in \mathcal{Q}$ (omitting permutations where $\psi_{1,1} = 0$ unless no other variable case exists):
   $$\sum_{j \ge 1} 1.00 G_{k,j} + \psi_{1,1} Q_{k,1} + \sum_{i > 1} \psi_{2,i} Q_{k,i}$$

4. **SLS Quasi-Permanent (`SLS_QP`, EN 1990 Eq. 6.16b):**
   Single deterministic combination (no leading variable distinction):
   $$\sum_{j \ge 1} 1.00 G_{k,j} + \sum_{i \ge 1} \psi_{2,i} Q_{k,i}$$

5. **EN 1998 Seismic Combination (`SEISMIC`, EN 1990 Eq. 6.12b / EN 1998-1 §3.2.4):**
   $$\sum_{j \ge 1} 1.00 G_{k,j} + \sum_{i \ge 1} \psi_{2,i} Q_{k,i} \pm A_{Ed}$$
   - When seismic action is represented by static lateral force or modal RSA `LoadCase` $E_x$ (and optional vertical $E_z$), generates signed directional combinations:
     - Horizontal only: $+1.00 E_x$ and $-1.00 E_x$.
     - Horizontal + Vertical (EN 1998-1 §4.3.3.5.2, $30\%$ rule in 2D): $\pm 1.00 E_x \pm 0.30 E_z$ and $\pm 0.30 E_x \pm 1.00 E_z$ (8 signed combinations).

### 4.3 EN 1998-1 Seismic Hazard, Ground Types, and Spectra (Greek National Annex)
1. **Seismic Hazard Zones (`GR`):**
   - `ZoneI`: $a_{gR} / g = 0.16$ ($a_{gR} = 1.5696\text{ m/s}^2$)
   - `ZoneII`: $a_{gR} / g = 0.24$ ($a_{gR} = 2.3544\text{ m/s}^2$)
   - `ZoneIII`: $a_{gR} / g = 0.36$ ($a_{gR} = 3.5316\text{ m/s}^2$)
   - `Custom`: user-specified $a_{gR}/g \in (0, 2.0]$
   - Standard gravity $g = 9.81\text{ m/s}^2$.
2. **Importance Classes (`GR`):**
   - `ClassI`: $\gamma_I = 0.80$
   - `ClassII` (default ordinary buildings): $\gamma_I = 1.00$
   - `ClassIII` (schools, assembly): $\gamma_I = 1.20$
   - `ClassIV` (hospitals, critical infrastructure): $\gamma_I = 1.40$
   - Design ground acceleration: $a_g = \gamma_I \cdot a_{gR}$.
3. **Ground Types (`A`–`E`, EN 1998-1 Table 3.2 Type 1 Horizontal Spectrum):**

| Ground Type | Soil Factor $S$ | $T_B$ (s) | $T_C$ (s) | $T_D$ (s) |
| :---: | :---: | :---: | :---: | :---: |
| `A` | 1.00 | 0.15 | 0.40 | 2.00 |
| `B` | 1.20 | 0.15 | 0.50 | 2.00 |
| `C` | 1.15 | 0.20 | 0.60 | 2.00 |
| `D` | 1.35 | 0.20 | 0.80 | 2.00 |
| `E` | 1.40 | 0.15 | 0.50 | 2.00 |

4. **Vertical Spectrum Parameters (EN 1998-1 Table 3.4 Type 1):**
   - For all ground types `A`–`E`: $a_{vg} / a_g = 0.90$, $S_v = 1.00$, $T_B = 0.05\text{ s}$, $T_C = 0.15\text{ s}$, $T_D = 1.00\text{ s}$, vertical behavior factor $q_v \in [1.0, 1.5]$ (default $1.50$).
5. **Behavior Factor $q$ Guidance per Structural System:**
   - The user can pick a preset structural system (e.g., RC Moment Frame DCM $q = 3.90$, RC Dual Wall-Frame DCM $q = 3.60$, RC Ductile Wall DCM $q = 3.00$, Inverted Pendulum DCM $q = 1.50$, Low Ductility DCL $q = 1.50$, Steel Moment Frame DCM $q = 4.00$) or enter any custom $q \in [1.0, 8.0]$.
6. **Horizontal Elastic Response Spectrum $S_e(T)$ (EN 1998-1 §3.2.2.2):**
   With damping correction $\eta = \max(\sqrt{10 / (5 + \zeta)}, 0.55)$ where $\zeta$ is viscous damping in percent (default $\zeta = 5\% \Rightarrow \eta = 1.0$):
   - $0 \le T < T_B$: $S_e(T) = a_g \cdot S \cdot \left[1 + \frac{T}{T_B}(\eta \cdot 2.5 - 1)\right]$
   - $T_B \le T \le T_C$: $S_e(T) = a_g \cdot S \cdot \eta \cdot 2.5$
   - $T_C < T \le T_D$: $S_e(T) = a_g \cdot S \cdot \eta \cdot 2.5 \left[\frac{T_C}{T}\right]$
   - $T_D < T \le 4.0\text{ s}$ (and beyond): $S_e(T) = a_g \cdot S \cdot \eta \cdot 2.5 \left[\frac{T_C T_D}{T^2}\right]$
7. **Horizontal Design Spectrum $S_d(T)$ (EN 1998-1 §3.2.2.5):**
   With lower-bound factor $\beta = 0.20$ (Greek NA default, editable):
   - $0 \le T < T_B$: $S_d(T) = a_g \cdot S \cdot \left[\frac{2}{3} + \frac{T}{T_B}\left(\frac{2.5}{q} - \frac{2}{3}\right)\right]$
   - $T_B \le T \le T_C$: $S_d(T) = a_g \cdot S \cdot \frac{2.5}{q}$
   - $T_C < T \le T_D$: $S_d(T) = \max\left(a_g \cdot S \cdot \frac{2.5}{q} \left[\frac{T_C}{T}\right],\; \beta \cdot a_g\right)$
   - $T > T_D$: $S_d(T) = \max\left(a_g \cdot S \cdot \frac{2.5}{q} \left[\frac{T_C T_D}{T^2}\right],\; \beta \cdot a_g\right)$
8. **Vertical Elastic and Design Spectra $S_{ve}(T), S_{vd}(T)$:**
   Same equations replacing $(a_g, S, q, T_B, T_C, T_D)$ with $(a_{vg} = 0.90 a_g, S = 1.0, q_v, 0.05, 0.15, 1.00)$.
9. **Seismic Mass Combination Rule (EN 1998-1 §3.2.4(2)):**
   Masses for modal analysis and lateral force method are computed from gravity loads using:
   $$\sum_{j \ge 1} 1.00 G_{k,j} + \sum_{i \ge 1} \psi_{E,i} Q_{k,i}, \quad \psi_{E,i} = \varphi \cdot \psi_{2,i}$$
   where default $\varphi = 1.0$ (or user-selectable EN 1998-1 §4.2.4 story reduction factor $\varphi \in \{0.5, 0.8, 1.0\}$).
10. **Lateral Force Method Distribution (EN 1998-1 §4.3.3.2):**
    Given fundamental period $T_1$, total seismic mass $m = \sum m_i$, and story/node heights $z_i$ above the foundation/top of rigid basement:
    - Correction factor $\lambda = 0.85$ if $T_1 \le 2 T_C$ and the building has more than 2 storeys ($n_{\text{storeys}} > 2$); otherwise $\lambda = 1.00$.
    - Base shear: $F_b = S_d(T_1) \cdot m \cdot \lambda$.
    - Horizontal force at level/node $i$: $F_i = F_b \cdot \frac{z_i m_i}{\sum_j z_j m_j}$.
11. **Modal Combination Rules (EN 1998-1 §4.3.3.3.2):**
    - `SRSS`: $E_E = \sqrt{\sum_i E_{Ei}^2}$.
    - `CQC`: $E_E = \sqrt{\sum_i \sum_j \rho_{ij} E_{Ei} E_{Ej}}$, with Der Kiureghian correlation coefficient for constant damping $\zeta$:
      $$\rho_{ij} = \frac{8 \zeta^2 (1 + r) r^{3/2}}{(1 - r^2)^2 + 4 \zeta^2 r (1 + r)^2}, \quad r = \frac{T_j}{T_i}$$

---

## 5. Required Workflows

1. **Configure National Annex & Factor Overrides:** View default Greek (`GR`) factors, toggle ULS equation (`Single6_10` vs `Twin6_10a_6_10b`) and permanent favorability policy, or override specific $\gamma$ or $\psi$ values; reset to normative `GR` defaults at any time.
2. **Generate EN 1990 / EN 1998 Combinations:** Submit load-case summaries (`id`, `name`, `action_type`) and existing combinations; receive the regenerated `Combination` table where `Generated` entries are updated and `UserEdited`/`Manual` entries are preserved, plus `EnvelopeSpecification` sets per limit state.
3. **Edit & Create Combinations:** Modify factors on a `Generated` combination (automatically promoting its `origin` to `UserEdited`), add a `Manual` combination, delete a combination, or reset `UserEdited` combinations back to `Generated`.
4. **Configure Seismic Settings & Evaluate Spectra:** Set zone, ground type, importance class, structural system / $q$, damping, and vertical option; compute `DesignSpectrum` with discrete curve points ($0 \le T \le 4.0\text{ s}$) and exact $S_e(T), S_d(T)$ queries for specific modal periods $T_k$.
5. **Compute Seismic Mass Multipliers & Lateral Forces:** Derive per-load-case mass factors ($1.0 G + \psi_{E,i} Q$) and compute lateral base shear $F_b$ and nodal forces $F_i$ from $(T_1, z_i, m_i)$.

---

## 6. Non-Functional Requirements

- **Performance:** Generating all combinations for up to 15 load cases and computing 200-point spectra must complete in $< 5\text{ ms}$ on the Rust backend.
- **Determinism:** Given identical `LoadCase` descriptors and `NationalAnnexProfile`, `GenerateCombinations` must emit combinations in a deterministic canonical order with stable `CO1..COn` identifiers and bilingual formulas.
- **Code Quality:** All Rust files in `src-tauri/src/actions/` and TypeScript files in `src/app/` must remain $\le 1000$ lines with companion `*_test.rs` unit tests.

---

## 7. Explicit Assumptions & Artifact Impact

- **Assumptions:**
  1. `LoadCase` entities are owned by `cap-frame-analysis-engine` inside `StructuralModel`; `cap-eurocode-actions-combinations` accepts lightweight `LoadCaseDescriptor` records (`id`, `code`, `name`, `action_type`) or operates on the model's load-case list.
  2. In 2D frame analysis, horizontal seismic action acts along the global $X$ axis and optional vertical seismic action acts along the global $Y$ axis.
- **Application Synthesis Impact:** **No impact** on [`docs/prd.md`](../../prd.md) or [`application-architecture-summary.md`](../application-architecture-summary.md); all scope, rules, and boundaries match the synthesis baseline established on 2026-10-31.
