## Issue Metadata

- Issue number: `035`
- Owning capability node: `/.okf/capabilities/cap-structural-mechanics-teaching.md`
- Artifact root: `.okf/capabilities/cap-structural-mechanics-teaching.md`
- Issue file: `docs/agents/issues/pending/20261103-035-teaching-virtual-work-deflections-lualatex-handout-export.md`
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

Implement **Vertical Slice 3** of `cap-structural-mechanics-teaching`: step-by-step deflection and rotation computation via the **Principle of Virtual Work (Αρχή Δυνατών Έργων — ΑΔΕ / Unit Load Method)** and **Castigliano's Theorem** (`Scenario 7`, `PedagogicalPhase::VirtualWorkDeflection`), interactive **Cross & Kani Cycle Spreadsheets** (`Scenario 5`, `Scenario 6`), and one-click **LuaLaTeX / Printable Handout & Exam Sheet Export** (`Scenario 8`).

### 1. Principle of Virtual Work (ΑΔΕ) & Castigliano Deflection Solver (`src-tauri/src/teaching/virtual_work.rs`):
- Allow the user to query vertical/horizontal deflection $\Delta_C$ or rotation $\theta_C$ at any node or span point of a solved beam, frame, or truss (`Scenario 7`).
- Construct the real internal force state ($M_0(x), N_0(x)$) and the virtual unit-load state ($\bar{M}(x), \bar{N}(x)$ under $\bar{P}=1$ or $\bar{M}=1$ at target point $C$).
- Generate step cards displaying the virtual system FBD, the Vereshchagin integration table (shape areas, centroid ordinates $\bar{y}_G$, and exact rational fraction products $\frac{1}{EI}\int M_0 \bar{M} dx$ or truss $\sum \frac{N_0 \bar{N} L}{EA}$), and Castigliano strain-energy derivative $\frac{\partial U}{\partial P}$.

### 2. Interactive Cross & Kani Iteration Table View & LuaLaTeX Handout Exporter (`src/app/features/teaching/`, `src-tauri/src/teaching/handout_export.rs`):
- Enhance Mode 3's `StepWalkerComponent` with an interactive cycle spreadsheet for Cross Moment Distribution (`Next Cycle`, `Auto-Balance` until unbalance $< 0.01\text{ kNm}`, `Scenario 5`) and Kani Sway/Rotation tables (`Scenario 6`).
- Implement `teaching_export_handout` (`Scenario 8`) generating a complete bilingual Greek/English **LuaLaTeX (`.tex`)** and printable **HTML/PDF** university handout containing: problem statement, TikZ vector structural schematic & supports/loads, step-by-step symbolic and numeric equations, Cross/Kani or Vereshchagin tables, and TikZ/SVG $M, Q, N$ diagrams.
- Add the **"Export Handout (.tex / Print)" («Εξαγωγή Τεχνικού Τεύχους / Handout»)** button in Mode 3's toolbar.

## Acceptance criteria

- [ ] Querying deflection at a target point via "Virtual Work Deflection (ΑΔΕ)" constructs the unit dummy load system ($\bar{P}=1$ or $\bar{M}=1$), renders both $M_0(x)$ and $\bar{M}(x)$ diagrams, and presents the Vereshchagin integration table with exact rational fractions (`Scenario 7`).
- [ ] Cross and Kani methods render interactive cycle spreadsheets in Mode 3 allowing cycle-by-cycle stepping (`Next Cycle` / `Auto-Balance`) until convergence $< 0.01\text{ kNm}$ (`Scenario 5`, `Scenario 6`).
- [ ] Clicking "Export Handout" exports a standalone LuaLaTeX (`.tex`) document (and printable handout view) with TikZ schematics, step derivations, Cross/Kani/Vereshchagin tables, and $M, Q, N$ plots (`Scenario 8`).
- [ ] Rust unit tests (`virtual_work_test.rs`, `handout_export_test.rs`), `npx vitest run`, `npx ng build`, and `npm run check:limits` pass with 0 errors.

## Artifact sync required

- Application PRD: `none - already covered in docs/prd.md`
- Application architecture summary: `none - matches docs/architecture/application-architecture-summary.md`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-structural-mechanics-teaching.md (transition state to implemented upon completion of 034-035)`
- Issue registry: `required`; node `issues:` reference: `required: .okf/capabilities/cap-structural-mechanics-teaching.md`
- Reason/no-impact decision: `Completes all remaining scenarios (Scenarios 1..8) of cap-structural-mechanics-teaching.`

## Human review gate

`visual-review`: User runs a Virtual Work (ΑΔΕ) deflection query in Mode 3, inspects the $M_0$ and $\bar{M}$ diagrams and Vereshchagin table, steps through the interactive Cross/Kani spreadsheet, and exports a LuaLaTeX `.tex` handout.

## Blocked by

- Blocked by `docs/agents/issues/pending/20261103-034-teaching-clapeyron-force-method-vereshchagin-steps.md`

## Artifact anchors

- Capability spec (`.okf/capabilities/cap-structural-mechanics-teaching.md`): Scope §2 (Cross & Kani tables), §3 (Principle of Virtual Work & Castigliano), §4 (Exportable Handout & Exam Generator)

## Acceptance scenarios addressed

- `Scenario 5`: Moment Distribution (Cross) with Interactive Cycle Table
- `Scenario 6`: Rigid Frame with Side Sway via Kani Method
- `Scenario 7`: Deflection via Principle of Virtual Work (ΑΔΕ)
- `Scenario 8`: Automated Handout & Exam Sheet Export

## Verification obligations

- Policy source: `/.okf/project.md`

| Scenario | Backend boundary | Frontend integration | End-to-end journey |
|---|---|---|---|
| `Scenario 5` | `implemented: src-tauri/src/teaching/cross_kani.rs` | `planned` | `catalog-only` |
| `Scenario 6` | `implemented: src-tauri/src/teaching/cross_kani.rs` | `planned` | `catalog-only` |
| `Scenario 7` | `planned` | `planned` | `catalog-only` |
| `Scenario 8` | `planned` | `planned` | `catalog-only` |
