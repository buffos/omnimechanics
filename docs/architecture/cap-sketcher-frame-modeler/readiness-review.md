# Architecture Readiness Review: Mode 2 Structural Frame Modeler (`cap-sketcher-frame-modeler`)

- **Capability:** [`cap-sketcher-frame-modeler`](../../../.okf/capabilities/cap-sketcher-frame-modeler.md)
- **State Policy:** `own` (leaf capability under roll-up [`cap-geometry-canvas-sketcher`](../../../.okf/capabilities/cap-geometry-canvas-sketcher.md))
- **Reviewed Artifacts:**
  - [`domain-glossary.md`](domain-glossary.md)
  - [`prd.md`](prd.md)
  - [`canonical-domain-model.md`](canonical-domain-model.md)
  - [`canonical-use-cases.md`](canonical-use-cases.md)
  - [`canonical-api-cli-contract.md`](canonical-api-cli-contract.md)
  - [`acceptance-scenarios.md`](acceptance-scenarios.md)
- **Root Synthesis Documents Checked:**
  - [`docs/prd.md`](../../prd.md)
  - [`docs/architecture/application-architecture-summary.md`](../application-architecture-summary.md)
  - [`docs/architecture/cross-capability-conventions.md`](../cross-capability-conventions.md)
- **Review Date:** 2026-11-01
- **Verdict:** **READY FOR `specified` STATE** (0 High, 0 Medium, 0 Low blocking findings)

---

## 1. Findings

**No material findings (`High`, `Medium`, or `Low` blockers) were found across the 6 canonical artifacts.**

All previously identified gaps from [`requirements-gap-analysis.md`](../cap-geometry-canvas-sketcher/requirements-gap-analysis.md) (`G1` through `G4`) have been systematically resolved:
1. **Single Model Ownership (`G3` Resolved):** The retired `SketchModel` is eliminated. Mode 2 edits `StructuralModel` (owned by `cap-frame-analysis-engine`) directly and preserves exact canonical names (`Node`, `Element`, `FrameMember`, `TrussBar`, `Spring`, `Cable`, `EndRelease`, `RigidOffset`, `Section`, `Material`, `Support`, `Load`, `LoadCase`, `Combination`, `ResultSet`, `Envelope`, `Diagnostics`, `MethodEligibility`, `ProjectDocument`).
2. **Internal Hinge & Member Load Semantics (`G3` Resolved):** Node-level internal hinges are explicitly specified as a modeler gesture (`InternalHingeGesture`) that expands to $k - 1$ member-end moment releases (`EndRelease.moment = true`) to prevent nodal rotational singularity (`INV-6`, `SC-005`). Member point and distributed loads attach at arbitrary positions $a \in [0, L]$ without splitting the element, whereas drawing a node/member that snaps to an interior point or intersection executes `TopologicalMemberSplit` (`SC-003`, `SC-006`).
3. **Real-Time Linear vs. Background Nonlinear Reconciliation (`G2` Resolved):** Every model mutation increments `StructuralModel.revision` (including `Undo`/`Redo`), triggers a debounced (~$150\text{ ms}$) synchronous linear DSM preview (`frame_solve_linear`), and manages cancellable background `AnalysisJob`s via `analysis://progress` (`SC-007`, `SC-008`).
4. **Mode 3 Classifier & Handoff Boundary (`G2` Resolved):** Mode 2 queries `teaching_classify_model` to enable "Open Step-by-Step" when eligible, explains ineligibility in Greek/English (such as models with `Cable` elements), always offers the DSM Walkthrough, and supports opening Mode 3 presets as editable sketches in Mode 2 (`SC-011`, `SC-012`, `SC-013`).
5. **Product-Surface Reachability & Verification (`G4` Resolved):** Every scenario (`SC-001` through `SC-014`) defines explicit `backend-boundary`, `frontend-integration` (Vitest), and `end-to-end` verification surfaces, ensuring no behavior remains hidden behind the UI surface.

