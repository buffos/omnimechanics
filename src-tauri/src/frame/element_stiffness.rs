use nalgebra::DMatrix;

use super::types::{EndRelease, LoadFrame, OffsetFrame, RigidOffset};

/// Evaluates Hermite cubic shape functions at coordinate x in [0, L]
/// Returns [N_v1, N_theta1, N_v2, N_theta2]
pub fn hermite_shape_functions(x: f64, l: f64) -> [f64; 4] {
    let xi = (x / l).clamp(0.0, 1.0);
    let xi2 = xi * xi;
    let xi3 = xi2 * xi;
    [
        1.0 - 3.0 * xi2 + 2.0 * xi3,
        x * (1.0 - xi).powi(2),
        3.0 * xi2 - 2.0 * xi3,
        x * xi * (xi - 1.0),
    ]
}

/// Evaluates derivatives of Hermite shape functions dN/dx at coordinate x
pub fn hermite_shape_derivs(x: f64, l: f64) -> [f64; 4] {
    let xi = (x / l).clamp(0.0, 1.0);
    [
        (-6.0 * xi + 6.0 * xi * xi) / l,
        1.0 - 4.0 * xi + 3.0 * xi * xi,
        (6.0 * xi - 6.0 * xi * xi) / l,
        -2.0 * xi + 3.0 * xi * xi,
    ]
}

/// Builds 6x6 local Euler-Bernoulli beam stiffness matrix
pub fn local_frame_stiffness(e: f64, a: f64, i: f64, l: f64) -> DMatrix<f64> {
    let mut k = DMatrix::<f64>::zeros(6, 6);
    let ea_l = (e * a) / l;
    let ei_l3 = (e * i) / (l * l * l);

    k[(0, 0)] = ea_l;
    k[(0, 3)] = -ea_l;
    k[(3, 0)] = -ea_l;
    k[(3, 3)] = ea_l;

    k[(1, 1)] = 12.0 * ei_l3;
    k[(1, 2)] = 6.0 * ei_l3 * l;
    k[(1, 4)] = -12.0 * ei_l3;
    k[(1, 5)] = 6.0 * ei_l3 * l;

    k[(2, 1)] = 6.0 * ei_l3 * l;
    k[(2, 2)] = 4.0 * ei_l3 * l * l;
    k[(2, 4)] = -6.0 * ei_l3 * l;
    k[(2, 5)] = 2.0 * ei_l3 * l * l;

    k[(4, 1)] = -12.0 * ei_l3;
    k[(4, 2)] = -6.0 * ei_l3 * l;
    k[(4, 4)] = 12.0 * ei_l3;
    k[(4, 5)] = -6.0 * ei_l3 * l;

    k[(5, 1)] = 6.0 * ei_l3 * l;
    k[(5, 2)] = 2.0 * ei_l3 * l * l;
    k[(5, 4)] = -6.0 * ei_l3 * l;
    k[(5, 5)] = 4.0 * ei_l3 * l * l;

    k
}

