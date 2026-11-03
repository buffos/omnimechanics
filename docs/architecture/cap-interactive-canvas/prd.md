# Product Requirements Document: Interactive 3D Visualization Canvas & UI (`cap-interactive-canvas`)

- **Capability Owner:** [`cap-interactive-canvas`](../../../.okf/capabilities/cap-interactive-canvas.md)
- **State:** `bounded` → target `specified`
- **Parent Product PRD:** [`docs/prd.md`](../../prd.md)
- **Cross-Capability Conventions:** [`cross-capability-conventions.md`](../cross-capability-conventions.md)
- **Domain Glossary:** [`domain-glossary.md`](domain-glossary.md)

---

## 1. Purpose & Product Summary

`cap-interactive-canvas` provides the unified visual workbench, application shell, and scientific visualization layer for **OmniMechanics**. It transforms multi-dimensional numerical outputs from the Rust computational core (`cap-solver-core`, `cap-material-field`, `cap-parametric-benchmarking`, `cap-frame-analysis-engine`, `cap-structural-mechanics-teaching`) into immediate, interactive 2D and 3D visual insights.

The capability owns four cohesive functional pillars:
1. **Tri-Mode Workspace Shell & Shared UI Infrastructure:** Top-level header mode switcher (`continuum` Mode 1, `structural` Mode 2, `teaching` Mode 3), instant bilingual Greek/English (`el`/`en`) localization (`I18nService`), stacked accessible modal dialogs (`DialogService` / `DialogHostComponent`), and deduplicated toast notifications (`NotificationService` / `NotificationHostComponent`).
2. **Mode 1 Three.js 3D/2D Continuum Viewport:** Hardware-accelerated WebGL viewport supporting orbit/pan/zoom navigation, perspective/orthographic camera switching, surface/wireframe/edges render modes, interactive 3D clipping plane slicing, raycast boundary face/edge picking for Dirichlet/Neumann/Robin boundary conditions, physical scalar colormaps (`Turbo`, `Viridis`, `Coolwarm`, `Jet`) for material stiffness $\mu(\mathbf{x})$, von Mises stress $\sigma_{\text{vM}}$, principal stresses, and displacement $\|\mathbf{u}\|$, deformed shape exaggeration ($\times 1 \dots \times 1000+$ and `Auto`), vector glyph arrows, and harmonic/transient playback scrubbing.
3. **Bottom Engineering Dock:** Collapsible SVG scientific charting dock displaying transient response history curves ($u_y(t)$ comparing Laplace-Talbot vs. Newmark-$\beta$ against analytical Euler-Bernoulli static deflection), live streaming solver convergence residuals, parametric benchmark sweep charts, and one-click CSV/SVG export.
4. **Shared 2D SVG Viewport Primitives & Conventions:** Reusable world-to-screen coordinate transformation (`ViewportTransform2D` with $+y$ up in metres and pan/zoom matrix), tension-side internal force diagram plotting (`TensionSideDiagram` for $M, Q, N$ and envelopes), and canonical `SemanticColorTokens` shared across Mode 2 (`cap-sketcher-frame-modeler`) and Mode 3 (`cap-structural-mechanics-teaching`).

---

## 2. Goals & Non-Goals

### 2.1 Goals
- **G-1 (Seamless Tri-Mode Navigation):** Allow users to switch effortlessly between Mode 1 (Continuum Workbench), Mode 2 (Structural Modeling & Analysis), and Mode 3 (Teaching Studio) and toggle Greek/English (`el`/`en`) without losing active model or simulation state.
- **G-2 (Interactive 3D/2D Continuum Inspection):** Render conformal 2D/3D FEM/BEM meshes (`Tri3`, `Quad4`, `Tet4`, `Hex8`) at 60 FPS with interactive camera controls, dynamic 3D clipping planes, and direct raycast boundary picking.
- **G-3 (Quantitative Field & Deformation Clarity):** Present physical fields ($\mu(\mathbf{x})$, $\sigma_{\text{vM}}$, principal stresses, $\|\mathbf{u}\|$) with calibrated colorbars, selectable scientific palettes (`Turbo`, `Viridis`, `Coolwarm`, `Jet`), vector glyphs, and controlled deformation exaggeration that never distorts reported numerical values.
- **G-4 (Synchronized Time-History & Convergence Analytics):** Couple the 3D/2D viewport playback scrubber directly to the Bottom Engineering Dock response curves and stream iterative solver convergence residuals in real time.
- **G-5 (Consistent 2D Structural Graphics Across Mode 2 & Mode 3):** Enforce a single world-to-screen coordinate transform ($+y$ up), tension-side bending moment convention, and semantic color palette across both the Mode 2 frame modeler and Mode 3 teaching canvas.

