## Issue Metadata

- Issue number: `006`
- Owning capability node: `/.okf/capabilities/cap-solver-core.md`
- Artifact root: `docs/architecture/cap-solver-core/`
- Issue file: `docs/agents/issues/done/20261029-006-dynamic-robin-schwarz-coupling.md`
- Category: `feature`
- Execution type: `AFK`
- Review gate: `visual-review`
- Suggested state: `done`

## Parent Artifacts

- `.okf/project.md`
- `.okf/capabilities/cap-solver-core.md`
- `papers/Variational_BEM_FEM_Predictor_Corrector_Elastostatics.tex` (Section 9)

## What to build

Implement the Impedance-Matched Dynamic Robin Transmission Conditions and Robin-Schwarz coupled Predictor-Corrector loop in the Rust solver engine (`src-tauri/src/solver/`):

1. **Dynamic Impedance Tensor $\mathbf{Z}(s)$ (`robin_transmission.rs`):**
   - Formulate the anisotropic impedance tensor $\mathbf{Z}(s) = \rho s [c_p (\mathbf{n} \otimes \mathbf{n}) + c_s (\mathbf{I} - \mathbf{n} \otimes \mathbf{n})]$ in 2D for boundary segments with normal $\mathbf{n} = [n_x, n_y]^T$.
   - Calculate primary dilatational wave speed $c_p = \sqrt{(\lambda + 2\mu)/\rho}$ and shear wave speed $c_s = \sqrt{\mu/\rho}$ from material parameters.
   - Implement element-level radiation damping matrix $\mathbf{C}_{\text{Robin}, e}(s) = \frac{L_e}{6} \begin{bmatrix} 2\mathbf{Z}(s) & \mathbf{Z}(s) \\ \mathbf{Z}(s) & 2\mathbf{Z}(s) \end{bmatrix}$.

2. **Robin FEM Corrector Assembly:**
   - Augment the dynamic FEM operator with the boundary radiation damping matrix: $\mathcal{A}_{\text{FEM}}(s) = \mathbf{K} + s^2 \mathbf{M} + \mathbf{C}_{\text{Robin}}(s)$.
   - Add Robin transmission load vector: $\mathbf{F}_{\text{Robin}} = \mathbf{S}_{\text{shape}} \hat{\mathbf{g}}_\Gamma$.
   - Verify that $\mathbf{C}_{\text{Robin}}(s)$ provides positive energy dissipation, preventing operator singularity even at internal resonant frequencies.

3. **Robin BEM Predictor Formulation:**
   - Incorporate the impedance condition into the boundary integral equation: $[\mathbf{H}(s) + \mathbf{G}(s) \mathbf{Z}(s)] \hat{\mathbf{u}}_\Gamma = \mathbf{G}(s) \hat{\mathbf{g}}_\Gamma$.
   - Eliminate fictitious interior Dirichlet eigenfrequencies (irregular frequencies).

4. **Dynamic Robin-Schwarz Alternating Predictor-Corrector Loop:**
   - Iterate between BEM Robin predictor and FEM Robin volume corrector with transmission update:
     $\hat{\mathbf{g}}_\Gamma^{(k+1/2)} = -\hat{\mathbf{t}}_\Gamma^{(k)} + \mathbf{Z}(s) \hat{\mathbf{u}}_\Gamma^{(k)}$ and $\hat{\mathbf{g}}_\Gamma^{(k+1)} = -\hat{\mathbf{t}}_{\text{FEM}}^{(k+1/2)} + \mathbf{Z}(s) \hat{\mathbf{u}}_{\text{FEM}}^{(k+1/2)}$.
   - Terminate when interface residual $\|\hat{\mathbf{g}}^{(k+1)} - \hat{\mathbf{g}}^{(k)}\| < 10^{-5}$.
   - Verify contraction rate ($\rho \le 0.2$), achieving convergence within 3--5 iterations.

5. **Expose Hybrid Robin Option to Tauri & Angular UI:**
   - Add "BEM-FEM Hybrid (Dynamic Robin)" solver option in `solver.service.ts` and sidebar UI.
   - Comprehensive unit tests verifying tensor positive-definiteness, radiation damping coercivity, and convergence.

## Acceptance criteria

- [x] Rust implementation of $\mathbf{Z}(s)$ for 2D boundary elements with outward unit normals.
- [x] Assembly of $\mathbf{C}_{\text{Robin}}(s)$ matrix into the global complex dynamic system.
- [x] Verified elimination of Helmholtz resonance singularity in FEM corrector under Robin conditions.
- [x] Working Dynamic Robin-Schwarz Predictor-Corrector loop converging in $\le 6$ iterations for inhomogeneous test meshes.
- [x] Tauri IPC command and Angular solver option to run Hybrid Dynamic Robin simulation.
- [x] Unit tests passing in `cargo test --lib`.

## Artifact sync required

- Application PRD: `none - solver capability implementation`
- Application architecture summary: `none - solver core internal`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-solver-core.md`
- Issue registry: `required`; node `issues:` reference: `required`
- Reason/no-impact decision: `Implements Section 9 of the monograph into the working engine.`
