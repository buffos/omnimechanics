use crate::material::traits::MaterialFieldStrategy;
use crate::material::types::ReferenceMaterial;
use crate::mesh::types::{ElementType, MeshData};
use crate::solver::types::StressState2D;

/// Calculates the 3x3 plane elasticity constitutive matrix C for either Plane Stress or Plane Strain
#[inline]
pub fn plane_elasticity_c_matrix(e: f64, nu: f64, stress_state: StressState2D) -> [f64; 9] {
    match stress_state {
        StressState2D::PlaneStress => {
            let factor = e / (1.0 - nu * nu);
            [
                factor,          factor * nu,     0.0,
                factor * nu,     factor,          0.0,
                0.0,             0.0,             factor * (1.0 - nu) / 2.0,
            ]
        }
        StressState2D::PlaneStrain => {
            let factor = e / ((1.0 + nu) * (1.0 - 2.0 * nu));
            [
                factor * (1.0 - nu), factor * nu,         0.0,
                factor * nu,         factor * (1.0 - nu), 0.0,
                0.0,                 0.0,                 e / (2.0 * (1.0 + nu)),
            ]
        }
    }
}

/// Solves linear or non-linear elastostatic problem using standalone Pure Finite Element Method with default Plane Stress.

/// 2D Constant Strain Triangle (CST / Tri3) stiffness matrix
pub fn compute_tri3_stiffness(
    mesh: &MeshData,
    nodes: &[usize],
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    stress_state: StressState2D,
) -> (Vec<f64>, Vec<usize>) {
    let p0 = mesh.nodes[nodes[0]];
    let p1 = mesh.nodes[nodes[1]];
    let p2 = mesh.nodes[nodes[2]];

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

    // Strain-displacement matrix B (3x6)
    let inv_2a = 1.0 / (2.0 * area);
    let b = [
        y23 * inv_2a, 0.0,          y31 * inv_2a, 0.0,          y12 * inv_2a, 0.0,
        0.0,          x32 * inv_2a, 0.0,          x13 * inv_2a, 0.0,          x21 * inv_2a,
        x32 * inv_2a, y23 * inv_2a, x13 * inv_2a, y31 * inv_2a, x21 * inv_2a, y12 * inv_2a,
    ];

    // Ke = area * B^T * C * B (6x6)
    let mut ke = vec![0.0; 36];
    for i in 0..6 {
        for j in 0..6 {
            let mut sum = 0.0;
            for r in 0..3 {
                for s in 0..3 {
                    sum += b[r * 6 + i] * c[r * 3 + s] * b[s * 6 + j];
                }
            }
            ke[i * 6 + j] = sum * area;
        }
    }

    let dofs = vec![
        nodes[0] * 2, nodes[0] * 2 + 1,
        nodes[1] * 2, nodes[1] * 2 + 1,
        nodes[2] * 2, nodes[2] * 2 + 1,
    ];

    (ke, dofs)
}

