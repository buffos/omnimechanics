use crate::material::traits::MaterialFieldStrategy;
use crate::material::types::ReferenceMaterial;
use crate::mesh::types::{ElementType, MeshData};
use crate::solver::dynamic_bem::assemble_dynamic_bem_matrices;
use crate::solver::fem_solver::{
    assemble_dynamic_stiffness_and_mass, compute_nodal_von_mises,
};
use crate::solver::krylov_solver::solve_complex_gmres;
use crate::solver::types::{ConvergenceLog, HarmonicSimulationResult};
use faer::sparse::SparseColMat;
use num_complex::Complex64;
use std::collections::HashSet;
use std::f64::consts::PI;

/// Anisotropic Dynamic Impedance Operator Z(s) for 2D/3D elastodynamics:
/// Z(s) = rho * s * [ c_p * (n \otimes n) + c_s * (I - n \otimes n) ]
pub struct DynamicImpedance {
    pub rho: f64,
    pub cp: f64,
    pub cs: f64,
}

impl DynamicImpedance {
    pub fn new(reference: &ReferenceMaterial) -> Self {
        let rho = reference.rho0;
        let mu = reference.mu0();
        let lambda = reference.lambda0();
        let cp = ((lambda + 2.0 * mu) / rho).sqrt();
        let cs = (mu / rho).sqrt();
        Self { rho, cp, cs }
    }

    /// Evaluates the 2x2 complex impedance tensor Z(s) for outward unit normal n = [nx, ny]
    pub fn tensor_2d(&self, normal: [f64; 2], s: Complex64) -> [[Complex64; 2]; 2] {
        let nx = normal[0];
        let ny = normal[1];
        let n_sq_x = nx * nx;
        let n_sq_y = ny * ny;
        let n_xy = nx * ny;

        let factor = self.rho * s;
        let cp = self.cp;
        let cs = self.cs;

        let z_xx = factor * (cp * n_sq_x + cs * (1.0 - n_sq_x));
        let z_yy = factor * (cp * n_sq_y + cs * (1.0 - n_sq_y));
        let z_xy = factor * (cp - cs) * n_xy;

        [[z_xx, z_xy], [z_xy, z_yy]]
    }

    /// Evaluates the 3x3 complex impedance tensor Z(s) for outward unit normal n = [nx, ny, nz]
    pub fn tensor_3d(&self, normal: [f64; 3], s: Complex64) -> [[Complex64; 3]; 3] {
        let factor = self.rho * s;
        let cp = self.cp;
        let cs = self.cs;
        let delta_c = cp - cs;

        let mut z = [[Complex64::new(0.0, 0.0); 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                let delta = if i == j { 1.0 } else { 0.0 };
                z[i][j] = factor * Complex64::new(cs * delta + delta_c * normal[i] * normal[j], 0.0);
            }
        }
        z
    }
}

/// Boundary Radiation Damping Matrix C_Robin(s) = \int_\Gamma N^T Z(s) N d\Gamma
pub struct BoundaryRadiationDamping {
    pub total_dofs: usize,
    /// Nonzero entries in triplet format (row, col, value)
    pub triplets: Vec<(usize, usize, Complex64)>,
    pub diag: Vec<Complex64>,
}

