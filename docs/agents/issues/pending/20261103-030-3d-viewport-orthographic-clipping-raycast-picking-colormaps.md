## Issue Metadata

- Issue number: `030`
- Owning capability node: `/.okf/capabilities/cap-interactive-canvas.md`
- Artifact root: `docs/architecture/cap-interactive-canvas/`
- Issue file: `docs/agents/issues/pending/20261103-030-3d-viewport-orthographic-clipping-raycast-picking-colormaps.md`
- Category: `feature`
- Execution type: `AFK`
- Review gate: `visual-review`
- Suggested state: `ready-for-agent`

## Parent Artifacts

- `docs/prd.md`
- `docs/architecture/application-architecture-summary.md`
- `docs/architecture/cross-capability-conventions.md`
- `.okf/capabilities/cap-interactive-canvas.md`
- `docs/architecture/cap-interactive-canvas/prd.md`
- `docs/architecture/cap-interactive-canvas/canonical-domain-model.md`
- `docs/architecture/cap-interactive-canvas/canonical-use-cases.md`
- `docs/architecture/cap-interactive-canvas/canonical-api-cli-contract.md`
- `docs/architecture/cap-interactive-canvas/acceptance-scenarios.md`
- `docs/architecture/cap-interactive-canvas/readiness-review.md`

## What to build

Implement **Vertical Slice 1** of the remaining `cap-interactive-canvas` specification: `Orthographic` vs. `Perspective` camera projection toggle, 3D dynamic clipping plane slider (`X`/`Y`/`Z` + normal inversion), interactive Three.js raycast boundary facet picking (`Dirichlet`, `Neumann`, `Robin`), multi-palette scientific colormaps (`Turbo`, `Viridis`, `Coolwarm`, `Jet`) with `PrincipalStress` channel and degenerate-span protection, and vector glyph arrows in `ViewportComponent`.

### 1. Camera Projection & 3D Dynamic Clipping Plane (`src/app/components/viewport/`):
- Support toggling between `PerspectiveCamera` and `OrthographicCamera` while preserving the spherical orbit target $\mathbf{c}$, polar clamp $\phi \in [0.05, \pi - 0.05]$, and `Reset View` framing (`SC-005`).
- Implement `ClippingPlaneConfig` controls (`enabled`, `axis: 'X' | 'Y' | 'Z'`, `normalizedOffset` $s \in [-1, 1]$, `invertNormal`) updating `renderer.localClippingEnabled = true` and `THREE.Plane` on mesh materials so users can slice into 3D cavity meshes to inspect internal stress and material fields (`SC-006`).

### 2. Interactive Raycast Boundary Facet Picking (`src/app/components/viewport/`):
- Add raycast picking (`THREE.Raycaster`) over `boundaryGroup` facets when boundary picking is active (`SC-007`).
- Clicking a boundary facet resolves its `BoundaryTag` (`Left`, `Right`, `Top`, `Bottom`, `Front`, `Back`, `Hole`), highlights all facets sharing that tag, emits `BoundaryFacetPickEvent`, and allows assigning/inspecting `BoundaryConditionAssignment` (`Dirichlet`, `Neumann`, `Robin`).
- Clicking empty canvas space (`RAYCAST_NO_BOUNDARY_HIT`) clears the selection highlight without mutating existing boundary assignments (`SC-007`).

### 3. Scientific Colormap Palettes, Principal Stress & Vector Glyphs (`src/app/components/viewport/colormaps.ts`):
- Implement `evaluateColormap(palette, t)` for `Turbo`, `Viridis`, `Coolwarm`, and `Jet` with degenerate-span protection ($\max - \min \le 10^{-9} \implies \text{span} = 1.0, t = 0.0$, zero `NaN`s) (`SC-008`), plus `PrincipalStress` scalar channel.
- Extract `computeAutoDeformationScale(nodes, maxDisp)` normalizing peak visual deflection to $8\%$ of $L_{\max}$ while never altering HUD/colorbar physical extrema (`SC-009`).
- Render 3D/2D `VectorGlyphOverlay` arrows (`THREE.ArrowHelper` / cones) for applied boundary tractions and support reactions.

