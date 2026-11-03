## Issue Metadata

- Issue number: `024`
- Owning capability node: `/.okf/capabilities/cap-sketcher-frame-modeler.md`
- Artifact root: `docs/architecture/cap-sketcher-frame-modeler/`
- Issue file: `docs/agents/issues/pending/20261102-024-palettes-inspector-live-linear-dsm-diagrams-mechanisms.md`
- Category: `feature`
- Execution type: `AFK`
- Review gate: `visual-review`
- Suggested state: `done`

## Parent Artifacts

- `docs/prd.md`
- `docs/architecture/application-architecture-summary.md`
- `docs/architecture/cross-capability-conventions.md`
- `.okf/capabilities/cap-sketcher-frame-modeler.md`
- `docs/architecture/cap-sketcher-frame-modeler/prd.md`
- `docs/architecture/cap-sketcher-frame-modeler/canonical-domain-model.md`
- `docs/architecture/cap-sketcher-frame-modeler/canonical-use-cases.md`
- `docs/architecture/cap-sketcher-frame-modeler/canonical-api-cli-contract.md`
- `docs/architecture/cap-sketcher-frame-modeler/acceptance-scenarios.md`
- `docs/architecture/cap-sketcher-frame-modeler/readiness-review.md`

## What to build

Implement **Vertical Slice 2** of `cap-sketcher-frame-modeler`: the Property Inspector (sections, materials, end releases, node-level internal hinge $k-1$ expansion, rigid offsets, cable parameters), full Support & Load Palettes, debounced (~$150\text{ ms}$) live Linear Static DSM coordination (`frame_solve_linear`), tension-side $M/Q/N$ diagram & deformed shape overlays, cross-section hover probe, member end-force table, and animated kinematic mechanism display when a structure is unstable.

### 1. Property Inspector & Internal Hinge Expansion (`src/app/features/modeler/inspector/`):
- Edit `Section` and `Material` from built-in catalogs (`frame_get_catalog`) or custom $(E\text{ in GPa}, A, I, h)$ inputs (`BR-1`), per-end `EndRelease` (`axial`, `shear`, `moment`), per-end `RigidOffset` $(d_x, d_y)$, `TrussBar` axial behavior, `Spring` stiffnesses, and `CableInputSpec` (`UnstressedLength`, `SagRatio`, `Pretension`) (`UC-C03`, `SC-005`).
- Implement `InternalHingeExpansionPolicy`: clicking a node shared by $k \ge 2$ `FrameMember`s sets `EndRelease.moment = true` on $k - 1$ incident member ends (keeping 1 continuous to prevent rotational singularity) and renders a nodal hinge ring; reject $k < 2$ with `INVALID_HINGE_TARGET` (`SC-005`).

### 2. Full Support & Load Palettes (`src/app/features/modeler/palettes/`):
- **Support Palette:** Assign `Fixed`, `Pinned`, inclined `Roller { angle_deg }`, `Guided { angle_deg }`, and `Elastic { kx, ky, ktheta }` with optional `unilateral` compression-only toggle (`UC-C04`, `SC-006`).
- **Load Palette:** Assign `NodalLoad` $(F_x, F_y, M_z)$, `MemberPointLoad` at arbitrary distance $a \in [0, L]$ without splitting the member, uniform/trapezoidal `MemberDistributedLoad` $(q_1, q_2, a, b)$ in `Local`, `Global`, or `Projected` frames, `SupportSettlement` $(\Delta_x, \Delta_y, \Delta_\theta)$, `ThermalLoad` $(\Delta T_{\text{uniform}}, \Delta T_{\text{gradient}})$, and `LoadCase.self_weight` toggle; reject $a < 0$ or $a > L$ with `LOAD_OUT_OF_MEMBER_BOUNDS` (`SC-006`).
- Render crisp SVG glyphs for all supports, releases, rigid offsets, and loads on the Mode 2 canvas.

### 3. Live Linear DSM Coordinator, Tension-Side Diagrams & Mechanism Animation (`src/app/features/modeler/results/`):
- Debounce `StructuralModel.revision` mutations by $150\text{ ms}$ (`UC-C06`) and invoke `frame_solve_linear`.
- Plot support reaction arrows, scaled deformed shape, and $M(x), Q(x), N(x)$ diagrams on the SVG canvas with **bending moment $M(x)$ always plotted on the tension fibre side** (Greek engineering convention, `BR-2`) and extrema labels (`SC-007`).
- Implement `CrossSectionHoverProbe` (`UC-Q02`) showing live $N(x), Q(x), M(x)$, tension side, and local displacements along any hovered member, plus the collapsible Member End-Force Table (`UC-Q03`) (`SC-007`).
- When `Diagnostics` reports a kinematic mechanism (`is_stable == false` / `KINEMATIC_INSTABILITY`), suppress invalid force diagrams, display the bilingual static indeterminacy & mechanism diagnostic banner, and smoothly animate the structure oscillating along `mechanism_modes[0]` (`SC-009`).