/// Condenses out released DOFs (axial, shear, moment) from local stiffness and fixed-end force vector
pub fn condense_releases(
    k_local: &DMatrix<f64>,
    f0_local: &[f64; 6],
    rel_i: &EndRelease,
    rel_j: &EndRelease,
) -> (DMatrix<f64>, [f64; 6]) {
    let mut released_dofs = Vec::new();
    if rel_i.axial {
        released_dofs.push(0);
    }
    if rel_i.shear {
        released_dofs.push(1);
    }
    if rel_i.moment {
        released_dofs.push(2);
    }
    if rel_j.axial {
        released_dofs.push(3);
    }
    if rel_j.shear {
        released_dofs.push(4);
    }
    if rel_j.moment {
        released_dofs.push(5);
    }

    if released_dofs.is_empty() {
        return (k_local.clone(), *f0_local);
    }

    let mut active_dofs = Vec::new();
    for i in 0..6 {
        if !released_dofs.contains(&i) {
            active_dofs.push(i);
        }
    }

    if active_dofs.is_empty() {
        return (DMatrix::zeros(6, 6), [0.0; 6]);
    }

    let na = active_dofs.len();
    let nr = released_dofs.len();
    let mut k_aa = DMatrix::<f64>::zeros(na, na);
    let mut k_ar = DMatrix::<f64>::zeros(na, nr);
    let mut k_ra = DMatrix::<f64>::zeros(nr, na);
    let mut k_rr = DMatrix::<f64>::zeros(nr, nr);

    for i in 0..na {
        for j in 0..na {
            k_aa[(i, j)] = k_local[(active_dofs[i], active_dofs[j])];
        }
        for j in 0..nr {
            k_ar[(i, j)] = k_local[(active_dofs[i], released_dofs[j])];
        }
    }
    for i in 0..nr {
        for j in 0..na {
            k_ra[(i, j)] = k_local[(released_dofs[i], active_dofs[j])];
        }
        for j in 0..nr {
            k_rr[(i, j)] = k_local[(released_dofs[i], released_dofs[j])];
        }
    }

    let mut f_a = vec![0.0; na];
    for i in 0..na {
        f_a[i] = f0_local[active_dofs[i]];
    }
    let mut f_r = vec![0.0; nr];
    for i in 0..nr {
        f_r[i] = f0_local[released_dofs[i]];
    }

    let mut k_res = DMatrix::<f64>::zeros(6, 6);
    let mut f_res = [0.0; 6];

    if let Some(k_rr_inv) = k_rr.try_inverse() {
        let k_cond = k_aa - &k_ar * &k_rr_inv * k_ra;
        for i in 0..na {
            for j in 0..na {
                k_res[(active_dofs[i], active_dofs[j])] = k_cond[(i, j)];
            }
        }

        // f_c = f_a - K_ar * K_rr^-1 * f_r
        let mut fr_vec = nalgebra::DVector::from_vec(f_r);
        let corr = &k_ar * (&k_rr_inv * fr_vec);
        for i in 0..na {
            f_res[active_dofs[i]] = f_a[i] - corr[i];
        }
    }

    (k_res, f_res)
}

/// Computes 6x6 transformation matrix mapping member clear ends to global node centers
/// Accounts for rigid offsets (dx, dy) and orientation angle alpha
pub fn element_transformation_matrix(
    cos_a: f64,
    sin_a: f64,
    off_i: &RigidOffset,
    off_j: &RigidOffset,
) -> DMatrix<f64> {
    // 1. Rotation matrix from global to local orientation: T_rot
    let mut t_rot = DMatrix::<f64>::zeros(6, 6);
    t_rot[(0, 0)] = cos_a;
    t_rot[(0, 1)] = sin_a;
    t_rot[(1, 0)] = -sin_a;
    t_rot[(1, 1)] = cos_a;
    t_rot[(2, 2)] = 1.0;

    t_rot[(3, 3)] = cos_a;
    t_rot[(3, 4)] = sin_a;
    t_rot[(4, 3)] = -sin_a;
    t_rot[(4, 4)] = cos_a;
    t_rot[(5, 5)] = 1.0;

    // 2. Rigid offset transformation in local frame
    let (dxi, dyi) = match off_i.frame {
        OffsetFrame::Local => (off_i.dx, off_i.dy),
        OffsetFrame::Global => (
            off_i.dx * cos_a + off_i.dy * sin_a,
            -off_i.dx * sin_a + off_i.dy * cos_a,
        ),
    };
    let (dxj, dyj) = match off_j.frame {
        OffsetFrame::Local => (off_j.dx, off_j.dy),
        OffsetFrame::Global => (
            off_j.dx * cos_a + off_j.dy * sin_a,
            -off_j.dx * sin_a + off_j.dy * cos_a,
        ),
    };

    let mut t_off = DMatrix::<f64>::identity(6, 6);
    t_off[(0, 2)] = -dyi;
    t_off[(1, 2)] = dxi;
    t_off[(3, 5)] = -dyj;
    t_off[(4, 5)] = dxj;

    // Overall transformation: T = T_off * T_rot
    t_off * t_rot
}

