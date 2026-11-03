# Acceptance Scenarios: Interactive 3D Visualization Canvas & UI (`cap-interactive-canvas`)

- **Capability Owner:** [`cap-interactive-canvas`](../../../.okf/capabilities/cap-interactive-canvas.md)
- **Upstream References:** [`prd.md`](prd.md), [`canonical-domain-model.md`](canonical-domain-model.md), [`canonical-use-cases.md`](canonical-use-cases.md), [`canonical-api-cli-contract.md`](canonical-api-cli-contract.md)
- **Root Verification Policy:** `backend-boundary: required`, `frontend-integration: required` (Vitest), `end-to-end: catalog-only` (visual review gate)

---

## 1. Scenario Coverage Matrix

| Scenario ID | Title | Primary Pillar | Branch / Rule Tested | `backend-boundary` | `frontend-integration` | `end-to-end` |
| --- | --- | --- | --- | --- | --- | --- |
| **SC-001** | Tri-Mode Workspace Switching Preserves Active Session State | Shell & Header | Happy path (`WF-1`) | `not-applicable` | `required` | `catalog-only` |
| **SC-002** | Instant Bilingual Toggle (`el`/`en`) Across Shell, Viewport & Dock | Shell & I18n | Policy (`BR-5`) | `not-applicable` | `required` | `catalog-only` |
| **SC-003** | Modal Dialog Stack LIFO Enforcement & DismissGuard Protection | Dialogs & Toasts | Blocked / Policy (`INV-1`) | `not-applicable` | `required` | `catalog-only` |
| **SC-004** | Toast Notification Deduplication & 5000 ms Auto-Dismissal | Dialogs & Toasts | Idempotency (`INV-2`) | `not-applicable` | `required` | `catalog-only` |
| **SC-005** | 3D/2D Camera Auto-Fit, Orbit Singularity Clamp & Orthographic Toggle | 3D Viewport | Invariant (`INV-3`, `WF-2`) | `not-applicable` | `required` | `catalog-only` |
| **SC-006** | 3D Dynamic Clipping Plane Slicing Across Volumetric Cavity Mesh | 3D Viewport | Policy (`INV-5`, `WF-2`) | `not-applicable` | `required` | `catalog-only` |
| **SC-007** | Interactive Raycast Boundary Facet Picking & Background Miss Handling | 3D Viewport | Happy + Exception (`WF-3`) | `not-applicable` | `required` | `catalog-only` |
| **SC-008** | Scalar Field Colormap Transfer & Degenerate Homogeneous Span Guard | Field Visualization | Invariant (`INV-4`, `BR-2`) | `not-applicable` | `required` | `catalog-only` |
| **SC-009** | Deformation Exaggeration (`Auto` & Manual Presets) Preserves Physical Readouts | Field Visualization | Policy (`POL-1`, `BR-1`) | `not-applicable` | `required` | `catalog-only` |
| **SC-010** | Synchronized Transient/Harmonic Playback Scrubbing Between Viewport & Dock | Dock & Playback | Stateful sync (`WF-5`) | `not-applicable` | `required` | `catalog-only` |
| **SC-011** | Live Streaming Solver Convergence Residuals & CSV/SVG Export Guard | Engineering Dock | Operational (`WF-5`) | `required` (DTO) | `required` | `catalog-only` |
| **SC-012** | Shared 2D SVG World-to-Screen Transform & Tension-Side Moment Diagram Parity | Shared 2D Kernel | Invariant (`INV-6`, `BR-4`) | `not-applicable` | `required` | `catalog-only` |

---

## 2. Detailed Canonical Scenarios

### SC-001: Tri-Mode Workspace Switching Preserves Active Session State
- **Target Behavior:** Switching between Mode 1 (`continuum`), Mode 2 (`structural`), and Mode 3 (`teaching`) updates the visible workspace while preserving in-memory parameters and pausing background animation timers cleanly.
- **Preconditions:** Mode 1 has an active `PlateWithHole2D` mesh and harmonic oscillation playing (`isPlayingHarmonic() === true`).
- **Action:** The user selects Mode 2 (`structural`) in the header switcher, then Mode 3 (`teaching`), and finally returns to Mode 1 (`continuum`).
- **Expected Outcome:**
  - Each workspace renders its dedicated main view on selection.
  - Returning to Mode 1 restores the `PlateWithHole2D` mesh, material settings, and simulation result without re-running the solver.
