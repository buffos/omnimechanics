# Domain Glossary: Interactive 3D Visualization Canvas & UI (`cap-interactive-canvas`)

- **Capability Owner:** [`cap-interactive-canvas`](../../../.okf/capabilities/cap-interactive-canvas.md)
- **Upstream Conventions:** [`cross-capability-conventions.md`](../cross-capability-conventions.md)
- **Status:** Canonical Reference (2026-11-01)

## 1. Purpose

This glossary stabilizes the ubiquitous language for `cap-interactive-canvas`, which owns the application-wide Tri-Mode Workspace Shell, shared modal dialog and toast notification infrastructure, the Mode 1 Three.js 3D/2D Continuum Viewport, the collapsible Bottom Engineering Dock, and the shared 2D SVG Viewport Primitives & Coordinate Transform conventions consumed across Mode 2 (`cap-sketcher-frame-modeler`) and Mode 3 (`cap-structural-mechanics-teaching`).

It strictly inherits cross-capability terms (`StructuralModel`, `Node`, `Element`, `Support`, `Load`, `LoadCase`, `Combination`, `ResultSet`, `Envelope`, `Diagnostics`, `ContinuumDomain`, `ProjectDocument`) from [`cross-capability-conventions.md`](../cross-capability-conventions.md) without redefining or renaming their domain ownership.

---

## 2. Actors

| Canonical Term | Category | Definition | Aliases / Discouraged |
| --- | --- | --- | --- |
| **Engineer / Instructor** | Actor | Models 2D/3D continuum and structural systems, inspects field colormaps and internal-force diagrams, configures boundary conditions and solver parameters, and prepares educational demonstrations. | Power User, Modeler |
| **Student** | Actor | Navigates between Mode 2 and Mode 3, toggles Greek/English (`el`/`en`) terminology, inspects tension-side M/Q/N diagrams, and observes visual mechanism animations. | Learner |
| **Researcher** | Actor | Executes continuum BEM/FEM/hybrid simulations in Mode 1, slices 3D volumes with clipping planes, scrubs transient wave propagation timelines, and exports publication-ready CSV/SVG response curves from the Bottom Engineering Dock. | Analyst, Scientist |

---

## 3. Workspace Shell & Shared UI Terms

| Canonical Term | Category | Definition | Aliases / Discouraged |
| --- | --- | --- | --- |
| **WorkspaceMode** | State / Status | Top-level application workspace selector in the header switcher: `continuum` (Mode 1: Continuum Workbench), `structural` (Mode 2: Structural Modeling & Analysis), or `teaching` (Mode 3: Teaching Studio). | AppTab, ScreenMode, PageView |
| **LanguageLocale** | State / Status | Active bilingual user interface locale managed by `I18nService`: `el` (Greek, default) or `en` (English). Switching updates UI labels, solver diagnostics, error messages, and educational derivations instantaneously without losing workspace state. | LangCode, TranslationMode |
| **DialogSession** | Business Object | A typed modal overlay instance managed in a LIFO stack by `DialogService` and rendered by `DialogHostComponent` inside `DialogShellComponent`, supporting custom input `DIALOG_DATA`, typed completion result `DialogRef<TResult>`, focus restoration, and an optional synchronous `DismissGuard`. | ModalWindow, Popup |
| **DismissGuard** | Policy / Rule | A predicate attached to an active `DialogSession` that blocks backdrop-click or `Escape`-key dismissal when unsaved or invalid modal configuration state exists. | CloseBlocker |
| **ToastNotification** | Business Object | A transient, non-blocking status banner managed by `NotificationService` and rendered by `NotificationHostComponent` with severity (`info`, `success`, `warning`, `error`), content-based deduplication, and a 5000 ms auto-dismiss timer. | Snackbar, AlertBanner |

---

## 4. Mode 1 Continuum 3D/2D Viewport Terms (Three.js)

