# Architecture Readiness Review: 2D Frame Analysis Engine (`cap-frame-analysis-engine`)

- **Capability Owner:** `cap-frame-analysis-engine`
- **Review Date:** 2026-11-01
- **State Policy:** `mode: own` (absent `state_policy` in `.okf/capabilities/cap-frame-analysis-engine.md` defaults to `own`)
- **Reviewed Artifact Set:**
  - [domain-glossary.md](domain-glossary.md)
  - [prd.md](prd.md)
  - [canonical-domain-model.md](canonical-domain-model.md)
  - [canonical-use-cases.md](canonical-use-cases.md)
  - [canonical-api-cli-contract.md](canonical-api-cli-contract.md)
  - [acceptance-scenarios.md](acceptance-scenarios.md)
- **Root Synthesis Baseline Checked:**
  - [docs/prd.md](../../prd.md)
  - [docs/architecture/application-architecture-summary.md](../application-architecture-summary.md)
  - [docs/architecture/cross-capability-conventions.md](../cross-capability-conventions.md)

---

## 1. Findings-First Audit

**No High or Medium severity findings were identified.** All terminology, units, Greek sign conventions, aggregate invariants, synchronous vs. asynchronous execution semantics, error codes, and verification surfaces are internally consistent across all six specification artifacts and strictly conform to `cross-capability-conventions.md`.

### Low Severity / Observational Notes (Non-Blocking)

| ID | Severity | Affected Artifacts | Observation | Resolution / Mitigation in Spec |
| --- | --- | --- | --- | --- |
| **RR-LOW-01** | Low | `canonical-domain-model.md`, `canonical-use-cases.md` | Unilateral contact active-set iterations can exhibit period-2 cycling (chattering) on symmetric structures with borderline uplift. | Explicitly mitigated in `NonlinearConvergencePolicy` (§5 of Domain Model) via cycle detection and relaxation damping within the 50-iteration cap. |
| **RR-LOW-02** | Low | `prd.md`, `acceptance-scenarios.md` | Offline OpenSeesPy CSV reference files in `docs/benchmarks/frame/` are not yet committed to the repository (tracked as Low open item in root `docs/prd.md` §9). | Closed-form analytical equations (Euler-Bernoulli, Euler buckling, Irvine catenary) provide immediate self-contained verification in `cargo test` while the offline CSV generator script is executed during slice implementation. |

---

## 2. Cross-Document Consistency Verification

| Check Dimension | Status | Evidence Across Artifacts |
| --- | --- | --- |
| **Ubiquitous Language & Naming** | Pass | `StructuralModel`, `Node`, `Element` (`FrameMember`, `TrussBar`, `Spring`, `Cable`), `EndRelease`, `RigidOffset`, `Section`, `Material`, `Support`, `Load`, `LoadCase`, `LumpedMass`, `AnalysisKind`, `AnalysisJob`, `ResultSet`, `Envelope`, `Diagnostics`, `DsmWalkthroughData` used identically everywhere. |
| **Units & Greek Sign Convention** | Pass | SI internally ($\text{kN}, \text{m}, \text{kNm}, \text{kN/m}, ^\circ\text{C}, \text{t}$), $E, G$ entered in $\text{GPa}$ and scaled by $10^6$ at boundary; $M > 0$ when bottom ($-y_{\text{local}}$) fibre is in tension, diagrams drawn on tension side, $N > 0$ in tension. |
| **Superposition & Nonlinear Policy** | Pass | Superposition restricted to purely linear models (`INV-05`); `SecondOrderPDelta`, `Cable`, and unilateral models set `superposition_valid: false` and solve each `Combination` independently. |
| **Execution & Concurrency Model** | Pass | Synchronous `SolveLinearStatic` (`< 50 ms`) + background cancellable `StartAnalysisJob` (`analysis://progress` event) + monotonic `revision` superseding (`STALE_MODEL_REVISION`). |
| **Error Vocabulary** | Pass | Bilingual `{ code, message_el, message_en, details? }` with identical `SCREAMING_SNAKE` codes (`KINEMATIC_INSTABILITY`, `NONLINEAR_NOT_CONVERGED`, `PDELTA_BUCKLING_DIVERGENCE`, `STALE_MODEL_REVISION`, etc.) in Use Cases, Contract, and Scenarios. |
| **Verification Surface Mapping** | Pass | All 9 canonical scenarios (`SC-001` to `SC-009`) declare `backend-boundary: required`, `frontend-integration: required`, and `end-to-end: catalog-only` (or `not-applicable` for pure race test `SC-008`), matching root `.okf/project.md` policy. |

---

## 3. Residual Risks & Implementation Guidance

1. **Ill-Conditioned Stiffness with Extreme Rigid Offsets or Stiff Springs:**
   - *Risk:* User enters penalty-like spring stiffnesses ($k_x > 10^{15}\text{ kN/m}$) that degrade floating-point conditioning of $\mathbf{K}_{ff}$.
   - *Mitigation:* `KinematicDiagnosticsService` emits a conditioning warning when the reciprocal condition number estimates $< 10^{-12}$ while distinguishing true kinematic mechanisms from numerical ill-conditioning.
2. **Zero-Weight Catenary Limit ($w_0 \to 0$):**
   - *Risk:* When a cable material has $\gamma = 0$ or self-weight is disabled, the hyperbolic $\sinh^{-1}(V/H)$ formula has a $0/0$ removable singularity.
   - *Mitigation:* As specified in `canonical-domain-model.md` §3.2.4, when $w_0 L_0 / T_0 < 10^{-7}$, the element smoothly switches to the analytical taut-string / tension-only secant-tangent limit.

---

## 4. Application Synthesis Gate & Readiness Decision

- **Artifact-Impact Classification:** `no impact` on root product truth ([`docs/prd.md`](../../prd.md)) and `no impact` on cross-capability architecture truth ([`docs/architecture/application-architecture-summary.md`](../application-architecture-summary.md)). Both synthesis documents already establish `cap-frame-analysis-engine` as the owner of `src-tauri/src/frame/` and define the 6 Mode 2 delivery slices.
- **State Evaluation:**
  - `mode`: `own`
  - Current `.okf` state: `bounded`
  - Recommended target `.okf` state: **`specified`**
- **Gate Verdict:** **READY FOR `specified` STATE AND ISSUE SLICING** (once paired with `cap-sketcher-frame-modeler` for Mode 2 Slice 1).