- **Verification Surfaces:** `backend-boundary: not-applicable` | `frontend-integration: required` | `end-to-end: catalog-only`

### SC-002: Instant Bilingual Toggle (`el`/`en`) Across Shell, Viewport & Dock
- **Target Behavior:** Toggling language updates all user-facing strings between Greek (`el`) and English (`en`) reactively (`BR-5`).
- **Preconditions:** Application starts in default Greek locale (`currentLang() === 'el'`).
- **Action:** The user clicks the header language switcher (`🇬🇷 EL` → `🇬🇧 EN`) and then toggles back to `🇬🇷 EL`.
- **Expected Outcome:**
  - Header navigation buttons, workspace labels, step walker controls, and diagnostic messages switch immediately to English and back to Greek without page reload or state loss.
- **Verification Surfaces:** `backend-boundary: not-applicable` | `frontend-integration: required` | `end-to-end: catalog-only`

### SC-003: Modal Dialog Stack LIFO Enforcement & DismissGuard Protection
- **Target Behavior:** `DialogService` enforces LIFO ordering, blocks dismissal when a `DismissGuard` returns `false`, and restores keyboard focus on close (`INV-1`).
- **Preconditions:** The user opens `SurroundingMediumDialogComponent` from the Solver tab trigger button, and the dialog registers a `DismissGuard` returning `false` while an invalid input is present.
- **Action:**
  1. The user presses `Escape` or clicks the backdrop while `dismissGuard() === false`.
  2. The user corrects the input (`dismissGuard() === true`) and closes the dialog.
- **Expected Outcome:**
  - Step 1 is blocked (`DIALOG_DISMISS_BLOCKED`); the dialog remains open and active in `DialogHostComponent`.
  - Step 2 closes the dialog, unlocks document body scroll, resolves `ref.closed` with the updated `SurroundingMediumConfig`, and restores focus to the trigger button.
- **Verification Surfaces:** `backend-boundary: not-applicable` | `frontend-integration: required` | `end-to-end: catalog-only`

### SC-004: Toast Notification Deduplication & 5000 ms Auto-Dismissal
- **Target Behavior:** `NotificationService` deduplicates identical active toasts and resets their 5000 ms expiration timer (`INV-2`).
- **Preconditions:** No active notifications exist.
- **Action:** `notificationService.show('Mesh generated', 'info')` is called twice within 2000 ms, then 5000 ms elapse after the second call.
- **Expected Outcome:**
  - `notifications()` contains exactly one entry (not two) during the active window.
  - The notification remains visible at 4000 ms after the first call (2000 ms after the second) and auto-dismisses 5000 ms after the second call.
- **Verification Surfaces:** `backend-boundary: not-applicable` | `frontend-integration: required` | `end-to-end: catalog-only`

### SC-005: 3D/2D Camera Auto-Fit, Orbit Singularity Clamp & Orthographic Toggle
- **Target Behavior:** Camera auto-fits 2D and 3D meshes, clamps polar angle $\phi$ away from poles, and switches between `Perspective` and `Orthographic` projections (`INV-3`, `WF-2`).
- **Preconditions:** A 3D `BlockWithCavity3D` mesh is loaded in `ContinuumViewport`.
- **Action:** The user drags vertically past the zenith/nadir limits, zooms with the wheel, toggles `Orthographic` projection, and clicks `Reset View`.
- **Expected Outcome:**
  - Polar angle $\phi$ stays strictly within $[0.05, \pi - 0.05]$ rad and radius $r \in [0.5, 250]$.
  - Switching to `Orthographic` renders parallel edges without perspective foreshortening; `Reset View` re-centers target $\mathbf{c}$ and restores default isometric framing ($\theta = \pi/4, \phi = \pi/3$).
