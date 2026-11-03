use super::element_stiffness::{hermite_shape_functions, integrate_distributed_load};
use super::types::{
    ExtremaPoint, Load, LoadFrame, MemberEndForces, MemberExtrema, MemberResult, StationResult,
};

#[derive(Debug, Clone)]
pub struct InternalLoadEffect {
    pub x: f64,
    pub delta_n: f64,
    pub delta_q: f64,
    pub delta_m: f64,
}

/// Evaluates internal forces N(x), Q(x), M(x) along a frame member
pub fn evaluate_member_diagram(
    element_id: &str,
    l: f64,
    e: f64,
    a: f64,
    i_inertia: f64,
    cos_a: f64,
    sin_a: f64,
    u_local: &[f64; 6],
    f_end_local: &[f64; 6],
    loads: &[&Load],
    self_weight: bool,
    gamma: f64,
) -> MemberResult {
    let n1 = f_end_local[0];
    let q1 = f_end_local[1];
    let m1 = f_end_local[2];
    let n2 = f_end_local[3];
    let q2 = f_end_local[4];
    let m2 = f_end_local[5];

    let end_forces = MemberEndForces {
        ni_kn: n1,
        qi_kn: q1,
        mi_knm: m1,
        nj_kn: n2,
        qj_kn: q2,
        mj_knm: m2,
    };

    // Gather station evaluation points: uniform 21 points + load points + zero shear roots
    let mut x_eval_set = Vec::new();
    let num_default_stations = 20;
    for k in 0..=num_default_stations {
        x_eval_set.push((k as f64 / num_default_stations as f64) * l);
    }

    // Add load boundaries / concentrated points
    for load in loads {
        match load {
            Load::MemberPointLoad { x_ratio, .. } => {
                let xp = (x_ratio * l).clamp(0.0, l);
                x_eval_set.push(xp);
                if xp > 1e-4 {
                    x_eval_set.push(xp - 1e-5);
                }
                if xp < l - 1e-4 {
                    x_eval_set.push(xp + 1e-5);
                }
            }
            Load::MemberDistributedLoad {
                a_ratio, b_ratio, ..
            } => {
                let xa = (a_ratio * l).clamp(0.0, l);
                let xb = (b_ratio * l).clamp(0.0, l);
                x_eval_set.push(xa);
                x_eval_set.push(xb);
            }
            _ => {}
        }
    }

    // Function to compute (N, Q, M) at any station x
    let eval_at = |x: f64| -> (f64, f64, f64) {
        let mut n = -n1;
        let mut q = q1;
        let mut m = -m1 + q1 * x;

        // Self-weight contribution
        if self_weight {
            let qx_sw = -gamma * a * sin_a;
            let qy_sw = -gamma * a * cos_a;
            n -= qx_sw * x;
            q += qy_sw * x;
            m += 0.5 * qy_sw * x * x;
        }

        // Interior loads
        for load in loads {
            match load {
                Load::MemberPointLoad {
                    fx,
                    fy,
                    mz,
                    x_ratio,
                    frame,
                    ..
                } => {
                    let xp = x_ratio * l;
                    if x >= xp {
                        let (fx_loc, fy_loc) = match frame {
                            LoadFrame::Local => (*fx, *fy),
                            LoadFrame::Global | LoadFrame::Projected => (
                                fx * cos_a + fy * sin_a,
                                -fx * sin_a + fy * cos_a,
                            ),
                        };
                        n += fx_loc;
                        q += fy_loc;
                        m += fy_loc * (x - xp) - mz;
                    }
                }
                Load::MemberDistributedLoad {
                    qx1,
                    qy1,
                    qx2,
                    qy2,
                    a_ratio,
                    b_ratio,
                    frame,
                    ..
                } => {
                    let xa = a_ratio * l;
                    let xb = b_ratio * l;
                    if x > xa {
                        let span = xb - xa;
                        let x_active = x.min(xb);
                        let sub_span = x_active - xa;

                        let s_end = (x_active - xa) / span;
                        let qy_end = qy1 + s_end * (qy2 - qy1);
                        let qx_end = qx1 + s_end * (qx2 - qx1);

                        // Average and trapezoidal moment
                        let (qx_loc_1, qy_loc_1) = match frame {
                            LoadFrame::Local => (*qx1, *qy1),
                            LoadFrame::Global => (
                                qx1 * cos_a + qy1 * sin_a,
                                -qx1 * sin_a + qy1 * cos_a,
                            ),
                            LoadFrame::Projected => {
                                let scale = cos_a.abs().max(1e-4);
                                (
                                    qx1 * scale * cos_a + qy1 * scale * sin_a,
                                    -qx1 * scale * sin_a + qy1 * scale * cos_a,
                                )
                            }
                        };
                        let (qx_loc_2, qy_loc_2) = match frame {
                            LoadFrame::Local => (qx_end, qy_end),
                            LoadFrame::Global => (
                                qx_end * cos_a + qy_end * sin_a,
                                -qx_end * sin_a + qy_end * cos_a,
                            ),
                            LoadFrame::Projected => {
                                let scale = cos_a.abs().max(1e-4);
                                (
                                    qx_end * scale * cos_a + qy_end * scale * sin_a,
                                    -qx_end * scale * sin_a + qy_end * scale * cos_a,
                                )
                            }
                        };

                        let f_axial = 0.5 * (qx_loc_1 + qx_loc_2) * sub_span;
                        let f_shear = 0.5 * (qy_loc_1 + qy_loc_2) * sub_span;

                        // Centroid of trapezoid from x
                        let lever = if (qy_loc_1 + qy_loc_2).abs() > 1e-9 {
                            (x - xa) - sub_span * (qy_loc_1 + 2.0 * qy_loc_2) / (3.0 * (qy_loc_1 + qy_loc_2))
                        } else {
                            x - xa - 0.5 * sub_span
                        };

                        n -= f_axial;
                        q += f_shear;
                        m += f_shear * lever;
                    }
                }
                _ => {}
            }
        }

        (n, q, m)
    };

    // Find zero-shear root in intervals to capture exact moment extrema
    for i in 0..x_eval_set.len().saturating_sub(1) {
        let x1 = x_eval_set[i];
        let x2 = x_eval_set[i + 1];
        if (x2 - x1) > 1e-4 {
            let (_, q_a, _) = eval_at(x1);
            let (_, q_b, _) = eval_at(x2);
            if q_a * q_b < 0.0 {
                // Linear root finding
                let x_root = x1 - q_a * (x2 - x1) / (q_b - q_a);
                if x_root > x1 && x_root < x2 {
                    x_eval_set.push(x_root);
                }
            }
        }
    }

    x_eval_set.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    x_eval_set.dedup_by(|a, b| (*a - *b).abs() < 1e-6);

    let mut stations = Vec::new();
    let mut extrema = MemberExtrema {
        max_m: ExtremaPoint {
            x_m: 0.0,
            value: -f64::INFINITY,
        },
        min_m: ExtremaPoint {
            x_m: 0.0,
            value: f64::INFINITY,
        },
        max_q: ExtremaPoint {
            x_m: 0.0,
            value: -f64::INFINITY,
        },
        min_q: ExtremaPoint {
            x_m: 0.0,
            value: f64::INFINITY,
        },
        max_n: ExtremaPoint {
            x_m: 0.0,
            value: -f64::INFINITY,
        },
        min_n: ExtremaPoint {
            x_m: 0.0,
            value: f64::INFINITY,
        },
        max_abs_deflection_m: ExtremaPoint {
            x_m: 0.0,
            value: 0.0,
        },
    };

    let u_y_ends = [u_local[1], u_local[2], u_local[4], u_local[5]];

    for x in x_eval_set {
        let (n_val, q_val, m_val) = eval_at(x);

        // Compute local deflection via Hermite shape functions
        let n_trans = hermite_shape_functions(x, l);
        let mut uy_local = n_trans[0] * u_y_ends[0]
            + n_trans[1] * u_y_ends[1]
            + n_trans[2] * u_y_ends[2]
            + n_trans[3] * u_y_ends[3];

        let ux_local = (1.0 - x / l) * u_local[0] + (x / l) * u_local[3];
        let theta_local = (1.0 - x / l) * u_local[2] + (x / l) * u_local[5];

        if m_val > extrema.max_m.value {
            extrema.max_m = ExtremaPoint {
                x_m: x,
                value: m_val,
            };
        }
        if m_val < extrema.min_m.value {
            extrema.min_m = ExtremaPoint {
                x_m: x,
                value: m_val,
            };
        }
        if q_val > extrema.max_q.value {
            extrema.max_q = ExtremaPoint {
                x_m: x,
                value: q_val,
            };
        }
        if q_val < extrema.min_q.value {
            extrema.min_q = ExtremaPoint {
                x_m: x,
                value: q_val,
            };
        }
        if n_val > extrema.max_n.value {
            extrema.max_n = ExtremaPoint {
                x_m: x,
                value: n_val,
            };
        }
        if n_val < extrema.min_n.value {
            extrema.min_n = ExtremaPoint {
                x_m: x,
                value: n_val,
            };
        }
        if uy_local.abs() > extrema.max_abs_deflection_m.value {
            extrema.max_abs_deflection_m = ExtremaPoint {
                x_m: x,
                value: uy_local.abs(),
            };
        }

        stations.push(StationResult {
            x_m: x,
            n_kn: n_val,
            q_kn: q_val,
            m_knm: m_val,
            ux_local_m: ux_local,
            uy_local_m: uy_local,
            theta_rad: theta_local,
        });
    }

    MemberResult {
        element_id: element_id.to_string(),
        length_eff_m: l,
        end_forces_local: end_forces,
        stations,
        extrema,
    }
}
