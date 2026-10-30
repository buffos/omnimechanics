use crate::mesh::types::{ElementType, MeshData};
use faer::sparse::SparseColMat;
use std::collections::HashMap;

/// Boundary Shape Matrix (S_shape) representing the variational consistency mass matrix:
/// S_shape = \int_\Gamma N(s)^T N(s) d\Gamma
/// Energetically couples surface tractions t (N/m^2) with nodal forces F (N).
pub struct BoundaryShapeMatrix {
    pub dimension: usize,
    /// Total number of boundary degrees of freedom (num_bnd_nodes * dimension)
    pub num_bnd_dofs: usize,
    /// Mapping: global node index -> local boundary node index
    pub global_to_bnd_node: HashMap<usize, usize>,
    /// Mapping: local boundary node index -> global node index
    pub bnd_to_global_node: Vec<usize>,
    /// Global assembled consistency matrix (sparse)
    pub matrix: SparseColMat<usize, f64>,
    /// Lumped diagonal inverse approximation for ultra-fast scaling if needed
    pub diag_inv: Vec<f64>,
}

impl BoundaryShapeMatrix {
    /// Assembles the global Boundary Shape Matrix from the mesh boundary elements
    pub fn assemble(mesh: &MeshData) -> Self {
        let dim = mesh.dimension;

        // 1. Collect all unique boundary nodes
        let mut bnd_nodes_set = std::collections::BTreeSet::new();
        for bnd in &mesh.boundary_elements {
            for &n in &bnd.nodes {
                bnd_nodes_set.insert(n);
            }
        }

        let bnd_to_global_node: Vec<usize> = bnd_nodes_set.into_iter().collect();
        let mut global_to_bnd_node = HashMap::new();
        for (local_idx, &global_idx) in bnd_to_global_node.iter().enumerate() {
            global_to_bnd_node.insert(global_idx, local_idx);
        }

        let num_bnd_nodes = bnd_to_global_node.len();
        let num_bnd_dofs = num_bnd_nodes * dim;

        // 2. Assemble element matrices using triplet format
        let mut triplets: Vec<(usize, usize, f64)> = Vec::new();
        let mut diag_sums = vec![0.0; num_bnd_dofs];

        for bnd in &mesh.boundary_elements {
            match bnd.elem_type {
                ElementType::Line2 => {
                    let n0 = bnd.nodes[0];
                    let n1 = bnd.nodes[1];
                    let p0 = mesh.nodes[n0];
                    let p1 = mesh.nodes[n1];

                    // Length of 1D boundary element
                    let dx = p1[0] - p0[0];
                    let dy = p1[1] - p0[1];
                    let dz = p1[2] - p0[2];
                    let length = (dx * dx + dy * dy + dz * dz).sqrt();

                    // Analytical S_shape for Line2: L/6 * [2, 1; 1, 2] (symmetric)
                    let s_self = length * (2.0 / 6.0);
                    let s_cross = length * (1.0 / 6.0);

                    let loc0 = global_to_bnd_node[&n0];
                    let loc1 = global_to_bnd_node[&n1];

                    for d in 0..dim {
                        let r0 = loc0 * dim + d;
                        let r1 = loc1 * dim + d;

                        triplets.push((r0, r0, s_self));
                        triplets.push((r0, r1, s_cross));
                        triplets.push((r1, r0, s_cross));
                        triplets.push((r1, r1, s_self));

                        diag_sums[r0] += s_self + s_cross;
                        diag_sums[r1] += s_self + s_cross;
                    }
                }

                ElementType::Tri3 => {
                    let n0 = bnd.nodes[0];
                    let n1 = bnd.nodes[1];
                    let n2 = bnd.nodes[2];
                    let p0 = mesh.nodes[n0];
                    let p1 = mesh.nodes[n1];
                    let p2 = mesh.nodes[n2];

                    // Cross product to find triangle area
                    let v01 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
                    let v02 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
                    let cross = [
                        v01[1] * v02[2] - v01[2] * v02[1],
                        v01[2] * v02[0] - v01[0] * v02[2],
                        v01[0] * v02[1] - v01[1] * v02[0],
                    ];
                    let area = 0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();

                    // Analytical S_shape for Tri3: A/12 * [2, 1, 1; 1, 2, 1; 1, 1, 2]
                    let s_self = area * (2.0 / 12.0);
                    let s_cross = area * (1.0 / 12.0);

                    let loc_nodes = [
                        global_to_bnd_node[&n0],
                        global_to_bnd_node[&n1],
                        global_to_bnd_node[&n2],
                    ];

                    for d in 0..dim {
                        for (i, &loc_i) in loc_nodes.iter().enumerate() {
                            for (j, &loc_j) in loc_nodes.iter().enumerate() {
                                let r_i = loc_i * dim + d;
                                let r_j = loc_j * dim + d;
                                let val = if i == j { s_self } else { s_cross };
                                triplets.push((r_i, r_j, val));
                            }
                            let r_i = loc_i * dim + d;
                            diag_sums[r_i] += s_self + 2.0 * s_cross;
                        }
                    }
                }

                _ => {
                    // Fallback diagonal weighting for other element types
                    for &n in &bnd.nodes {
                        let loc = global_to_bnd_node[&n];
                        for d in 0..dim {
                            let r = loc * dim + d;
                            triplets.push((r, r, 1.0));
                            diag_sums[r] += 1.0;
                        }
                    }
                }
            }
        }

        // Build SparseColMat
        let matrix = SparseColMat::try_new_from_triplets(
            num_bnd_dofs,
            num_bnd_dofs,
            &triplets,
        )
        .expect("Failed to build S_shape sparse matrix");

        let diag_inv: Vec<f64> = diag_sums
            .into_iter()
            .map(|s| if s.abs() > 1e-12 { 1.0 / s } else { 1.0 })
            .collect();

        Self {
            dimension: dim,
            num_bnd_dofs,
            global_to_bnd_node,
            bnd_to_global_node,
            matrix,
            diag_inv,
        }
    }

