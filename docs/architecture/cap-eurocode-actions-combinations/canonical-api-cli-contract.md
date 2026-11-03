# Canonical API / IPC & CLI Contract: Eurocode Actions, Load Combinations & Seismic Loading (`cap-eurocode-actions-combinations`)

- **Owning capability:** [`cap-eurocode-actions-combinations`](../../../.okf/capabilities/cap-eurocode-actions-combinations.md)
- **Cross-capability conventions:** [`cross-capability-conventions.md`](../cross-capability-conventions.md)
- **Glossary:** [`domain-glossary.md`](domain-glossary.md)
- **Use-Case Model:** [`canonical-use-cases.md`](canonical-use-cases.md)
- **Status:** Canonical Contract (2026-11-01)

---

## 1. Purpose & Transport-Neutral Conventions

This document defines the canonical external contract for `cap-eurocode-actions-combinations` across Tauri IPC (`actions_*` commands), direct Rust crate calls from `cap-frame-analysis-engine` and `cap-eurocode-rc-design`, and CLI/test-runner invocations.

### Transport-Neutral Conventions
- **Command Prefix:** Every Tauri IPC command owned by this capability begins with `actions_` per [`cross-capability-conventions.md`](../cross-capability-conventions.md).
- **Units:** SI internally and across payloads: acceleration in $\text{m/s}^2$ and normalized $g$ ($g = 9.81\text{ m/s}^2$), period $T$ in seconds ($\text{s}$), height $z$ in metres ($\text{m}$), seismic mass in tonnes ($\text{t} = 10^3\text{ kg} = \text{kN}/(\text{m/s}^2)$), lateral forces in $\text{kN}$, damping ratio $\zeta$ in percent ($\%$), and all $\gamma, \xi, \psi, q$ factors dimensionless.
- **Identifiers:**
  - Load case references use model-unique string IDs (`"LC1"`, `"LC2"`, ...).
  - Combinations use model-unique string IDs (`"CO1"`, `"CO2"`, ...).
  - Envelope specifications use stable IDs (`"ENV_ULS_STR"`, `"ENV_ULS_GEO"`, `"ENV_SEISMIC"`, `"ENV_SLS_CHAR"`, `"ENV_SLS_FREQ"`, `"ENV_SLS_QP"`).
- **Bilingual Text Shape:** All user-facing strings use `{ el: string, en: string }`.
- **Error Envelope:** Every error returns `{ code: string, message_el: string, message_en: string, details?: object }`.

---

## 2. Canonical Payload Shapes (JSON / Serde DTOs)

### 2.1 `ActionType` & `LoadCaseDescriptor`
```json
{
  "id": "LC2",
  "code": "Q1",
  "name": { "el": "Κινητό Κατοικιών", "en": "Residential Imposed" },
  "action_type": { "kind": "Imposed", "category": "A" }
}
```
Supported `action_type` variants:
- `{ "kind": "Permanent" }`
- `{ "kind": "Imposed", "category": "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" }`
- `{ "kind": "Snow" }`
- `{ "kind": "Wind" }`
- `{ "kind": "Temperature" }`
- `{ "kind": "Seismic" }`
- `{ "kind": "Accidental" }` *(reserved)*

