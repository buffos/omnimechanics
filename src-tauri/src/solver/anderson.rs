//! Anderson Acceleration for Fixed-Point Iterations (Walker & Ni, SIAM 2011).
//!
//! Accelerates convergence of sequences $x_{k+1} = g(x_k)$ by computing
//! optimal linear combinations of past iterates in a Krylov-like subspace.

#[derive(Debug, Clone)]
pub struct AndersonAccelerator {
    m: usize,
    beta: f64,
    x_hist: Vec<Vec<f64>>,
    f_hist: Vec<Vec<f64>>,
}

impl AndersonAccelerator {
    /// Create a new Anderson accelerator with subspace depth `m` (typically 3-5)
    /// and relaxation parameter `beta` (typically 0.9 - 1.0).
    pub fn new(m: usize, beta: f64) -> Self {
        Self {
            m: m.max(1),
            beta,
            x_hist: Vec::with_capacity(m + 2),
            f_hist: Vec::with_capacity(m + 2),
        }
    }

    /// Reset internal history.
    pub fn reset(&mut self) {
        self.x_hist.clear();
        self.f_hist.clear();
    }

    /// Compute next accelerated iterate $x_{k+1}$ given current state $x_k$
    /// and fixed-point residual $f_k = g(x_k) - x_k = \Delta x_k$.
    pub fn compute_update(&mut self, x_k: &[f64], f_k: &[f64]) -> Vec<f64> {
        let n = x_k.len();
        self.x_hist.push(x_k.to_vec());
        self.f_hist.push(f_k.to_vec());

        // Keep at most m + 1 entries
        if self.x_hist.len() > self.m + 1 {
            self.x_hist.remove(0);
            self.f_hist.remove(0);
        }

        let num_states = self.x_hist.len();
        if num_states < 2 {
            // First iteration: standard Picard step with relaxation beta
            let mut x_next = vec![0.0; n];
            for i in 0..n {
                x_next[i] = x_k[i] + self.beta * f_k[i];
            }
            return x_next;
        }

        let k = num_states - 1; // number of available difference vectors

        // Form difference vectors:
        // delta_x[j] = x_{j+1} - x_j
        // delta_f[j] = f_{j+1} - f_j
        let mut delta_x: Vec<Vec<f64>> = Vec::with_capacity(k);
        let mut delta_f: Vec<Vec<f64>> = Vec::with_capacity(k);

        for j in 0..k {
            let mut dx = vec![0.0; n];
            let mut df = vec![0.0; n];
            let xj = &self.x_hist[j];
            let xj1 = &self.x_hist[j + 1];
            let fj = &self.f_hist[j];
            let fj1 = &self.f_hist[j + 1];
            for i in 0..n {
                dx[i] = xj1[i] - xj[i];
                df[i] = fj1[i] - fj[i];
            }
            delta_x.push(dx);
            delta_f.push(df);
        }

        // Solve least squares problem: min || f_k - sum_{j=0}^{k-1} gamma_j delta_f[j] ||_2
        // Normal equations: M * gamma = b, where M_{ij} = <delta_f[i], delta_f[j]>, b_i = <delta_f[i], f_k>
        let mut m_mat = vec![vec![0.0; k]; k];
        let mut b_vec = vec![0.0; k];
        let mut trace = 0.0;

        for i in 0..k {
            let mut b_val = 0.0;
            for p in 0..n {
                b_val += delta_f[i][p] * f_k[p];
            }
            b_vec[i] = b_val;

            for j in 0..k {
                if j >= i {
                    let mut dot = 0.0;
                    for p in 0..n {
                        dot += delta_f[i][p] * delta_f[j][p];
                    }
                    m_mat[i][j] = dot;
                    m_mat[j][i] = dot;
                    if i == j {
                        trace += dot;
                    }
                }
            }
        }

        // Tikhonov regularization
        let reg = 1e-10 * (trace / k as f64).max(1.0);
        for i in 0..k {
            m_mat[i][i] += reg;
        }

        // Solve M * gamma = b via Gaussian elimination with partial pivoting (k <= m <= 5)
        let gamma = solve_dense_system(&mut m_mat, &mut b_vec);

        let mut x_next = vec![0.0; n];
        match gamma {
            Some(g) => {
                for i in 0..n {
                    let mut sum_terms = 0.0;
                    for j in 0..k {
                        sum_terms += g[j] * (delta_x[j][i] + self.beta * delta_f[j][i]);
                    }
                    x_next[i] = x_k[i] + self.beta * f_k[i] - sum_terms;
                }
            }
            None => {
                // Fallback to standard Picard step if system is degenerate
                for i in 0..n {
                    x_next[i] = x_k[i] + self.beta * f_k[i];
                }
            }
        }

        x_next
    }
}

/// Gaussian elimination with partial pivoting for small dense systems (k <= 8).
fn solve_dense_system(a: &mut [Vec<f64>], b: &mut [f64]) -> Option<Vec<f64>> {
    let k = b.len();
    for col in 0..k {
        // Pivot selection
        let mut max_val = a[col][col].abs();
        let mut max_row = col;
        for row in (col + 1)..k {
            let val = a[row][col].abs();
            if val > max_val {
                max_val = val;
                max_row = row;
            }
        }

        if max_val < 1e-14 {
            return None; // Singular
        }

        if max_row != col {
            a.swap(col, max_row);
            b.swap(col, max_row);
        }

        let pivot = a[col][col];
        for row in (col + 1)..k {
            let factor = a[row][col] / pivot;
            for c in col..k {
                a[row][c] -= factor * a[col][c];
            }
            b[row] -= factor * b[col];
        }
    }

    // Back substitution
    let mut x = vec![0.0; k];
    for row in (0..k).rev() {
        let mut sum = b[row];
        for c in (row + 1)..k {
            sum -= a[row][c] * x[c];
        }
        let diag = a[row][row];
        if diag.abs() < 1e-14 {
            return None;
        }
        x[row] = sum / diag;
    }

    Some(x)
}

#[cfg(test)]
#[path = "anderson_test.rs"]
mod tests;