/// 2D 4-node quadrilateral (Quad4) with 2x2 Gauss quadrature
pub fn compute_quad4_stiffness(
    mesh: &MeshData,
    nodes: &[usize],
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    stress_state: StressState2D,
) -> (Vec<f64>, Vec<usize>) {
    let pts = [
        mesh.nodes[nodes[0]],
        mesh.nodes[nodes[1]],
        mesh.nodes[nodes[2]],
        mesh.nodes[nodes[3]],
    ];

    let centroid = [
        (pts[0][0] + pts[1][0] + pts[2][0] + pts[3][0]) / 4.0,
        (pts[0][1] + pts[1][1] + pts[2][1] + pts[3][1]) / 4.0,
        0.0,
    ];

    let props = strategy.evaluate(centroid, reference);
    let c_mat = plane_elasticity_c_matrix(props.e, props.nu, stress_state);

    let g_pt = 1.0 / (3.0_f64).sqrt();
    let gauss_pts = [-g_pt, g_pt];
    let mut ke = vec![0.0; 64]; // 8x8

    for &xi in &gauss_pts {
        for &eta in &gauss_pts {
            // Shape function derivatives dN/dxi, dN/deta
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

            let det_j = j11 * j22 - j12 * j21;
            let inv_det_j = 1.0 / det_j.abs().max(1e-12);

            let inv_j11 =  j22 * inv_det_j;
            let inv_j12 = -j12 * inv_det_j;
            let inv_j21 = -j21 * inv_det_j;
            let inv_j22 =  j11 * inv_det_j;

            // B-matrix (3x8)
            let mut b = [0.0; 24];
            for a in 0..4 {
                let dnx = inv_j11 * dn_dxi[a] + inv_j12 * dn_deta[a];
                let dny = inv_j21 * dn_dxi[a] + inv_j22 * dn_deta[a];

                b[0 * 8 + 2 * a + 0] = dnx;
                b[1 * 8 + 2 * a + 1] = dny;
                b[2 * 8 + 2 * a + 0] = dny;
                b[2 * 8 + 2 * a + 1] = dnx;
            }

            let weight = det_j.abs();
            for i in 0..8 {
                for j in 0..8 {
                    let mut sum = 0.0;
                    for r in 0..3 {
                        for s in 0..3 {
                            sum += b[r * 8 + i] * c_mat[r * 3 + s] * b[s * 8 + j];
                        }
                    }
                    ke[i * 8 + j] += sum * weight;
                }
            }
        }
    }

    let dofs = vec![
        nodes[0] * 2, nodes[0] * 2 + 1,
        nodes[1] * 2, nodes[1] * 2 + 1,
        nodes[2] * 2, nodes[2] * 2 + 1,
        nodes[3] * 2, nodes[3] * 2 + 1,
    ];

    (ke, dofs)
}

/// Returns the 6x6 isotropic elasticity constitutive matrix C in 3D Voigt notation:
/// [sigma_xx, sigma_yy, sigma_zz, tau_xy, tau_yz, tau_zx]^T = C * [eps_xx, eps_yy, eps_zz, gamma_xy, gamma_yz, gamma_zx]^T
pub fn isotropic_3d_c_matrix(e: f64, nu: f64) -> [f64; 36] {
    let factor = e / ((1.0 + nu) * (1.0 - 2.0 * nu));
    let c11 = factor * (1.0 - nu);
    let c12 = factor * nu;
    let g = e / (2.0 * (1.0 + nu));

    let mut c = [0.0; 36];
    c[0 * 6 + 0] = c11; c[0 * 6 + 1] = c12; c[0 * 6 + 2] = c12;
    c[1 * 6 + 0] = c12; c[1 * 6 + 1] = c11; c[1 * 6 + 2] = c12;
    c[2 * 6 + 0] = c12; c[2 * 6 + 1] = c12; c[2 * 6 + 2] = c11;
    c[3 * 6 + 3] = g;
    c[4 * 6 + 4] = g;
    c[5 * 6 + 5] = g;
    c
}

