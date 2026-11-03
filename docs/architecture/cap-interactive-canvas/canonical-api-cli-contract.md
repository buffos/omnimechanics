# Canonical API, IPC & UI Contract: Interactive 3D Visualization Canvas & UI (`cap-interactive-canvas`)

- **Capability Owner:** [`cap-interactive-canvas`](../../../.okf/capabilities/cap-interactive-canvas.md)
- **Upstream References:** [`prd.md`](prd.md), [`canonical-domain-model.md`](canonical-domain-model.md), [`canonical-use-cases.md`](canonical-use-cases.md), [`cross-capability-conventions.md`](../cross-capability-conventions.md)
- **Status:** Canonical Reference (2026-11-01)

---

## 1. Purpose & Contract Goals

`cap-interactive-canvas` is the primary visual frontend and shared UI kernel of **OmniMechanics**. Its canonical external contract defines:
1. **Transport-Neutral Data Shapes & State Contracts** consumed and exposed by the Tri-Mode Workspace Shell, `DialogService`, `NotificationService`, `I18nService`, `ViewportComponent`, `BottomDockComponent`, and the Shared 2D SVG Viewport Kernel.
2. **Tauri IPC Command & Event Subscriptions** through which the canvas consumes mesh, material field, continuum solver, and live progress streams (`analysis://progress`) and serializes UI workspace state into `ProjectDocument.ui`.
3. **Canonical Component & Service API Surface** (Angular 22 signal inputs/outputs and pure TypeScript helper signatures) and **CLI / Headless Verification Mapping** for automated testing under Vitest.

---

## 2. Transport-Neutral Conventions & Identifier Rules

- **Units:** Physical quantities arriving over Tauri IPC follow SI conventions from [`cross-capability-conventions.md`](../cross-capability-conventions.md): coordinates in $\text{m}$, continuum displacements in $\text{m}$ (displayed in $\text{mm}$ via $\times 1000$), stresses in $\text{MPa}$ (or $\text{Pa}$ for $E_0$), forces in $\text{kN}$ (Mode 2/3) or $\text{N}$ (Mode 1), moments in $\text{kNm}$, time in $\text{s}$ (displayed in $\text{ms}$ or $\text{s}$).
- **Axes & Sign Conventions:**
  - **2D Structural World (`ViewportTransform2D`):** $+x$ right, $+y$ up, counter-clockwise rotation positive. Bending moment $M > 0$ when the local bottom ($-y$) fibre is in tension; diagrams are always plotted on the tension side.
  - **3D Continuum World (`ContinuumViewport`):** Right-handed $(x, y, z)$ Cartesian frame with spherical orbit camera $(r, \theta, \phi)$ centered on mesh bounding-box centroid $\mathbf{c}$.
- **Bilingual Error & Status Contract:** All user-visible errors conform to:
  ```ts
  export interface CanonicalIpcError {
    readonly code: string; // SCREAMING_SNAKE_CASE
    readonly message_el: string;
    readonly message_en: string;
    readonly details?: Record<string, unknown>;
  }
  ```

---

## 3. Canonical Payload & State Shapes

### 3.1 Workspace Shell, I18n, Dialog & Notification Contracts

```ts
export type WorkspaceMode = 'continuum' | 'structural' | 'teaching';
export type Language = 'el' | 'en';

export interface DialogConfig<TData = undefined> {
  readonly title: string;
  readonly eyebrow?: string;
  readonly description?: string;
  readonly data?: TData;
  readonly dismissible?: boolean; // default: true
  readonly size?: 'standard' | 'wide'; // default: 'standard'
}

export interface DialogRef<TResult> {
  readonly id: number;
  readonly closed: Promise<TResult | undefined>;
  close(result?: TResult): void;
  dismiss(): void;
  setDismissGuard(guard: () => boolean): void;
}

export type NotificationType = 'success' | 'error' | 'warning' | 'info';
export interface Notification {
  readonly id: number;
  readonly message: string;
  readonly type: NotificationType;
}
```

### 3.2 Mode 1 Continuum Viewport Contracts (Three.js)

