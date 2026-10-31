## Issue Metadata

- Issue number: `014`
- Owning capability node: `/.okf/capabilities/cap-structural-mechanics-teaching.md`
- Artifact root: `docs/architecture/cap-structural-mechanics-teaching/`
- Issue file: `docs/agents/issues/done/20261030-014-structural-mechanics-teaching-slice1.md`
- Category: `feature`
- Execution type: `AFK`
- Review gate: `visual-review (passed & approved by user)`
- State: `done`

## Parent Artifacts

- `.okf/project.md`
- `.okf/capabilities/cap-structural-mechanics-teaching.md`
- `.okf/capabilities/cap-interactive-canvas.md`
- `docs/roadmap.md`

## What to build

Implement **Vertical Slice 1** of Mode 3 (Structural Mechanics Step-by-Step Educational Studio), delivering end-to-end analytical solutions, exact rational math derivations, an interactive step-walker timeline, and real-time internal force diagrams ($M, Q, N$):

### 1. Rust Domain Model & Solver (`src-tauri/src/teaching/`):
- **Core Entities & Types (`types.rs`):**
  - Structural system representation: Nodes with coordinates $(x, y)$, support conditions (Pinned, Roller, Clamped), beam/truss members with $(A, I, E)$, point loads $(P_x, P_y)$, moments $(M_z)$, and distributed loads ($q$).
  - `PedagogicalStep`: step index, phase, title, rationale, active entity ID, active diagram ($M, Q, N$), symbolic LaTeX equation, numerical substitution, and solved quantities.
  - `ExactFraction`: Rational arithmetic struct $(p / q)$ with exact multiplication, division, addition, GCD reduction, and formatting to LaTeX fractions $\frac{a}{b}$.
- **Analytical Isostatic Beam & Frame Solver (`isostatic_beam.rs`):**
  - Reaction equilibrium solver ($\Sigma F_x = 0, \Sigma F_y = 0, \Sigma M = 0$) generating step-by-step reaction derivations.
  - Piecewise continuous function builder for internal forces $N(x), Q(x), M(x)$.
  - Analytical detector for shear roots $Q(x_0)=0 \Rightarrow M_{\max} = M(x_0)$ and inflection points $M(x)=0$.
- **Analytical Truss Solver (`truss.rs`):**
  - Method of Joints (Μέθοδος Κόμβων): automatic sequence search for joints with $\le 2$ unknowns, isolated joint diagrams, and $\Sigma F_x=0, \Sigma F_y=0$ equations.
  - Method of Sections (Μέθοδος Ritter): selection of 3 cut bars, isolated Free-Body Diagram, and moment equilibrium about the Ritter pole.
- **Teaching Presets (`presets.rs`):**
  - *Cantilever with Point & Distributed Load*
  - *Simply Supported Beam with Midspan Load*
  - *Beam with Overhang (Πρόβολος)*
  - *Classic 2D Pratt Truss*
  - *Classic 2D Howe Truss*
  - *Continuous 2-Span Beam (Cross & Kani)*
- **Moment Distribution & Kani Engine (`cross_kani.rs`):**
  - Member stiffness $k$, distribution factors $DF$, fixed-end moments $FEM$.
  - Multi-cycle distribution and carry-over ($COF=0.5$) table generation.

### 2. Tauri IPC Integration (`src-tauri/src/`):
- Tauri command: `solve_teaching_system(config: TeachingInputConfig) -> Result<TeachingSolutionDto, String>`.
- Expose DTOs with full step list, member curves, reaction arrows, diagram samples, and Cross tables.

### 3. Angular 22 Frontend (`src/app/features/teaching/`):
- **Teaching Mode Workspace:**
  - Header Mode Switcher: Mode 1 (Continuum Workbench) vs Mode 3 (Teaching Studio).
  - Presets & Parameter Inspector (Signal Forms with schema validation):
    - Preset selector (Beams, Overhangs, Trusses, Continuous Beams).
    - Dimension inputs ($L, H$), load inputs ($q, P$), support types.
- **Pedagogical Step-Walker Component (`step-walker.component`):**
  - Timeline controls: `[ ⏮ First ]`, `[ ◀ Prev ]`, `[ Next ▶ ]`, `[ Last ⏭ ]`, `[ ▶ Auto-Play ]`.
  - Progress indicator (e.g. *"Step 3 of 7: Equilibrium of Joint B"*).
  - Interactive Step Card:
    - Phase badge, rationale & engineering principle.
    - Symbolic equation and numerical substitution.
    - Result banner with exact LaTeX output and solved quantities chips.
- **Synchronized Canvas Viewport (`teaching-canvas.component`):**
  - Visual highlighting of the active joint, cut, or beam interval.
  - Progressive plotting of the $M(x), Q(x), N(x)$ diagrams with extreme values ($M_{\max}$) and roots.
  - Interactive Cross iteration table display.

## Acceptance criteria

- [x] Rust analytical teaching solver implemented with unit tests in separate companion test files (`*_test.rs`).
- [x] Exact rational arithmetic module formatting clean LaTeX fractions.
- [x] Method of Joints and Ritter cuts fully verified analytically.
- [x] Piecewise $M(x), Q(x), N(x)$ internal force equations and $M_{\max}$ detection verified.
- [x] Tauri IPC command `solve_teaching_system` registered and functional.
- [x] Angular Teaching Studio UI built with Signal Forms and OnPush change detection.
- [x] Pedagogical Step-Walker interactive timeline step-by-step navigation functional with KaTeX equations.
- [x] All code files strictly obey limits: Rust/TS $\le 1000$ lines, HTML $\le 500$ lines.
- [x] `npm run check:limits` and `cargo test` pass cleanly with 0 errors.

## Artifact sync required

- Application PRD: `none - initial Mode 3 vertical slice`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-structural-mechanics-teaching.md`
- Issue registry: `required: docs/agents/issues/issues.md`
- Reason/no-impact decision: `Delivers Vertical Slice 1 of Mode 3 Educational Structural Mechanics Studio.`

## Verification & Delivery Summary

- **Automated Verification:**
  - `cargo test --lib -- teaching`: 11/11 tests pass (isostatic beams, cantilevers, Pratt truss joints/ritter, Cross continuous beam, rational fractions).
  - `npm run check:limits`: All 131 files within strict line limits (TS/Rust <= 1000 lines, HTML <= 500 lines).
  - `npx ng build`: 0 errors.
- **Visual & Interactive Human Review:**
  - Point load position slider $x_P$ bounds clamped to beam length.
  - Applied loads ($P$, $q$) visibly rendered with vectors and values.
  - Shear force diagram right boundaries strictly vertical.
  - Segmented control styling restored for all solver domain buttons.
  - Pratt truss diagram centered with compact 12px envelope.
  - Pinned structural SVG preview layout with independent scrollable derivation area and sticky stepper toolbar.
  - Continuous beam (Cross method) complete with reactions and $M, Q, N$ diagrams.
  - KaTeX LaTeX rendering verified with unescaped ampersand escaping.
