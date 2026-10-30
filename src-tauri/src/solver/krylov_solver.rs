use num_complex::Complex64;

#[derive(Debug, Clone)]
pub struct GmresResult {
    pub solution: Vec<Complex64>,
    pub converged: bool,
    pub iterations: usize,
    pub relative_residual: f64,
}

/// Solves the complex linear system A * x = b using restarted Generalized Minimal Residual (GMRES).
///
/// Matrix-free: `apply_a` computes the operator-vector product A * v without explicitly forming A.
/// In our BEM-FEM framework:
///   A * v = v + E(s)^{-1} [ (K_FEM + s^2 * M_FEM) * v ]
pub fn solve_complex_gmres<F>(
    n: usize,
    b: &[Complex64],
    apply_a: F,
    max_iter: usize,
    tol: f64,
    restart: usize,
) -> GmresResult
where
    F: Fn(&[Complex64]) -> Vec<Complex64>,
{
    let b_norm = l2_norm(b);
    if b_norm < 1e-15 {
        return GmresResult {
            solution: vec![Complex64::new(0.0, 0.0); n],
            converged: true,
            iterations: 0,
            relative_residual: 0.0,
        };
    }

    let mut x = vec![Complex64::new(0.0, 0.0); n];
    let m = restart.min(n).max(5);
    let mut total_iter = 0;

    while total_iter < max_iter {
        // Compute initial residual r0 = b - A * x
        let ax = apply_a(&x);
        let mut r = vec![Complex64::new(0.0, 0.0); n];
        for i in 0..n {
            r[i] = b[i] - ax[i];
        }

        let r_norm = l2_norm(&r);
        if r_norm / b_norm < tol {
            return GmresResult {
                solution: x,
                converged: true,
                iterations: total_iter,
                relative_residual: r_norm / b_norm,
            };
        }

        // V basis: (m + 1) vectors of dimension n
        let mut v: Vec<Vec<Complex64>> = Vec::with_capacity(m + 1);
        v.push(scale_vec(&r, 1.0 / r_norm));

        // Upper Hessenberg matrix H: (m + 1) x m
        let mut h = vec![vec![Complex64::new(0.0, 0.0); m]; m + 1];

        // Givens rotations: cs (cos), sn (sin)
        let mut cs: Vec<f64> = vec![0.0; m];
        let mut sn: Vec<Complex64> = vec![Complex64::new(0.0, 0.0); m];

        // Right-hand side for Hessenberg least squares: g = [r_norm, 0, ..., 0]
        let mut g = vec![Complex64::new(0.0, 0.0); m + 1];
        g[0] = Complex64::new(r_norm, 0.0);

        let mut k = 0;
        let mut converged = false;

        while k < m && total_iter < max_iter {
            total_iter += 1;

            // w = A * v_k
            let mut w = apply_a(&v[k]);

            // Arnoldi Modified Gram-Schmidt orthonormalization
            for i in 0..=k {
                let h_ik = dot_product(&v[i], &w);
                h[i][k] = h_ik;
                for j in 0..n {
                    w[j] -= h_ik * v[i][j];
                }
            }

            let h_next = l2_norm(&w);
            h[k + 1][k] = Complex64::new(h_next, 0.0);

            if h_next > 1e-14 {
                v.push(scale_vec(&w, 1.0 / h_next));
            } else {
                v.push(vec![Complex64::new(0.0, 0.0); n]);
            }

            // Apply existing Givens rotations to the new column
            for i in 0..k {
                let c = cs[i];
                let s = sn[i];
                let h0 = h[i][k];
                let h1 = h[i + 1][k];
                h[i][k] = c * h0 + s * h1;
                h[i + 1][k] = -s.conj() * h0 + c * h1;
            }

            // Generate new Givens rotation to eliminate H(k+1, k)
            let (c, s) = givens_rotation(h[k][k], h[k + 1][k]);
            cs[k] = c;
            sn[k] = s;

            let h0 = h[k][k];
            let h1 = h[k + 1][k];
            h[k][k] = c * h0 + s * h1;
            h[k + 1][k] = Complex64::new(0.0, 0.0);

            // Apply rotation to g
            let g0 = g[k];
            let g1 = g[k + 1];
            g[k] = c * g0 + s * g1;
            g[k + 1] = -s.conj() * g0 + c * g1;

            let current_residual = g[k + 1].norm();
            if current_residual / b_norm < tol {
                k += 1;
                converged = true;
                break;
            }

            k += 1;
        }

        // Back-substitution to solve upper triangular H(1..k, 1..k) * y = g(1..k)
        let mut y = vec![Complex64::new(0.0, 0.0); k];
        for i in (0..k).rev() {
            let mut sum = g[i];
            for j in (i + 1)..k {
                sum -= h[i][j] * y[j];
            }
            y[i] = sum / h[i][i];
        }

        // Update solution x = x + sum_{i=0}^{k-1} y_i * v_i
        for i in 0..k {
            for j in 0..n {
                x[j] += y[i] * v[i][j];
            }
        }

        if converged {
            let final_res = g[k].norm() / b_norm;
            return GmresResult {
                solution: x,
                converged: true,
                iterations: total_iter,
                relative_residual: final_res,
            };
        }
    }

    let final_res = l2_norm(&apply_a(&x)) / b_norm;
    GmresResult {
        solution: x,
        converged: false,
        iterations: total_iter,
        relative_residual: final_res,
    }
}

fn l2_norm(vec: &[Complex64]) -> f64 {
    vec.iter().map(|z| z.norm_sqr()).sum::<f64>().sqrt()
}

fn dot_product(u: &[Complex64], v: &[Complex64]) -> Complex64 {
    // Conjugate linear in first argument (Hermitian inner product: u^H * v)
    u.iter().zip(v.iter()).map(|(ui, vi)| ui.conj() * vi).sum()
}

fn scale_vec(vec: &[Complex64], scale: f64) -> Vec<Complex64> {
    vec.iter().map(|z| z * scale).collect()
}

fn givens_rotation(a: Complex64, b: Complex64) -> (f64, Complex64) {
    if b.norm_sqr() < 1e-30 {
        return (1.0, Complex64::new(0.0, 0.0));
    }
    let norm = (a.norm_sqr() + b.norm_sqr()).sqrt();
    let c = a.norm() / norm;
    let s = (a / a.norm()) * (b.conj() / norm);
    (c, s)
}

#[cfg(test)]
#[path = "krylov_solver_test.rs"]
mod tests;
