# Architecture Readiness Review: Interactive 3D Visualization Canvas & UI (`cap-interactive-canvas`)

- **Capability Owner:** [`cap-interactive-canvas`](../../../.okf/capabilities/cap-interactive-canvas.md)
- **State Policy:** `mode: own` (absent in node frontmatter → defaults to `own`)
- **Reviewed Artifact Set:**
  - [`domain-glossary.md`](domain-glossary.md)
  - [`prd.md`](prd.md)
  - [`canonical-domain-model.md`](canonical-domain-model.md)
  - [`canonical-use-cases.md`](canonical-use-cases.md)
  - [`canonical-api-cli-contract.md`](canonical-api-cli-contract.md)
  - [`acceptance-scenarios.md`](acceptance-scenarios.md)
- **Root Synthesis Baseline Checked:**
  - [`docs/prd.md`](../../prd.md)
  - [`docs/architecture/application-architecture-summary.md`](../application-architecture-summary.md)
  - [`docs/architecture/cross-capability-conventions.md`](../cross-capability-conventions.md)
- **Review Date:** 2026-11-01

---

## 1. Findings Summary (Findings-First Gate)

- **High Severity Blockers:** `0`
- **Medium Severity Blockers:** `0`
- **Low Severity Observations (Non-Blocking):** `1`

**Gate Statement:** No High or Medium findings were identified across the 6 canonical reference artifacts. All terminology, sign conventions, error structures, verification surfaces, and capability boundaries are internally consistent and aligned with [`cross-capability-conventions.md`](../cross-capability-conventions.md).

### Low-Severity Observation
- **[LOW-01] Brownfield vs. Target Surface Delta in `ViewportComponent` and `HeaderComponent`:**
  - *Affected Artifacts:* [`canonical-api-cli-contract.md`](canonical-api-cli-contract.md) (§7), [`acceptance-scenarios.md`](acceptance-scenarios.md) (`SC-001`, `SC-005`, `SC-006`, `SC-007`, `SC-008`).
  - *Observation:* The existing Angular 22 implementation already delivers Mode 1/Mode 3 switching, `I18nService` (`el`/`en`), `DialogService`/`NotificationService`, Three.js orbit/pan/zoom, `Turbo` colormaps, deformation scaling, harmonic/transient playback, `BottomDockComponent` SVG plotting + CSV/SVG export, and Mode 3 2D tension-side SVG diagrams. The remaining target items—adding the Mode 2 (`structural`) button to `HeaderComponent`, `Orthographic` camera toggle, 3D clipping plane slider, interactive raycast boundary picking, additional palettes (`Viridis`, `Coolwarm`, `Jet`), vector glyphs, and extracting the shared 2D SVG transform helper for Mode 2—are cleanly isolated and explicitly cataloged in §7 of the contract.
  - *Disposition:* Non-blocking; ready for issue slicing alongside Mode 2 slice 1 and continuum viewport polish slices.

---

## 2. Cross-Document Consistency Audit

| Check Dimension | Status | Verification Evidence |
| --- | --- | --- |
| **Canonical Vocabulary & Ownership** | PASS | Strictly preserves `StructuralModel`, `Node`, `Element`, `Support`, `Load`, `LoadCase`, `Combination`, `ResultSet`, `Envelope`, `Diagnostics`, `ContinuumDomain`, and `ProjectDocument` ownership from `cross-capability-conventions.md`. |
| **Axes, Units & Diagram Signs** | PASS | SI units (`m`, `mm`, `MPa`, `kN`, `kNm`, `s`), global $+y$ up, and tension-side bending moment plotting ($M > 0$ when bottom/local $-y$ fibre is in tension) are invariant across PRD (`BR-4`), Domain Model (`INV-6`), Contract (§3.3), and Scenarios (`SC-012`). |
| **Bilingual & Error Conventions** | PASS | `LanguageLocale` (`el`/`en`) and `{ code: SCREAMING_SNAKE, message_el, message_en, details? }` are uniform across Use Cases (§5) and Contract (§2, §4). |
| **Product-Surface Reachability** | PASS | Every user-visible feature (header mode/language switchers, modal dialogs, 3D clipping plane, raycast boundary picking, colormaps, deformation scaling, dock scrubbing, CSV/SVG export, 2D tension-side diagrams) has explicit UI reachability in `SC-001`–`SC-012`. |
| **Code Limit Compliance** | PASS | `ViewportComponent` is currently 898 lines (close to the 1000-line ceiling). Extracting pure colormap/clipping/picking helpers and shared 2D SVG transform utilities into dedicated helper modules keeps all `.ts` files well under 1000 lines and `.html` under 500 lines. |

---

## 3. Testability & Verification Surface Alignment

All 12 acceptance scenarios (`SC-001` through `SC-012`) carry stable IDs and explicit verification surface classifications matching the root project policy:
- **`backend-boundary` (`required`):** Applicable to solver result DTO consumption (`SC-011`); `not-applicable` for pure frontend camera/dialog/viewport state scenarios.
- **`frontend-integration` (`required`):** All 12 scenarios expose deterministic TypeScript/signal entry points (`DialogService`, `NotificationService`, `I18nService`, `evaluateColormap`, `computeAutoDeformationScale`, `ViewportTransform2D`, `buildTensionSideDiagramPath`, dock path/cursor computed signals) testable under the Vitest harness without requiring a physical GPU display.
- **`end-to-end` (`catalog-only`):** Visual review gate cataloged for all 12 scenarios.

---

## 4. Residual Risks & Mitigations

1. **Risk — `viewport.component.ts` Approaching 1000-Line Limit (Currently 898 Lines):**
   - *Mitigation:* New 3D features (clipping plane math, raycast picking helper, additional colormaps, vector glyph builders) must be placed in companion pure TypeScript helper files under `src/app/components/viewport/` rather than inline in `ViewportComponent`.
2. **Risk — WebGL Buffer Memory Leaks During Rapid Transient Scrubbing or Live Re-Meshing:**
   - *Mitigation:* `renderMesh()` and `ngOnDestroy()` must explicitly call `.dispose()` on superseded `THREE.BufferGeometry` and `THREE.Material` instances when rebuilding meshes (`NFR-1`).

---

## 5. Application Synthesis Gate & Readiness Verdict

- **Application Synthesis Impact Classification:** `no impact` (explicitly verified).
  - [`docs/prd.md`](../../prd.md) and [`docs/architecture/application-architecture-summary.md`](../application-architecture-summary.md) already define the Tri-Mode topology (Mode 1 Three.js viewport + dock, Mode 2 SVG frame modeler, Mode 3 teaching studio), shared rules, and verification strategy. No root document changes are required from this node specification run.
- **Readiness Verdict:** **READY FOR `specified` STATE.** `cap-interactive-canvas` has a complete, coherent, blocker-free canonical reference set and is ready for transition from `bounded` to `specified`.
