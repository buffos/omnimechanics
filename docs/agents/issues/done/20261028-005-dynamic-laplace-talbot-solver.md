## Issue Metadata

- Issue number: `005`
- Owning capability node: `/.okf/capabilities/cap-solver-core.md`
- Artifact root: `docs/architecture/cap-solver-core/`
- Issue file: `docs/agents/issues/done/20261028-005-dynamic-laplace-talbot-solver.md`
- Category: `feature`
- Execution type: `AFK`
- Review gate: `visual-review`
- Suggested state: `done`

## Parent Artifacts

- `.okf/project.md`
- `.okf/capabilities/cap-solver-core.md`
- `.okf/capabilities/cap-geometry-meshing.md`
- `.okf/capabilities/cap-material-field.md`
- `.okf/capabilities/cap-interactive-canvas.md`

## What to build

Implement the dynamic elastodynamic multi-solver engine in Rust (`src-tauri/src/solver/`):
1. **FEM Consistent Mass Matrix & Dynamic Operator (`fem_solver.rs`):**
   - Assemble consistent mass matrix $\mathbf{M}_{\text{FEM}} = \sum_e \int_{\Omega_e} \rho(\mathbf{x}) \mathbf{N}^T \mathbf{N} d\Omega$ with spatially varying density $\rho(\mathbf{x})$.
   - Implement complex matrix-vector evaluator $\mathbf{v} \mapsto (\mathbf{K}_{\text{FEM}} + s^2 \mathbf{M}_{\text{FEM}}) \mathbf{v}$ for complex Laplace parameter $s \in \mathbb{C}$ or harmonic frequency $s = i\omega$.
2. **Dynamic BEM Fundamental Kernels in Transformed Domain (`dynamic_bem.rs`):**
   - Implement complex Kelvin/Stokes dynamic Green's tensors in 2D using modified Bessel functions $K_0(z)$ and $K_1(z)$ for longitudinal ($c_p$) and shear ($c_s$) wave speeds.
   - Formulate and factorize the homogeneous boundary operator $\mathbf{E}(s)$ at sample points.
3. **BEM-Preconditioned GMRES Solver (`krylov_solver.rs`):**
   - Formulate left-preconditioned Krylov subspace iteration: $(\mathbf{I} + \mathbf{E}(s)^{-1} \mathbf{K}(s)) \hat{\mathbf{u}} = \mathbf{E}(s)^{-1} \hat{\mathbf{F}}$.
   - Implement complex GMRES with Arnoldi orthonormalization.
   - Ensure strictly matrix-free operation: FEM evaluates internal/inertial forces, BEM back-substitutes boundary corrections.
4. **Talbot Contour Quadrature & Time Inversion (`talbot.rs`):**
   - Parameterize modified Talbot contour in complex $s$-plane: $s(\theta) = \sigma + \lambda ((1 - \sin\theta) + i\beta\theta)$ for $\theta \in (-\pi, \pi)$.
   - Generate $N_q$ quadrature points ($N_q = 16 \dots 24$) and complex integration weights $w_k$.
   - Parallel solve across all $N_q$ points via `rayon`.
   - Synthesize transient time response: $\mathbf{u}(\mathbf{x}, t) = \sum_{k=1}^{N_q} \text{Re}(w_k e^{s_k t} \hat{\mathbf{u}}(\mathbf{x}, s_k))$.
5. **IPC Commands & Frontend Integration:**
   - Expose `run_harmonic_simulation` (frequency $f$, amplitude, phase lag) and `run_transient_simulation` (time duration $T$, time steps, wave pulses).
   - Add Dynamic Controls to Angular sidebar: Mode switch ("Harmonic Steady-State" vs "Transient Wave Talbot"), Frequency input, Pulse duration.
   - Three.js Viewport animation: Time playback controls (Play/Pause, scrub slider, loop) showing elastodynamic wave propagation and stress waves.

## Acceptance criteria

- [x] Rust FEM mass matrix assembly $\mathbf{M}_{\text{FEM}}$ for 2D Tri3/Quad4 and 3D Tet4/Hex8.
- [x] Rust dynamic BEM kernels with complex modified Bessel functions $K_0, K_1$ for transformed domain $s \in \mathbb{C}$.
- [x] Rust complex GMRES solver with BEM preconditioning converging unconditionally in the presence of strong dynamic inertia.
- [x] Rust Talbot contour integrator recovering transient elastodynamic time-history from independent complex frequency solves.
- [x] Multi-threaded execution of Talbot quadrature points using `rayon`.
- [x] Tauri IPC commands supporting both Harmonic Steady-State and Transient Wave modes.
- [x] Angular UI with dynamic frequency/time controls and Three.js time-scrubbing wave playback.
- [x] Rust unit tests verifying mass matrix conservation, dynamic Bessel kernel evaluation, and GMRES convergence.

## Artifact sync required

- Application PRD: `none - capability foundation`
- Application architecture summary: `none - capability foundation`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-solver-core.md`
- Issue registry: `required`; node `issues:` reference: `required`
- Reason/no-impact decision: `Major dynamic elastodynamics extension to cap-solver-core.`

## Human review gate

- Visual review: User verifies in the desktop window that running a Harmonic Steady-State simulation displays the oscillating amplitude/phase response, and running a Transient Talbot simulation animates the wave propagation through the beam with the time slider.
- Verification status: Automated verification complete (21 Rust unit tests passing, Angular production build passing). Interactive playback and controls reviewed and approved.

## Blocked by

None - Issue 004 completed

## Artifact anchors

- Capability: `cap-solver-core`
- Capability: `cap-geometry-meshing`
- Capability: `cap-material-field`
- Capability: `cap-interactive-canvas`

## Verification obligations

- mode: `when-supported`
- surfaces:
  - backend-boundary: `when-supported` (Rust unit tests for dynamic kernels, mass matrices, and GMRES)
  - frontend-integration: `when-supported` (Angular build and IPC integration)
  - end-to-end: `visual-review` (Interactive wave playback and harmonic amplitude visualization)
