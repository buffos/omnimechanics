# Domain Glossary: Eurocode Actions, Load Combinations & Seismic Loading (`cap-eurocode-actions-combinations`)

- **Owning capability:** [`cap-eurocode-actions-combinations`](../../../.okf/capabilities/cap-eurocode-actions-combinations.md)
- **Cross-capability conventions:** [`cross-capability-conventions.md`](../cross-capability-conventions.md)
- **Shared with:** [`cap-frame-analysis-engine`](../../../.okf/capabilities/cap-frame-analysis-engine.md), [`cap-eurocode-rc-design`](../../../.okf/capabilities/cap-eurocode-rc-design.md)
- **Status:** Canonical vocabulary (2026-11-01)

## 1. Purpose

This glossary establishes the single normative vocabulary for EN 1990 action classification, partial and combination factor tables, ULS/SLS/Seismic load combinations, and EN 1998-1 seismic response spectra across OmniMechanics. Every downstream specification (`prd.md`, `canonical-domain-model.md`, `canonical-use-cases.md`, `canonical-api-cli-contract.md`, `acceptance-scenarios.md`) and shared consumer (`cap-frame-analysis-engine`, `cap-eurocode-rc-design`, `cap-sketcher-frame-modeler`) must use these exact canonical terms.

---

## 2. Canonical Terms by Category

### 2.1 Actors

| Canonical Term | Definition | Aliases / Discouraged Terms |
| --- | --- | --- |
| **Engineer / Instructor** | Primary user who configures the National Annex profile, tags load cases with action types, generates or edits EN 1990/EN 1998 combinations, defines seismic hazard parameters, and inspects governing envelopes. | Designer, Structural Analyst, Teacher |
| **Student** | Learner who inspects how Eurocode partial factors ($\gamma$), combination coefficients ($\psi$), and design spectra ($S_d(T)$) transform individual load cases into design combinations and envelopes in Greek or English. | Pupil, Learner |
| **Frame Analysis Engine** (`cap-frame-analysis-engine`) | Downstream solver consumer that owns `StructuralModel` and `LoadCase` entities, queries `Combination` tables, `SeismicMassContribution` rules, and `DesignSpectrum` ordinate functions, and computes `ResultSet` and `Envelope` outputs. | DSM Solver, Frame Engine |
| **RC Design Engine** (`cap-eurocode-rc-design`) | Downstream consumer that reads ULS (`ULS_STR`, `SEISMIC`) and SLS (`SLS_CHAR`, `SLS_FREQ`, `SLS_QP`) internal-force envelopes and governing combination metadata to perform EN 1992-1-1 section checks. | Concrete Designer, EC2 Module |

### 2.2 Business Objects & Normative Entities