| Canonical Term | Category | Definition | Aliases / Discouraged |
| --- | --- | --- | --- |
| **ContinuumViewport** | Business Object | The Three.js WebGL interactive canvas in Mode 1 that renders 2D planar and 3D volumetric conformal meshes (`Tri3`, `Quad4`, `Tet4`, `Hex8`), boundary element facets, scalar colormaps, deformed geometry, vector glyphs, and HUD overlays. | ThreeCanvas, SceneView |
| **CameraProjectionMode** | State / Status | Camera projection matrix mode in `ContinuumViewport`: `Perspective` (45° FOV default for 3D depth perception) or `Orthographic` (parallel projection for true-scale 2D/3D engineering elevation and plan inspection). | ViewMode, CameraType |
| **MeshRenderMode** | State / Status | Visual representation of mesh topology: `SurfaceSolid` (shaded faces with vertex colors), `WireframeOverlay` (element edge segments over shaded faces), or `VolumeEdgesOnly` (unshaded structural wireframe). | DisplayStyle, ShadingMode |
| **ClippingPlaneFilter** | Business Object | An interactive 3D sectioning plane defined by normal axis (`X`, `Y`, `Z`, or camera-aligned), signed normalized offset $s \in [-1, 1]$ across the mesh bounding box, and flip direction, enabling inspection of internal 3D stress fields and cavity inclusions. | SectionCut, VolumeSlicer |
| **BoundaryFacetPick** | Workflow / Action | Interactive raycast selection of a 2D boundary edge or 3D boundary facet in `ContinuumViewport` that resolves the hit element's `BoundaryTag` (`Left`, `Right`, `Top`, `Bottom`, `Front`, `Back`, `Hole`, `Default`) and opens or updates boundary condition assignment. | FaceClick, RaycastHit |
| **BoundaryConditionAssignment** | Business Object | Physics prescription bound to a picked `BoundaryTag` or facet group: `Dirichlet` (prescribed displacement $\bar{u}_i$), `Neumann` (prescribed traction $\bar{t}_i$), or `Robin` (radiating/impedance interface $\mathbf{t} + \mathbf{Z}\mathbf{u} = \mathbf{g}$). | SupportLoadTag, BCType |
| **ScalarFieldChannel** | State / Status | Physical scalar field mapped to vertex colors on the mesh: `Material` (spatial stiffness/property distribution $\mu(\mathbf{x})$ or $E(\mathbf{x})$), `VonMises` (equivalent stress $\sigma_{\text{vM}}$ in MPa), `PrincipalStress` ($\sigma_1, \sigma_3$ in MPa), or `Displacement` (magnitude $\|\mathbf{u}\|$ in mm). | ColormapType, ActiveField |
| **ColormapPalette** | Policy / Rule | Continuous scientific color transfer function mapping normalized scalar values $t \in [0, 1]$ to RGB triples: `Turbo` (polynomial high-contrast default), `Viridis` (perceptually uniform), `Coolwarm` (diverging blue-to-red), or `Jet` (legacy rainbow). | ColorScale, GradientTheme |
| **DeformationScalePolicy** | Policy / Rule | Geometric exaggeration multiplier $\alpha_{\text{def}} \in [1, 20000]$ (with standard presets $\times 1, \times 10, \times 100, \times 1000$ and `Auto` normalization to $8\%$ of maximum model dimension $L_{\max}$) applied to nodal displacements $\mathbf{x}_{\text{def}} = \mathbf{x}_0 + \alpha_{\text{def}}\mathbf{u}$. | WarpFactor, DispMultiplier |
| **VectorGlyphOverlay** | Business Object | Instanced 3D/2D arrow glyphs anchored at boundary facet centroids or nodes depicting applied tractions $\bar{\mathbf{t}}$ and support reaction tractions $\mathbf{r}$, scaled proportionally to vector magnitude and colored by physical role. | ForceArrows, TractionVectors |
| **PlaybackTimelineController** | Business Object | Interactive temporal scrubbing bar in `ContinuumViewport` supporting `TimeDomain` transient frame stepping ($t_k \in [0, T_{\text{total}}]$) and `LaplaceTalbot` harmonic phase oscillation ($\theta \in [0^\circ, 360^\circ)$) with play/pause, frame-index slider, and synchronized dock cursor. | AnimationBar, TimeSlider |

---

## 5. Bottom Engineering Dock Terms

| Canonical Term | Category | Definition | Aliases / Discouraged |
| --- | --- | --- | --- |
| **EngineeringDock** | Business Object | Collapsible bottom analytics panel providing synchronized SVG time-history charts, live solver convergence residual plots, and parametric benchmark sweep comparisons, plus publication SVG and tabular CSV export. | BottomDrawer, PlotPanel |
| **ResponseHistoryCurve** | Metric / Reporting | Multi-series 2D SVG time-series plot comparing transient displacement $u_y(t)$ (e.g., Laplace-Talbot contour inversion vs. Newmark-$\beta$ direct integration) against the analytical Euler-Bernoulli static reference line $\delta_{\text{EB}} = PL^3/(3EI)$, interactive with the `PlaybackTimelineController` cursor. | TimeHistoryPlot, WaveChart |
| **ConvergenceResidualTrace** | Metric / Reporting | Live-streamed logarithmic/tabular trace of iterative solver relative residuals $\|R^{(k)}\|/\|F\|$ and residual norms $\|r^{(k)}\|$ across iterations $k$, updated via Tauri IPC events. | ResidualPlot, SolverLog |
| **ParametricSweepChart** | Metric / Reporting | Comparative SVG plot generated during `cap-parametric-benchmarking` runs that charts peak deflection, stress concentration factor, or $L_2$ error across graded material parameters ($\alpha$, inclusion ratio, mesh refinement). | BenchmarkPlot, SweepGraph |

