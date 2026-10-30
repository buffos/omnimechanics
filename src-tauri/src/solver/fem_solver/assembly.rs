use super::elements::*;
use super::pcg::sparse_mat_vec_mult;
use crate::material::traits::MaterialFieldStrategy;
use crate::material::types::ReferenceMaterial;
use crate::mesh::types::{ElementType, MeshData};
use crate::solver::types::StressState2D;
use faer::sparse::SparseColMat;
use std::collections::HashSet;

fn add_element_stiffness(
    triplets: &mut Vec<(usize, usize, f64)>,
    ke: &[f64],
    dofs: &[usize],
) {
    let n = dofs.len();
    for i in 0..n {
        let r = dofs[i];
        for j in 0..n {
            let c = dofs[j];
            let val = ke[i * n + j];
            if val.abs() > 1e-15 {
                triplets.push((r, c, val));
            }
        }
    }
}


/// Assembles the constant reference stiffness matrix K_0 with Dirichlet penalties on fixed nodes with default Plane Stress.
pub fn assemble_reference_stiffness(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
) -> (SparseColMat<usize, f64>, HashSet<usize>, HashSet<usize>) {
    assemble_reference_stiffness_with_stress_state(mesh, reference, StressState2D::PlaneStress)
}

/// Assembles the constant reference stiffness matrix K_0 with Dirichlet penalties on fixed nodes with specified 2D stress state.
pub fn assemble_reference_stiffness_with_stress_state(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    stress_state: StressState2D,
) -> (SparseColMat<usize, f64>, HashSet<usize>, HashSet<usize>) {
    let dim = mesh.dimension;
    let num_nodes = mesh.nodes.len();
    let total_dofs = num_nodes * dim;

    let mut fixed_nodes = HashSet::new();
    for bnd in &mesh.boundary_elements {
        if bnd.tag == "Left" {
            for &n in &bnd.nodes {
                fixed_nodes.insert(n);
            }
        }
    }

    let mut loaded_nodes = HashSet::new();
    for bnd in &mesh.boundary_elements {
        if bnd.tag == "Right" {
            for &n in &bnd.nodes {
                loaded_nodes.insert(n);
            }
        }
    }

    let mut fixed_dofs = HashSet::new();
    for &n in &fixed_nodes {
        for d in 0..dim {
            fixed_dofs.insert(n * dim + d);
        }
    }

    let homogeneous_strategy = crate::material::strategies::DirectionalGradientField::new(
        crate::material::types::Axis::Y,
        0.0,
        1.0,
        1.0,
        crate::material::types::GradientForm::Linear,
    );

    let mut triplets: Vec<(usize, usize, f64)> = Vec::new();

    for elem in &mesh.volume_elements {
        match elem.elem_type {
            ElementType::Tri3 => {
                let n = &elem.nodes;
                let (ke, dof_indices) = compute_tri3_stiffness(mesh, n, reference, &homogeneous_strategy, stress_state);
                add_element_stiffness_filtered(&mut triplets, &ke, &dof_indices, &fixed_dofs);
            }
            ElementType::Quad4 => {
                let n = &elem.nodes;
                let (ke, dof_indices) = compute_quad4_stiffness(mesh, n, reference, &homogeneous_strategy, stress_state);
                add_element_stiffness_filtered(&mut triplets, &ke, &dof_indices, &fixed_dofs);
            }
            ElementType::Tet4 => {
                let n = &elem.nodes;
                let (ke, dof_indices) = compute_tet4_stiffness(mesh, n, reference, &homogeneous_strategy);
                add_element_stiffness_filtered(&mut triplets, &ke, &dof_indices, &fixed_dofs);
            }
            ElementType::Hex8 => {
                let n = &elem.nodes;
                let (ke, dof_indices) = compute_hex8_stiffness(mesh, n, reference, &homogeneous_strategy);
                add_element_stiffness_filtered(&mut triplets, &ke, &dof_indices, &fixed_dofs);
            }
            _ => {}
        }
    }

    // Exact Dirichlet: 1.0 on diagonal for fixed DOFs
    for &dof in &fixed_dofs {
        triplets.push((dof, dof, 1.0));
    }

    let k_mat = SparseColMat::try_new_from_triplets(
        total_dofs,
        total_dofs,
        &triplets,
    )
    .expect("Failed to build reference K_0 matrix");

    (k_mat, fixed_nodes, loaded_nodes)
}