```ts
export type BoundaryTag =
  | 'Left'
  | 'Right'
  | 'Top'
  | 'Bottom'
  | 'Front'
  | 'Back'
  | 'Hole'
  | 'Default';

export type ColormapType = 'Material' | 'VonMises' | 'PrincipalStress' | 'Displacement';
export type ColormapPalette = 'Turbo' | 'Viridis' | 'Coolwarm' | 'Jet';
export type CameraProjectionMode = 'Perspective' | 'Orthographic';
export type MeshRenderMode = 'SurfaceSolid' | 'WireframeOverlay' | 'VolumeEdgesOnly';

export interface ClippingPlaneConfig {
  readonly enabled: boolean;
  readonly axis: 'X' | 'Y' | 'Z';
  readonly normalizedOffset: number; // [-1.0, +1.0] across mesh bounding box
  readonly invertNormal: boolean;
}

export interface BoundaryConditionAssignment {
  readonly tag: BoundaryTag;
  readonly kind: 'Dirichlet' | 'Neumann' | 'Robin';
  readonly prescribedDisplacement?: [number | null, number | null, number | null];
  readonly prescribedTraction?: [number, number, number];
  readonly robinImpedance?: { eMediumMpa: number; nuMedium: number; rhoMedium: number };
}

export interface BoundaryFacetPickEvent {
  readonly tag: BoundaryTag;
  readonly boundaryElementIndex: number;
  readonly worldPoint: [number, number, number];
}
```

### 3.3 Shared 2D SVG Viewport & Diagram Contracts (Mode 2 & Mode 3)

```ts
export interface ViewportTransform2DState {
  readonly viewWidth: number;
  readonly viewHeight: number;
  readonly margin: number;
  readonly panX: number; // screen pixel offset
  readonly panY: number; // screen pixel offset
  readonly scale: number; // pixels per metre
  readonly refWorldX: number;
  readonly refWorldY: number;
}

export interface DiagramRenderOptions2D {
  readonly kind: 'M' | 'Q' | 'N' | 'EnvelopeM' | 'EnvelopeQ' | 'EnvelopeN';
  readonly maxAbsValue: number;
  readonly maxPixelOffset: number; // default: 70 px for beams/frames, 12 px for trusses
  readonly tensionSidePositiveM: true; // invariant: positive M plotted toward local -y fibre
}
```

### 3.4 UI Persistence Section (`ProjectDocument.ui` Handoff)

When `cap-project-persistence` saves or loads a `ProjectDocument`, `cap-interactive-canvas` contributes the `ui` section:

```ts
export interface ProjectUiSectionDto {
  readonly activeWorkspaceMode: WorkspaceMode;
  readonly language: Language;
  readonly continuumViewport?: {
    readonly cameraProjection: CameraProjectionMode;
    readonly activeColormapType: ColormapType;
    readonly activePalette: ColormapPalette;
    readonly deformationScale: number;
    readonly clippingPlane: ClippingPlaneConfig;
    readonly dockExpanded: boolean;
  };
  readonly structuralViewport2D?: {
    readonly panX: number;
    readonly panY: number;
    readonly zoomScale: number;
  };
}
```

---

## 4. Error Model & Canonical Error Codes

| Error Code | Category | Trigger Condition | User-Visible Behavior (`el` / `en`) |
| --- | --- | --- | --- |
| `DIALOG_DISMISS_BLOCKED` | Validation / Policy | `dismiss()` invoked while active `DialogSession.dismissGuard()` returns `false` | Dialog remains open; warning toast explains unsaved/invalid modal inputs. |
| `FIELD_SPAN_DEGENERATE` | Numerical Guard | Scalar field $\max - \min \le 10^{-9}$ | Handled silently via fallback span $1.0$ (`t = 0.0`); colorbar shows equal min/max without `NaN`. |
| `RAYCAST_NO_BOUNDARY_HIT` | Interaction | User clicks empty canvas space in picking mode | Clears active boundary highlight; no error toast shown. |
| `EXPORT_DATA_EMPTY` | Precondition | User invokes `exportCsv()` or `exportSvg()` with no transient/sweep series | Action button disabled in UI; if called programmatically, shows info toast. |
| `WEBGL_CONTEXT_UNAVAILABLE` | Infrastructure | Browser/WebView2 fails to initialize `THREE.WebGLRenderer` | Displays inline fallback card in viewport and dispatches `error` toast. |

---

## 5. Canonical Tauri IPC & Event Mapping