## Acceptance criteria

- [ ] Users can toggle between `Perspective` and `Orthographic` camera projections, orbit with polar angle clamped to $[0.05, \pi - 0.05]$, and auto-fit 2D/3D meshes via `Reset View` (`SC-005`).
- [ ] Enabling the 3D clipping plane on axis `X`, `Y`, or `Z` and scrubbing $s \in [-1, 1]$ (or toggling `invertNormal`) slices the 3D volume mesh to reveal internal cavity surfaces and stress colors (`SC-006`).
- [ ] Clicking a boundary facet in the viewport highlights all facets with that `BoundaryTag` and emits `BoundaryFacetPickEvent` for BC assignment; clicking empty canvas clears the highlight cleanly (`SC-007`).
- [ ] Users can switch between `Turbo`, `Viridis`, `Coolwarm`, and `Jet` palettes across `Material`, `VonMises`, `PrincipalStress`, and `Displacement` channels with zero `NaN` values on uniform fields (`SC-008`), and changing deformation scale (`Auto`, `1000×`) never alters reported physical stress/displacement numbers (`SC-009`).
- [ ] Vitest unit tests (`viewport-colormaps.spec.ts`), `npx ng build`, and `npm run check:limits` pass with 0 errors.

## Artifact sync required

- Application PRD: `none - already covered in docs/prd.md`
- Application architecture summary: `none - matches docs/architecture/application-architecture-summary.md`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-interactive-canvas.md`
- Issue registry: `required`; node `issues:` reference: `required: .okf/capabilities/cap-interactive-canvas.md`
- Reason/no-impact decision: `Delivers the 3D/2D continuum viewport controls, clipping plane, raycast picking, and colormap palettes specified in cap-interactive-canvas.`

## Human review gate

`visual-review`: User loads a 3D `BlockWithCavity3D` mesh in Mode 1, toggles Orthographic projection, slices the cavity with the 3D clipping plane slider, clicks boundary facets to pick tags, and cycles through `Turbo`, `Viridis`, `Coolwarm`, and `Jet` colormaps.

## Blocked by

- None - can start immediately

## Artifact anchors

- PRD: `WF-2`, `WF-3`, `WF-4`, `BR-1`, `BR-2`, `FR-2`, `FR-3`, `FR-4`, `FR-5`
- Domain model: `ContinuumViewport`, `ClippingPlaneConfig`, `BoundaryFacetPickEvent`, `DeformationScalePolicy`
- Use cases: `UC-C03..UC-C07`
- Contract: `evaluateColormap`, `computeAutoDeformationScale`, `ClippingPlaneConfig`, `BoundaryFacetPickEvent`, error codes `FIELD_SPAN_DEGENERATE`, `RAYCAST_NO_BOUNDARY_HIT`

## Acceptance scenarios addressed

- `SC-005`: 3D/2D Camera Auto-Fit, Orbit Singularity Clamp & Orthographic Toggle
- `SC-006`: 3D Dynamic Clipping Plane Slicing Across Volumetric Cavity Mesh
- `SC-007`: Interactive Raycast Boundary Facet Picking & Background Miss Handling
- `SC-008`: Scalar Field Colormap Transfer & Degenerate Homogeneous Span Guard
- `SC-009`: Deformation Exaggeration (`Auto` & Manual Presets) Preserves Physical Readouts

## Verification obligations

- Policy source: `/.okf/project.md`

| Scenario | Backend boundary | Frontend integration | End-to-end journey |
|---|---|---|---|
| `SC-005` | `not-applicable` | `planned` | `catalog-only` |
| `SC-006` | `not-applicable` | `planned` | `catalog-only` |
| `SC-007` | `not-applicable` | `planned` | `catalog-only` |
| `SC-008` | `not-applicable` | `planned` | `catalog-only` |
| `SC-009` | `not-applicable` | `planned` | `catalog-only` |