/// Computes internal resisting force vector F_int(u) = \int_\Omega B^T \sigma d\Omega
/// evaluated at each element's Gauss quadrature points with the actual inhomogeneous constitutive law and default Plane Stress.
pub fn compute_internal_forces(
    mesh: &MeshData,
    displacements: &[f64],
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
) -> Vec<f64> {
    compute_internal_forces_with_stress_state(mesh, displacements, reference, strategy, StressState2D::PlaneStress)
}

/// Computes internal resisting force vector F_int(u) = \int_\Omega B^T \sigma d\Omega
/// evaluated at each element's Gauss quadrature points with the actual inhomogeneous constitutive law and specified 2D stress state.
pub fn compute_internal_forces_with_stress_state(
    mesh: &MeshData,
    displacements: &[f64],
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    stress_state: StressState2D,
) -> Vec<f64> {
    let dim = mesh.dimension;
    let num_nodes = mesh.nodes.len();
    let total_dofs = num_nodes * dim;
    let mut f_int = vec![0.0; total_dofs];

    for elem in &mesh.volume_elements {
        match elem.elem_type {
            ElementType::Tri3 => {
                let n = &elem.nodes;
                let p0 = mesh.nodes[n[0]];
                let p1 = mesh.nodes[n[1]];
                let p2 = mesh.nodes[n[2]];

                let centroid = [
                    (p0[0] + p1[0] + p2[0]) / 3.0,
                    (p0[1] + p1[1] + p2[1]) / 3.0,
                    0.0,
                ];
                let props = strategy.evaluate(centroid, reference);
                let c = plane_elasticity_c_matrix(props.e, props.nu, stress_state);

                let x1 = p0[0]; let y1 = p0[1];
                let x2 = p1[0]; let y2 = p1[1];
                let x3 = p2[0]; let y3 = p2[1];

                let det_j = (x2 - x1) * (y3 - y1) - (x3 - x1) * (y2 - y1);
                let area = 0.5 * det_j.abs().max(1e-9);

                let y23 = y2 - y3; let y31 = y3 - y1; let y12 = y1 - y2;
                let x32 = x3 - x2; let x13 = x1 - x3; let x21 = x2 - x1;

                let inv_2a = 1.0 / (2.0 * area);
                let b = [
                    y23 * inv_2a, 0.0,          y31 * inv_2a, 0.0,          y12 * inv_2a, 0.0,
                    0.0,          x32 * inv_2a, 0.0,          x13 * inv_2a, 0.0,          x21 * inv_2a,
                    x32 * inv_2a, y23 * inv_2a, x13 * inv_2a, y31 * inv_2a, x21 * inv_2a, y12 * inv_2a,
                ];

                let u_elem = [
                    displacements[n[0] * 2 + 0], displacements[n[0] * 2 + 1],
                    displacements[n[1] * 2 + 0], displacements[n[1] * 2 + 1],
                    displacements[n[2] * 2 + 0], displacements[n[2] * 2 + 1],
                ];

                let mut eps = [0.0; 3];
                for r in 0..3 {
                    for c in 0..6 {
                        eps[r] += b[r * 6 + c] * u_elem[c];
                    }
                }

                let sigma_xx = c[0] * eps[0] + c[1] * eps[1];
                let sigma_yy = c[3] * eps[0] + c[4] * eps[1];
                let tau_xy   = c[8] * eps[2];
                let sigma = [sigma_xx, sigma_yy, tau_xy];

                for c in 0..6 {
                    let mut sum = 0.0;
                    for r in 0..3 {
                        sum += b[r * 6 + c] * sigma[r];
                    }
                    let dof = if c < 2 {
                        n[0] * 2 + c
                    } else if c < 4 {
                        n[1] * 2 + (c - 2)
                    } else {
                        n[2] * 2 + (c - 4)
                    };
                    f_int[dof] += sum * area;
                }
            }

            ElementType::Quad4 => {
                let n = &elem.nodes;
                let pts = [
                    mesh.nodes[n[0]],
                    mesh.nodes[n[1]],
                    mesh.nodes[n[2]],
                    mesh.nodes[n[3]],
                ];

                let u_elem = [
                    displacements[n[0] * 2 + 0], displacements[n[0] * 2 + 1],
                    displacements[n[1] * 2 + 0], displacements[n[1] * 2 + 1],
                    displacements[n[2] * 2 + 0], displacements[n[2] * 2 + 1],
                    displacements[n[3] * 2 + 0], displacements[n[3] * 2 + 1],
                ];

                let g_pt = 1.0 / (3.0_f64).sqrt();
                let gauss_pts = [-g_pt, g_pt];

                for &xi in &gauss_pts {
                    for &eta in &gauss_pts {
                        let dn_dxi = [
                            -0.25 * (1.0 - eta),
                             0.25 * (1.0 - eta),
                             0.25 * (1.0 + eta),
                            -0.25 * (1.0 + eta),
                        ];
                        let dn_deta = [
                            -0.25 * (1.0 - xi),
                            -0.25 * (1.0 + xi),
                             0.25 * (1.0 + xi),
                             0.25 * (1.0 - xi),
                        ];

                        let mut j11 = 0.0; let mut j12 = 0.0;
                        let mut j21 = 0.0; let mut j22 = 0.0;
                        for a in 0..4 {
                            j11 += dn_dxi[a] * pts[a][0];
                            j12 += dn_dxi[a] * pts[a][1];
                            j21 += dn_deta[a] * pts[a][0];
                            j22 += dn_deta[a] * pts[a][1];
                        }

                        let det_j = (j11 * j22 - j12 * j21).abs().max(1e-12);
                        let inv_det_j = 1.0 / det_j;

                        let inv_j11 =  j22 * inv_det_j;
                        let inv_j12 = -j12 * inv_det_j;
                        let inv_j21 = -j21 * inv_det_j;
                        let inv_j22 =  j11 * inv_det_j;

                        let mut b = [0.0; 24]; // 3x8
                        for a in 0..4 {
                            let dnx = inv_j11 * dn_dxi[a] + inv_j12 * dn_deta[a];
                            let dny = inv_j21 * dn_dxi[a] + inv_j22 * dn_deta[a];

                            b[0 * 8 + 2 * a + 0] = dnx;
                            b[1 * 8 + 2 * a + 1] = dny;
                            b[2 * 8 + 2 * a + 0] = dny;
                            b[2 * 8 + 2 * a + 1] = dnx;
                        }

                        let mut xg = [0.0, 0.0, 0.0];
                        let n_vals = [
                            0.25 * (1.0 - xi) * (1.0 - eta),
                            0.25 * (1.0 + xi) * (1.0 - eta),
                            0.25 * (1.0 + xi) * (1.0 + eta),
                            0.25 * (1.0 - xi) * (1.0 + eta),
                        ];
                        for a in 0..4 {
                            xg[0] += n_vals[a] * pts[a][0];
                            xg[1] += n_vals[a] * pts[a][1];
                        }

                        let props = strategy.evaluate(xg, reference);
                        let c = plane_elasticity_c_matrix(props.e, props.nu, stress_state);

                        let mut eps = [0.0; 3];
                        for r in 0..3 {
                            for col in 0..8 {
                                eps[r] += b[r * 8 + col] * u_elem[col];
                            }
                        }

                        let sigma_xx = c[0] * eps[0] + c[1] * eps[1];
                        let sigma_yy = c[3] * eps[0] + c[4] * eps[1];
                        let tau_xy   = c[8] * eps[2];
                        let sigma = [sigma_xx, sigma_yy, tau_xy];

                        for col in 0..8 {
                            let mut sum = 0.0;
                            for r in 0..3 {
                                sum += b[r * 8 + col] * sigma[r];
                            }
                            let node_local = col / 2;
                            let dof_local = col % 2;
                            let dof = n[node_local] * 2 + dof_local;
                            f_int[dof] += sum * det_j;
                        }
                    }
                }
            }

            ElementType::Tet4 => {
                let n = &elem.nodes;
                let (ke, dof_indices) = compute_tet4_stiffness(mesh, n, reference, strategy);
                let num_dofs = dof_indices.len();
                let mut u_elem = vec![0.0; num_dofs];
                for (i, &dof) in dof_indices.iter().enumerate() {
                    u_elem[i] = displacements[dof];
                }
                for i in 0..num_dofs {
                    let mut sum = 0.0;
                    for j in 0..num_dofs {
                        sum += ke[i * num_dofs + j] * u_elem[j];
                    }
                    f_int[dof_indices[i]] += sum;
                }
            }

            ElementType::Hex8 => {
                let n = &elem.nodes;
                let (ke, dof_indices) = compute_hex8_stiffness(mesh, n, reference, strategy);
                let num_dofs = dof_indices.len();
                let mut u_elem = vec![0.0; num_dofs];
                for (i, &dof) in dof_indices.iter().enumerate() {
                    u_elem[i] = displacements[dof];
                }
                for i in 0..num_dofs {
                    let mut sum = 0.0;
                    for j in 0..num_dofs {
                        sum += ke[i * num_dofs + j] * u_elem[j];
                    }
                    f_int[dof_indices[i]] += sum;
                }
            }

            _ => {}
        }
    }

    f_int
}