### 2.2 Non-Goals
- **NG-1:** VR/AR headset rendering or stereoscopic WebXR output.
- **NG-2:** Owning 2D structural sketching tools, snapping engines, or frame model mutation logic (owned by `cap-sketcher-frame-modeler`).
- **NG-3:** Owning FEM/BEM/DSM numerical solver algorithms, mesh generation algorithms, or Eurocode combination generation (owned by `cap-solver-core`, `cap-geometry-presets`, `cap-frame-analysis-engine`, and `cap-eurocode-actions-combinations`).
- **NG-4:** Photorealistic ray-traced rendering or texture-mapped architectural material shaders.

---

## 3. Actors & User Journeys

| Actor | Key Responsibilities & Needs in `cap-interactive-canvas` |
| --- | --- |
| **Researcher** | Inspects 2D/3D graded material fields $\mu(\mathbf{x})$, slices 3D cavities with the clipping plane, compares Talbot vs. Newmark-$\beta$ wave histories in the Bottom Engineering Dock, monitors GMRES/Anderson residual convergence, and exports SVG/CSV figures for mini-papers. |
| **Engineer / Instructor** | Switches between Mode 1, Mode 2, and Mode 3; picks boundary facets to assign Dirichlet/Neumann/Robin conditions; configures soil impedance in a modal dialog; inspects deformed shapes and tension-side M/Q/N diagrams. |
| **Student** | Toggles language between Greek (`el`) and English (`en`), steps through harmonic/transient animations or Mode 3 derivations, and reads color-coded support, load, and internal force diagrams. |

---

## 4. Required Workflows

### WF-1: Tri-Mode Workspace Switching, Bilingual Toggle & Modal/Toast Lifecycle
1. The user clicks the header mode switcher to select **Mode 1 (`continuum`)**, **Mode 2 (`structural`)**, or **Mode 3 (`teaching`)**.
2. The shell activates the target workspace while preserving in-memory signals for the inactive workspaces.
3. Clicking the language button (`🇬🇷 EL` / `🇬🇧 EN`) toggles `I18nService.currentLang` between `'el'` and `'en'`, updating all header controls, sidebar labels, dialog titles, and canvas legends immediately.
4. Triggering an advanced configuration action (e.g., *Soil / Surrounding Medium Interaction* or *Solvers Taxonomy & Roles*) opens a typed `DialogSession` in `DialogHostComponent`, locking background scroll and trapping focus.
5. **Policy/Blocked Branch:** If the dialog registers a `DismissGuard` (e.g., invalid negative soil modulus), pressing `Escape` or clicking the backdrop is blocked and a warning `ToastNotification` is shown; when valid and closed, focus returns to the trigger button.

### WF-2: 3D/2D Continuum Mesh Exploration, Camera Projection & Clipping Plane Slicing
1. When a 2D or 3D mesh (`MeshData`) loads, `ContinuumViewport` automatically fits the camera to the mesh bounding box (`fitCameraToMesh`), orienting perpendicular to the $xy$-plane for 2D models or at an isometric spherical angle ($\theta = \pi/4, \phi = \pi/3$) for 3D solids.
2. The user orbits (left-drag), pans (right-drag), zooms (mouse wheel), switches between `Perspective` and `Orthographic` projection, or toggles `SurfaceSolid`, `WireframeOverlay`, `BoundaryTags`, and `Nodes`.
3. In 3D mode, enabling the **3D Clipping Plane Slider** and dragging normalized offset $s \in [-1, 1]$ along axis `X`, `Y`, or `Z` slices the rendered volume mesh in real time via WebGL clipping planes, exposing internal stress distributions around 3D cavities or inclusions.