/// 3D Linear Tetrahedron (Tet4) stiffness matrix (Constant Strain Tetrahedron)
pub fn compute_tet4_stiffness(
    mesh: &MeshData,
    nodes: &[usize],
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
) -> (Vec<f64>, Vec<usize>) {
    let p = [
        mesh.nodes[nodes[0]],
        mesh.nodes[nodes[1]],
        mesh.nodes[nodes[2]],
        mesh.nodes[nodes[3]],
    ];

    let centroid = [
        (p[0][0] + p[1][0] + p[2][0] + p[3][0]) / 4.0,
        (p[0][1] + p[1][1] + p[2][1] + p[3][1]) / 4.0,
        (p[0][2] + p[1][2] + p[2][2] + p[3][2]) / 4.0,
    ];

    let props = strategy.evaluate(centroid, reference);
    let c_mat = isotropic_3d_c_matrix(props.e, props.nu);

    // Compute determinant 6V = det([1, x, y, z])
    let x = [p[0][0], p[1][0], p[2][0], p[3][0]];
    let y = [p[0][1], p[1][1], p[2][1], p[3][1]];
    let z = [p[0][2], p[1][2], p[2][2], p[3][2]];

    let m11 = y[1] * (z[2] - z[3]) - y[2] * (z[1] - z[3]) + y[3] * (z[1] - z[2]);
    let m12 = x[1] * (z[2] - z[3]) - x[2] * (z[1] - z[3]) + x[3] * (z[1] - z[2]);
    let m13 = x[1] * (y[2] - y[3]) - x[2] * (y[1] - y[3]) + x[3] * (y[1] - y[2]);
    let m14 = x[1] * (y[2] * z[3] - y[3] * z[2]) - x[2] * (y[1] * z[3] - y[3] * z[1]) + x[3] * (y[1] * z[2] - y[2] * z[1]);

    let det_6v = m14 - x[0] * m11 + y[0] * m12 - z[0] * m13;
    let six_v = det_6v.abs().max(1e-14);
    let vol = six_v / 6.0;

    // Linear shape function spatial derivatives: dN_a / dx = b_a / (6V), etc.
    let b = [
        -(y[1] * (z[2] - z[3]) - y[2] * (z[1] - z[3]) + y[3] * (z[1] - z[2])),
         (y[0] * (z[2] - z[3]) - y[2] * (z[0] - z[3]) + y[3] * (z[0] - z[2])),
        -(y[0] * (z[1] - z[3]) - y[1] * (z[0] - z[3]) + y[3] * (z[0] - z[1])),
         (y[0] * (z[1] - z[2]) - y[1] * (z[0] - z[2]) + y[2] * (z[0] - z[1])),
    ];

    let c_deriv = [
         (x[1] * (z[2] - z[3]) - x[2] * (z[1] - z[3]) + x[3] * (z[1] - z[2])),
        -(x[0] * (z[2] - z[3]) - x[2] * (z[0] - z[3]) + x[3] * (z[0] - z[2])),
         (x[0] * (z[1] - z[3]) - x[1] * (z[0] - z[3]) + x[3] * (z[0] - z[1])),
        -(x[0] * (z[1] - z[2]) - x[1] * (z[0] - z[2]) + x[2] * (z[0] - z[1])),
    ];

    let d_deriv = [
        -(x[1] * (y[2] - y[3]) - x[2] * (y[1] - y[3]) + x[3] * (y[1] - y[2])),
         (x[0] * (y[2] - y[3]) - x[2] * (y[0] - y[3]) + x[3] * (y[0] - y[2])),
        -(x[0] * (y[1] - y[3]) - x[1] * (y[0] - y[3]) + x[3] * (y[0] - y[1])),
         (x[0] * (y[1] - y[2]) - x[1] * (y[0] - y[2]) + x[2] * (y[0] - y[1])),
    ];

    let sign = if det_6v < 0.0 { -1.0 } else { 1.0 };
    let inv_6v = sign / six_v;

    // Assemble constant 6x12 B matrix
    let mut b_mat = [0.0; 72]; // 6x12
    for a in 0..4 {
        let dnx = b[a] * inv_6v;
        let dny = c_deriv[a] * inv_6v;
        let dnz = d_deriv[a] * inv_6v;

        b_mat[0 * 12 + 3 * a + 0] = dnx;
        b_mat[1 * 12 + 3 * a + 1] = dny;
        b_mat[2 * 12 + 3 * a + 2] = dnz;

        b_mat[3 * 12 + 3 * a + 0] = dny;
        b_mat[3 * 12 + 3 * a + 1] = dnx;

        b_mat[4 * 12 + 3 * a + 1] = dnz;
        b_mat[4 * 12 + 3 * a + 2] = dny;

        b_mat[5 * 12 + 3 * a + 0] = dnz;
        b_mat[5 * 12 + 3 * a + 2] = dnx;
    }

    // Ke = V * B^T * C * B
    let mut cb = [0.0; 72]; // 6x12
    for r in 0..6 {
        for col in 0..12 {
            let mut sum = 0.0;
            for k in 0..6 {
                sum += c_mat[r * 6 + k] * b_mat[k * 12 + col];
            }
            cb[r * 12 + col] = sum;
        }
    }

    let mut ke = vec![0.0; 144]; // 12x12
    for i in 0..12 {
        for j in 0..12 {
            let mut sum = 0.0;
            for k in 0..6 {
                sum += b_mat[k * 12 + i] * cb[k * 12 + j];
            }
            ke[i * 12 + j] = sum * vol;
        }
    }

    let mut dofs = Vec::with_capacity(12);
    for &n in nodes {
        dofs.push(n * 3 + 0);
        dofs.push(n * 3 + 1);
        dofs.push(n * 3 + 2);
    }

    (ke, dofs)
}