/// Computes internal perturbation resisting force vector F_inh(u) = \int_\Omega B^T (C(x) - C0) B u d\Omega.
/// For a homogeneous medium where C(x) == C0, this returns identically zero.
pub fn compute_inhomogeneous_forces_with_stress_state(
    mesh: &MeshData,
    displacements: &[f64],
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    stress_state: StressState2D,
) -> Vec<f64> {
    let dim = mesh.dimension;
    let num_nodes = mesh.nodes.len();
    let total_dofs = num_nodes * dim;
    let mut f_inh = vec![0.0; total_dofs];

    let c0 = plane_elasticity_c_matrix(reference.e0, reference.nu0, stress_state);

    for elem in &mesh.volume_elements {
        match elem.elem_type {
            ElementType::Tri3 => {
                let n = &elem.nodes;
                let p0 = mesh.nodes[n[0]];
                let p1 = mesh.nodes[n[1]];
                let p2 = mesh.nodes[n[2]];

                let centroid = [
                    (p0[0] + p1[0] + p2[0]) / 3.0,
                    (p0[1] + p1[1] + p2[1]) / 3.0,
                    0.0,
                ];
                let props = strategy.evaluate(centroid, reference);
                let c = plane_elasticity_c_matrix(props.e, props.nu, stress_state);

                let delta_c0 = c[0] - c0[0];
                let delta_c1 = c[1] - c0[1];
                let delta_c3 = c[3] - c0[3];
                let delta_c4 = c[4] - c0[4];
                let delta_c8 = c[8] - c0[8];

                if delta_c0.abs() < 1e-12 && delta_c1.abs() < 1e-12 && delta_c8.abs() < 1e-12 {
                    continue;
                }

                let x1 = p0[0]; let y1 = p0[1];
                let x2 = p1[0]; let y2 = p1[1];
                let x3 = p2[0]; let y3 = p2[1];

                let det_j = (x2 - x1) * (y3 - y1) - (x3 - x1) * (y2 - y1);
                let area = 0.5 * det_j.abs().max(1e-9);

                let y23 = y2 - y3; let y31 = y3 - y1; let y12 = y1 - y2;
                let x32 = x3 - x2; let x13 = x1 - x3; let x21 = x2 - x1;

                let inv_2a = 1.0 / (2.0 * area);
                let b = [
                    y23 * inv_2a, 0.0,          y31 * inv_2a, 0.0,          y12 * inv_2a, 0.0,
                    0.0,          x32 * inv_2a, 0.0,          x13 * inv_2a, 0.0,          x21 * inv_2a,
                    x32 * inv_2a, y23 * inv_2a, x13 * inv_2a, y31 * inv_2a, x21 * inv_2a, y12 * inv_2a,
                ];

                let u_elem = [
                    displacements[n[0] * 2 + 0], displacements[n[0] * 2 + 1],
                    displacements[n[1] * 2 + 0], displacements[n[1] * 2 + 1],
                    displacements[n[2] * 2 + 0], displacements[n[2] * 2 + 1],
                ];

                let mut eps = [0.0; 3];
                for r in 0..3 {
                    for col in 0..6 {
                        eps[r] += b[r * 6 + col] * u_elem[col];
                    }
                }

                let sigma_xx = delta_c0 * eps[0] + delta_c1 * eps[1];
                let sigma_yy = delta_c3 * eps[0] + delta_c4 * eps[1];
                let tau_xy   = delta_c8 * eps[2];
                let sigma = [sigma_xx, sigma_yy, tau_xy];

                for col in 0..6 {
                    let mut sum = 0.0;
                    for r in 0..3 {
                        sum += b[r * 6 + col] * sigma[r];
                    }
                    let dof = if col < 2 {
                        n[0] * 2 + col
                    } else if col < 4 {
                        n[1] * 2 + (col - 2)
                    } else {
                        n[2] * 2 + (col - 4)
                    };
                    f_inh[dof] += sum * area;
                }
            }

            ElementType::Quad4 => {
                let n = &elem.nodes;
                let pts = [
                    mesh.nodes[n[0]],
                    mesh.nodes[n[1]],
                    mesh.nodes[n[2]],
                    mesh.nodes[n[3]],
                ];

                let u_elem = [
                    displacements[n[0] * 2 + 0], displacements[n[0] * 2 + 1],
                    displacements[n[1] * 2 + 0], displacements[n[1] * 2 + 1],
                    displacements[n[2] * 2 + 0], displacements[n[2] * 2 + 1],
                    displacements[n[3] * 2 + 0], displacements[n[3] * 2 + 1],
                ];

                let g_pt = 1.0 / (3.0_f64).sqrt();
                let gauss_pts = [-g_pt, g_pt];

                for &xi in &gauss_pts {
                    for &eta in &gauss_pts {
                        let dn_dxi = [
                            -0.25 * (1.0 - eta),
                             0.25 * (1.0 - eta),
                             0.25 * (1.0 + eta),
                            -0.25 * (1.0 + eta),
                        ];
                        let dn_deta = [
                            -0.25 * (1.0 - xi),
                            -0.25 * (1.0 + xi),
                             0.25 * (1.0 + xi),
                             0.25 * (1.0 - xi),
                        ];

                        let mut j11 = 0.0; let mut j12 = 0.0;
                        let mut j21 = 0.0; let mut j22 = 0.0;
                        for a in 0..4 {
                            j11 += dn_dxi[a] * pts[a][0];
                            j12 += dn_dxi[a] * pts[a][1];
                            j21 += dn_deta[a] * pts[a][0];
                            j22 += dn_deta[a] * pts[a][1];
                        }

                        let det_j = (j11 * j22 - j12 * j21).abs().max(1e-12);
                        let inv_det_j = 1.0 / det_j;

                        let inv_j11 =  j22 * inv_det_j;
                        let inv_j12 = -j12 * inv_det_j;
                        let inv_j21 = -j21 * inv_det_j;
                        let inv_j22 =  j11 * inv_det_j;

                        let mut b = [0.0; 24]; // 3x8
                        for a in 0..4 {
                            let dnx = inv_j11 * dn_dxi[a] + inv_j12 * dn_deta[a];
                            let dny = inv_j21 * dn_dxi[a] + inv_j22 * dn_deta[a];

                            b[0 * 8 + 2 * a + 0] = dnx;
                            b[1 * 8 + 2 * a + 1] = dny;
                            b[2 * 8 + 2 * a + 0] = dny;
                            b[2 * 8 + 2 * a + 1] = dnx;
                        }

                        let mut xg = [0.0, 0.0, 0.0];
                        let n_vals = [
                            0.25 * (1.0 - xi) * (1.0 - eta),
                            0.25 * (1.0 + xi) * (1.0 - eta),
                            0.25 * (1.0 + xi) * (1.0 + eta),
                            0.25 * (1.0 - xi) * (1.0 + eta),
                        ];
                        for a in 0..4 {
                            xg[0] += n_vals[a] * pts[a][0];
                            xg[1] += n_vals[a] * pts[a][1];
                        }

                        let props = strategy.evaluate(xg, reference);
                        let c = plane_elasticity_c_matrix(props.e, props.nu, stress_state);

                        let delta_c0 = c[0] - c0[0];
                        let delta_c1 = c[1] - c0[1];
                        let delta_c3 = c[3] - c0[3];
                        let delta_c4 = c[4] - c0[4];
                        let delta_c8 = c[8] - c0[8];

                        if delta_c0.abs() < 1e-12 && delta_c1.abs() < 1e-12 && delta_c8.abs() < 1e-12 {
                            continue;
                        }

                        let mut eps = [0.0; 3];
                        for r in 0..3 {
                            for col in 0..8 {
                                eps[r] += b[r * 8 + col] * u_elem[col];
                            }
                        }

                        let sigma_xx = delta_c0 * eps[0] + delta_c1 * eps[1];
                        let sigma_yy = delta_c3 * eps[0] + delta_c4 * eps[1];
                        let tau_xy   = delta_c8 * eps[2];
                        let sigma = [sigma_xx, sigma_yy, tau_xy];

                        for col in 0..8 {
                            let mut sum = 0.0;
                            for r in 0..3 {
                                sum += b[r * 8 + col] * sigma[r];
                            }
                            let node_local = col / 2;
                            let dof_local = col % 2;
                            let dof = n[node_local] * 2 + dof_local;
                            f_inh[dof] += sum * det_j;
                        }
                    }
                }
            }

            ElementType::Tet4 => {
                let n = &elem.nodes;
                let homogeneous_strategy = crate::material::strategies::DirectionalGradientField::new(
                    crate::material::types::Axis::X,
                    0.0,
                    1.0,
                    1.0,
                    crate::material::types::GradientForm::Linear,
                );
                let (ke, dof_indices) = compute_tet4_stiffness(mesh, n, reference, strategy);
                let (ke0, _) = compute_tet4_stiffness(mesh, n, reference, &homogeneous_strategy);
                let num_dofs = dof_indices.len();
                let mut u_elem = vec![0.0; num_dofs];
                for (i, &dof) in dof_indices.iter().enumerate() {
                    u_elem[i] = displacements[dof];
                }
                for i in 0..num_dofs {
                    let mut sum = 0.0;
                    for j in 0..num_dofs {
                        sum += (ke[i * num_dofs + j] - ke0[i * num_dofs + j]) * u_elem[j];
                    }
                    f_inh[dof_indices[i]] += sum;
                }
            }

            ElementType::Hex8 => {
                let n = &elem.nodes;
                let homogeneous_strategy = crate::material::strategies::DirectionalGradientField::new(
                    crate::material::types::Axis::X,
                    0.0,
                    1.0,
                    1.0,
                    crate::material::types::GradientForm::Linear,
                );
                let (ke, dof_indices) = compute_hex8_stiffness(mesh, n, reference, strategy);
                let (ke0, _) = compute_hex8_stiffness(mesh, n, reference, &homogeneous_strategy);
                let num_dofs = dof_indices.len();
                let mut u_elem = vec![0.0; num_dofs];
                for (i, &dof) in dof_indices.iter().enumerate() {
                    u_elem[i] = displacements[dof];
                }
                for i in 0..num_dofs {
                    let mut sum = 0.0;
                    for j in 0..num_dofs {
                        sum += (ke[i * num_dofs + j] - ke0[i * num_dofs + j]) * u_elem[j];
                    }
                    f_inh[dof_indices[i]] += sum;
                }
            }

            _ => {}
        }
    }

    f_inh
}


