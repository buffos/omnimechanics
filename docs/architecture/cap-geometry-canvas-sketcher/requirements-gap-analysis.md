# Requirements Gap Analysis: cap-geometry-canvas-sketcher

- Recorded: 2026-10-31, during `$process-reference-issue 015` pre-implementation checks
- Owning node: `.okf/capabilities/cap-geometry-canvas-sketcher.md` (leaf, `state: in-progress`)
- Issue: `docs/agents/issues/pending/20261031-015-interactive-canvas-sketcher.md`
- Status: **routed (2026-10-31).** Issue 015 is superseded. Resolution:
  - G1: application synthesis drafted (`docs/prd.md`, `docs/architecture/application-architecture-summary.md`).
  - G2: new `cap-frame-analysis-engine` (general DSM) decided.
  - G3: the structural model is owned by the frame engine (Q35/Q36).
  - G4: verification block normalised, a Vitest harness decided, continuum scenario moved to
    `cap-sketcher-continuum-domain`.
  - See [discovery-notes.md](discovery-notes.md).

## G1 (High): Missing reference artifacts

- `docs/prd.md` does not exist.
- `docs/architecture/application-architecture-summary.md` does not exist.
- The issue's artifact root `docs/architecture/cap-geometry-canvas-sketcher/` held no capability PRD,
  domain model, use-case model, contract, acceptance scenarios, or readiness review. The OKF node's
  inline "Architecture & Domain Model" and "Acceptance Scenarios" sections are the only source.
- Route: `$fog-of-war-planning` application synthesis gate, or an explicit user waiver that accepts the
  OKF node plus this issue as the reference set (the same basis issue 014 used).

## G2 (High): The teaching engine cannot solve arbitrary sketched frames

Issue criterion: "`Analyze in Teaching Studio` transfers the custom structure into Mode 3 with instant
analytical solution and step-by-step derivations."

The current `src-tauri/src/teaching/` solvers cover only:

| Solver | Coverage |
| --- | --- |
| `isostatic_beam::solve_isostatic_beam_system` | One straight horizontal beam (1D `x` positions), isostatic supports |
| `truss::solve_teaching_truss` | Pin-jointed trusses with nodal loads (joints and Ritter) |
| `cross_kani::solve_continuous_beam_cross` | Collinear continuous beams (moment distribution) |

No solver handles general 2D frames. The issue's own "Portal Frame" sample preset and OKF Scenarios 1 and 2
(fixed-base portal frame, statically indeterminate to degree 3) therefore have no solving path.
Product decision needed. Options:

1. **Classify and route:** handoff recognises collinear beams, continuous beams, and trusses, dispatches to
   the existing solvers, and rejects other topologies with a diagnostic (degree of indeterminacy, reason).
   The portal-frame preset becomes "draw only" or is replaced.
2. **Add an isostatic frame solver:** pedagogical equilibrium for statically determinate frames
   (e.g. three-hinged portal, simply supported L-frame). Hyperstatic frames are still rejected.
3. **Add a direct stiffness solver:** numeric solution with M, Q, N diagrams for any stable frame,
   including hyperstatic ones, with fewer pedagogical derivation steps.

## G3 (Medium): Two divergent sketch domain models

| Aspect | OKF node | Issue 015 |
| --- | --- | --- |
| Node coordinates | `point: [f64; 3]` | `(x, y)` |
| Supports | `Free, Pinned, RollerX, RollerY, Fixed, Spring(f64)` | No `Spring` |
| Loads | `NodalForce`, `NodalMoment`, `DistributedLineLoad{perpendicular}` | `PointLoad` on node **or member at distance x**, `PointMoment`, `DistributedLoad{direction}` |
| Continuum | `boundaries: Vec<SketchBoundaryLoop>` | Not included |
| Internal hinge | Listed in the support palette (node) | Member end releases `start_hinge` / `end_hinge` |

Also unspecified: the truss and beam member distinction (both ends released?), units and sign
conventions (kN, kN/m, positive `q` direction), and how a node-level "Internal Hinge" maps onto member
releases. Proposed default: use the issue's 2D model, keep `boundaries` for a later slice, and implement
internal hinges as releases on every member end at the node.

## G4 (Medium): Scenario coverage and verification policy

- The root `verification.policy` is `required`.
- Issue behaviours with no acceptance scenario: invalid topology (orphan node, isolated member,
  mechanism), handoff failure presentation, load-dialog validation, select/move/delete, sample presets,
  and clear/reset.
- OKF Scenario 3 (polygonal plate with hole and CDT mesh) is outside this slice. It needs a named
  deferral owner (a future slice issue).
- The frontend has no test harness (0 `*.spec.ts`, no `test` script). Under `required`, snapping, ortho,
  and the sketch-model serialization in the UI cannot have executed evidence. Either add a harness
  (e.g. Vitest for the pure snapping and geometry helpers) or record a policy exception.

## Working trace (provisional)

| Source rule | Scenario | Issue criterion | Planned verification |
| --- | --- | --- | --- |
| Sketch data model + JSON | gap (G4) | AC1 | Rust `sketcher/types_test.rs` |
| Topology validation / indeterminacy | gap (G4) | AC2 | Rust `sketcher/handoff_test.rs` |
| Ortho + magnetic snap | OKF Scenario 1 | AC4 | needs frontend harness (G4) + visual review |
| Support assignment | OKF Scenario 2 | AC5 | handoff test + visual review |
| Load assignment | OKF Scenario 2 | AC6 | handoff test + visual review |
| Handoff to Mode 3 | OKF Scenario 2.3 | AC7 | blocked by G2 |
| Mode 2 header switch | gap | AC3 | visual review |
| Closed polygon + CDT mesh | OKF Scenario 3 | none | deferred: owner TBD |