impl BoundaryRadiationDamping {
    /// Assembles C_Robin(s) for all boundary elements in the mesh (2D and 3D)
    pub fn assemble(mesh: &MeshData, impedance: &DynamicImpedance, s: Complex64) -> Self {
        let dim = mesh.dimension;
        let total_dofs = mesh.nodes.len() * dim;
        let mut triplets = Vec::new();
        let mut diag = vec![Complex64::new(0.0, 0.0); total_dofs];

        if dim == 2 {
            for bnd in &mesh.boundary_elements {
                if bnd.elem_type != ElementType::Line2 || bnd.nodes.len() < 2 {
                    continue;
                }

                let n0 = bnd.nodes[0];
                let n1 = bnd.nodes[1];
                let p0 = mesh.nodes[n0];
                let p1 = mesh.nodes[n1];

                let dx = p1[0] - p0[0];
                let dy = p1[1] - p0[1];
                let length = (dx * dx + dy * dy).sqrt().max(1e-6);

                // Outward unit normal: from tag or geometry
                let normal = match bnd.tag.as_str() {
                    "Left" => [-1.0, 0.0],
                    "Right" => [1.0, 0.0],
                    "Bottom" => [0.0, -1.0],
                    "Top" => [0.0, 1.0],
                    _ => {
                        let mut nx = dy / length;
                        let mut ny = -dx / length;
                        let norm = (nx * nx + ny * ny).sqrt();
                        if norm > 1e-8 {
                            nx /= norm;
                            ny /= norm;
                        }
                        [nx, ny]
                    }
                };

                let z = impedance.tensor_2d(normal, s);

                // Line2 quadrature weights: L/6 * [2*Z, Z; Z, 2*Z]
                let c_self_pre = length / 3.0; // 2 * L / 6
                let c_cross_pre = length / 6.0; // 1 * L / 6

                for d1 in 0..dim {
                    for d2 in 0..dim {
                        let r0 = n0 * dim + d1;
                        let c0 = n0 * dim + d2;
                        let r1 = n1 * dim + d1;
                        let c1 = n1 * dim + d2;

                        let val_self = z[d1][d2] * c_self_pre;
                        let val_cross = z[d1][d2] * c_cross_pre;

                        // Node 0 - Node 0
                        triplets.push((r0, c0, val_self));
                        if r0 == c0 {
                            diag[r0] += val_self;
                        }

                        // Node 1 - Node 1
                        triplets.push((r1, c1, val_self));
                        if r1 == c1 {
                            diag[r1] += val_self;
                        }

                        // Cross terms
                        triplets.push((r0, c1, val_cross));
                        triplets.push((r1, c0, val_cross));
                    }
                }
            }
        } else if dim == 3 {
            for bnd in &mesh.boundary_elements {
                let normal = [bnd.normal[0], bnd.normal[1], bnd.normal[2]];
                let z = impedance.tensor_3d(normal, s);

                if bnd.nodes.len() >= 4 {
                    // Quad4 boundary surface element
                    let n0 = bnd.nodes[0];
                    let n1 = bnd.nodes[1];
                    let n2 = bnd.nodes[2];
                    let n3 = bnd.nodes[3];
                    let p0 = mesh.nodes[n0];
                    let p1 = mesh.nodes[n1];
                    let p2 = mesh.nodes[n2];
                    let p3 = mesh.nodes[n3];
                    let d1 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
                    let d2 = [p3[0] - p1[0], p3[1] - p1[1], p3[2] - p1[2]];
                    let cx = d1[1] * d2[2] - d1[2] * d2[1];
                    let cy = d1[2] * d2[0] - d1[0] * d2[2];
                    let cz = d1[0] * d2[1] - d1[1] * d2[0];
                    let area = 0.5 * (cx * cx + cy * cy + cz * cz).sqrt().max(1e-6);

                    let c_self = area / 9.0;
                    let c_adj = area / 18.0;
                    let c_opp = area / 36.0;
                    let nodes = [n0, n1, n2, n3];
                    for a in 0..4 {
                        for b in 0..4 {
                            let na = nodes[a];
                            let nb = nodes[b];
                            let diff = (a as isize - b as isize).abs();
                            let wt = if a == b {
                                c_self
                            } else if diff == 1 || diff == 3 {
                                c_adj
                            } else {
                                c_opp
                            };
                            for d1 in 0..3 {
                                for d2 in 0..3 {
                                    let r = na * 3 + d1;
                                    let c = nb * 3 + d2;
                                    let val = z[d1][d2] * wt;
                                    triplets.push((r, c, val));
                                    if r == c {
                                        diag[r] += val;
                                    }
                                }
                            }
                        }
                    }
                } else if bnd.nodes.len() >= 3 {
                    // Tri3 boundary surface element
                    let n0 = bnd.nodes[0];
                    let n1 = bnd.nodes[1];
                    let n2 = bnd.nodes[2];
                    let p0 = mesh.nodes[n0];
                    let p1 = mesh.nodes[n1];
                    let p2 = mesh.nodes[n2];
                    let v1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
                    let v2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
                    let cx = v1[1] * v2[2] - v1[2] * v2[1];
                    let cy = v1[2] * v2[0] - v1[0] * v2[2];
                    let cz = v1[0] * v2[1] - v1[1] * v2[0];
                    let area = 0.5 * (cx * cx + cy * cy + cz * cz).sqrt().max(1e-6);

                    let c_self = area / 6.0;
                    let c_cross = area / 12.0;
                    let nodes = [n0, n1, n2];
                    for a in 0..3 {
                        for b in 0..3 {
                            let na = nodes[a];
                            let nb = nodes[b];
                            let wt = if a == b { c_self } else { c_cross };
                            for d1 in 0..3 {
                                for d2 in 0..3 {
                                    let r = na * 3 + d1;
                                    let c = nb * 3 + d2;
                                    let val = z[d1][d2] * wt;
                                    triplets.push((r, c, val));
                                    if r == c {
                                        diag[r] += val;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Self {
            total_dofs,
            triplets,
            diag,
        }
    }

    /// Evaluates matrix-vector product w = C_Robin(s) * v
    pub fn multiply(&self, v: &[Complex64]) -> Vec<Complex64> {
        let mut out = vec![Complex64::new(0.0, 0.0); self.total_dofs];
        for &(r, c, val) in &self.triplets {
            out[r] += val * v[c];
        }
        out
    }
}

/// Solves the coupled BEM-FEM system via the Impedance-Matched Dynamic Robin-Schwarz cycle
pub fn solve_dynamic_robin_hybrid(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    frequency_hz: f64,
    load_magnitude: f64,
    max_iterations: usize,
    tolerance: f64,
) -> HarmonicSimulationResult {
    let start_time = std::time::Instant::now();
    let dim = mesh.dimension;
    let num_nodes = mesh.nodes.len();
    let total_dofs = num_nodes * dim;

    let omega = 2.0 * PI * frequency_hz.max(0.1);
    let eta = 0.02; // Structural hysteretic loss factor damping
    let s = Complex64::new(eta * omega, omega);
    let s_sq = s * s;

    // 1. Initialize Impedance and Assemble Boundary Radiation Damping Matrix C_Robin
    let impedance = DynamicImpedance::new(reference);
    let c_robin = BoundaryRadiationDamping::assemble(mesh, &impedance, s);

    // 2. Assemble FEM Dynamic Stiffness K and Mass M with exact Dirichlet filtering
    let (k0_mat, m_mat, fixed_nodes) =
        assemble_dynamic_stiffness_and_mass(mesh, reference, strategy);

    if dim == 3 {
        // 3D Soil-Structure Interaction: Coupled Structural Dynamic Stiffness + Mass + Radiation Damping Matrix C_Robin(s)
        let mut loaded_nodes_set = HashSet::new();
        for b in &mesh.boundary_elements {
            if b.tag == "Right" {
                for &n in &b.nodes {
                    loaded_nodes_set.insert(n);
                }
            }
        }
        let loaded_nodes: Vec<usize> = loaded_nodes_set.into_iter().collect();

        let mut f_ext = vec![Complex64::new(0.0, 0.0); total_dofs];
        if !loaded_nodes.is_empty() {
            let force_per_node = -load_magnitude / loaded_nodes.len() as f64;
            for &node_idx in &loaded_nodes {
                let dof_y = node_idx * dim + 1;
                f_ext[dof_y] = Complex64::new(force_per_node, 0.0);
            }
        }

        for &node_idx in &fixed_nodes {
            for d in 0..dim {
                f_ext[node_idx * dim + d] = Complex64::new(0.0, 0.0);
            }
        }

        let mut diag_inv = vec![Complex64::new(1.0, 0.0); total_dofs];
        for col in 0..total_dofs {
            let mut k_val = 0.0;
            let mut m_val = 0.0;
            for (row, &val) in k0_mat.row_indices_of_col(col).zip(k0_mat.values_of_col(col)) {
                if row == col {
                    k_val = val;
                    break;
                }
            }
            for (row, &val) in m_mat.row_indices_of_col(col).zip(m_mat.values_of_col(col)) {
                if row == col {
                    m_val = val;
                    break;
                }
            }
            let total = Complex64::new(k_val, 0.0) + s_sq * m_val + c_robin.diag[col];
            if total.norm() > 1e-8 {
                diag_inv[col] = Complex64::new(1.0, 0.0) / total;
            }
        }

        let b_prec: Vec<Complex64> = f_ext
            .iter()
            .zip(diag_inv.iter())
            .map(|(&f, &d)| f * d)
            .collect();

        let apply_3d = |v: &[Complex64]| -> Vec<Complex64> {
            let kv = crate::solver::dynamic_solver::sp_mat_mul_complex(&k0_mat, v);
            let mv = crate::solver::dynamic_solver::sp_mat_mul_complex(&m_mat, v);
            let cv = c_robin.multiply(v);
            let mut out = vec![Complex64::new(0.0, 0.0); total_dofs];
            for i in 0..total_dofs {
                let node_idx = i / dim;
                if fixed_nodes.contains(&node_idx) {
                    out[i] = v[i];
                } else {
                    out[i] = (kv[i] + s_sq * mv[i] + cv[i]) * diag_inv[i];
                }
            }
            out
        };

        let gmres_res = solve_complex_gmres(
            total_dofs,
            &b_prec,
            apply_3d,
            max_iterations.max(120),
            tolerance.max(1e-7),
            max_iterations.max(120),
        );

        let u_sol = gmres_res.solution;
        let mut amplitudes = Vec::with_capacity(num_nodes);
        let mut phase_lags = Vec::with_capacity(num_nodes);
        let mut deformed_nodes = Vec::with_capacity(num_nodes);
        let mut max_amp: f64 = 0.0;

        for i in 0..num_nodes {
            let p = mesh.nodes[i];
            let ux = u_sol[i * 3 + 0];
            let uy = u_sol[i * 3 + 1];
            let uz = u_sol[i * 3 + 2];
            let amp_x = ux.norm();
            let amp_y = uy.norm();
            let amp_z = uz.norm();
            let total_amp = (amp_x * amp_x + amp_y * amp_y + amp_z * amp_z).sqrt();
            if total_amp > max_amp {
                max_amp = total_amp;
            }
            amplitudes.push([amp_x, amp_y, amp_z]);
            phase_lags.push([ux.arg(), uy.arg(), uz.arg()]);
            deformed_nodes.push([p[0] + ux.re, p[1] + uy.re, p[2] + uz.re]);
        }

        let von_mises = compute_nodal_von_mises(mesh, &amplitudes, reference, strategy);
        let max_vm = von_mises.iter().cloned().fold(0.0, f64::max);
        let exec_time = start_time.elapsed().as_secs_f64() * 1000.0;

        return HarmonicSimulationResult {
            converged: gmres_res.converged,
            iterations: gmres_res.iterations,
            relative_residual: gmres_res.relative_residual,
            frequency_hz,
            amplitudes,
            phase_lags,
            deformed_nodes,
            von_mises_stresses: von_mises,
            max_amplitude: max_amp,
            max_von_mises: max_vm,
            execution_time_ms: exec_time,
        };
    }

    // 3. Boundary element lengths and normals
    let bnd_lengths: Vec<f64> = mesh
        .boundary_elements
        .iter()
        .map(|bnd| {
            let p0 = mesh.nodes[bnd.nodes[0]];
            let p1 = mesh.nodes[bnd.nodes[1]];
            ((p1[0] - p0[0]).powi(2) + (p1[1] - p0[1]).powi(2)).sqrt().max(1e-6)
        })
        .collect();

    // 4. Assemble BEM Dynamic Matrices H(s) and G(s)
    let (h_bem, g_bem) = assemble_dynamic_bem_matrices(mesh, reference, s);
    let num_bnd = mesh.boundary_elements.len();
    let bnd_dofs = num_bnd * dim;

    // 5. Construct Robin BEM System Matrix A:
    // For fixed Dirichlet boundary ("Left"): u = 0 is known, t is unknown -> column is -G
    // For transmission boundary (Top, Bottom, Right): t + Z*u = g -> column is H + G*Z
    let mut a_bem = h_bem.clone();
    for j in 0..num_bnd {
        let bnd = &mesh.boundary_elements[j];
        if bnd.tag == "Left" {
            for r in 0..bnd_dofs {
                for d in 0..dim {
                    let col = j * dim + d;
                    a_bem[(r, col)] = -g_bem[(r, col)];
                }
            }
        } else {
            let normal = [bnd.normal[0], bnd.normal[1]];
            let z_j = impedance.tensor_2d(normal, s);

            for r in 0..bnd_dofs {
                for d2 in 0..dim {
                    let col = j * dim + d2;
                    let mut gz = Complex64::new(0.0, 0.0);
                    for k in 0..dim {
                        gz += g_bem[(r, j * dim + k)] * z_j[k][d2];
                    }
                    a_bem[(r, col)] += gz;
                }
            }
        }
    }

    // LU factorize A_bem for rapid boundary predictor solutions
    let lu_bem = a_bem.lu();

    // 6. External Applied Force Vector F_ext
    let mut loaded_nodes_set = HashSet::new();
    for b in &mesh.boundary_elements {
        if b.tag == "Right" {
            for &n in &b.nodes {
                loaded_nodes_set.insert(n);
            }
        }
    }
    let loaded_nodes: Vec<usize> = loaded_nodes_set.into_iter().collect();

    let mut f_ext = vec![Complex64::new(0.0, 0.0); total_dofs];
    if !loaded_nodes.is_empty() {
        let force_per_node = -load_magnitude / loaded_nodes.len() as f64;
        for &node_idx in &loaded_nodes {
            let dof_y = node_idx * dim + 1;
            f_ext[dof_y] = Complex64::new(force_per_node, 0.0);
        }
    }

    for &node_idx in &fixed_nodes {
        for d in 0..dim {
            f_ext[node_idx * dim + d] = Complex64::new(0.0, 0.0);
        }
    }

    // 7. Initial Robin boundary transmission vector g_trans^(0)
    let mut g_bem_trans = vec![Complex64::new(0.0, 0.0); bnd_dofs];
    let total_right_len: f64 = mesh
        .boundary_elements
        .iter()
        .enumerate()
        .filter(|(_, b)| b.tag == "Right")
        .map(|(i, _)| bnd_lengths[i])
        .sum();
    let right_len_safe = if total_right_len > 1e-6 { total_right_len } else { 1.0 };
    for i in 0..num_bnd {
        let bnd = &mesh.boundary_elements[i];
        if bnd.tag == "Right" {
            let traction_y = -load_magnitude / right_len_safe;
            g_bem_trans[i * dim + 1] = Complex64::new(traction_y, 0.0);
        }
    }

    // Jacobi diagonal preconditioner for FEM Volume Corrector:
    // D_ii = K_ii + s^2 * M_ii + C_Robin,ii
    let mut diag_inv = vec![Complex64::new(1.0, 0.0); total_dofs];
    for col in 0..total_dofs {
        let mut k_val = 0.0;
        let mut m_val = 0.0;
        for (row, &val) in k0_mat.row_indices_of_col(col).zip(k0_mat.values_of_col(col)) {
            if row == col {
                k_val = val;
                break;
            }
        }
        for (row, &val) in m_mat.row_indices_of_col(col).zip(m_mat.values_of_col(col)) {
            if row == col {
                m_val = val;
                break;
            }
        }
        let total = Complex64::new(k_val, 0.0) + s_sq * m_val + c_robin.diag[col];
        if total.norm() > 1e-8 {
            diag_inv[col] = Complex64::new(1.0, 0.0) / total;
        }
    }

    let mut u_fem = vec![Complex64::new(0.0, 0.0); total_dofs];
    let mut u_fem_prev = vec![Complex64::new(0.0, 0.0); total_dofs];
    let mut convergence_history = Vec::new();
    let mut converged = false;

    // 8. Dynamic Robin Predictor-Corrector Loop (Accelerated Schwarz)
    for iter in 1..=max_iterations.max(1) {
        // --- STEP A: BEM Robin Predictor ---
        // RHS = \sum_{j \notin Left} G_{:, j} * g_bem_trans_j
        let mut g_vec = vec![Complex64::new(0.0, 0.0); bnd_dofs];
        for j in 0..num_bnd {
            let bnd = &mesh.boundary_elements[j];
            if bnd.tag != "Left" {
                for d in 0..dim {
                    g_vec[j * dim + d] = g_bem_trans[j * dim + d];
                }
            }
        }
        let g_dvec = nalgebra::DVector::from_column_slice(&g_vec);
        let rhs_bem = &g_bem * g_dvec;
        let x_sol = lu_bem.solve(&rhs_bem).unwrap_or(nalgebra::DVector::zeros(bnd_dofs));

        let mut u_bnd_pred = vec![Complex64::new(0.0, 0.0); bnd_dofs];
        for j in 0..num_bnd {
            let bnd = &mesh.boundary_elements[j];
            if bnd.tag != "Left" {
                for d in 0..dim {
                    u_bnd_pred[j * dim + d] = x_sol[j * dim + d];
                }
            }
        }

        // --- STEP B: Robin Transmission Data Update for FEM ---
        // g_fem_trans = 2 * Z * u_pred - g_bem_trans
        let mut g_fem_trans = vec![Complex64::new(0.0, 0.0); bnd_dofs];
        for i in 0..num_bnd {
            let bnd = &mesh.boundary_elements[i];
            if bnd.tag == "Left" {
                continue;
            }
            let normal = [bnd.normal[0], bnd.normal[1]];
            let z_i = impedance.tensor_2d(normal, s);

            for d1 in 0..dim {
                let mut z_u = Complex64::new(0.0, 0.0);
                for d2 in 0..dim {
                    z_u += z_i[d1][d2] * u_bnd_pred[i * dim + d2];
                }
                g_fem_trans[i * dim + d1] = Complex64::new(2.0, 0.0) * z_u - g_bem_trans[i * dim + d1];
            }
        }

        // Variational surface integral of g_fem_trans onto FEM nodes: \int_\Gamma N^T g d\Gamma
        let mut f_fem_total = vec![Complex64::new(0.0, 0.0); total_dofs];
        for i in 0..num_bnd {
            let bnd = &mesh.boundary_elements[i];
            let p0 = bnd.nodes[0];
            let p1 = bnd.nodes[1];
            let half_l = bnd_lengths[i] * 0.5;

            for d in 0..dim {
                let g_val = g_fem_trans[i * dim + d];
                f_fem_total[p0 * dim + d] += g_val * half_l;
                f_fem_total[p1 * dim + d] += g_val * half_l;
            }
        }

        for &node_idx in &fixed_nodes {
            for d in 0..dim {
                f_fem_total[node_idx * dim + d] = Complex64::new(0.0, 0.0);
            }
        }

        // --- STEP C: FEM Robin Volume Corrector ---
        // Solves [K + s^2 * M + C_Robin] u = F_fem_total via complex GMRES
        let apply_fem_robin = |v: &[Complex64]| -> Vec<Complex64> {
            let kv = sp_mat_mul_complex(&k0_mat, v);
            let mv = sp_mat_mul_complex(&m_mat, v);
            let cv = c_robin.multiply(v);

            let mut out = vec![Complex64::new(0.0, 0.0); total_dofs];
            for i in 0..total_dofs {
                let node_idx = i / dim;
                if fixed_nodes.contains(&node_idx) {
                    out[i] = v[i];
                } else {
                    let unprec = kv[i] + s_sq * mv[i] + cv[i];
                    out[i] = unprec * diag_inv[i];
                }
            }
            out
        };

        let mut b_prec = vec![Complex64::new(0.0, 0.0); total_dofs];
        for i in 0..total_dofs {
            let node_idx = i / dim;
            if fixed_nodes.contains(&node_idx) {
                b_prec[i] = Complex64::new(0.0, 0.0);
            } else {
                b_prec[i] = f_fem_total[i] * diag_inv[i];
            }
        }

        let gmres_res = solve_complex_gmres(
            total_dofs,
            &b_prec,
            apply_fem_robin,
            60,
            1e-9,
            40,
        );
        u_fem = gmres_res.solution;

        // --- STEP D: Interface Transmission Residual Feedback ---
        let mut g_bem_next = vec![Complex64::new(0.0, 0.0); bnd_dofs];

        for i in 0..num_bnd {
            let bnd = &mesh.boundary_elements[i];
            if bnd.tag == "Left" {
                continue;
            }
            let normal = [bnd.normal[0], bnd.normal[1]];
            let z_i = impedance.tensor_2d(normal, s);
            let p0 = bnd.nodes[0];
            let p1 = bnd.nodes[1];

            let mut u_fem_elem = [Complex64::new(0.0, 0.0); 2];
            for d in 0..dim {
                u_fem_elem[d] = (u_fem[p0 * dim + d] + u_fem[p1 * dim + d]) * 0.5;
            }

            for d1 in 0..dim {
                let mut z_u = Complex64::new(0.0, 0.0);
                for d2 in 0..dim {
                    z_u += z_i[d1][d2] * u_fem_elem[d2];
                }
                g_bem_next[i * dim + d1] = Complex64::new(2.0, 0.0) * z_u - g_fem_trans[i * dim + d1];
            }
        }

        let mut step_diff_sq = 0.0;
        let mut u_fem_norm_sq = 0.0;
        for i in 0..total_dofs {
            let du = u_fem[i] - u_fem_prev[i];
            step_diff_sq += du.norm_sqr();
            u_fem_norm_sq += u_fem[i].norm_sqr();
        }
        let step_diff = step_diff_sq.sqrt();
        let u_fem_norm = u_fem_norm_sq.sqrt().max(1e-12);
        let rel_step_change = if iter == 1 { 1.0 } else { step_diff / u_fem_norm };
        u_fem_prev = u_fem.clone();

        println!("iter {}: rel_err={:.6e}, step_diff={:.6e}", iter, rel_step_change, step_diff);
        convergence_history.push(ConvergenceLog {
            iteration: iter,
            residual_norm: step_diff,
            relative_error: rel_step_change,
        });

        // Fast Robin contraction check (typically 3-5 iterations)
        if iter > 1 && (rel_step_change < tolerance || step_diff < 1e-6) {
            converged = true;
            break;
        }

        // Relaxed transmission update: g_bem^(k+1) = (1 - omega) * g_bem^(k) + omega * g_bem_next
        let omega_relax = 0.35;
        for j in 0..bnd_dofs {
            g_bem_trans[j] = g_bem_trans[j] * (1.0 - omega_relax) + g_bem_next[j] * omega_relax;
        }
    }

    // 9. Synthesize physical nodal field quantities (Amplitude, Phase lag, Deformed nodes)
    let mut amplitudes = Vec::with_capacity(num_nodes);
    let mut phase_lags = Vec::with_capacity(num_nodes);
    let mut deformed_nodes = Vec::with_capacity(num_nodes);
    let mut u_real_disp = Vec::with_capacity(num_nodes);
    let mut max_amp = 0.0_f64;

    for i in 0..num_nodes {
        let ux = u_fem[i * dim];
        let uy = u_fem[i * dim + 1];
        let uz = if dim == 3 {
            u_fem[i * dim + 2]
        } else {
            Complex64::new(0.0, 0.0)
        };

        let amp_x = (ux.re * ux.re + ux.im * ux.im).sqrt();
        let amp_y = (uy.re * uy.re + uy.im * uy.im).sqrt();
        let amp_z = (uz.re * uz.re + uz.im * uz.im).sqrt();
        let node_amp = (amp_x * amp_x + amp_y * amp_y + amp_z * amp_z).sqrt();

        if node_amp > max_amp {
            max_amp = node_amp;
        }

        amplitudes.push([amp_x, amp_y, amp_z]);
        phase_lags.push([ux.arg(), uy.arg(), uz.arg()]);

        // Peak instantaneous displacement: Re(u)
        let disp_re = [ux.re, uy.re, if dim == 3 { uz.re } else { 0.0 }];
        u_real_disp.push(disp_re);

        let orig = mesh.nodes[i];
        deformed_nodes.push([orig[0] + ux.re, orig[1] + uy.re, orig[2] + uz.re]);
    }

    // Zero out fixed Dirichlet displacements
    for &node_idx in &fixed_nodes {
        amplitudes[node_idx] = [0.0, 0.0, 0.0];
        phase_lags[node_idx] = [0.0, 0.0, 0.0];
        u_real_disp[node_idx] = [0.0, 0.0, 0.0];
        deformed_nodes[node_idx] = mesh.nodes[node_idx];
    }

    // Real part baseline for static/peak stress evaluation
    let von_mises = compute_nodal_von_mises(mesh, &u_real_disp, reference, strategy);
    let max_vm = von_mises.iter().cloned().fold(0.0_f64, f64::max);

    HarmonicSimulationResult {
        converged,
        iterations: convergence_history.len(),
        relative_residual: convergence_history.last().map(|c| c.relative_error).unwrap_or(0.0),
        frequency_hz,
        amplitudes,
        phase_lags,
        deformed_nodes,
        von_mises_stresses: von_mises,
        max_amplitude: max_amp,
        max_von_mises: max_vm,
        execution_time_ms: start_time.elapsed().as_secs_f64() * 1000.0,
    }
}

/// Helper function to multiply a real SparseColMat by a complex vector
fn sp_mat_mul_complex(mat: &SparseColMat<usize, f64>, v: &[Complex64]) -> Vec<Complex64> {
    let n = mat.nrows();
    let mut out = vec![Complex64::new(0.0, 0.0); n];

    for col in 0..mat.ncols() {
        let v_col = v[col];
        if v_col.norm_sqr() < 1e-30 {
            continue;
        }
        for (row, &val) in mat.row_indices_of_col(col).zip(mat.values_of_col(col)) {
            out[row] += v_col * val;
        }
    }

    out
}