/// Assembles dynamic stiffness and mass matrices with exact Dirichlet DOF filtering (no penalties)
pub fn assemble_dynamic_stiffness_and_mass(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
) -> (SparseColMat<usize, f64>, SparseColMat<usize, f64>, HashSet<usize>) {
    assemble_heterogeneous_dynamic_stiffness_and_mass(mesh, reference, strategy)
}

/// Assembles reference homogeneous dynamic stiffness matrix K_0 and mass matrix M_0
pub fn assemble_reference_dynamic_stiffness_and_mass(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
) -> (SparseColMat<usize, f64>, SparseColMat<usize, f64>, HashSet<usize>) {
    assemble_custom_reference_dynamic_stiffness_and_mass(mesh, reference, &["Left"])
}

/// Assembles true heterogeneous dynamic stiffness matrix K_hetero and consistent mass matrix M_hetero
pub fn assemble_heterogeneous_dynamic_stiffness_and_mass(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
) -> (SparseColMat<usize, f64>, SparseColMat<usize, f64>, HashSet<usize>) {
    assemble_custom_heterogeneous_dynamic_stiffness_and_mass(mesh, reference, strategy, &["Left"])
}

/// Assembles reference homogeneous dynamic stiffness and mass with customizable fixed boundary tags
pub fn assemble_custom_reference_dynamic_stiffness_and_mass(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    fixed_boundary_tags: &[&str],
) -> (SparseColMat<usize, f64>, SparseColMat<usize, f64>, HashSet<usize>) {
    let homogeneous_strategy = crate::material::strategies::DirectionalGradientField::new(
        crate::material::types::Axis::Y,
        0.0,
        1.0,
        1.0,
        crate::material::types::GradientForm::Linear,
    );
    assemble_custom_heterogeneous_dynamic_stiffness_and_mass(
        mesh,
        reference,
        &homogeneous_strategy,
        fixed_boundary_tags,
    )
}

