## Issue Metadata

- Issue number: `021`
- Owning capability node: `/.okf/capabilities/cap-eurocode-actions-combinations.md`
- Artifact root: `docs/architecture/cap-eurocode-actions-combinations/`
- Issue file: `docs/agents/issues/pending/20261102-021-eurocode-na-profile-en1990-combination-engine.md`
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

Implement **Vertical Slice 1** of `cap-eurocode-actions-combinations`: the Greek National Annex (`GR`) profile & structural system $q$-factor guidance catalog, the EN 1990 combination generator (`ULS_STR`, `ULS_GEO`, `SLS_CHAR`, `SLS_FREQ`, `SLS_QP`), user provenance management (`Generated` $\to$ `UserEdited`, `Manual`, delete/reset), and `EnvelopeSpecification` grouping exposed via `actions_*` Tauri IPC commands.

### 1. Normative Domain & Greek National Annex (`src-tauri/src/actions/`):
- **`types.rs` & `na_profile.rs`:**
  - `ActionType` (`Permanent`, `Imposed { category: A..H }`, `Snow`, `Wind`, `Temperature`, `Seismic`, `Accidental` reserved), `LoadCaseDescriptor`, `NationalAnnexProfile`, `UlsEquationPolicy` (`Single6_10`, `Twin6_10a_6_10b`), `PermanentFavorabilityPolicy` (`BothBranches`, `UnfavorableOnly`, `PerCasePermutation`), and `StructuralSystemGuidanceEntry`.
  - Exact Greek NA (`GR`) defaults: $\gamma_{G,\text{sup}}=1.35$, $\gamma_{G,\text{inf}}=1.00$, $\gamma_{Q,\text{sup}}=1.50$, $\gamma_{Q,\text{inf}}=0.00$, $\xi=0.85$, $\psi_0/\psi_1/\psi_2$ for Categories A–H, Snow, Wind, and Temperature (`SC-001`).
  - Normative invariant validation (`INV-ACT-01..04`, `INV-ACT-06`) returning bilingual `ACTIONS_*` error codes (`ACTIONS_UNKNOWN_NA_CODE`, `ACTIONS_INVALID_GAMMA_FACTOR`, `ACTIONS_INVALID_PSI_FACTOR`, `ACTIONS_INVALID_XI_FACTOR`, `ACTIONS_NO_PERMANENT_LOAD_CASE`, `ACTIONS_ACCIDENTAL_NOT_SUPPORTED`, `ACTIONS_TOO_MANY_PERMANENT_CASES`, `ACTIONS_DUPLICATE_LOAD_CASE_ID`, `ACTIONS_EMPTY_COMBINATION_TERMS`, `ACTIONS_UNKNOWN_LOAD_CASE_REF`, `ACTIONS_INVALID_COMBINATION_FACTOR`, `ACTIONS_COMBINATION_NOT_FOUND`) (`SC-009`).
- **`combinations.rs`:**
  - Deterministic leading-variable permutation generator for `ULS_STR` / `ULS_GEO` (both `Single6_10` and `Twin6_10a_6_10b` with $\xi = 0.85$), `SLS_CHAR` (Eq. 6.14b), `SLS_FREQ` (Eq. 6.15b), and `SLS_QP` (Eq. 6.16b), omitting zero-factor accompanying terms ($\psi_0 = 0$ or $\psi_2 = 0$) (`SC-002`, `SC-003`).
  - Bilingual (`el`/`en`) symbolic formula builder (e.g., `"1.35·G1 + 1.50·Q1 + 0.90·W1 (Εξ. 6.10)"`) and `EnvelopeSpecification` group builder (`ENV_ULS_STR`, `ENV_ULS_GEO`, `ENV_SLS_CHAR`, `ENV_SLS_FREQ`, `ENV_SLS_QP`).
  - Provenance state machine (`Generated` $\to$ `UserEdited` on edit via `generation_signature` matching, `Manual` creation, deletion, and `ResetToGenerated` restoration) (`SC-004`).
- **Tauri IPC Commands:**
  - Register `actions_get_default_na_profile`, `actions_validate_na_profile`, `actions_generate_combinations`, `actions_save_custom_combination`, and `actions_delete_or_reset_combination` in `src-tauri/src/lib.rs`.

## Acceptance criteria