| Canonical Term | Definition | Aliases / Discouraged Terms |
| --- | --- | --- |
| **ActionType** | Normative Eurocode 0 classification assigned to a `LoadCase`: `Permanent`, `Imposed{category: A..H}`, `Snow`, `Wind`, `Temperature`, `Seismic`, or `Accidental` (reserved). Owned by `cap-eurocode-actions-combinations`. | Load Category, Action Nature, Case Type |
| **ImposedCategory** | Occupancy/use sub-category `A` through `H` under EN 1991-1-1 for `Imposed` variable actions (`A`: residential, `B`: office, `C`: congregation, `D`: shopping, `E`: storage, `F`: light traffic $\le 30\text{ kN}$, `G`: medium traffic $30\text{--}160\text{ kN}$, `H`: roofs). Determines $\psi_0, \psi_1, \psi_2$. | Live Load Class, Occupancy Class |
| **NationalAnnexProfile** | Editable normative parameter set specifying partial safety factors ($\gamma_G, \gamma_Q$), reduction factor $\xi$, combination coefficients ($\psi_0, \psi_1, \psi_2$) per action category, and EN 1998-1 seismic zone/ground/importance tables. Default profile code is `GR` (Greek National Annex). | NA Config, Code Profile, Country Annex |
| **Combination** | Named linear or modal superposition specification consisting of a stable string identifier (`CO1`, `CO2`, ...), a `LimitState`, an `origin` (`Generated`, `UserEdited`, `Manual`), an optional equation tag (`Eq6_10`, `Eq6_10a`, `Eq6_10b`, `Eq6_12b`), and a list of `CombinationTerm` entries plus optional `SeismicDirectionalCombination`. | Load Combo, Design Case, Factored Case |
| **CombinationTerm** | Single factored entry within a `Combination` pairing a `load_case_id` (`LC1`, ...) with a scalar multiplier `factor` ($c_i$) and a `CombinationRole` (`PermanentUnfavorable`, `PermanentFavorable`, `LeadingVariable`, `AccompanyingVariable`, `SeismicAction`). | Load Factor Pair, Case Multiplier |
| **SeismicSettings** | Complete EN 1998-1 configuration for a project: seismic hazard zone (`ZoneI`, `ZoneII`, `ZoneIII` or custom $a_{gR}$), `GroundType` (`A`–`E`), `ImportanceClass` (`I`–`IV` or custom $\gamma_I$), `StructuralSystemGuidance`, behavior factor $q$, viscous damping ratio $\zeta$ (default $5\%$), combination rule (`SRSS` or `CQC`), and vertical component enablement. | EC8 Parameters, Earthquake Settings |
| **DesignSpectrum** | Sampled and analytically evaluable EN 1998-1 Type 1 horizontal (and optional vertical) spectrum curves containing both elastic response spectrum $S_e(T)$ and design spectrum $S_d(T)$ in $\text{m/s}^2$ (and normalized $S_d(T)/g$), alongside characteristic periods $(T_B, T_C, T_D)$, soil factor $S$, design ground acceleration $a_g = \gamma_I a_{gR}$, and lower-bound factor $\beta = 0.20$. | Response Spectrum Curve, EC8 Spectrum |
| **SeismicMassCombinationRule** | Normative rule $\sum G_{k,j} + \sum \psi_{E,i} Q_{k,i}$ (with $\psi_{E,i} = \varphi \cdot \psi_{2,i}$, default $\varphi = 1.0$ or occupancy-based EN 1998-1 §4.2.4) exported as per-`LoadCase` mass participation multipliers for modal and lateral force analysis. | Seismic Gravity Weight, Mass Combo |
| **EnvelopeSpecification** | Normative grouping of `Combination` IDs by `LimitState` category (e.g., all ULS persistent/transient, all ULS seismic, all SLS quasi-permanent) instructing `cap-frame-analysis-engine` which combinations to envelope for maximum/minimum internal forces and displacements. | Envelope Filter, Governing Set |

### 2.3 Workflows & Actions

| Canonical Term | Definition | Aliases / Discouraged Terms |
| --- | --- | --- |
| **GenerateCombinations** | Deterministic expansion of a set of tagged `LoadCase` descriptors and a `NationalAnnexProfile` into the complete permutation set of EN 1990 ULS, SLS, and EN 1998 Seismic combinations while preserving existing `UserEdited` and `Manual` combinations. | Auto-Combo, Expand Permutations |
| **ComputeDesignSpectrum** | Calculation of EN 1998-1 Type 1 horizontal and optional vertical elastic ($S_e$) and design ($S_d$) spectra from `SeismicSettings` and `NationalAnnexProfile`. | Build Spectrum, Generate Sd(T) |
| **ComputeLateralForceDistribution** | Normative EN 1998-1 §4.3.3.2 calculation of total seismic base shear $F_b = S_d(T_1) \cdot m \cdot \lambda$ and story/nodal horizontal forces $F_i = F_b \frac{z_i m_i}{\sum z_j m_j}$ from fundamental period $T_1$ and elevation-mass pairs $(z_i, m_i)$. | Equivalent Static Seismic, ELF Calculation |

### 2.4 Policies, Rules & Enumerations

