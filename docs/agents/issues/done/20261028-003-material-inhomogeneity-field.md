## Issue Metadata

- Issue number: `003`
- Owning capability node: `/.okf/capabilities/cap-material-field.md`
- Artifact root: `docs/architecture/cap-material-field/`
- Issue file: `docs/agents/issues/done/20261028-003-material-inhomogeneity-field.md`
- Category: `feature`
- Execution type: `AFK`
- Review gate: `visual-review`
- Suggested state: `done`

## Parent Artifacts

- `.okf/project.md`
- `.okf/capabilities/cap-material-field.md`
- `.okf/capabilities/cap-geometry-meshing.md`
- `.okf/capabilities/cap-interactive-canvas.md`

## What to build

Implement the Open-Closed Material Modeling & Spatial Inhomogeneity Engine in the Rust core. Decouple the baseline homogeneous reference parameters $(E_0, \nu_0, \rho_0)$ from spatial perturbation strategies:
1. Directional Gradient (FGM): $\mu(y) = \mu_0 (1 + \alpha y/L)^p$
2. Analytical Math Expressions evaluated at coordinates $(x, y, z)$
3. Localized Inclusions / Voids: discrete subdomains with distinct elastic moduli
4. Vecchio-Collins Non-Linear Concrete Model (Modified Compression Field Theory - MCFT)
5. J2 von Mises Elastoplasticity (Yield stress, hardening, return mapping)

Expose Tauri IPC command `evaluate_material_field` that samples the field at all mesh nodes/elements and returns nodal scalars and bounds for 3D/2D Three.js vertex-color colormap rendering with an interactive HUD colorbar legend.

## Acceptance criteria

- [x] Rust `MaterialField` trait adhering to the Open-Closed Principle, decoupling reference properties $(E_0, \nu_0, \rho_0)$ from spatial perturbations.
- [x] Concrete strategy implementations in Rust:
  - `DirectionalGradientField` (FGM linear/quadratic/exponential along X, Y, or Z).
  - `MathExpressionField` (spatial expression evaluator at $(x, y, z)$).
  - `InclusionField` (spherical/cylindrical inclusions with distinct moduli $(E_1, \nu_1)$ and interface transition).
  - `VecchioCollinsField` (MCFT compressive softening $\beta$, tension stiffening, and secant modulus evaluation).
  - `J2PlasticityField` (von Mises yield function $\Phi = \sqrt{3 J_2} - \sigma_y$).
- [x] Field sampling engine computing nodal and quadrature-point values across any conformal `MeshData`.
- [x] Tauri IPC command `evaluate_material_field` returning scalar arrays, bounds $[v_{\min}, v_{\max}]$, and reference values.
- [x] Angular `MaterialService` managing active material configuration and IPC communication.
- [x] Three.js `ViewportComponent` enhanced to render scalar material fields as smooth vertex colormaps (Turbo / Viridis / Jet / Coolwarm) across 2D and 3D meshes.
- [x] Interactive HUD Colorbar displaying property name ($E$, $\mu$, $\sigma_y$), units (MPa / GPa), and numerical scale.
- [x] Rust unit tests verifying all 5 material strategies and bounds.

## Artifact sync required

- Application PRD: `none - capability foundation`
- Application architecture summary: `none - capability foundation`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-material-field.md`
- Issue registry: `required`; node `issues:` reference: `required`
- Reason/no-impact decision: `Grounding issue for material modeling and inhomogeneity field engine.`

## Human review gate

- Visual review: User verified in the desktop window that selecting different inhomogeneity strategies (Gradient, Inclusion, Vecchio-Collins, etc.) dynamically updates the mesh vertex colors with the colormap and displays the HUD colorbar with correct bounds. Confirmed: "Ολα τώρα είναι σούπερ".

## Blocked by

None - completed

## Artifact anchors

- Capability: `cap-material-field`
- Capability: `cap-interactive-canvas`
- Capability: `cap-solver-core`

## Acceptance scenarios addressed

- Inhomogeneity field evaluation, MCFT concrete degradation, and J2 plasticity rendering.

## Verification obligations

- mode: `when-supported`
- surfaces:
  - backend-boundary: `passed` (10/10 Rust unit tests for mesh and material strategies)
  - frontend-integration: `passed` (Angular build and IPC integration)
  - end-to-end: `passed` (Colormap rendering, live slider updates, and HUD colorbar confirmed in visual review)