- [x] `actions_get_default_na_profile` returns the complete Greek National Annex (`GR`) $\gamma, \xi, \psi_0, \psi_1, \psi_2$ table and structural system $q$-factor guidance catalog with bilingual descriptions (`SC-001`).
- [x] `actions_generate_combinations` produces exact deterministic leading-variable permutations and favorable/unfavorable permanent branches for `Single6_10`, `Twin6_10a_6_10b` ($\xi = 0.85$), `SLS_CHAR`, `SLS_FREQ`, and `SLS_QP`, along with `EnvelopeSpecification` sets (`SC-002`, `SC-003`).
- [x] `actions_save_custom_combination` and `actions_delete_or_reset_combination` promote edited generated combinations to `UserEdited`, preserve `UserEdited` and `Manual` combinations across subsequent `actions_generate_combinations` runs (`reset_user_edited: false`), and restore normative factors on `ResetToGenerated` (`SC-004`).
- [x] Normative validation rejects invalid $\psi$ hierarchies ($\psi_1 > \psi_0$), missing permanent load cases, and reserved `Accidental` load cases with the exact `ACTIONS_*` error codes (`SC-009`).
- [x] Companion unit tests (`*_test.rs`) verify `SC-001`, `SC-002`, `SC-003`, `SC-004`, and `SC-009` in $< 5\text{ ms}$; all files obey $\le 1000$ lines (`npm run check:limits` and `cargo test` pass).

## Artifact sync required

- Application PRD: `none - already synthesized in docs/prd.md`
- Application architecture summary: `none - matches src-tauri/src/actions/ boundary in docs/architecture/application-architecture-summary.md`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-eurocode-actions-combinations.md`
- Issue registry: `required`; node `issues:` reference: `required: .okf/capabilities/cap-eurocode-actions-combinations.md`
- Reason/no-impact decision: `Implements Slice 1 of the specified cap-eurocode-actions-combinations contract without changing product or cross-capability architecture truth.`

## Human review gate

`None` (backend normative engine and IPC boundary slice with no rendered UI/UX changes).

## Blocked by

`None - can start immediately`

## Artifact anchors

- PRD: §4.1 (`ActionType` & `GR` table), §4.2 (EN 1990 equations & permutations), §5 Workflows 1–3
- Domain model: `NationalAnnexProfile`, `CombinationSet`, `Combination`, `EnvelopeSpecification`, `INV-ACT-01..04`, `INV-ACT-06`
- Use cases: `GetDefaultNationalAnnexProfile`, `ValidateAndApplyNationalAnnexProfile`, `GenerateCombinations`, `SaveCustomCombination`, `DeleteOrResetCombination`
- Contract: `actions_get_default_na_profile`, `actions_validate_na_profile`, `actions_generate_combinations`, `actions_save_custom_combination`, `actions_delete_or_reset_combination`, error codes §4

## Acceptance scenarios addressed

- `SC-001`: Greek National Annex (`GR`) Default Factor & Guidance Catalog Retrieval
- `SC-002`: ULS STR Eq. 6.10 & SLS Combination Generation with Leading-Variable Permutations
- `SC-003`: Twin ULS Equations (Eq. 6.10a & 6.10b) with Reduction Factor $\xi = 0.85$
- `SC-004`: User-Edited (`UserEdited`) and Custom (`Manual`) Combinations Survive Regeneration
- `SC-009`: Normative Validation Rejections (Invalid $\psi$ Hierarchy, Missing $G$, Reserved `Accidental`)

## Verification obligations

- Policy source: `/.okf/project.md`

| Scenario | Backend boundary | Frontend integration | End-to-end journey |
|---|---|---|---|
| `SC-001` | `verified: passed in actions_test::test_sc001_greek_national_annex_retrieval` | `deferred: owned by cap-sketcher-frame-modeler UI slice` | `not-applicable` |
| `SC-002` | `verified: passed in actions_test::test_sc002_uls_str_eq6_10_and_sls_permutations` | `deferred: owned by cap-sketcher-frame-modeler UI slice` | `not-applicable` |
| `SC-003` | `verified: passed in actions_test::test_sc003_twin_equations_6_10a_and_6_10b` | `deferred: owned by cap-sketcher-frame-modeler UI slice` | `not-applicable` |
| `SC-004` | `verified: passed in actions_test::test_sc004_user_edited_and_manual_survive_regeneration` | `deferred: owned by cap-sketcher-frame-modeler UI slice` | `not-applicable` |
| `SC-009` | `verified: passed in actions_test::test_sc009_normative_validation_rejections` | `deferred: owned by cap-sketcher-frame-modeler UI slice` | `not-applicable` |
