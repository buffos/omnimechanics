# Canonical Use-Case & Application-Service Model: Interactive 3D Visualization Canvas & UI (`cap-interactive-canvas`)

- **Capability Owner:** [`cap-interactive-canvas`](../../../.okf/capabilities/cap-interactive-canvas.md)
- **Upstream References:** [`prd.md`](prd.md), [`canonical-domain-model.md`](canonical-domain-model.md), [`cross-capability-conventions.md`](../cross-capability-conventions.md)
- **Status:** Canonical Reference (2026-11-01)

---

## 1. Purpose & Application-Layer Goals

This document defines the canonical commands, queries, application service groupings, orchestration responsibilities, and end-to-end use-case chains for `cap-interactive-canvas`. It sits between the domain model (`WorkspaceShellState`, `ContinuumSceneState`, `EngineeringDockState`, `ViewportTransform2D`) and the concrete UI components / Tauri IPC adapters.

---

## 2. Application Service Groupings

| Application Service | Owned Aggregate / Kernel | Core Responsibilities |
| --- | --- | --- |
| **WorkspaceShellOrchestrator** (`AppComponent` + `I18nService` + `DialogService` + `NotificationService`) | `WorkspaceShellState` | Switches active workspace mode (`continuum`, `structural`, `teaching`), toggles `el`/`en` locale, manages stacked modal `DialogSession` lifecycles with `DismissGuard`, and dispatches deduplicated `ToastNotification` messages. |
| **ContinuumViewportApplicationService** (`ViewportComponent` + `colormaps.ts`) | `ContinuumSceneState` | Orchestrates Three.js camera projection & spherical controls, mesh buffer generation, 3D dynamic clipping plane slicing, raycast boundary face/edge picking (`Dirichlet`/`Neumann`/`Robin`), scalar field colormaps, deformation scaling, vector glyphs, and harmonic/transient playback timers. |
| **EngineeringDockApplicationService** (`BottomDockComponent`) | `EngineeringDockState` | Computes response history SVG paths ($u_y(t)$ Talbot, Newmark-$\beta$, Static Euler-Bernoulli), synchronizes frame scrubbing with the viewport, displays live convergence residuals and parametric sweeps, and exports CSV/SVG artifacts. |
| **SharedViewport2DKernelService** (`ViewportTransform2D` + `TensionSideDiagram` helpers) | Shared 2D Kernel | Pure coordinate transformation (`worldToScreen`, `screenToWorld`, `fitBounds`) and tension-side internal force polygon generation ($M, Q, N$, envelopes) shared by Mode 2 and Mode 3. |

---

## 3. Canonical Commands (State-Mutating Intents)

### 3.1 Workspace Shell & Shared Infrastructure Commands

| Command | Input Parameters | Preconditions | Orchestration & Outcome |
| --- | --- | --- | --- |
| **`SwitchWorkspaceMode`** | `{ targetMode: 'continuum' \| 'structural' \| 'teaching' }` | None | Updates `activeWorkspaceMode` signal; pauses active viewport timers if leaving Mode 1 while preserving all mode states in memory. |
| **`ToggleLanguageLocale`** | `{ locale?: 'el' \| 'en' }` | None | Switches `I18nService.currentLang` (`el` ↔ `en`); triggers immediate reactive re-evaluation of all translated UI labels, diagram legends, and diagnostic banners. |
| **`OpenModalDialog`** | `{ component, config: DialogConfig<TData> }` | None | Creates child `EnvironmentInjector` with `DIALOG_REF` and `DIALOG_DATA`, locks body scroll on first entry, pushes `DialogSession` onto LIFO stack, and returns `DialogHandle<TResult>`. |
| **`CloseOrDismissModalDialog`** | `{ id: number, mode: 'close' \| 'dismiss', result?: TResult }` | `id` matches `dialogStack.at(-1)` | For `dismiss`: checks `dismissible !== false` and `dismissGuard() === true`. Destroys child injector, resolves `closed` promise, unlocks scroll if stack empty, and restores focus to trigger element. |
| **`ShowToastNotification`** | `{ message: string, type: 'info' \| 'success' \| 'warning' \| 'error' }` | None | Checks if `(message, type)` is already active; if so, resets its 5000 ms timer (idempotent deduplication). Otherwise appends toast and schedules auto-dismissal. |

### 3.2 Mode 1 Continuum Viewport Commands