- **Verification Surfaces:** `backend-boundary: not-applicable` | `frontend-integration: required` | `end-to-end: catalog-only`

### SC-006: 3D Dynamic Clipping Plane Slicing Across Volumetric Cavity Mesh
- **Target Behavior:** Scrubbing the 3D clipping plane slider slices the volume mesh to reveal internal stress and material fields (`INV-5`, `WF-2`).
- **Preconditions:** A 3D `BlockWithCavity3D` mesh with solved `VonMises` stress field is displayed.
- **Action:** The user enables the clipping plane on axis `Z` and drags `normalizedOffset` from $1.0$ to $0.0$ (mid-plane), then toggles `invertNormal`.
- **Expected Outcome:**
  - At $s = 0.0$, the front half ($z > c_z$) of the mesh is clipped away, exposing the internal spherical cavity boundary and interior `VonMises` vertex colors.
  - Toggling `invertNormal` flips the visible half to $z < c_z$.
- **Verification Surfaces:** `backend-boundary: not-applicable` | `frontend-integration: required` | `end-to-end: catalog-only`

### SC-007: Interactive Raycast Boundary Facet Picking & Background Miss Handling
- **Target Behavior:** Clicking a boundary facet selects its `BoundaryTag` for Dirichlet/Neumann/Robin assignment; clicking empty space clears selection safely (`WF-3`).
- **Preconditions:** `showBoundaries()` is active on a 3D beam mesh with tags `Left`, `Right`, `Top`, `Bottom`, `Front`, `Back`.
- **Action:**
  1. The user clicks on a `Right` boundary facet and assigns a `Neumann` traction load.
  2. The user clicks on empty canvas background.
- **Expected Outcome:**
  - Step 1 emits `BoundaryFacetPicked` with `tag: 'Right'`, highlights all `Right` facets, and binds the `Neumann` traction assignment and vector glyphs.
  - Step 2 (`RAYCAST_NO_BOUNDARY_HIT`) clears the selection highlight while preserving the assigned `Neumann` condition on `Right`.
- **Verification Surfaces:** `backend-boundary: not-applicable` | `frontend-integration: required` | `end-to-end: catalog-only`

### SC-008: Scalar Field Colormap Transfer & Degenerate Homogeneous Span Guard
- **Target Behavior:** Colormap palettes (`Turbo`, `Viridis`, `Coolwarm`, `Jet`) map scalar values to RGB accurately and guard against zero-span division (`INV-4`, `BR-2`).
- **Preconditions:** A homogeneous mesh has uniform Young's modulus $E(\mathbf{x}) = 200\text{ GPa}$ ($\min = \max = 200$).
- **Action:** `ContinuumViewport` evaluates vertex colors for `Material` and the user cycles palettes (`Turbo` → `Viridis` → `Coolwarm` → `Jet`).
- **Expected Outcome:**
  - Normalization uses fallback span $1.0$ (`t = 0.0`), producing valid finite RGB values in $[0, 1]^3$ with zero `NaN` buffer entries, and the colorbar displays `200.00 GPa` min/max cleanly.
- **Verification Surfaces:** `backend-boundary: not-applicable` | `frontend-integration: required` | `end-to-end: catalog-only`

### SC-009: Deformation Exaggeration (`Auto` & Manual Presets) Preserves Physical Readouts
- **Target Behavior:** `Auto` deformation scaling normalizes visual peak deflection to $8\%$ of $L_{\max}$, and changing scale presets never alters physical stress/displacement numbers (`POL-1`, `BR-1`).
- **Preconditions:** A $10\text{ m}$ cantilever beam simulation finishes with $\max \|\mathbf{u}\| = 0.002\text{ m}$ ($2.000\text{ mm}$) and $\max \sigma_{\text{vM}} = 45.20\text{ MPa}$.
- **Action:** `autoScale()` runs automatically on solve completion; the user then clicks `1000×`.
- **Expected Outcome:**
  - `autoScale()` sets `deformationScale` to $400\times$ ($0.08 \times 10 / 0.002 = 400$).
  - Switching to `1000×` updates the Three.js displaced vertex positions $\mathbf{x}_{\text{def}} = \mathbf{x}_0 + 1000\mathbf{u}$, while the HUD badge and colorbar still read `2.000 mm` and `45.20 MPa`.
