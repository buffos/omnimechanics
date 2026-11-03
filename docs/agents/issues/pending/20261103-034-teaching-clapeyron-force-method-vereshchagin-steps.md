## Issue Metadata

- Issue number: `034`
- Owning capability node: `/.okf/capabilities/cap-structural-mechanics-teaching.md`
- Artifact root: `.okf/capabilities/cap-structural-mechanics-teaching.md`
- Issue file: `docs/agents/issues/pending/20261103-034-teaching-clapeyron-force-method-vereshchagin-steps.md`
- Category: `feature`
- Execution type: `AFK`
- Review gate: `visual-review`
- Suggested state: `ready-for-agent`

## Parent Artifacts

- `docs/prd.md`
- `docs/architecture/application-architecture-summary.md`
- `docs/architecture/cross-capability-conventions.md`
- `.okf/capabilities/cap-structural-mechanics-teaching.md`

## What to build

Implement **Vertical Slice 2** of `cap-structural-mechanics-teaching`: analytical step-by-step derivations for the **Clapeyron Three-Moment Equation** (`Scenario 4`, `PedagogicalPhase::ClapeyronEquations`) on multi-span continuous beams and the **Force Method / Flexibility Method** (`Μέθοδος των Δυνάμεων`, `PedagogicalPhase::ForceMethodFlexibilityMatrix`) for hyperstatic beams and frames of static indeterminacy degree $1 \le n \le 3$ using exact Vereshchagin / Simpson $\int \frac{M_i M_k}{EI} dx$ integration, wired into both Mode 3 presets and the Mode 2 $\to$ Mode 3 step-by-step handoff (`teaching_open_step_by_step`).

### 1. Clapeyron Three-Moment Equation Solver (`src-tauri/src/teaching/clapeyron.rs`):
- Formulate the three-moment equation $M_{i-1}\frac{L_i}{I_i} + 2 M_i \left(\frac{L_i}{I_i} + \frac{L_{i+1}}{I_{i+1}}\right) + M_{i+1}\frac{L_{i+1}}{I_{i+1}} = -6\left(\frac{A_i a_i}{L_i I_i} + \frac{A_{i+1} b_{i+1}}{L_{i+1} I_{i+1}}\right) + 6 E \Delta_{\text{settlement}}$ at each internal support (`Scenario 4`).
- Emit explicit `PedagogicalStep` cards showing the isostatic span moment areas $A_i$, centroid distances $a_i, b_i$, the assembled tridiagonal system, solved support moments $M_B, M_C, \dots$ (with `ExactFraction` where rational), and final superposed $M(x), Q(x)$ diagrams.

### 2. Force Method (Flexibility) Step-by-Step Solver (`src-tauri/src/teaching/force_method.rs`):
- Compute static indeterminacy degree $n$ ($1 \le n \le 3$), release redundant restraints to form the primary isostatic system ($M_0, Q_0, N_0$), and construct unit virtual systems ($M_1, \dots, M_n$) for redundant forces/moments $X_1, \dots, X_n$.
- Evaluate flexibility coefficients $\delta_{ik} = \sum \int \frac{M_i M_k}{EI} dx$ and load terms $\delta_{i0} = \sum \int \frac{M_i M_0}{EI} dx$ with explicit Vereshchagin shape-area $\times$ centroid-ordinate step tables.
- Solve compatibility equations $[\delta]\{X\} + \{\delta_0\} = \{0\}$, superpose $M = M_0 + \sum X_i M_i$, and emit interactive diagram overlays (`M_0`, `M_1..M_n`, final `M, Q, N`) in Mode 3's `TeachingStudioComponent` and `teaching_open_step_by_step`.

## Acceptance criteria

- [ ] Selecting "Clapeyron Method" on a 3-span continuous beam ($L_1=4\text{ m}, L_2=6\text{ m}, L_3=4\text{ m}$) derives each three-moment equation with explicit $6 A_i a_i / L_i$ load terms, solves for internal support moments $M_B, M_C$, and plots the superposed continuous $M(x)$ and $Q(x)$ diagrams (`Scenario 4`).
- [ ] Selecting "Force Method (Μέθοδος των Δυνάμεων)" on a hyperstatic beam or frame ($1 \le n \le 3$) displays the primary isostatic system $M_0$, unit systems $M_1 \dots M_n$, Vereshchagin integration tables for $\delta_{ik}$ and $\delta_{i0}$, solved redundants $X_i$, and final superposed internal force diagrams.
- [ ] Mode 2 sketches classified as eligible for `ContinuousBeamClapeyronCross` or `ForceMethod` open directly into these analytical step-by-step derivations via `teaching_open_step_by_step`.
- [ ] Rust unit tests (`clapeyron_test.rs`, `force_method_test.rs`), `npx vitest run`, `npx ng build`, and `npm run check:limits` pass with 0 errors.

## Artifact sync required

- Application PRD: `none - already covered in docs/prd.md`
- Application architecture summary: `none - matches docs/architecture/application-architecture-summary.md`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-structural-mechanics-teaching.md`
- Issue registry: `required`; node `issues:` reference: `required: .okf/capabilities/cap-structural-mechanics-teaching.md`
- Reason/no-impact decision: `Delivers Clapeyron and Force Method step-by-step pedagogical solvers in Mode 3.`

## Human review gate

`visual-review`: User opens Mode 3 Teaching Studio (and hands off a continuous beam / hyperstatic frame from Mode 2), steps through the Clapeyron Three-Moment derivation and the Force Method $M_0 / M_1$ Vereshchagin tables, and inspects the rendered diagrams.

## Blocked by

- None - can start immediately

## Artifact anchors

- Capability spec (`.okf/capabilities/cap-structural-mechanics-teaching.md`): Scope §2 (Force Method & Clapeyron Three-Moment Equation), `PedagogicalPhase::ForceMethodFlexibilityMatrix`, `PedagogicalPhase::ClapeyronEquations`

## Acceptance scenarios addressed

- `Scenario 4`: Continuous Beam via Clapeyron Three-Moment Equation (plus Force Method compatibility & Vereshchagin integration)

## Verification obligations

- Policy source: `/.okf/project.md`

| Scenario | Backend boundary | Frontend integration | End-to-end journey |
|---|---|---|---|
| `Scenario 4` | `planned` | `planned` | `catalog-only` |
