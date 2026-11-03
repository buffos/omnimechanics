# Architecture Readiness Review: Eurocode Actions, Load Combinations & Seismic Loading (`cap-eurocode-actions-combinations`)

- **Owning capability:** [`cap-eurocode-actions-combinations`](../../../.okf/capabilities/cap-eurocode-actions-combinations.md)
- **Reviewed artifact set:**
  - [`domain-glossary.md`](domain-glossary.md)
  - [`prd.md`](prd.md)
  - [`canonical-domain-model.md`](canonical-domain-model.md)
  - [`canonical-use-cases.md`](canonical-use-cases.md)
  - [`canonical-api-cli-contract.md`](canonical-api-cli-contract.md)
  - [`acceptance-scenarios.md`](acceptance-scenarios.md)
- **Root synthesis checked:** [`docs/prd.md`](../../prd.md), [`application-architecture-summary.md`](../application-architecture-summary.md), [`cross-capability-conventions.md`](../cross-capability-conventions.md)
- **Review Date:** 2026-11-01
- **Gate Verdict:** **READY FOR `specified` STATE** (0 High, 0 Medium, 0 Low blocking findings)

---

## 1. Findings Summary

**No material findings (0 High, 0 Medium, 0 Low blockers).**

All six canonical specification artifacts for `cap-eurocode-actions-combinations` were audited for cross-document consistency, Eurocode 0 (EN 1990) and Eurocode 8 (EN 1998-1) normative completeness, boundary hygiene with `cap-frame-analysis-engine` and `cap-eurocode-rc-design`, IPC naming/error conventions, and verification surface coverage.

---

## 2. Cross-Document & Normative Consistency Audit

| Check Dimension | Status | Evidence across Artifact Set |
| --- | :---: | --- |
| **Canonical Vocabulary Alignment** | PASS | `ActionType`, `ImposedCategory` (`A`–`H`), `Combination`, `LimitState` (`ULS_STR`, `ULS_GEO`, `SLS_CHAR`, `SLS_FREQ`, `SLS_QP`, `SEISMIC`), `CombinationOrigin` (`Generated`, `UserEdited`, `Manual`), `NationalAnnexProfile` (`GR`), `SeismicSettings`, and `DesignSpectrum` match [`cross-capability-conventions.md`](../cross-capability-conventions.md) verbatim. |
| **Greek National Annex (`GR`) Completeness** | PASS | $\gamma_{G,\text{sup}}=1.35$, $\gamma_{G,\text{inf}}=1.00$, $\gamma_{Q,\text{sup}}=1.50$, $\gamma_{Q,\text{inf}}=0.00$, $\xi=0.85$, $\psi_0/\psi_1/\psi_2$ for categories A–H, Snow, Wind, Temperature, Seismic hazard zones I/II/III ($0.16g, 0.24g, 0.36g$), Ground Types A–E, and Importance Classes I–IV ($0.8, 1.0, 1.2, 1.4$) are specified with identical numerical values in `prd.md`, `canonical-domain-model.md`, `canonical-api-cli-contract.md`, and `acceptance-scenarios.md`. |
| **EN 1990 Permutation & Provenance Rules** | PASS | Both Eq. 6.10 (`Single6_10`) and Eq. 6.10a/6.10b (`Twin6_10a_6_10b`) policies, favorable/unfavorable permanent branches (`BothBranches`, `UnfavorableOnly`, `PerCasePermutation`), and the `Generated` $\to$ `UserEdited` / `Manual` preservation lifecycle are consistently modeled and tested in `SC-002`, `SC-003`, and `SC-004`. |
| **EN 1998-1 Spectra, Mass & Lateral Force Rules** | PASS | Type 1 horizontal and optional vertical $S_e(T)$ and $S_d(T)$ formulas, $\beta = 0.20$ lower bound, $G + \psi_{E,i} Q$ mass participation, $30\%$ bidirectional signed combinations, lateral force distribution ($F_b, F_i, \lambda$), and CQC correlation matrix $\rho_{ij}$ are fully specified and covered by `SC-005` through `SC-008`. |
| **Pure Normative Boundary Enforcement** | PASS | The capability never solves stiffness matrices or performs section reinforcement checks; it emits `CombinationSet`, `DesignSpectrum`, `SeismicMassCombinationRule`, `LateralForceDistributionResult`, and `ModalCorrelationMatrix` payloads consumed by `cap-frame-analysis-engine` and `cap-eurocode-rc-design`. |
| **IPC & Error Contract Discipline** | PASS | All 9 Tauri IPC commands use the `actions_` prefix and all 15 domain error codes use the `ACTIONS_` prefix with bilingual `{ code, message_el, message_en, details? }` envelopes. |
| **Verification Surface Traceability** | PASS | `SC-001` through `SC-009` declare explicit `backend-boundary`, `frontend-integration`, and `end-to-end` verification modes aligned with [`docs/prd.md`](../../prd.md) §8. |

---

## 3. State Policy & Application Synthesis Gate

- **Node State Policy:** `mode: own` (absent `state_policy` in [`.okf/capabilities/cap-eurocode-actions-combinations.md`](../../../.okf/capabilities/cap-eurocode-actions-combinations.md) defaults to `own`).
- **Effective State Transition:** Ready to transition from `bounded` $\to$ `specified`.
- **Application Synthesis Gate Assessment:**
  - Checked [`docs/prd.md`](../../prd.md) and [`docs/architecture/application-architecture-summary.md`](../application-architecture-summary.md).
  - **Impact Classification:** `no impact` — the root synthesis documents already establish `cap-eurocode-actions-combinations` (`src-tauri/src/actions/`) as the shared normative owner for EN 1990 combinations and EN 1998 spectra in Mode 2 Slice 2 and Slice 6.

---

## 4. Residual Risks & Implementation Guidance

1. **Combinatorial Growth with Many Variable Load Cases (Low Risk):**
   - With $n$ variable load cases and `Twin6_10a_6_10b` + `BothBranches`, ULS generates $4n$ combinations. For typical 2D engineering models ($n \le 8$), this is $\le 32$ combinations ($< 1\text{ ms}$). For nonlinear analysis (P-$\Delta$, cables) where superposition does not hold, the Mode 2 UI should allow the user to select either a single combination or a filtered subset of combinations to solve iteratively.
2. **Deleted LoadCase Cleanup in Custom Combinations (Low Risk):**
   - `GenerateCombinations` prunes terms referencing deleted `LoadCase` IDs and emits a bilingual warning in `CombinationSet.warnings` if a `UserEdited` or `Manual` combination is modified or dropped. Companion unit tests in `src-tauri/src/actions/combinations_test.rs` must verify this pruning path alongside `SC-004`.