## Acceptance criteria

- [x] Property Inspector edits catalog/custom sections & materials ($E$ in $\text{GPa}$ converted at boundary), `EndRelease`, `RigidOffset`, and `CableInputSpec` (rejecting invalid cable parameters with `INVALID_CABLE_SPEC`), while `PlaceInternalHinge` expands a $k$-member joint into $k - 1$ moment releases and rejects $k < 2$ with `INVALID_HINGE_TARGET` (`SC-005`).
- [x] Support & Load Palettes place all support types (including inclined/unilateral rollers and elastic springs) and all load types (including interior point/trapezoidal loads without splitting the member, thermal loads, settlements, and self-weight), rejecting out-of-bounds distances with `LOAD_OUT_OF_MEMBER_BOUNDS` (`SC-006`).
- [x] Within ~$150\text{ ms}$ of any edit, reactions, scaled deformed shape, tension-side $M/Q/N$ diagrams ($M > 0$ plotted on bottom tension fibre), extrema labels, `CrossSectionHoverProbe`, and Member End-Force Table update live on the canvas (`SC-007`).
- [x] Kinematically unstable structures hide force diagrams, show the bilingual diagnostic banner (with indeterminacy degree $n_s$ and instability degree $n_k$), and animate the mechanism mode on the canvas (`SC-009`).
- [x] Vitest tests (`npx vitest run`), `cargo test`, `npm run check:limits`, and `npx ng build` pass with 0 errors.

## Artifact sync required

- Application PRD: `none - already covered in docs/prd.md`
- Application architecture summary: `none - matches docs/architecture/application-architecture-summary.md`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-sketcher-frame-modeler.md`
- Issue registry: `required`; node `issues:` reference: `required: .okf/capabilities/cap-sketcher-frame-modeler.md`
- Reason/no-impact decision: `Completes the full Mode 2 Delivery Slice 1 visual journey (drafting + supports/loads + live linear DSM diagrams + mechanism animation).`

## Human review gate

`visual-review`: User sketches beams, frames, and trusses in Mode 2, assigns supports, hinges, and loads, verifies live tension-side $M/Q/N$ diagrams, deformed shape, hover probe, end-force table, and observes the animated mechanism mode when a support is removed.

## Blocked by

- Blocked by `docs/agents/issues/pending/20261102-023-mode2-drafting-canvas-snapping-splits-transforms.md`

## Artifact anchors

- PRD: Goals 3, 4 & 5, `WF-2`, `WF-3`, `WF-4`, `BR-1`, `BR-2`, `NFR-2`
- Domain model: `InternalHingeExpansionPolicy`, `ResultOverlayState`, `CrossSectionHoverProbe`, `INV-3`, `INV-4`, `INV-6`
- Use cases: `UC-C03 ConfigureElementPropertiesAndHinges`, `UC-C04 AssignSupportOrLoad`, `UC-C06 CoordinateLiveAnalysis`, `UC-Q02 ProbeMemberCrossSection`, `UC-Q03 GetMemberEndForceTable`
- Contract: `CrossSectionHoverProbeDto`, `ModelerStore.applyInternalHinge`, `ModelerStore.updateElementProperties`, `ModelerStore.assignSupportOrLoad`, `frame_solve_linear`, error codes `INVALID_HINGE_TARGET`, `LOAD_OUT_OF_MEMBER_BOUNDS`, `INVALID_CABLE_SPEC`, `KINEMATIC_MECHANISM_DETECTED`

## Acceptance scenarios addressed

- `SC-005`: Property Inspector, Internal Hinge Expansion, Rigid Offsets & Cable Input Modes
- `SC-006`: Full Support & Load Palettes Without Spurious Member Splitting
- `SC-007`: Live Real-Time Linear DSM Feedback, Tension-Side M/Q/N Diagrams & Hover Probe
- `SC-009`: Kinematic Mechanism Detection & Animated Instability Mode

## Verification obligations

- Policy source: `/.okf/project.md`

| Scenario | Backend boundary | Frontend integration | End-to-end journey |
|---|---|---|---|
| `SC-005` | `implemented: src-tauri/src/frame/` | `planned` | `planned` |
| `SC-006` | `implemented: src-tauri/src/frame/` | `planned` | `planned` |
| `SC-007` | `implemented: src-tauri/src/frame/` | `planned` | `planned` |
| `SC-009` | `implemented: src-tauri/src/frame/` | `planned` | `planned` |