---

## 6. Shared 2D SVG Viewport & Diagram Primitives (Mode 2 & Mode 3)

| Canonical Term | Category | Definition | Aliases / Discouraged |
| --- | --- | --- | --- |
| **ViewportTransform2D** | Value Object | Immutable 2D affine coordinate transform between engineering world coordinates $(x, y)$ in metres (with $+x$ right, $+y$ up, counter-clockwise rotation positive) and SVG screen coordinates $(u, v)$ in pixels (with $+u$ right, $+v$ down): $u = u_0 + s \cdot x$, $v = v_0 - s \cdot y$. | ScreenMatrix, PanZoomState |
| **TensionSideDiagram** | Policy / Rule | Mandatory Greek/European structural engineering diagram plotting convention: bending moment $M(x)$ is positive when the local bottom ($-y$) fibre is in tension and its ordinate is always plotted perpendicular to the member axis on the tension side without sign-flip ambiguity. | MomentCurve, BMD |
| **InternalForceDiagramSet** | Business Object | Analytical SVG polygon/path overlays for a 2D member's axial force $N(x)$, shear force $Q(x)$, and bending moment $M(x)$ (or `Envelope` min/max bounds), rendered perpendicular to the member's local axis using canonical semantic color tokens. | MQNDiagrams, ForcePlots |
| **SemanticColorTokens** | Policy / Rule | Shared visual palette across all viewports: `#3b82f6` (Blue: Fixed support / Dirichlet / positive M), `#ef4444` (Red: Applied load / Neumann traction / negative M / truss compression), `#10b981` (Emerald: Pinned/Bottom boundary / truss tension), `#f59e0b` (Amber: Top boundary / Newmark-$\beta$ / Q shear), `#a855f7` (Purple: Cavity / Static EB reference / Envelope). | ThemePalette, ColorMapTokens |

---

## 7. Critical Semantic Distinctions

1. **`ContinuumViewport` (Three.js WebGL) vs. Shared 2D SVG Canvas Primitives:**
   - `ContinuumViewport` renders 2D/3D continuum meshes (`MeshData`) with continuous vertex-interpolated scalar fields (`ScalarFieldChannel`).
   - Shared 2D SVG primitives (`ViewportTransform2D`, `TensionSideDiagram`) render discrete 1D/2D skeletal structural models (`StructuralModel`) in Mode 2 and Mode 3 with crisp vector diagrams and KaTeX-aligned annotations.
2. **Physical Displacement $\|\mathbf{u}\|$ vs. Visual Deformed Position $\mathbf{x}_{\text{def}}$:**
   - Colorbars, tooltips, and dock charts always report true physical quantities (mm, MPa, kN, kNm). `DeformationScalePolicy` ($\alpha_{\text{def}}$) mutates only the rendered vertex coordinates $\mathbf{x}_{\text{def}} = \mathbf{x}_0 + \alpha_{\text{def}}\mathbf{u}$, never the underlying simulation arrays.
3. **`BoundaryFacetPick` (Mode 1 Continuum) vs. Sketcher Selection (Mode 2 Frame):**
   - `BoundaryFacetPick` performs 3D raycasting against `MeshData.boundary_elements` to assign continuum `Dirichlet`, `Neumann`, or `Robin` conditions by `BoundaryTag`. Mode 2 node/member selection operates on `StructuralModel` entities owned by `cap-frame-analysis-engine`.
4. **`DialogSession` vs. Inspector Sidebar:**
   - High-frequency modeling controls (mesh divisions, deformation scale, colormap selector, diagram toggles) live in non-blocking sidebars or HUD overlays. Advanced or multi-parameter configurations (soil impedance medium $Z(s)$, solver taxonomy guide, National Annex tables) use `DialogSession` via `DialogService`.

## 8. Open Terminology Issues

- **None.** All vocabulary aligns with [`cross-capability-conventions.md`](../cross-capability-conventions.md), [`docs/prd.md`](../../prd.md), and the implemented Angular 22 codebase.