### WF-3: Interactive Raycast Face/Edge Picking for Boundary Conditions
1. With boundary picking enabled in `ContinuumViewport`, the user hovers or clicks directly on a boundary edge (2D) or boundary facet (3D).
2. A Three.js `Raycaster` intersects the rendered `boundaryGroup` meshes, resolves the hit facet's `BoundaryTag` (`Left`, `Right`, `Top`, `Bottom`, `Front`, `Back`, `Hole`), highlights all facets sharing that tag, and emits a `BoundaryFacetPick` event.
3. The user assigns or edits the `BoundaryConditionAssignment` (`Dirichlet` fixed displacement $\bar{u}_i = 0$, `Neumann` prescribed traction $\bar{t}_i = P$, or `Robin` impedance interface $\mathbf{t} + \mathbf{Z}\mathbf{u} = \mathbf{g}$).
4. **Exception Path:** Clicking empty canvas background clears the active facet highlight without mutating boundary assignments.

### WF-4: Physical Field Colormaps, Deformation Exaggeration & Vector Glyphs
1. Before or after solving, the user selects a `ScalarFieldChannel` (`Material`, `VonMises`, `PrincipalStress`, or `Displacement`) and a `ColormapPalette` (`Turbo`, `Viridis`, `Coolwarm`, `Jet`).
2. `ContinuumViewport` normalizes nodal values across $[v_{\min}, v_{\max}]$ (handling uniform fields where $v_{\max} - v_{\min} \le 10^{-9}$ safely without division by zero) and updates vertex colors and the HUD colorbar legend.
3. When simulation results exist, `DeformationScalePolicy` computes displaced node coordinates $\mathbf{x}_{\text{def}} = \mathbf{x}_0 + \alpha_{\text{def}}\mathbf{u}$ using either `Auto` ($8\%$ of $L_{\max}$) or user-selected presets ($\times 1, \times 10, \times 100, \times 1000, \times 5000, \times 20000$).
4. Enabling `VectorGlyphOverlay` renders directional arrow glyphs at loaded and supported boundaries proportional to applied traction $\bar{\mathbf{t}}$ and reaction $\mathbf{r}$.

### WF-5: Transient & Harmonic Playback Scrubbing Synchronized with Bottom Dock
1. After a `TimeDomain` or `LaplaceTalbot` simulation completes, the viewport displays the `PlaybackTimelineController` bar and the `EngineeringDock` plots the `ResponseHistoryCurve` ($u_y(t)$ for Talbot and Newmark-$\beta$ plus the dashed purple Static Euler-Bernoulli reference line).
2. Clicking **Play** advances the frame index (or harmonic phase $\theta \in [0^\circ, 360^\circ)$), animating the deformed 3D/2D mesh and moving the vertical white cursor line on the dock SVG chart in lockstep.
3. Clicking or dragging anywhere on the dock's `ResponseHistoryCurve` SVG chart computes the nearest time frame $t_k$ and immediately updates both the dock cursor and the 3D/2D viewport mesh state.
4. Clicking **CSV** or **SVG** in the dock header exports the time-series table or vector chart file.

### WF-6: Shared 2D SVG Structural Rendering (Mode 2 & Mode 3)
1. Mode 2 and Mode 3 canvases project 2D world coordinates $(x, y)$ in metres ($+y$ up) through `ViewportTransform2D` to SVG pixels $(u, v)$ ($+v$ down) with aspect-locked scaling and interactive pan/zoom.
2. Internal force diagrams ($M, Q, N$ and envelopes) are constructed perpendicular to each member's axis using `TensionSideDiagram` rules: bending moment $M$ ordinates are always offset toward the tension fibre (downward/local $-y$ for positive $M$), and colored with `SemanticColorTokens`.

---

## 5. Business Rules & Constraints

