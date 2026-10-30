use num_complex::Complex64;
use std::f64::consts::PI;

/// Represents a single quadrature node and weight along the Talbot contour in the complex plane.
#[derive(Debug, Clone, Copy)]
pub struct TalbotPoint {
    pub s: Complex64,
    pub weight: Complex64,
}

/// Generates the modified Talbot contour quadrature nodes and integration weights
/// for numerical inversion of the Laplace transform at target time `t`.
///
/// Uses the optimal parameters from Weideman & Trefethen (2007):
/// s(theta) = (N / t) * [ -0.6122 + 0.5017 * theta * cot(0.5017 * theta) + 0.2645 * i * theta ]
pub fn generate_talbot_nodes(num_points: usize, t: f64) -> Vec<TalbotPoint> {
    let n = num_points.max(8);
    let mut points = Vec::with_capacity(n);
    let t_scale = t.max(1e-4);

    let d_theta = 2.0 * PI / n as f64;

    for k in 0..n {
        let theta = -PI + (k as f64 + 0.5) * d_theta;

        let alpha = 0.5017;
        let nu = 0.2645;
        let sigma0 = -0.6122;

        let a_theta = alpha * theta;
        let (cot_a, csc2_a) = if a_theta.abs() < 1e-6 {
            // Taylor expansion near theta = 0: theta * cot(alpha*theta) ~ 1/alpha - (alpha/3)*theta^2
            (1.0 / a_theta, 1.0 / (a_theta * a_theta))
        } else {
            let sin_val = a_theta.sin();
            let cos_val = a_theta.cos();
            (cos_val / sin_val, 1.0 / (sin_val * sin_val))
        };

        let s_re = if a_theta.abs() < 1e-6 {
            (n as f64 / t_scale) * (sigma0 + 1.0 - (a_theta * a_theta) / 3.0)
        } else {
            (n as f64 / t_scale) * (sigma0 + a_theta * cot_a)
        };
        let s_im = (n as f64 / t_scale) * nu * theta;
        let s = Complex64::new(s_re, s_im);

        // Derivative ds/dtheta
        let ds_re = if a_theta.abs() < 1e-6 {
            -(n as f64 / t_scale) * (2.0 * alpha * a_theta / 3.0)
        } else {
            (n as f64 / t_scale) * alpha * (cot_a - a_theta * csc2_a)
        };
        let ds_im = (n as f64 / t_scale) * nu;
        let ds = Complex64::new(ds_re, ds_im);

        // Integration weight w_k = (1 / (2*pi*i)) * ds * d_theta = -i * ds / n
        let weight = Complex64::new(0.0, -1.0) * ds / (n as f64);

        points.push(TalbotPoint { s, weight });
    }

    points
}

/// Evaluates the time-domain response u(x, t) from complex frequency solutions u_hat(x, s_k)
/// using the Talbot contour quadrature sum:
/// u(x, t) = sum_{k=1}^N Re( weight_k * exp(s_k * t) * u_hat(x, s_k) )
pub fn invert_laplace_talbot(
    points: &[TalbotPoint],
    u_hat_list: &[Vec<Complex64>],
    t: f64,
) -> Vec<f64> {
    assert_eq!(points.len(), u_hat_list.len());
    let dof_count = u_hat_list[0].len();
    let mut u_t = vec![0.0; dof_count];

    for (pt, u_hat) in points.iter().zip(u_hat_list.iter()) {
        let exp_st = (pt.s * t).exp();
        let coeff = pt.weight * exp_st;

        for dof in 0..dof_count {
            let term = coeff * u_hat[dof];
            u_t[dof] += term.re;
        }
    }

    u_t
}

#[cfg(test)]
#[path = "talbot_test.rs"]
mod tests;
