# Architecture Readiness Review: Continuum Domain Sketcher (`cap-sketcher-continuum-domain`)

- **Owning Capability:** [`cap-sketcher-continuum-domain`](../../../.okf/capabilities/cap-sketcher-continuum-domain.md)
- **Reviewed Artifacts:**
  - [`domain-glossary.md`](domain-glossary.md)
  - [`prd.md`](prd.md)
  - [`canonical-domain-model.md`](canonical-domain-model.md)
  - [`canonical-use-cases.md`](canonical-use-cases.md)
  - [`canonical-api-cli-contract.md`](canonical-api-cli-contract.md)
  - [`acceptance-scenarios.md`](acceptance-scenarios.md)
- **Root Synthesis Gate References:** [`docs/prd.md`](../../prd.md), [`application-architecture-summary.md`](../application-architecture-summary.md), [`cross-capability-conventions.md`](../cross-capability-conventions.md)
- **Review Date:** 2026-11-01
- **Gate Verdict:** **READY FOR `specified` STATE (0 High / 0 Medium Blockers)**

---

## 1. Findings-First Audit

**No High or Medium severity blockers were found across the 6 canonical reference artifacts.**

| Severity | Finding | Affected Artifacts | Resolution / Status |
| --- | --- | --- | --- |
| **None (0 High / 0 Medium)** | All cross-document consistency, domain invariant, contract parity, and verification surface checks passed. | All 6 reference artifacts | Clean pass. |
| **Low (Informational)** | Custom semantic boundary tags (`Custom(String)`) beyond the 5 standard preset tags (`Left`, `Right`, `Top`, `Bottom`, `Hole`) require either mapping to standard tags or passing `tag_conditions` when `cap-solver-core` assembles custom BCs. | `prd.md` §8, `canonical-api-cli-contract.md` §3.1 | Explicitly documented: outer rectangle/polygon edges auto-assign `Bottom`, `Right`, `Top`, `Left` and cutouts default to `Hole` so existing `cap-solver-core` solvers work immediately out-of-the-box, while `tag_conditions` preserves full Dirichlet/Neumann/Robin metadata. |

---

## 2. Cross-Document Consistency Verification

1. **Vocabulary & Ownership Alignment (`cross-capability-conventions.md`):**
   - `ContinuumDomain` is strictly separated from `StructuralModel` (`cap-frame-analysis-engine`).
   - Sketch vertices use `Vertex2D` (`V1`, `V2`, ...) rather than `Node` (`N1`), eliminating any collision with Mode 2 frame nodes or `MeshData.nodes` index arrays.
   - Tauri IPC commands strictly follow the `continuum_` prefix (`continuum_validate_domain`, `continuum_preview_segment`, `continuum_close_and_normalize_loop`, `continuum_generate_mesh`).
   - Error payloads strictly follow `{ code: SCREAMING_SNAKE, message_el, message_en, details? }`.
2. **Mathematical & Topological Invariant Alignment:**
   - Outer boundary CCW winding ($A > 0$) and interior cutout CW winding ($A < 0$) ensure that the outward unit normal formula $\mathbf{n} = [\Delta y / \ell, -\Delta x / \ell, 0.0]$ points away from the solid continuum material on both exterior boundaries and interior cavities, matching `generate_plate_with_hole_2d` in `src-tauri/src/mesh/generators_2d.rs`.
   - Minimum edge length ($\ell_{\min} = 10^{-4}\text{ m}$), minimum area ($A_{\min} = 10^{-6}\text{ m}^2$), and minimum feature clearance ($d_{\min} = \max(10^{-3}\text{ m}, 0.1 h \alpha_{\text{hole}})$) are identical across `prd.md`, `canonical-domain-model.md`, `canonical-use-cases.md`, and `acceptance-scenarios.md`.
3. **Solver & Preset Compatibility (`cap-geometry-presets` & `cap-solver-core`):**
   - `ConformalMeshHandoff.mesh_data` emits the exact `crate::mesh::types::MeshData` structure (`dimension: 2`, `nodes: Vec<[f64; 3]>`, `boundary_elements: Vec<BoundaryElement>`, `volume_elements: Vec<VolumeElement>`, `stats: MeshStats`) consumed by `SimulationRequest`, `HarmonicSimulationRequest`, and `TransientSimulationRequest`.
4. **Traceability & Verification Surface Completeness:**
   - All 9 acceptance scenarios (`SC-001` through `SC-009`) declare explicit `backend-boundary`, `frontend-integration`, and `end-to-end` verification surfaces aligned with the root project policy (`backend-boundary: required`, `frontend-integration: required`, `end-to-end: catalog-only`).
   - The carried-over acceptance seed ($10 \times 10\text{ m}$ square with $R = 1.5\text{ m}$ central circular hole) is directly covered in `SC-001` and chained into the Mode 1 BEM-FEM solver in `SC-008`.

---

## 3. Residual Risks & Implementation Guidance

1. **Delaunay Refinement Termination Near Sharp Input Angles:**
   - *Risk:* Standard Ruppert refinement can ping-pong when two user-drawn polygon segments meet at an interior angle $< 20^\circ$.
   - *Mitigation:* `ConformalDelaunayMesherService` must apply concentric circular-shell splitting (collar protection) around input vertices with interior angles $< 30^\circ$ and enforce a hard cap of $50{,}000$ Steiner nodes before returning `CDT_CONVERGENCE_FAILURE`.
2. **Circular Cutout Boundary Node Snapping:**
   - *Risk:* Naive linear subdivision of chord segments on a `CircularCutout` leaves mid-nodes slightly inside the true radius $R$.
   - *Mitigation:* `PslgBuilderService` must project any subdivided vertices on a `CircularCutout` perimeter onto the exact circle $(c_x + R\cos\theta, c_y + R\sin\theta)$ prior to CDT triangulation (`SC-001`).

---

## 4. State Policy & Application Synthesis Gate

- **Node State Policy:** `mode: own` (leaf capability under roll-up parent `cap-geometry-canvas-sketcher`).
- **Recommended Capability State Transition:** `bounded` $\to$ `specified`.
- **Application Synthesis Impact:** `no impact`
  - Checked [`docs/prd.md`](../../prd.md) and [`docs/architecture/application-architecture-summary.md`](../application-architecture-summary.md). Both documents already accurately reflect `cap-sketcher-continuum-domain` as the 2D continuum polygon/hole/CDT sketcher feeding Mode 1 solvers. No product truth or cross-capability boundary changed.