- **BR-1 (True Physical Reporting):** Scaling deformed geometry by $\alpha_{\text{def}}$ must never alter scalar colorbar extrema, HUD peak metrics, or exported CSV values.
- **BR-2 (Degenerate Span Protection):** If a scalar field has $\max - \min \le 10^{-9}$ (e.g., homogeneous material or zero displacement), normalization uses a fallback span of $1.0$ so vertex colors map cleanly to $t = 0.0$ without `NaN` artifacts.
- **BR-3 (Stale Result Clearing):** Any mesh regeneration or geometry preset change immediately clears active simulation overlays (`solverService.clearResult()`) so stale stress/displacement fields are never rendered on a new mesh.
- **BR-4 (Tension-Side Moment Invariant):** Across all 2D structural SVG viewports (Mode 2 and Mode 3), positive bending moment ($M > 0$, bottom fibre in tension) is plotted on the local $-y$ (bottom) side of the member, and negative moment ($M < 0$, top fibre in tension) on the local $+y$ (top) side.
- **BR-5 (Bilingual Completeness):** Every user-facing label, HUD badge, dock tab, dialog shell action, and error toast must resolve through `I18nService` (`el`/`en`) or carry `{ message_el, message_en }`.

---

## 6. Functional & Non-Functional Requirements

### Functional Requirements
- **FR-1:** Header bar with tri-mode workspace selector (`continuum`, `structural`, `teaching`), language toggle (`el`/`en`), Mode 1 dimension (`2D`/`3D`) and regime (`Static`/`TimeDomain`/`LaplaceTalbot`) segmented controls, backend status indicator, and solver taxonomy guide trigger.
- **FR-2:** Three.js `ContinuumViewport` supporting `Perspective` and `Orthographic` cameras, `SurfaceSolid` / `WireframeOverlay` / `VolumeEdgesOnly` modes, boundary tag legend, node point toggle, and `Reset View` camera auto-fit.
- **FR-3:** 3D clipping plane controls (enable toggle, axis selector `X`/`Y`/`Z`, normalized position slider $s \in [-1, 1]$, invert normal toggle) updating `renderer.clippingPlanes` and material clipping planes.
- **FR-4:** Raycast boundary picking on 2D boundary lines and 3D boundary surfaces with hover/selection highlight and emission of picked `BoundaryTag` for Dirichlet/Neumann/Robin assignment.
- **FR-5:** Scientific colormaps (`Turbo`, `Viridis`, `Coolwarm`, `Jet`) for `Material`, `VonMises`, `PrincipalStress`, and `Displacement` channels, plus vector glyph arrows (`THREE.ArrowHelper` / instanced cones+cylinders) for boundary tractions/reactions.
- **FR-6:** Collapsible `EngineeringDock` with `Response History u_y(t)`, `Convergence & Diagnostics`, and `Parametric Sweep` views, interactive time scrubbing, and CSV/SVG export.
- **FR-7:** Shared 2D SVG coordinate transform utility (`ViewportTransform2D`), tension-side diagram path builder, and semantic color tokens for Mode 2 and Mode 3.

### Non-Functional Requirements
- **NFR-1 (Frame Rate):** 60 FPS camera orbit/pan/zoom and clipping plane scrubbing for meshes up to $50{,}000$ elements; WebGL buffers (`THREE.BufferGeometry`) must be disposed on mesh replacement and component destruction to prevent GPU memory leaks.
- **NFR-2 (Accessibility & Keyboard UX):** `DialogShellComponent` traps `Tab`/`Shift+Tab` focus, supports `Escape` dismissal (subject to `DismissGuard`), and restores focus to the opener element upon close.
- **NFR-3 (Code Size Discipline):** All TypeScript files $\le 1000$ lines and HTML templates $\le 500$ lines (`npm run check:limits`).

---

## 7. Acceptance Scenarios Summary & Assumptions

Canonical acceptance scenarios (`SC-001` through `SC-012`) are fully specified in [`acceptance-scenarios.md`](acceptance-scenarios.md).

### Explicit Assumptions
- **A-1:** Mode 1 continuum solver runs and mesh generation occur in Rust via Tauri IPC (`MeshService`, `MaterialService`, `SolverService`); `cap-interactive-canvas` consumes their typed DTO signals and progress events.
- **A-2:** Mode 2 (`cap-sketcher-frame-modeler`) and Mode 3 (`cap-structural-mechanics-teaching`) consume the shared 2D SVG transform and diagram conventions from `src/app/shared/` while owning their respective sidebar/tool workflows.

### Artifact-Impact Classification
- **Impact on `docs/prd.md` and `docs/architecture/application-architecture-summary.md`:** `no impact` — this node PRD formalizes the exact specification for `cap-interactive-canvas` within the boundaries already established in the 2026-10-31 application synthesis gate.