/// 3D Hex8 element stiffness matrix with standard 2x2x2 Gauss integration (24x24)
pub fn compute_hex8_stiffness(
    mesh: &MeshData,
    nodes: &[usize],
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
) -> (Vec<f64>, Vec<usize>) {
    let p: [[f64; 3]; 8] = [
        mesh.nodes[nodes[0]],
        mesh.nodes[nodes[1]],
        mesh.nodes[nodes[2]],
        mesh.nodes[nodes[3]],
        mesh.nodes[nodes[4]],
        mesh.nodes[nodes[5]],
        mesh.nodes[nodes[6]],
        mesh.nodes[nodes[7]],
    ];

    // Local coordinates of the 8 nodes in [-1, 1]^3
    let xi_n: [f64; 8]  = [-1.0,  1.0,  1.0, -1.0, -1.0,  1.0,  1.0, -1.0];
    let eta_n: [f64; 8] = [-1.0, -1.0,  1.0,  1.0, -1.0, -1.0,  1.0,  1.0];
    let zeta_n: [f64; 8]= [-1.0, -1.0, -1.0, -1.0,  1.0,  1.0,  1.0,  1.0];

    // 2-point Gauss-Legendre quadrature points in [-1, 1]
    let g_pt = 1.0 / (3.0_f64).sqrt();
    let gauss_pts = [-g_pt, g_pt];

    let mut ke = vec![0.0; 576]; // 24x24

    for &xi in &gauss_pts {
        for &eta in &gauss_pts {
            for &zeta in &gauss_pts {
                // 1. Evaluate shape functions and local derivatives
                let mut dn_dxi = [0.0; 8];
                let mut dn_deta = [0.0; 8];
                let mut dn_dzeta = [0.0; 8];
                let mut xg = [0.0, 0.0, 0.0];

                for a in 0..8 {
                    let na = 0.125 * (1.0 + xi_n[a] * xi) * (1.0 + eta_n[a] * eta) * (1.0 + zeta_n[a] * zeta);
                    xg[0] += na * p[a][0];
                    xg[1] += na * p[a][1];
                    xg[2] += na * p[a][2];

                    dn_dxi[a]   = 0.125 * xi_n[a]   * (1.0 + eta_n[a] * eta)   * (1.0 + zeta_n[a] * zeta);
                    dn_deta[a]  = 0.125 * eta_n[a]  * (1.0 + xi_n[a] * xi)     * (1.0 + zeta_n[a] * zeta);
                    dn_dzeta[a] = 0.125 * zeta_n[a] * (1.0 + xi_n[a] * xi)     * (1.0 + eta_n[a] * eta);
                }

                // 2. Jacobian matrix J = [dx/dxi, dx/deta, dx/dzeta]^T
                let mut j11 = 0.0; let mut j12 = 0.0; let mut j13 = 0.0;
                let mut j21 = 0.0; let mut j22 = 0.0; let mut j23 = 0.0;
                let mut j31 = 0.0; let mut j32 = 0.0; let mut j33 = 0.0;

                for a in 0..8 {
                    j11 += dn_dxi[a] * p[a][0];
                    j12 += dn_dxi[a] * p[a][1];
                    j13 += dn_dxi[a] * p[a][2];

                    j21 += dn_deta[a] * p[a][0];
                    j22 += dn_deta[a] * p[a][1];
                    j23 += dn_deta[a] * p[a][2];

                    j31 += dn_dzeta[a] * p[a][0];
                    j32 += dn_dzeta[a] * p[a][1];
                    j33 += dn_dzeta[a] * p[a][2];
                }

                let det_j = j11 * (j22 * j33 - j23 * j32)
                          - j12 * (j21 * j33 - j23 * j31)
                          + j13 * (j21 * j32 - j22 * j31);

                let det_j_abs = det_j.abs().max(1e-14);
                let inv_det = 1.0 / det_j_abs;

                // 3. Inverse Jacobian J^{-1}
                let inv_j11 = (j22 * j33 - j23 * j32) * inv_det;
                let inv_j12 = (j13 * j32 - j12 * j33) * inv_det;
                let inv_j13 = (j12 * j23 - j13 * j22) * inv_det;

                let inv_j21 = (j23 * j31 - j21 * j33) * inv_det;
                let inv_j22 = (j11 * j33 - j13 * j31) * inv_det;
                let inv_j23 = (j13 * j21 - j11 * j23) * inv_det;

                let inv_j31 = (j21 * j32 - j22 * j31) * inv_det;
                let inv_j32 = (j12 * j31 - j11 * j32) * inv_det;
                let inv_j33 = (j11 * j22 - j12 * j21) * inv_det;

                // 4. Assemble 6x24 B matrix
                let mut b = [0.0; 144]; // 6x24
                for a in 0..8 {
                    let dnx = inv_j11 * dn_dxi[a] + inv_j12 * dn_deta[a] + inv_j13 * dn_dzeta[a];
                    let dny = inv_j21 * dn_dxi[a] + inv_j22 * dn_deta[a] + inv_j23 * dn_dzeta[a];
                    let dnz = inv_j31 * dn_dxi[a] + inv_j32 * dn_deta[a] + inv_j33 * dn_dzeta[a];

                    b[0 * 24 + 3 * a + 0] = dnx;
                    b[1 * 24 + 3 * a + 1] = dny;
                    b[2 * 24 + 3 * a + 2] = dnz;

                    b[3 * 24 + 3 * a + 0] = dny;
                    b[3 * 24 + 3 * a + 1] = dnx;

                    b[4 * 24 + 3 * a + 1] = dnz;
                    b[4 * 24 + 3 * a + 2] = dny;

                    b[5 * 24 + 3 * a + 0] = dnz;
                    b[5 * 24 + 3 * a + 2] = dnx;
                }

                // 5. Evaluate material constitutive matrix at Gauss point
                let props = strategy.evaluate(xg, reference);
                let c_mat = isotropic_3d_c_matrix(props.e, props.nu);

                // 6. C * B (6x24)
                let mut cb = [0.0; 144];
                for r in 0..6 {
                    for col in 0..24 {
                        let mut sum = 0.0;
                        for k in 0..6 {
                            sum += c_mat[r * 6 + k] * b[k * 24 + col];
                        }
                        cb[r * 24 + col] = sum;
                    }
                }

                // 7. Ke += B^T * (C * B) * det(J) * w (w=1.0)
                let d_vol = det_j_abs;
                for i in 0..24 {
                    for j in 0..24 {
                        let mut sum = 0.0;
                        for k in 0..6 {
                            sum += b[k * 24 + i] * cb[k * 24 + j];
                        }
                        ke[i * 24 + j] += sum * d_vol;
                    }
                }
            }
        }
    }

    let mut dofs = Vec::with_capacity(24);
    for &n in nodes {
        dofs.push(n * 3 + 0);
        dofs.push(n * 3 + 1);
        dofs.push(n * 3 + 2);
    }

    (ke, dofs)
}


