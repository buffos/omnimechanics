use super::elements::*;
use crate::material::traits::MaterialFieldStrategy;
use crate::material::types::ReferenceMaterial;
use crate::mesh::types::{ElementType, MeshData};
use crate::solver::types::{ConvergenceLog, StressState2D};
use faer::sparse::SparseColMat;

/// Computes the diagonal inverse preconditioner M^{-1} for sparse PCG
pub fn compute_diagonal_inverse(k_mat: &SparseColMat<usize, f64>) -> Vec<f64> {
    let n = k_mat.ncols();
    let mut diag_inv = vec![1.0; n];
    for col in 0..n {
        for (row, &val) in k_mat.row_indices_of_col(col).zip(k_mat.values_of_col(col)) {
            if row == col && val.abs() > 1e-12 {
                diag_inv[row] = 1.0 / val;
            }
        }
    }
    diag_inv
}

/// Preconditioned Conjugate Gradient (PCG) with precomputed preconditioner and optional initial guess
pub fn solve_pcg_sparse_with_preconditioner(
    k_mat: &SparseColMat<usize, f64>,
    f_ext: &[f64],
    diag_inv: &[f64],
    u_init: Option<&[f64]>,
    max_iter: usize,
    tol: f64,
) -> Vec<f64> {
    let n = f_ext.len();
    let mut u = match u_init {
        Some(init) => init.to_vec(),
        None => vec![0.0; n],
    };

    // Residual r = F - K * u
    let mut r = vec![0.0; n];
    if u_init.is_some() {
        let mut ku = vec![0.0; n];
        for col in 0..n {
            let u_val = u[col];
            if u_val.abs() > 1e-16 {
                for (row, &val) in k_mat.row_indices_of_col(col).zip(k_mat.values_of_col(col)) {
                    ku[row] += val * u_val;
                }
            }
        }
        for i in 0..n {
            r[i] = f_ext[i] - ku[i];
        }
    } else {
        r.copy_from_slice(f_ext);
    }

    let mut z: Vec<f64> = r.iter().zip(diag_inv).map(|(&ri, &di)| ri * di).collect();
    let mut p = z.clone();

    let mut rz_old: f64 = r.iter().zip(&z).map(|(&a, &b)| a * b).sum();
    let norm_f0 = f_ext.iter().map(|&x| x * x).sum::<f64>().sqrt().max(1e-12);

    for _ in 0..max_iter {
        let norm_r = r.iter().map(|&x| x * x).sum::<f64>().sqrt();
        if norm_r / norm_f0 < tol {
            break;
        }

        // Ap = K * p
        let mut ap = vec![0.0; n];
        for col in 0..n {
            let p_val = p[col];
            if p_val.abs() > 1e-16 {
                for (row, &val) in k_mat.row_indices_of_col(col).zip(k_mat.values_of_col(col)) {
                    ap[row] += val * p_val;
                }
            }
        }

        let p_ap: f64 = p.iter().zip(&ap).map(|(&a, &b)| a * b).sum();
        if p_ap.abs() < 1e-20 {
            break;
        }

        let alpha = rz_old / p_ap;

        for i in 0..n {
            u[i] += alpha * p[i];
            r[i] -= alpha * ap[i];
            z[i] = r[i] * diag_inv[i];
        }

        let rz_new: f64 = r.iter().zip(&z).map(|(&a, &b)| a * b).sum();
        let beta = rz_new / rz_old;
        rz_old = rz_new;

        for i in 0..n {
            p[i] = z[i] + beta * p[i];
        }
    }

    u
}

/// Preconditioned Conjugate Gradient (PCG) for solving K * u = F
pub fn solve_pcg_sparse(
    k_mat: &SparseColMat<usize, f64>,
    f_ext: &[f64],
    max_iter: usize,
    tol: f64,
) -> Vec<f64> {
    let diag_inv = compute_diagonal_inverse(k_mat);
    solve_pcg_sparse_with_preconditioner(k_mat, f_ext, &diag_inv, None, max_iter, tol)
}

/// Estimates nodal von Mises stresses from displacement field
pub fn compute_nodal_von_mises(
    mesh: &MeshData,
    displacements: &[[f64; 3]],
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
) -> Vec<f64> {
    let num_nodes = mesh.nodes.len();
    let mut vm_stresses = vec![0.0; num_nodes];
    let dim = mesh.dimension;

    for i in 0..num_nodes {
        let p = mesh.nodes[i];
        let u = displacements[i];
        let props = strategy.evaluate(p, reference);

        // Approximate strain gradient proportional to displacement / span
        let lx = mesh.nodes.iter().map(|pt| pt[0]).fold(0.0, f64::max).max(1.0);
        let ly = mesh.nodes.iter().map(|pt| pt[1]).fold(0.0, f64::max).max(1.0);

        // Bending stress distribution: max tension/compression at edges, proportional to distance from neutral axis
        let y_norm = (p[1] - ly / 2.0) / (ly / 2.0);
        let bending_strain = (u[1].abs() / ly) * y_norm * 3.0;
        let sigma_x = props.e * bending_strain.abs();
        let tau_xy = props.mu * (u[1].abs() / lx);

        let vm = if dim == 2 {
            (sigma_x * sigma_x + 3.0 * tau_xy * tau_xy).sqrt() * 1e-6 // convert to MPa
        } else {
            (sigma_x * sigma_x + 3.0 * tau_xy * tau_xy).sqrt() * 1e-6
        };

        vm_stresses[i] = vm;
    }

    vm_stresses
}


/// Multiplies a sparse CSR/CSC matrix (SparseColMat) by a dense slice: y = A * x
pub fn sparse_mat_vec_mult(mat: &SparseColMat<usize, f64>, x: &[f64]) -> Vec<f64> {
    let n = x.len();
    let mut y = vec![0.0; mat.nrows()];
    for col in 0..n {
        let x_val = x[col];
        if x_val.abs() > 1e-16 {
            for (row, &val) in mat.row_indices_of_col(col).zip(mat.values_of_col(col)) {
                y[row] += val * x_val;
            }
        }
    }
    y
}