| Command | Input Parameters | Preconditions | Orchestration & Outcome |
| --- | --- | --- | --- |
| **`UpdateCameraView`** | `{ action: 'orbit' \| 'pan' \| 'zoom' \| 'reset' \| 'setProjection', delta?: { dx, dy, wheel }, projection?: 'Perspective' \| 'Orthographic' }` | `ContinuumViewport` initialized | Updates spherical coordinates (clamping $\phi \in [0.05, \pi-0.05]$, $r \in [0.5, 250]$), switches camera matrix (`Perspective` ↔ `Orthographic`), or auto-fits bounding box on `reset`. |
| **`SetMeshRenderLayers`** | `{ showColormap?: boolean, showWireframe?: boolean, showBoundaries?: boolean, showNodes?: boolean, showVectorGlyphs?: boolean }` | Active `MeshData` present | Updates layer visibility flags and toggles Three.js scene group visibility without rebuilding geometry buffers when only visibility flags change. |
| **`UpdateClippingPlane`** | `{ enabled: boolean, axis: 'X' \| 'Y' \| 'Z', normalizedOffset: number, invertNormal?: boolean }` | Active 3D `MeshData` present | Computes world-space clipping plane $\mathbf{n}\cdot\mathbf{x} + d = 0$ from `normalizedOffset` $\in [-1, 1]$ across mesh bounding box and updates `renderer.localClippingEnabled` and material clipping planes. |
| **`PickBoundaryFacet`** | `{ screenX: number, screenY: number }` | `showBoundaries() === true` | Casts ray from camera through normalized device coordinates; resolves hit facet's `BoundaryTag`, highlights matching boundary group, and opens/updates `BoundaryConditionAssignment` (`Dirichlet`, `Neumann`, `Robin`). |
| **`AssignBoundaryConditionToTag`** | `{ tag: BoundaryTag, condition: { kind: 'Dirichlet' \| 'Neumann' \| 'Robin', values: Record<string, number> } }` | Valid `BoundaryTag` on current mesh | Binds the physical boundary prescription to `tag`, updates `VectorGlyphOverlay` arrows and legend badges, and marks simulation state ready for solve. |
| **`SelectScalarFieldAndPalette`** | `{ channel: ScalarFieldChannel, palette: ColormapPalette }` | `MeshData` present | Recomputes normalized vertex colors using `ColormapTransferPolicy` (`Turbo`, `Viridis`, `Coolwarm`, `Jet`) and updates the HUD colorbar range and unit. |
| **`ConfigureDeformationScale`** | `{ mode: 'Auto' \| 'Manual', scaleFactor?: number }` | Simulation result present | Applies `AutoScaleDeformationPolicy` ($8\%$ of $L_{\max}$) or manual preset ($\times 1 \dots \times 20000$), updating displaced vertex buffer $\mathbf{x}_{\text{def}} = \mathbf{x}_0 + \alpha_{\text{def}}\mathbf{u}$. |
| **`ControlTemporalPlayback`** | `{ action: 'play' \| 'pause' \| 'scrubFrame' \| 'scrubPhase', value?: number }` | `Transient` or `Harmonic` result present | Starts/stops animation interval timer or jumps directly to frame index $k$ / phase angle $\theta$, synchronizing both 3D mesh deformation and the `EngineeringDock` cursor. |

### 3.3 Bottom Engineering Dock Commands

| Command | Input Parameters | Preconditions | Orchestration & Outcome |
| --- | --- | --- | --- |
| **`SwitchDockTabOrCollapse`** | `{ tab?: 'history' \| 'diagnostics' \| 'sweep', isExpanded?: boolean }` | None | Switches active dock view and expands/collapses the bottom drawer. |
| **`ScrubHistoryChartToTime`** | `{ clickScreenX: number, svgClientWidth: number }` | `TransientSimulationResult` present | Maps SVG click X coordinate to normalized time $t \in [0, T_{\max}]$, finds nearest frame index $k$, and invokes `ControlTemporalPlayback({ action: 'scrubFrame', value: k })`. |
| **`ExportDockArtifact`** | `{ format: 'csv' \| 'svg' }` | Transient or sweep data present | Serializes time-series data (`time_s, talbot_displacement_mm, newmark_displacement_mm, static_euler_bernoulli_mm`) to UTF-8 CSV or serializes `#transient-response-svg` vector DOM to `.svg` file download. |

---

## 4. Canonical Queries (Read-Only Projections)