`cap-interactive-canvas` does not expose its own Rust solver commands; instead it binds to the platform IPC surface defined in [`cross-capability-conventions.md`](../cross-capability-conventions.md):

| Direction | Channel / Command | Owner Capability | Role in `cap-interactive-canvas` |
| --- | --- | --- | --- |
| Frontend → Rust | `generate_mesh` | `cap-geometry-presets` | Supplies `MeshData` (`nodes`, `volume_elements`, `boundary_elements`, `stats`) to `ViewportComponent`. |
| Frontend → Rust | `evaluate_material_field` | `cap-material-field` | Supplies `MaterialFieldResponse` (`values`, `min_val`, `max_val`, `unit`) for `Material` heatmap. |
| Frontend → Rust | `solve_continuum`, `solve_harmonic`, `solve_transient` | `cap-solver-core` | Supplies `SimulationResult`, `HarmonicSimulationResult`, `TransientSimulationResult` to viewport and dock. |
| Rust → Frontend Event | `analysis://progress` | `cap-solver-core` / `cap-frame-analysis-engine` | Streams `{ jobId, revision?, iteration, residualNorm, relativeError, status }` to live convergence HUD and `BottomDockComponent`. |

---

## 6. Headless / CLI & Vitest Verification Mapping

Because `cap-interactive-canvas` is a desktop UI capability, its transport-neutral logic is exposed as pure TypeScript functions and Angular services that execute headlessly in the `frontend-integration` Vitest suite (`npm test`):

| Canonical Operation | Pure TS / Service Entry Point | Verification Assertion |
| --- | --- | --- |
| Colormap evaluation | `evaluateColormap(palette, t)` (`colormaps.ts`) | Clamps $t \in [0, 1]$ and returns exact RGB triples in $[0, 1]^3$ for `Turbo`, `Viridis`, `Coolwarm`, `Jet`. |
| Auto-scale deformation | `computeAutoDeformationScale(nodes, maxDisp)` | Normalizes peak deflection to $8\%$ of $L_{\max}$, returning $\ge 1$ and rounding to multiples of 50 above 100. |
| 2D coordinate transform | `ViewportTransform2D` (`worldToScreen`, `screenToWorld`) | Preserves $+y$ up to $+v$ down inversion and exact round-trip identity $\text{screenToWorld}(\text{worldToScreen}(x, y)) = (x, y)$. |
| Tension-side diagram path | `buildTensionSideDiagramPath(member, samples, opts)` | Offsets positive $M > 0$ toward the local $-y$ (bottom) tension fibre and closes polygon onto member axis. |
| Modal & Toast lifecycle | `DialogService` / `NotificationService` | Enforces LIFO close, `DismissGuard` blocking, and 5000 ms toast deduplication. |

---

## 7. Parity Rules & Minimum First-Slice Surface

- **Cross-Mode Parity:** Mode 2 (`cap-sketcher-frame-modeler`) and Mode 3 (`cap-structural-mechanics-teaching`) must use the exact same `SemanticColorTokens`, `+y` up world coordinate orientation, and `TensionSideDiagram` sign rule so switching a model from Mode 2 to Mode 3 never flips a bending moment diagram.
- **Brownfield Baseline vs. Extensions:**
  - *Already implemented in codebase:* Header switcher (Mode 1/3), `I18nService` (`el`/`en`), `DialogService`/`DialogHostComponent`, `NotificationService`/`NotificationHostComponent`, `ViewportComponent` (Three.js orbit/pan/zoom, `Turbo` colormap, `Material`/`VonMises`/`Displacement` channels, deformation scaling, harmonic/transient playback), `BottomDockComponent` (Talbot vs. Newmark-$\beta$ vs. Static EB SVG plot, convergence table, CSV/SVG export), and `TeachingCanvasComponent` 2D SVG diagram rendering.
  - *Specified for completion in `cap-interactive-canvas` & Mode 2 integration:* Adding Mode 2 (`structural`) button to `HeaderComponent`, `Orthographic` camera toggle, 3D dynamic clipping plane slider, interactive raycast boundary facet picking (`Dirichlet`/`Neumann`/`Robin`), additional colormap palettes (`Viridis`, `Coolwarm`, `Jet`), vector glyph arrows, and extracting the shared 2D SVG transform/diagram helpers for Mode 2 reuse.