/// Computes consistent mass matrix for 2D Tri3
pub fn compute_tri3_mass(
    mesh: &MeshData,
    nodes: &[usize],
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
) -> (Vec<f64>, Vec<usize>) {
    let p0 = mesh.nodes[nodes[0]];
    let p1 = mesh.nodes[nodes[1]];
    let p2 = mesh.nodes[nodes[2]];

    let centroid = [
        (p0[0] + p1[0] + p2[0]) / 3.0,
        (p0[1] + p1[1] + p2[1]) / 3.0,
        0.0,
    ];
    let rho = strategy.evaluate(centroid, reference).rho;
    let det_j = (p1[0] - p0[0]) * (p2[1] - p0[1]) - (p2[0] - p0[0]) * (p1[1] - p0[1]);
    let area = 0.5 * det_j.abs().max(1e-9);

    let m_coeff = rho * area / 12.0;
    let mut me = vec![0.0; 36];
    for i in 0..3 {
        for j in 0..3 {
            let factor = if i == j { 2.0 } else { 1.0 };
            me[(i * 2 + 0) * 6 + (j * 2 + 0)] = factor * m_coeff;
            me[(i * 2 + 1) * 6 + (j * 2 + 1)] = factor * m_coeff;
        }
    }

    let dofs = vec![
        nodes[0] * 2, nodes[0] * 2 + 1,
        nodes[1] * 2, nodes[1] * 2 + 1,
        nodes[2] * 2, nodes[2] * 2 + 1,
    ];
    (me, dofs)
}