### 2.2 `NationalAnnexProfile`
```json
{
  "code": "GR",
  "name": {
    "el": "Ελληνικό Εθνικό Προσάρτημα (ΕΛΟΤ EN 1990/1998)",
    "en": "Greek National Annex (ELOT EN 1990/1998)"
  },
  "uls_equation_policy": "Single6_10",
  "permanent_favorability_policy": "BothBranches",
  "gamma_g_sup": 1.35,
  "gamma_g_inf": 1.00,
  "gamma_q_sup": 1.50,
  "gamma_q_inf": 0.00,
  "xi_reduction": 0.85,
  "psi_factors": {
    "Imposed_A": { "psi_0": 0.70, "psi_1": 0.50, "psi_2": 0.30 },
    "Imposed_B": { "psi_0": 0.70, "psi_1": 0.50, "psi_2": 0.30 },
    "Imposed_C": { "psi_0": 0.70, "psi_1": 0.70, "psi_2": 0.60 },
    "Imposed_D": { "psi_0": 0.70, "psi_1": 0.70, "psi_2": 0.60 },
    "Imposed_E": { "psi_0": 1.00, "psi_1": 0.90, "psi_2": 0.80 },
    "Imposed_F": { "psi_0": 0.70, "psi_1": 0.70, "psi_2": 0.60 },
    "Imposed_G": { "psi_0": 0.70, "psi_1": 0.50, "psi_2": 0.30 },
    "Imposed_H": { "psi_0": 0.00, "psi_1": 0.00, "psi_2": 0.00 },
    "Snow":      { "psi_0": 0.50, "psi_1": 0.20, "psi_2": 0.00 },
    "Wind":      { "psi_0": 0.60, "psi_1": 0.20, "psi_2": 0.00 },
    "Temperature": { "psi_0": 0.60, "psi_1": 0.50, "psi_2": 0.00 }
  }
}
```

### 2.3 `Combination` & `CombinationSet`
```json
{
  "revision": 4,
  "combinations": [
    {
      "id": "CO1",
      "name": {
        "el": "ΟΚA STR - Κύρια Q1 (Δυσμενής G)",
        "en": "ULS STR - Leading Q1 (Unfavorable G)"
      },
      "limit_state": "ULS_STR",
      "origin": "Generated",
      "equation": "Eq6_10",
      "leading_load_case_id": "LC2",
      "terms": [
        { "load_case_id": "LC1", "factor": 1.35, "role": "PermanentUnfavorable" },
        { "load_case_id": "LC2", "factor": 1.50, "role": "LeadingVariable" },
        { "load_case_id": "LC3", "factor": 0.90, "role": "AccompanyingVariable" }
      ],
      "symbolic_formula": {
        "el": "1.35·G1 + 1.50·Q1 + 0.90·W1 (Εξ. 6.10)",
        "en": "1.35·G1 + 1.50·Q1 + 0.90·W1 (Eq. 6.10)"
      },
      "generation_signature": "ULS_STR:Eq6_10:G_SUP:LEAD_LC2"
    }
  ],
  "envelopes": [
    {
      "id": "ENV_ULS_STR",
      "name": { "el": "Περιβάλλουσα ΟΚΑ (STR)", "en": "ULS (STR) Envelope" },
      "limit_state": "ULS_STR",
      "combination_ids": ["CO1", "CO2", "CO3", "CO4"]
    }
  ],
  "warnings": []
}
```

### 2.4 `SeismicSettings` & `DesignSpectrum`
```json
{
  "zone": "ZoneII",
  "ag_reference_g": 0.24,
  "ground_type": "B",
  "soil_factor_s": 1.20,
  "tb_s": 0.15,
  "tc_s": 0.50,
  "td_s": 2.00,
  "importance_class": "ClassII",
  "gamma_i": 1.00,
  "system_guidance": "RcMomentFrameDcm",
  "behavior_factor_q": 3.90,
  "damping_ratio_percent": 5.0,
  "beta_lower_bound": 0.20,
  "modal_combination_rule": "CQC",
  "include_vertical_component": false,
  "vertical_behavior_factor_qv": 1.50,
  "psi_e_phi": 1.00
}
```

`DesignSpectrum` output payload:
```json
{
  "ag_m_s2": 2.3544,
  "ag_g": 0.24,
  "avg_m_s2": null,
  "eta_damping": 1.0,
  "soil_factor_s": 1.20,
  "tb_s": 0.15,
  "tc_s": 0.50,
  "td_s": 2.00,
  "beta_lower_bound": 0.20,
  "horizontal_curve": [
    { "period_s": 0.0, "se_m_s2": 2.82528, "sd_m_s2": 1.88352, "se_g": 0.288, "sd_g": 0.192 },
    { "period_s": 0.15, "se_m_s2": 7.0632, "sd_m_s2": 1.81108, "se_g": 0.720, "sd_g": 0.184615 }
  ],
  "vertical_curve": null,
  "queried_ordinates": []
}
```