| Query | Input | Output Projection |
| --- | --- | --- |
| **`GetActiveSimulationViewModel`** | Current regime (`Static`, `TimeDomain`, `LaplaceTalbot`), results, frame/phase | `ActiveSimData` (`type`, `time`, `totalTime`, `displacements`, `von_mises_stresses`, `max_displacement`, `max_von_mises`, `converged`, `iterations`, `execution_time_ms`, `mode`, `history`). |
| **`GetColorbarLegendModel`** | `activeChannel`, `activePalette`, `ActiveSimData`, `MaterialFieldResponse` | `{ title, min, max, mid, unit, paletteGradientCss }` or `null` when no field is loaded. |
| **`GetDetectedBoundaryTagsLegend`** | `MeshData` | Array of `{ name: BoundaryTag, hex: string, count: number, assignedCondition?: BoundaryConditionAssignment }`. |
| **`GetResponseHistoryPlotModel`** | `talbotResult`, `newmarkResult`, `mesh`, `materialRef`, `loadMagnitude`, `currentFrameIndex` | `{ plotBounds: { maxT, maxDispMm }, staticEulerBernoulliMm, talbotSvgPath, newmarkSvgPath, newmarkPoints, staticLineY, currentCursorX }`. |
| **`Project2DMemberDiagramPath`** | `member: Element`, `diagramSamples`, `diagramKind: 'M' \| 'Q' \| 'N' \| 'Envelope'`, `transform: ViewportTransform2D` | Closed SVG path `d` string with ordinates offset on the tension side (for $M$) and semantic CSS class (`diagram-m`, `diagram-q`, `diagram-n`, `diagram-truss-tension`, `diagram-truss-compression`). |

---

## 5. Transaction Boundaries, Idempotency & Failure Model

### Transaction & Consistency Expectations
- **Single-Aggregate Synchronous UI Mutations:** All viewport, dock, and dialog commands execute synchronously within Angular's reactive signal graph (`OnPush` change detection) and schedule WebGL buffer updates inside reactive `effect()` blocks.
- **Stale State Cancellation:** When `MeshService.generateMesh()` or Mode 2 model `revision` increments, dependent simulation overlays are atomically cleared before new geometry renders.

### Idempotency Rules
- `ShowToastNotification` is strictly idempotent over active `(message, type)` tuples within the 5000 ms window.
- `SelectScalarFieldAndPalette`, `ConfigureDeformationScale`, and `UpdateClippingPlane` are pure state replacements—repeating the same command produces identical WebGL vertex buffers and plane uniforms.

### Failure Categories
| Failure Category | Canonical Error Code | Handling in Application Layer |
| --- | --- | --- |
| **Dialog Dismiss Guard Blocked** | `DIALOG_DISMISS_BLOCKED` | Keeps top-most `DialogSession` open; emits warning `ToastNotification` in active locale (`el`/`en`). |
| **Empty or Degenerate Field Span** | `FIELD_SPAN_DEGENERATE` | Handled gracefully by `INV-4` (fallback span $1.0$); never throws or writes `NaN` to WebGL vertex buffers. |
| **Raycast Miss on Canvas Click** | `RAYCAST_NO_BOUNDARY_HIT` | Clears active boundary facet highlight without mutating existing `BoundaryConditionAssignment` state. |
| **Export Artifact Unavailable** | `EXPORT_DATA_EMPTY` | Export buttons (`CSV`, `SVG`) are disabled when no transient/sweep result exists; programmatic invocation returns early with an info toast. |
| **WebGL Context Creation / Loss** | `WEBGL_CONTEXT_UNAVAILABLE` | Renders non-crashing fallback banner inside `.viewport-container` and surfaces an error `ToastNotification`. |

---

## 6. End-to-End Use-Case Chains

1. **Chain 1 — 3D Inhomogeneous Cavity Slicing & Stress Inspection (Mode 1):**
   `SwitchWorkspaceMode('continuum')` → generate `BlockWithCavity3D` mesh → `PickBoundaryFacet` (assign `Dirichlet` on `Bottom`, `Neumann` traction on `Top`) → run hybrid solver → `SelectScalarFieldAndPalette('VonMises', 'Turbo')` → `UpdateClippingPlane({ enabled: true, axis: 'Z', normalizedOffset: 0.0 })` → inspect internal cavity stress concentration.
2. **Chain 2 — Transient Wave Scrubbing & Publication Export (Mode 1):**
   Run `TimeDomain` simulation (Talbot + Newmark-$\beta$) → `SwitchDockTabOrCollapse({ tab: 'history', isExpanded: true })` → `ScrubHistoryChartToTime` on peak deflection → observe synchronized 3D deformed mesh (`Auto` scale) and dock cursor → `ExportDockArtifact('svg')` and `ExportDockArtifact('csv')`.
3. **Chain 3 — Mode 2 to Mode 3 Shared 2D Diagram Hand-Off:**
   Sketch frame in Mode 2 (`ViewportTransform2D`, live DSM `TensionSideDiagram` for $M$) → click *Open step-by-step* → `SwitchWorkspaceMode('teaching')` → step through derivation in Greek (`el`) with identical coordinate orientation ($+y$ up) and tension-side moment plotting.