/// Computes consistent mass matrix for 2D Quad4
pub fn compute_quad4_mass(
    mesh: &MeshData,
    nodes: &[usize],
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
) -> (Vec<f64>, Vec<usize>) {
    let pts = [
        mesh.nodes[nodes[0]],
        mesh.nodes[nodes[1]],
        mesh.nodes[nodes[2]],
        mesh.nodes[nodes[3]],
    ];

    let g_pt = 1.0 / (3.0_f64).sqrt();
    let gauss_coords = [
        (-g_pt, -g_pt),
        ( g_pt, -g_pt),
        ( g_pt,  g_pt),
        (-g_pt,  g_pt),
    ];

    let mut me = vec![0.0; 64];

    for &(xi, eta) in &gauss_coords {
        let n = [
            0.25 * (1.0 - xi) * (1.0 - eta),
            0.25 * (1.0 + xi) * (1.0 - eta),
            0.25 * (1.0 + xi) * (1.0 + eta),
            0.25 * (1.0 - xi) * (1.0 + eta),
        ];

        let d_n = [
            [-0.25 * (1.0 - eta), -0.25 * (1.0 - xi)],
            [ 0.25 * (1.0 - eta), -0.25 * (1.0 + xi)],
            [ 0.25 * (1.0 + eta),  0.25 * (1.0 + xi)],
            [-0.25 * (1.0 + eta),  0.25 * (1.0 - xi)],
        ];

        let mut j11 = 0.0; let mut j12 = 0.0;
        let mut j21 = 0.0; let mut j22 = 0.0;
        let mut gx = 0.0; let mut gy = 0.0;

        for a in 0..4 {
            j11 += d_n[a][0] * pts[a][0];
            j12 += d_n[a][0] * pts[a][1];
            j21 += d_n[a][1] * pts[a][0];
            j22 += d_n[a][1] * pts[a][1];
            gx += n[a] * pts[a][0];
            gy += n[a] * pts[a][1];
        }

        let det_j = (j11 * j22 - j12 * j21).abs().max(1e-9);
        let rho = strategy.evaluate([gx, gy, 0.0], reference).rho;

        for a in 0..4 {
            for b in 0..4 {
                let term = rho * n[a] * n[b] * det_j;
                me[(a * 2 + 0) * 8 + (b * 2 + 0)] += term;
                me[(a * 2 + 1) * 8 + (b * 2 + 1)] += term;
            }
        }
    }

    let dofs = vec![
        nodes[0] * 2, nodes[0] * 2 + 1,
        nodes[1] * 2, nodes[1] * 2 + 1,
        nodes[2] * 2, nodes[2] * 2 + 1,
        nodes[3] * 2, nodes[3] * 2 + 1,
    ];
    (me, dofs)
}