/// Assembles heterogeneous dynamic stiffness and mass with customizable fixed boundary tags
pub fn assemble_custom_heterogeneous_dynamic_stiffness_and_mass(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    fixed_boundary_tags: &[&str],
) -> (SparseColMat<usize, f64>, SparseColMat<usize, f64>, HashSet<usize>) {
    let dim = mesh.dimension;
    let num_nodes = mesh.nodes.len();
    let total_dofs = num_nodes * dim;

    let mut fixed_nodes = HashSet::new();
    let mut fixed_dofs = HashSet::new();
    for bnd in &mesh.boundary_elements {
        if fixed_boundary_tags.contains(&bnd.tag.as_str()) {
            for &n in &bnd.nodes {
                fixed_nodes.insert(n);
                for d in 0..dim {
                    fixed_dofs.insert(n * dim + d);
                }
            }
        }
    }

    let mut k_triplets = Vec::new();
    let mut m_triplets = Vec::new();

    for elem in &mesh.volume_elements {
        match elem.elem_type {
            ElementType::Tri3 => {
                let n = &elem.nodes;
                let (ke, dofs) = compute_tri3_stiffness(mesh, n, reference, strategy, StressState2D::PlaneStress);
                let (me, _) = compute_tri3_mass(mesh, n, reference, strategy);
                add_element_stiffness_filtered(&mut k_triplets, &ke, &dofs, &fixed_dofs);
                add_element_stiffness_filtered(&mut m_triplets, &me, &dofs, &fixed_dofs);
            }
            ElementType::Quad4 => {
                let n = &elem.nodes;
                let (ke, dofs) = compute_quad4_stiffness(mesh, n, reference, strategy, StressState2D::PlaneStress);
                let (me, _) = compute_quad4_mass(mesh, n, reference, strategy);
                add_element_stiffness_filtered(&mut k_triplets, &ke, &dofs, &fixed_dofs);
                add_element_stiffness_filtered(&mut m_triplets, &me, &dofs, &fixed_dofs);
            }
            ElementType::Tet4 => {
                let n = &elem.nodes;
                let (ke, dofs) = compute_tet4_stiffness(mesh, n, reference, strategy);
                let (me, _) = compute_tet4_mass(mesh, n, reference, strategy);
                add_element_stiffness_filtered(&mut k_triplets, &ke, &dofs, &fixed_dofs);
                add_element_stiffness_filtered(&mut m_triplets, &me, &dofs, &fixed_dofs);
            }
            ElementType::Hex8 => {
                let n = &elem.nodes;
                let (ke, dofs) = compute_hex8_stiffness(mesh, n, reference, strategy);
                let (me, _) = compute_hex8_mass(mesh, n, reference, strategy);
                add_element_stiffness_filtered(&mut k_triplets, &ke, &dofs, &fixed_dofs);
                add_element_stiffness_filtered(&mut m_triplets, &me, &dofs, &fixed_dofs);
            }
            _ => {}
        }
    }

    // Exact Dirichlet: set 1.0 on diagonal for fixed DOFs, 0 off-diagonal
    for &dof in &fixed_dofs {
        k_triplets.push((dof, dof, 1.0));
    }

    let k_mat = SparseColMat::try_new_from_triplets(total_dofs, total_dofs, &k_triplets)
        .expect("Failed to build heterogeneous dynamic stiffness matrix");
    let m_mat = SparseColMat::try_new_from_triplets(total_dofs, total_dofs, &m_triplets)
        .expect("Failed to build heterogeneous dynamic mass matrix");

    (k_mat, m_mat, fixed_nodes)
}

