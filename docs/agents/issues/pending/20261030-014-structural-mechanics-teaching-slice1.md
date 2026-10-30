## Issue Metadata

- Issue number: `014`
- Owning capability node: `/.okf/capabilities/cap-structural-mechanics-teaching.md`
- Artifact root: `docs/architecture/cap-structural-mechanics-teaching/`
- Issue file: `docs/agents/issues/pending/20261030-014-structural-mechanics-teaching-slice1.md`
- Category: `feature`
- Execution type: `AFK`
- Review gate: `visual-review`
- Suggested state: `pending`

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
  - *Three-Hinged Frame (Τριαρθρωτό Πλαίσιο)*

### 2. Tauri IPC Integration (`src-tauri/src/`):
- Tauri command: `solve_teaching_system(config: TeachingInputConfig) -> Result<TeachingSolutionDto, String>`.
- Expose DTOs with full step list, member curves, reaction arrows, and diagram coordinate samples.

### 3. Angular 22 Frontend (`src/app/features/teaching/`):
- **Teaching Mode Workspace:**
  - Header / Tab Mode Switcher: Mode 1 (Continuum Workbench) vs Mode 3 (Teaching Studio).
  - Presets & Parameter Inspector (Signal Forms with schema validation):
    - Preset selector (Beams, Overhangs, Trusses).
    - Dimension inputs ($L, H$), load inputs ($q, P, M$), support types.
- **Pedagogical Step-Walker Component (`step-walker.component`):**
  - Timeline controls: `[ ⏮ First ]`, `[ ◀ Prev ]`, `[ Next ▶ ]`, `[ Last ⏭ ]`, `[ ▶ Auto-Play ]`.
  - Progress indicator (e.g. *"Step 3 of 7: Equilibrium of Joint B"*).
  - Interactive Step Card:
    - Rationale & engineering principle.
    - Symbolic equation (rendered with KaTeX math).
    - Exact numerical substitution and result ($N_{AB} = +14.14\text{ kN}$ [Tension]).
- **Synchronized Canvas Viewport:**
  - Visual highlighting of the active joint, cut, or beam interval.
  - Progressive plotting of the $M(x), Q(x), N(x)$ diagrams with extreme values and roots.

## Acceptance criteria

- [ ] Rust analytical teaching solver implemented with unit tests in separate companion test files (`*_test.rs`).
- [ ] Exact rational arithmetic module formatting clean LaTeX fractions.
- [ ] Method of Joints and Ritter cuts fully verified analytically.
- [ ] Piecewise $M(x), Q(x), N(x)$ internal force equations and $M_{\max}$ detection verified.
- [ ] Tauri IPC command `solve_teaching_system` registered and functional.
- [ ] Angular Teaching Studio UI built with Signal Forms and OnPush change detection.
- [ ] Pedagogical Step-Walker interactive timeline step-by-step navigation functional with KaTeX equations.
- [ ] All code files strictly obey limits: Rust/TS $\le 1000$ lines, HTML $\le 500$ lines.
- [ ] `npm run check:limits` and `cargo test` pass cleanly with 0 errors.

## Artifact sync required

- Application PRD: `none - initial Mode 3 vertical slice`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-structural-mechanics-teaching.md`
- Issue registry: `required: docs/agents/issues/issues.md`
- Reason/no-impact decision: `Delivers Vertical Slice 1 of Mode 3 Educational Structural Mechanics Studio.`