/// Computes consistent mass matrix for 3D Tet4
pub fn compute_tet4_mass(
    mesh: &MeshData,
    nodes: &[usize],
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
) -> (Vec<f64>, Vec<usize>) {
    let p0 = mesh.nodes[nodes[0]];
    let p1 = mesh.nodes[nodes[1]];
    let p2 = mesh.nodes[nodes[2]];
    let p3 = mesh.nodes[nodes[3]];

    let centroid = [
        (p0[0] + p1[0] + p2[0] + p3[0]) * 0.25,
        (p0[1] + p1[1] + p2[1] + p3[1]) * 0.25,
        (p0[2] + p1[2] + p2[2] + p3[2]) * 0.25,
    ];
    let rho = strategy.evaluate(centroid, reference).rho;

    let v1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let v2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
    let v3 = [p3[0] - p0[0], p3[1] - p0[1], p3[2] - p0[2]];

    let det_j = v1[0] * (v2[1] * v3[2] - v2[2] * v3[1])
        - v1[1] * (v2[0] * v3[2] - v2[2] * v3[0])
        + v1[2] * (v2[0] * v3[1] - v2[1] * v3[0]);
    let vol = (det_j.abs() / 6.0).max(1e-12);

    let m_coeff = rho * vol / 20.0;
    let mut me = vec![0.0; 144]; // 12x12
    for i in 0..4 {
        for j in 0..4 {
            let factor = if i == j { 2.0 } else { 1.0 };
            for d in 0..3 {
                me[(i * 3 + d) * 12 + (j * 3 + d)] = factor * m_coeff;
            }
        }
    }

    let mut dofs = Vec::with_capacity(12);
    for &n in nodes {
        dofs.push(n * 3 + 0);
        dofs.push(n * 3 + 1);
        dofs.push(n * 3 + 2);
    }
    (me, dofs)
}

