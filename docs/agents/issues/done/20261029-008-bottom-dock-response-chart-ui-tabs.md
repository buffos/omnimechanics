## Issue Metadata

- Issue number: `008`
- Owning capability node: `/.okf/capabilities/cap-parametric-benchmarking.md`
- Artifact root: `docs/architecture/cap-parametric-benchmarking/`
- Issue file: `docs/agents/issues/done/20261029-008-bottom-dock-response-chart-ui-tabs.md`
- Category: `feature`
- Execution type: `AFK`
- Review gate: `visual-review`
- Suggested state: `done`

## Parent Artifacts

- `.okf/project.md`
- `.okf/capabilities/cap-parametric-benchmarking.md`
- `.okf/capabilities/cap-interactive-canvas.md`
- `docs/roadmap.md`

## What to build

Implement a collapsible Bottom Engineering Dock with an interactive Time-History Response Chart ($u_y(t)$ curve) and reorganize the Left Sidebar into clean workflow Tabs:

1. **Collapsible Bottom Engineering Dock (`src/app/dock/`):**
   - Docked at the bottom of the main viewport wrapper (default height ~240px, collapsible to a sleek 34px status bar).
   - Reactive SVG/Canvas Time-History Response Plot:
     - X-axis: Time $t \in [0, T_{\text{total}}]$ (seconds / ms).
     - Y-axis: Max / Tip vertical displacement $u_y(t)$ (mm).
     - Curves plotted side-by-side:
       - **Deformed Talbot Contour** (Cyan `#38bdf8`)
       - **Newmark-$\beta$ Direct Marching** (Amber/Orange `#f59e0b`)
       - **Theoretical Static Euler-Bernoulli Line** (Dashed gray `#94a3b8`)
     - Interactive cursor tooltip showing $(t, u)$ and scrubbing the 3D viewport frame.
     - Actions: `[ 📥 Export CSV ]` and `[ 📷 Export SVG ]` for direct publication in the monograph.
   - Secondary Tab: **📊 Convergence & Diagnostics** showing iterations, residual norms, and timing.

2. **Tabbed Left Sidebar Organization (`app.component`):**
   - Partition the congested vertical scroll into 3 workflow-driven tabs:
     - 📐 **Geometry & Mesh:** Presets, dimensions ($L, H$), mesh resolution ($n_x, n_y$), element type.
     - 🧪 **Material & Medium:** $E_0, \nu, \rho$, inhomogeneity strategies, `[ ⚙️ Soil / Surrounding Medium... ]` modal trigger.
     - ⚡ **Solver & Dynamics:** Analysis domain, Integration method, Excitation waveform, total time, steps, load, damping.
   - Pinned Action Footer:
     - `[ Run Simulation ]` button and execution status banner stay permanently docked at the bottom of the sidebar across all tabs.

3. **Solver Multi-Run History Synchronization (`solver.service.ts`):**
   - Store both `lastTalbotResult` and `lastNewmarkResult` signals so executing one method preserves the curve of the other, enabling instant side-by-side benchmarking.

## Acceptance criteria

- [x] Collapsible bottom dock component created and integrated below the Three.js viewport.
- [x] Interactive time-history response plot rendering both Talbot and Newmark curves alongside static benchmark line.
- [x] Tooltip inspection and CSV export functional.
- [x] Left sidebar reorganized into 3 workflow tabs (Mesh, Material, Solver) with pinned bottom action button.
- [x] Multi-run curve caching functional in `solver.service.ts`.
- [x] Angular production build compiles cleanly (`ng build`).

## Artifact sync required

- Application PRD: `none - UI visualization capability`
- Application architecture summary: `none - internal UI reorganization`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-parametric-benchmarking.md`
- Issue registry: `required`; node `issues:` reference: `required`
- Reason/no-impact decision: `Delivers interactive dynamic response benchmarking dock and tabbed UI layout.`