- **Verification Surfaces:** `backend-boundary: not-applicable` | `frontend-integration: required` | `end-to-end: catalog-only`

### SC-010: Synchronized Transient/Harmonic Playback Scrubbing Between Viewport & Dock
- **Target Behavior:** Scrubbing either the viewport timeline slider or clicking the Bottom Engineering Dock `ResponseHistoryCurve` chart synchronizes the 3D deformed mesh and dock cursor line (`WF-5`).
- **Preconditions:** A 25-frame `TimeDomain` simulation ($T_{\max} = 0.50\text{ s}$) is loaded in Mode 1 with `BottomDockComponent` expanded on the `Response History u_y(t)` tab.
- **Action:** The user clicks at the horizontal midpoint ($t = 0.25\text{ s}$) of `#transient-response-svg` in the Bottom Engineering Dock.
- **Expected Outcome:**
  - `currentFrameIndex` updates to the frame closest to $t = 0.25\text{ s}$.
  - Both the white dashed playback cursor in `BottomDockComponent` and the deformed mesh / time readout in `ViewportComponent` jump immediately to that frame.
- **Verification Surfaces:** `backend-boundary: not-applicable` | `frontend-integration: required` | `end-to-end: catalog-only`

### SC-011: Live Streaming Solver Convergence Residuals & CSV/SVG Export Guard
- **Target Behavior:** Iterative solver residuals populate the HUD and dock diagnostics table, and CSV/SVG exports are guarded when data is empty (`WF-5`).
- **Preconditions:** Mode 1 starts with no transient results; then an iterative hybrid solve and a transient benchmark solve are executed.
- **Action:**
  1. Check export buttons before solving.
  2. Run solver and inspect `Convergence & Diagnostics` tab, then click `CSV` and `SVG` export after transient solve.
- **Expected Outcome:**
  - Before solving, `CSV` and `SVG` export buttons are disabled (`EXPORT_DATA_EMPTY` guard).
  - After solving, residual iterations (`Iter`, `||r||`, `Relative Error`) render in scientific notation, and `CSV` / `SVG` generate valid downloadable blobs containing Talbot, Newmark-$\beta$, and Static Euler-Bernoulli series.
- **Verification Surfaces:** `backend-boundary: required` (solver result DTO contract) | `frontend-integration: required` | `end-to-end: catalog-only`

### SC-012: Shared 2D SVG World-to-Screen Transform & Tension-Side Moment Diagram Parity
- **Target Behavior:** `ViewportTransform2D` and `TensionSideDiagram` produce identical coordinate orientation ($+y$ up) and tension-side bending moment diagrams across Mode 2 and Mode 3 (`INV-6`, `BR-4`).
- **Preconditions:** A horizontal simply supported beam ($L = 6\text{ m}$) under downward uniform load $q = 10\text{ kN/m}$ has midspan sagging moment $M(3) = +45\text{ kNm}$ (bottom fibre in tension).
- **Action:** The beam and its $M(x)$ diagram are projected through `ViewportTransform2D` and `buildTensionSideDiagramPath` for both Mode 2 and Mode 3 canvases.
- **Expected Outcome:**
  - World $+y$ maps to screen $-v$ (upward on screen), and `screenToWorld(worldToScreen(x, y))` equals $(x, y)$ within $10^{-9}$.
  - The positive sagging moment ordinate $+45\text{ kNm}$ is plotted downward on screen (on the bottom/local $-y$ tension side of the beam) in both Mode 2 and Mode 3.
- **Verification Surfaces:** `backend-boundary: not-applicable` | `frontend-integration: required` | `end-to-end: catalog-only`