---

## 3. Canonical Tauri IPC Commands

| Tauri Command Name | Use Case Mapping | Request Payload | Response Payload |
| --- | --- | --- | --- |
| `actions_get_default_na_profile` | `GetDefaultNationalAnnexProfile` | `{ code?: string }` | `{ profile: NationalAnnexProfile, action_catalog: ActionCategoryMetadata[], system_guidance_catalog: StructuralSystemGuidanceEntry[] }` |
| `actions_validate_na_profile` | `ValidateAndApplyNationalAnnexProfile` | `{ profile: NationalAnnexProfile, load_cases?: LoadCaseDescriptor[], existing_combinations?: Combination[], seismic_settings?: SeismicSettings }` | `{ profile: NationalAnnexProfile, combination_set?: CombinationSet }` |
| `actions_generate_combinations` | `GenerateCombinations` | `{ load_cases: LoadCaseDescriptor[], profile: NationalAnnexProfile, seismic_settings?: SeismicSettings, existing_combinations: Combination[], include_uls_geo?: boolean, reset_user_edited?: boolean }` | `CombinationSet` |
| `actions_save_custom_combination` | `SaveCustomCombination` | `{ combination: CombinationDraft, load_cases: LoadCaseDescriptor[], existing_combinations: Combination[] }` | `CombinationSet` |
| `actions_delete_or_reset_combination` | `DeleteOrResetCombination` | `{ combination_id: string, action: "Delete" \| "ResetToGenerated", load_cases: LoadCaseDescriptor[], profile: NationalAnnexProfile, seismic_settings?: SeismicSettings, existing_combinations: Combination[] }` | `CombinationSet` |
| `actions_compute_design_spectrum` | `ComputeDesignSpectrum` | `{ settings: SeismicSettings, max_period_s?: number, num_points?: number, query_periods_s?: number[] }` | `{ spectrum: DesignSpectrum, queried_ordinates: SpectrumPoint[] }` |
| `actions_compute_seismic_mass_rule` | `ComputeSeismicMassRule` | `{ load_cases: LoadCaseDescriptor[], profile: NationalAnnexProfile, settings: SeismicSettings }` | `SeismicMassCombinationRule` |
| `actions_compute_lateral_forces` | `ComputeLateralForceDistribution` | `{ settings: SeismicSettings, fundamental_period_t1_s: number, n_storeys: number, levels: MassLevelInput[] }` | `LateralForceDistributionResult` |
| `actions_compute_cqc_matrix` | `ComputeCqcCorrelationMatrix` | `{ rule: "SRSS" \| "CQC", modal_periods_s: number[], damping_ratio_percent: number }` | `ModalCorrelationMatrix` |

---

## 4. Canonical Error Vocabulary

All domain errors raised by `cap-eurocode-actions-combinations` use the `ACTIONS_` prefix:

| Error Code | Trigger Condition | Bilingual Message Summary (`en` / `el`) |
| --- | --- | --- |
| `ACTIONS_UNKNOWN_NA_CODE` | Requested National Annex code is not `"GR"` or `"CUSTOM"`. | Unknown National Annex code / Άγνωστος κωδικός Εθνικού Προσαρτήματος |
| `ACTIONS_INVALID_GAMMA_FACTOR` | $\gamma_{G,\text{sup}} < \gamma_{G,\text{inf}}$, $\gamma_{G,\text{inf}} < 0$, or $\gamma_{Q,\text{sup}} \le 0$. | Invalid partial safety factor $\gamma$ / Μη έγκυρος επιμέρους συντελεστής ασφαλείας $\gamma$ |
| `ACTIONS_INVALID_PSI_FACTOR` | Any category violates $1.0 \ge \psi_0 \ge \psi_1 \ge \psi_2 \ge 0.0$. | Combination factors must satisfy $1 \ge \psi_0 \ge \psi_1 \ge \psi_2 \ge 0$ / Οι συντελεστές $\psi$ πρέπει να ικανοποιούν $1 \ge \psi_0 \ge \psi_1 \ge \psi_2 \ge 0$ |
| `ACTIONS_INVALID_XI_FACTOR` | Reduction factor $\xi \notin (0.0, 1.0]$. | Reduction factor $\xi$ must be in $(0, 1]$ / Ο συντελεστής μείωσης $\xi$ πρέπει να ανήκει στο $(0, 1]$ |
| `ACTIONS_NO_PERMANENT_LOAD_CASE` | `load_cases` contains zero `ActionType::Permanent` entries. | At least one Permanent ($G$) load case is required / Απαιτείται τουλάχιστον μία Μόνιμη ($G$) φόρτιση |
| `ACTIONS_ACCIDENTAL_NOT_SUPPORTED` | A load case is tagged `ActionType::Accidental`. | Accidental combinations are not supported in this version / Οι τυχαίοι συνδυασμοί δεν υποστηρίζονται στην παρούσα έκδοση |
| `ACTIONS_TOO_MANY_PERMANENT_CASES` | `PerCasePermutation` selected with $> 6$ `Permanent` cases. | Per-case permanent permutation is limited to at most 6 permanent load cases / Η συνδυαστική ανά μόνιμη φόρτιση επιτρέπεται έως 6 μόνιμες φορτίσεις |
| `ACTIONS_DUPLICATE_LOAD_CASE_ID` | Duplicate `id` in `load_cases`. | Duplicate load case ID / Διπλό αναγνωριστικό περίπτωσης φόρτισης |
| `ACTIONS_EMPTY_COMBINATION_TERMS` | `SaveCustomCombination` submitted with 0 non-zero terms. | Combination must contain at least one non-zero load case factor / Ο συνδυασμός πρέπει να περιέχει τουλάχιστον έναν μη μηδενικό όρο |
| `ACTIONS_UNKNOWN_LOAD_CASE_REF` | A `CombinationTerm` references a non-existent `load_case_id`. | Combination references an unknown load case / Ο συνδυασμός αναφέρεται σε άγνωστη περίπτωση φόρτισης |
| `ACTIONS_INVALID_COMBINATION_FACTOR` | A `CombinationTerm` has `NaN` or `±Infinity` factor. | Load factor must be a finite number / Ο συντελεστής φόρτισης πρέπει να είναι πεπερασμένος αριθμός |
| `ACTIONS_COMBINATION_NOT_FOUND` | `combination_id` not found in `existing_combinations`. | Combination ID not found / Δεν βρέθηκε ο συνδυασμός |
| `ACTIONS_INVALID_SEISMIC_PARAMETERS` | Invalid $a_{gR}, \gamma_I, S, T_B, T_C, T_D, q, \zeta, \beta$, or $\varphi$. | Invalid EN 1998-1 seismic parameters / Μη έγκυρες σεισμικές παράμετροι ΕΛΟΤ EN 1998-1 |
| `ACTIONS_INVALID_SPECTRUM_PERIOD` | Negative or non-finite period $T < 0$ in spectrum or CQC query. | Period $T$ must be non-negative / Η ιδιοπερίοδος $T$ πρέπει να είναι μη αρνητική |
| `ACTIONS_INVALID_LATERAL_FORCE_INPUT` | $T_1 < 0$, $\sum m_i \le 0$, or $\sum z_j m_j \le 0$. | Lateral force distribution requires positive mass and positive story elevation / Η κατανομή σεισμικών δυνάμεων απαιτεί θετική μάζα και θετικό ύψος ορόφων |

---

## 5. Canonical CLI & Parity Mapping

For headless verification or batch scripting (`cargo test --package bem_app --lib actions`), the same serde JSON payloads map 1-to-1 to the Rust functions in `src-tauri/src/actions/`:
- `actions::get_default_na_profile("GR")`
- `actions::generate_combinations(&request)`
- `actions::compute_design_spectrum(&request)`
- `actions::compute_seismic_mass_rule(&request)`
- `actions::compute_lateral_forces(&request)`
- `actions::compute_cqc_matrix(&request)`

All implementations (Rust core, Tauri IPC, TypeScript frontend store) must preserve exact numerical parity within $10^{-12}$ floating-point tolerance.