/// Integrates exact fixed-end forces on a beam member under a distributed load
/// using 3-point Gauss-Legendre quadrature (exact for cubics/quartics)
pub fn integrate_distributed_load(
    l: f64,
    qx1: f64,
    qy1: f64,
    qx2: f64,
    qy2: f64,
    a_ratio: f64,
    b_ratio: f64,
    cos_a: f64,
    sin_a: f64,
    frame: LoadFrame,
) -> [f64; 6] {
    let xa = a_ratio * l;
    let xb = b_ratio * l;
    let span = xb - xa;
    if span <= 1e-9 {
        return [0.0; 6];
    }

    // 3-point Gauss-Legendre rule on [-1, 1]
    let gauss_pts = [-0.7745966692414834, 0.0, 0.7745966692414834];
    let gauss_wts = [0.5555555555555556, 0.8888888888888888, 0.5555555555555556];

    let mut f0 = [0.0; 6];

    for (pt, wt) in gauss_pts.iter().zip(gauss_wts.iter()) {
        let x = xa + 0.5 * (pt + 1.0) * span;
        let s = (x - xa) / span; // interpolation ratio in [0, 1]
        let raw_qx = qx1 + s * (qx2 - qx1);
        let raw_qy = qy1 + s * (qy2 - qy1);

        // Convert load components to local (qx_loc, qy_loc)
        let (qx_loc, qy_loc) = match frame {
            LoadFrame::Local => (raw_qx, raw_qy),
            LoadFrame::Global => (
                raw_qx * cos_a + raw_qy * sin_a,
                -raw_qx * sin_a + raw_qy * cos_a,
            ),
            LoadFrame::Projected => {
                // Projected onto global horizontal (length L_proj = L * |cos_a|)
                let proj_scale = cos_a.abs().max(1e-4);
                let qx_g = raw_qx * proj_scale;
                let qy_g = raw_qy * proj_scale;
                (
                    qx_g * cos_a + qy_g * sin_a,
                    -qx_g * sin_a + qy_g * cos_a,
                )
            }
        };

        // Shape function evaluations
        let n_trans = hermite_shape_functions(x, l);
        let n_ax_1 = 1.0 - x / l;
        let n_ax_2 = x / l;

        let d_vol = wt * 0.5 * span;

        f0[0] += qx_loc * n_ax_1 * d_vol;
        f0[3] += qx_loc * n_ax_2 * d_vol;

        f0[1] += qy_loc * n_trans[0] * d_vol;
        f0[2] += qy_loc * n_trans[1] * d_vol;
        f0[4] += qy_loc * n_trans[2] * d_vol;
        f0[5] += qy_loc * n_trans[3] * d_vol;
    }

    f0
}

/// Evaluates exact fixed-end forces for a concentrated point load at x = x_ratio * L
pub fn evaluate_point_load(
    l: f64,
    fx: f64,
    fy: f64,
    mz: f64,
    x_ratio: f64,
    cos_a: f64,
    sin_a: f64,
    frame: LoadFrame,
) -> [f64; 6] {
    let x = (x_ratio * l).clamp(0.0, l);
    let (fx_loc, fy_loc) = match frame {
        LoadFrame::Local => (fx, fy),
        LoadFrame::Global | LoadFrame::Projected => (
            fx * cos_a + fy * sin_a,
            -fx * sin_a + fy * cos_a,
        ),
    };

    let n_trans = hermite_shape_functions(x, l);
    let dn_trans = hermite_shape_derivs(x, l);

    let mut f0 = [0.0; 6];
    f0[0] = fx_loc * (1.0 - x / l);
    f0[3] = fx_loc * (x / l);

    f0[1] = fy_loc * n_trans[0] - mz * dn_trans[0];
    f0[2] = fy_loc * n_trans[1] - mz * dn_trans[1];
    f0[4] = fy_loc * n_trans[2] - mz * dn_trans[2];
    f0[5] = fy_loc * n_trans[3] - mz * dn_trans[3];

    f0
}

/// Evaluates fixed-end forces for uniform thermal and temperature gradient loads
pub fn evaluate_thermal_load(
    e: f64,
    a: f64,
    i: f64,
    alpha_t: f64,
    depth_h: Option<f64>,
    dt_uniform: f64,
    dt_gradient: f64,
) -> [f64; 6] {
    let mut f0 = [0.0; 6];
    // Axial thermal expansion
    if dt_uniform.abs() > 1e-9 {
        let n0 = e * a * alpha_t * dt_uniform;
        f0[0] = -n0;
        f0[3] = n0;
    }
    // Thermal curvature gradient
    if dt_gradient.abs() > 1e-9 {
        if let Some(h) = depth_h {
            let m0 = e * i * alpha_t * dt_gradient / h;
            f0[2] = m0;
            f0[5] = -m0;
        }
    }
    f0
}