/// Computes consistent mass matrix for 3D Hex8 via 2x2x2 Gauss quadrature
pub fn compute_hex8_mass(
    mesh: &MeshData,
    nodes: &[usize],
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
) -> (Vec<f64>, Vec<usize>) {
    let mut pts = [[0.0; 3]; 8];
    for a in 0..8 {
        pts[a] = mesh.nodes[nodes[a]];
    }

    let g_pt = 1.0 / 3.0_f64.sqrt();
    let gauss_1d = [-g_pt, g_pt];

    let mut me = vec![0.0; 576]; // 24x24

    for &xi in &gauss_1d {
        for &eta in &gauss_1d {
            for &zeta in &gauss_1d {
                let n = [
                    0.125 * (1.0 - xi) * (1.0 - eta) * (1.0 - zeta),
                    0.125 * (1.0 + xi) * (1.0 - eta) * (1.0 - zeta),
                    0.125 * (1.0 + xi) * (1.0 + eta) * (1.0 - zeta),
                    0.125 * (1.0 - xi) * (1.0 + eta) * (1.0 - zeta),
                    0.125 * (1.0 - xi) * (1.0 - eta) * (1.0 + zeta),
                    0.125 * (1.0 + xi) * (1.0 - eta) * (1.0 + zeta),
                    0.125 * (1.0 + xi) * (1.0 + eta) * (1.0 + zeta),
                    0.125 * (1.0 - xi) * (1.0 + eta) * (1.0 + zeta),
                ];

                let dn_dxi = [
                    -0.125 * (1.0 - eta) * (1.0 - zeta),
                     0.125 * (1.0 - eta) * (1.0 - zeta),
                     0.125 * (1.0 + eta) * (1.0 - zeta),
                    -0.125 * (1.0 + eta) * (1.0 - zeta),
                    -0.125 * (1.0 - eta) * (1.0 + zeta),
                     0.125 * (1.0 - eta) * (1.0 + zeta),
                     0.125 * (1.0 + eta) * (1.0 + zeta),
                    -0.125 * (1.0 + eta) * (1.0 + zeta),
                ];
                let dn_deta = [
                    -0.125 * (1.0 - xi) * (1.0 - zeta),
                    -0.125 * (1.0 + xi) * (1.0 - zeta),
                     0.125 * (1.0 + xi) * (1.0 - zeta),
                     0.125 * (1.0 - xi) * (1.0 - zeta),
                    -0.125 * (1.0 - xi) * (1.0 + zeta),
                    -0.125 * (1.0 + xi) * (1.0 + zeta),
                     0.125 * (1.0 + xi) * (1.0 + zeta),
                     0.125 * (1.0 - xi) * (1.0 + zeta),
                ];
                let dn_dzeta = [
                    -0.125 * (1.0 - xi) * (1.0 - eta),
                    -0.125 * (1.0 + xi) * (1.0 - eta),
                    -0.125 * (1.0 + xi) * (1.0 + eta),
                    -0.125 * (1.0 - xi) * (1.0 + eta),
                     0.125 * (1.0 - xi) * (1.0 - eta),
                     0.125 * (1.0 + xi) * (1.0 - eta),
                     0.125 * (1.0 + xi) * (1.0 + eta),
                     0.125 * (1.0 - xi) * (1.0 + eta),
                ];

                let mut j = [[0.0; 3]; 3];
                let mut gx = 0.0; let mut gy = 0.0; let mut gz = 0.0;
                for a in 0..8 {
                    gx += n[a] * pts[a][0];
                    gy += n[a] * pts[a][1];
                    gz += n[a] * pts[a][2];

                    j[0][0] += dn_dxi[a] * pts[a][0];
                    j[0][1] += dn_dxi[a] * pts[a][1];
                    j[0][2] += dn_dxi[a] * pts[a][2];

                    j[1][0] += dn_deta[a] * pts[a][0];
                    j[1][1] += dn_deta[a] * pts[a][1];
                    j[1][2] += dn_deta[a] * pts[a][2];

                    j[2][0] += dn_dzeta[a] * pts[a][0];
                    j[2][1] += dn_dzeta[a] * pts[a][1];
                    j[2][2] += dn_dzeta[a] * pts[a][2];
                }

                let det_j = (j[0][0] * (j[1][1] * j[2][2] - j[1][2] * j[2][1])
                    - j[0][1] * (j[1][0] * j[2][2] - j[1][2] * j[2][0])
                    + j[0][2] * (j[1][0] * j[2][1] - j[1][1] * j[2][0])).abs().max(1e-12);

                let rho = strategy.evaluate([gx, gy, gz], reference).rho;

                for a in 0..8 {
                    for b in 0..8 {
                        let term = rho * n[a] * n[b] * det_j;
                        for d in 0..3 {
                            me[(a * 3 + d) * 24 + (b * 3 + d)] += term;
                        }
                    }
                }
            }
        }
    }

    let mut dofs = Vec::with_capacity(24);
    for &node in nodes {
        dofs.push(node * 3 + 0);
        dofs.push(node * 3 + 1);
        dofs.push(node * 3 + 2);
    }
    (me, dofs)
}