---

## 2. Cross-Document Consistency Audit

| Check Dimension | Status | Evidence Across Artifacts |
| --- | --- | --- |
| **Canonical Vocabulary** | Pass | Matches [`cross-capability-conventions.md`](../cross-capability-conventions.md) 100% across Glossary, PRD, Domain Model, Use Cases, Contract, and Scenarios. |
| **Units & Sign Conventions** | Pass | SI internally (`kN`, `m`, `kNm`, `kN/m`, `°C`), $E$ entered in `GPa`, global $y$ up, $N > 0$ tension, $M > 0$ bottom ($-y$) fibre in tension, diagrams plotted on tension side (`BR-1`, `BR-2`, `SC-005`, `SC-007`). |
| **IPC Command & Error Naming** | Pass | Prefixes `frame_`, `actions_`, `teaching_`, `project_`, event `analysis://progress`, and bilingual `{ code, message_el, message_en, details? }` error envelopes aligned in Use Cases and Contract. |
| **Verification Surfaces** | Pass | All 14 scenarios (`SC-001`..`SC-014`) map pure-TS geometry/snapping/store logic to `frontend-integration` (Vitest), IPC/solver behavior to `backend-boundary` (`cargo test`), and UI reachability to `end-to-end`. |
| **Code Size & Modularity** | Pass | Pure TypeScript services (`SnappingPolicyService`, `TopologicalSplitService`, `InternalHingeExpansionPolicy`, `CadTransformationService`, `ModelerStore`) are cleanly separated from SVG rendering components to stay well within the $\le 1000\text{ line}$ TS and $\le 500\text{ line}$ HTML limits. |

---

## 3. Residual Risks & Implementation Watch-Items

1. **Vitest Harness Bootstrap (Slice 1 Prerequisite):**
   - *Risk:* The repository currently has `0` `*.spec.ts` files and no Vitest configuration (`docs/prd.md` §9).
   - *Mitigation:* Delivery Slice 1 for Mode 2 must bootstrap the Vitest runner (`frontend-integration: required`) as its first vertical task so `SC-001`..`SC-014` pure-TS tests execute in CI and pre-commit checks.
2. **SVG Rendering Performance at 1000 Elements:**
   - *Risk:* Re-rendering SVG paths and text labels on every pointer move during live cursor hover probing could cause frame drops on large frames (~$500$ nodes / $1000$ members).
   - *Mitigation:* Keep the static geometry/diagram SVG layer in an Angular `OnPush` computed signal that only updates on `revision` or `ResultOverlayState` changes, and render the `CrossSectionHoverProbe` and rubber-band snap overlays in a lightweight dedicated overlay `<g>` layer.
3. **Topological Split Load Partitioning Precision:**
   - *Risk:* Floating-point rounding when splitting a member under a partial trapezoidal load could shift $a$ or $b$ by $10^{-15}\text{ m}$ outside $[0, L_{\text{sub}}]$.
   - *Mitigation:* Clamp split local coordinates to $[0, L_{\text{sub}}]$ with tolerance $\epsilon = 10^{-9}\text{ m}$ inside `TopologicalSplitService` and cover with dedicated Vitest unit tests (`SC-003`).

---

## 4. Application Synthesis Gate & Readiness Assessment

- **Application Synthesis Status:** **PASSED (`no impact`)**. Both [`docs/prd.md`](../../prd.md) and [`docs/architecture/application-architecture-summary.md`](../application-architecture-summary.md) already define Mode 2's exact role, 6-slice delivery sequence, shared rules, and verification strategy. No root product or cross-capability architectural truth was altered or contradicted by this specification set.
- **Capability Readiness:** `cap-sketcher-frame-modeler` satisfies all criteria to transition from `bounded` to **`specified`** and is ready for vertical-slice issue generation (`reference-docs-to-issues`) once sibling `cap-frame-analysis-engine` completes its specification gate.