    /// Converts continuous surface tractions t (N/m^2) to nodal forces F (N):
    /// F = S_shape * t
    pub fn tractions_to_forces(&self, tractions: &[f64]) -> Vec<f64> {
        assert_eq!(tractions.len(), self.num_bnd_dofs);
        let mut forces = vec![0.0; self.num_bnd_dofs];

        // Sparse matrix-vector multiplication F = S * t
        for col in 0..self.num_bnd_dofs {
            let t_val = tractions[col];
            for (row, &val) in self.matrix.row_indices_of_col(col).zip(self.matrix.values_of_col(col)) {
                forces[row] += val * t_val;
            }
        }

        forces
    }

    /// Converts discrete nodal forces F (N) to continuous surface tractions t (N/m^2):
    /// t = S_shape^{-1} * F
    /// Uses Jacobi preconditioned Conjugate Gradient (CG) for rapid, exact solution.
    pub fn forces_to_tractions(&self, forces: &[f64], max_iter: usize, tol: f64) -> Vec<f64> {
        assert_eq!(forces.len(), self.num_bnd_dofs);
        let n = self.num_bnd_dofs;

        // Initial guess x0 using lumped diagonal inverse
        let mut x: Vec<f64> = forces
            .iter()
            .zip(&self.diag_inv)
            .map(|(&f, &d)| f * d)
            .collect();

        // r = b - A * x
        let ax = self.tractions_to_forces(&x);
        let mut r: Vec<f64> = forces.iter().zip(&ax).map(|(&f, &a)| f - a).collect();

        // z = M^{-1} * r (Jacobi preconditioner)
        let mut z: Vec<f64> = r.iter().zip(&self.diag_inv).map(|(&ri, &di)| ri * di).collect();
        let mut p = z.clone();

        let mut rz_old = dot_product(&r, &z);
        let norm_b = dot_product(forces, forces).sqrt().max(1e-12);

        for _ in 0..max_iter {
            let norm_r = dot_product(&r, &r).sqrt();
            if norm_r / norm_b < tol {
                break;
            }

            let ap = self.tractions_to_forces(&p);
            let pap = dot_product(&p, &ap);
            if pap.abs() < 1e-16 {
                break;
            }

            let alpha = rz_old / pap;

            for i in 0..n {
                x[i] += alpha * p[i];
                r[i] -= alpha * ap[i];
                z[i] = r[i] * self.diag_inv[i];
            }

            let rz_new = dot_product(&r, &z);
            let beta = rz_new / rz_old;
            rz_old = rz_new;

            for i in 0..n {
                p[i] = z[i] + beta * p[i];
            }
        }

        x
    }
}

fn dot_product(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(&x, &y)| x * y).sum()
}

#[cfg(test)]
#[path = "boundary_shape_test.rs"]
mod tests;