pub fn add_element_stiffness_filtered(
    triplets: &mut Vec<(usize, usize, f64)>,
    ke: &[f64],
    dofs: &[usize],
    fixed_dofs: &HashSet<usize>,
) {
    let n = dofs.len();
    for i in 0..n {
        let r = dofs[i];
        if fixed_dofs.contains(&r) {
            continue;
        }
        for j in 0..n {
            let c = dofs[j];
            if fixed_dofs.contains(&c) {
                continue;
            }
            let val = ke[i * n + j];
            if val.abs() > 1e-15 {
                triplets.push((r, c, val));
            }
        }
    }
}

/// Assembles the global consistent mass matrix M_FEM
pub fn assemble_consistent_mass_matrix(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
) -> SparseColMat<usize, f64> {
    let total_dofs = mesh.nodes.len() * mesh.dimension;
    let mut triplets = Vec::new();

    for elem in &mesh.volume_elements {
        match elem.elem_type {
            ElementType::Tri3 => {
                let (me, dofs) = compute_tri3_mass(mesh, &elem.nodes, reference, strategy);
                add_element_stiffness(&mut triplets, &me, &dofs);
            }
            ElementType::Quad4 => {
                let (me, dofs) = compute_quad4_mass(mesh, &elem.nodes, reference, strategy);
                add_element_stiffness(&mut triplets, &me, &dofs);
            }
            _ => {}
        }
    }

    SparseColMat::try_new_from_triplets(total_dofs, total_dofs, &triplets)
        .expect("Failed to build mass matrix")
}