| Canonical Term | Definition | Aliases / Discouraged Terms |
| --- | --- | --- |
| **LimitState** | Canonical enumeration of design verification states: `ULS_STR` (structural resistance), `ULS_GEO` (geotechnical), `SLS_CHAR` (characteristic/rare), `SLS_FREQ` (frequent), `SLS_QP` (quasi-permanent), and `SEISMIC` (EN 1998-1 seismic ULS). | Design State, Check Type |
| **CombinationOrigin** | Lifecycle provenance of a `Combination`: `Generated` (created automatically by `GenerateCombinations` and replaced on regeneration), `UserEdited` (originally generated but modified by the user; preserved across regenerations unless explicitly reset), `Manual` (created from scratch by the user; always preserved). | Combo Source, Provenance |
| **UlsEquationPolicy** | Selection in `NationalAnnexProfile` between EN 1990 Eq. 6.10 (`Single6_10`) and the twin expressions Eq. 6.10a / 6.10b (`Twin6_10a_6_10b` with reduction factor $\xi$, default $\xi = 0.85$). | ULS Formula Choice, 6.10 vs 6.10a/b |
| **PermanentActionFavorabilityPolicy** | Policy controlling whether permanent load cases (`Permanent`) are combined with unfavorable ($\gamma_{G,\text{sup}} = 1.35$) and favorable ($\gamma_{G,\text{inf}} = 1.00$) branches in ULS combinations (`UnfavorableOnly` vs `BothBranches`). | Gamma_G Split Policy |
| **ModalCombinationRule** | Rule for combining modal peak responses in Response Spectrum Analysis: `SRSS` (Square Root of Sum of Squares) or `CQC` (Complete Quadratic Combination with Der Kiureghian correlation coefficients $\rho_{ij}$). | Modal Superposition Method |

---

## 3. Critical Semantic Distinctions

1. **`LoadCase` [frame] vs `ActionType` [actions]:**
   - `LoadCase` is a container of physical forces, settlements, or thermal loads owned by `cap-frame-analysis-engine`.
   - `ActionType` is the normative Eurocode classification (`Permanent`, `Imposed{A..H}`, `Snow`, `Wind`, `Temperature`, `Seismic`, `Accidental`) owned by `cap-eurocode-actions-combinations`. A `LoadCase` carries an `ActionType` tag so `cap-eurocode-actions-combinations` can assign $\gamma$ and $\psi$ factors without owning the physical loads.
2. **`Combination` [actions] vs `ResultSet` / `Envelope` [frame]:**
   - `Combination` is a pure normative recipe of `(load_case_id, factor)` terms and limit-state metadata owned by `cap-eurocode-actions-combinations`.
   - `ResultSet` and `Envelope` are solved structural response quantities (reactions, displacements, $M/Q/N$ diagrams, extrema with governing `Combination` IDs) computed and owned by `cap-frame-analysis-engine`.
3. **Linear Superposition vs Nonlinear Per-Combination Solve:**
   - A `Combination` defines exact scalar multipliers $c_i$ for each `LoadCase`. Whether the solver applies $c_i$ by linear superposition of solved `LoadCase` results ($\sum c_i R_i$) or by assembling a combined load vector $\sum c_i F_i$ for iterative nonlinear analysis (P-$\Delta$, cables, unilateral supports) is decided by `cap-frame-analysis-engine`, using the exact same `Combination` definition.
4. **`Generated` vs `UserEdited` vs `Manual` Combinations:**
   - Regenerating combinations after adding a new `LoadCase` or changing a $\psi$ factor replaces only `Generated` combinations; `UserEdited` and `Manual` combinations remain untouched unless the user invokes an explicit reset.
5. **Elastic Spectrum $S_e(T)$ vs Design Spectrum $S_d(T)$:**
   - $S_e(T)$ includes viscous damping correction $\eta = \sqrt{10/(5+\zeta)} \ge 0.55$ for $\zeta \ne 5\%$.
   - $S_d(T)$ incorporates the behavior factor $q$ (with $\eta$ set to $1.0$ per EN 1998-1 §3.2.2.5) and enforces the lower bound $\beta \cdot a_g = 0.20 a_g$ for $T \ge T_B$.

---

## 4. Open Terminology Issues

None. All vocabulary aligns with [`cross-capability-conventions.md`](../cross-capability-conventions.md) and discovery batches 1–4.
