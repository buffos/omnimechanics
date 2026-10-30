use num_complex::Complex64;
use std::f64::consts::PI;

/// Computes modified Bessel functions of the second kind K_0(z), K_1(z), K_2(z)
/// for complex arguments z in the right half-plane (Re(z) > 0).
///
/// These functions are the fundamental wave decay kernels for the 2D elastodynamic
/// Green's tensor in the transformed Laplace / frequency domain.

/// Evaluates K_0(z) for Re(z) > 0.
pub fn bessel_k0(z: Complex64) -> Complex64 {
    let r = z.norm();
    if r < 1e-12 {
        // Logarithmic singular behavior for small |z|
        return -(z / 2.0).ln() - 0.5772156649015329;
    }

    if r <= 3.5 {
        // Ascending power series expansion with digamma function psi(k)
        // K_0(z) = -ln(z/2) I_0(z) + sum_{k=0}^inf (z^2/4)^k / (k!)^2 * psi(k+1)
        let z2_4 = (z * z) * 0.25;
        let mut term = Complex64::new(1.0, 0.0);
        let mut i0 = term;
        let mut sum_psi = term * (-0.5772156649015329); // psi(1) = -gamma
        let mut psi = -0.5772156649015329;

        for k in 1..25 {
            term = term * z2_4 / (k as f64 * k as f64);
            psi += 1.0 / (k as f64);
            i0 += term;
            sum_psi += term * psi;
            if term.norm() < 1e-15 * sum_psi.norm() {
                break;
            }
        }
        -(z * 0.5).ln() * i0 + sum_psi
    } else {
        // Asymptotic expansion for large |z|:
        // K_0(z) ~ sqrt(pi / (2z)) * e^{-z} * [1 - 1/(8z) + 9/(128z^2) - 225/(3072z^3) + ...]
        let prefactor = (Complex64::new(PI, 0.0) / (z * 2.0)).sqrt() * (-z).exp();
        let inv_8z = 1.0 / (z * 8.0);
        let mut poly = Complex64::new(1.0, 0.0);

        // Coefficients: a_k = prod_{j=1}^k (2j - 1)^2 / (j * 8)
        let mu_coeffs = [1.0, -1.0, 9.0 / 2.0, -75.0 / 6.0, 3675.0 / 24.0];
        let mut cur_inv = Complex64::new(1.0, 0.0);
        for &c in &mu_coeffs[1..] {
            cur_inv = cur_inv * inv_8z;
            poly += cur_inv * c;
        }
        prefactor * poly
    }
}

/// Evaluates K_1(z) for Re(z) > 0.
pub fn bessel_k1(z: Complex64) -> Complex64 {
    let r = z.norm();
    if r < 1e-12 {
        return 1.0 / z;
    }

    if r <= 3.5 {
        // Ascending series for K_1(z):
        // K_1(z) = 1/z + (z/2) ln(z/2) I_1(z) - (z/4) sum_{k=0}^inf (z^2/4)^k / (k!(k+1)!) * (psi(k+1) + psi(k+2))
        let z2_4 = (z * z) * 0.25;
        let mut term = Complex64::new(1.0, 0.0); // k=0 term
        let gamma = 0.5772156649015329;
        let mut psi_k = -gamma;
        let mut psi_k1 = -gamma + 1.0;
        let mut sum_terms = term * (psi_k + psi_k1);
        let mut i1 = term * 0.5 * z;

        let mut term_i = term * 0.5 * z;
        for k in 1..25 {
            term = term * z2_4 / (k as f64 * (k + 1) as f64);
            psi_k += 1.0 / (k as f64);
            psi_k1 += 1.0 / ((k + 1) as f64);
            sum_terms += term * (psi_k + psi_k1);

            term_i = term_i * z2_4 / (k as f64 * (k + 1) as f64);
            i1 += term_i;

            if term.norm() < 1e-15 * sum_terms.norm() {
                break;
            }
        }
        (1.0 / z) + (z * 0.5).ln() * i1 - (z * 0.25) * sum_terms
    } else {
        // Asymptotic expansion:
        // K_1(z) ~ sqrt(pi / (2z)) * e^{-z} * [1 + 3/(8z) - 15/(128z^2) + ...]
        let prefactor = (Complex64::new(PI, 0.0) / (z * 2.0)).sqrt() * (-z).exp();
        let inv_8z = 1.0 / (z * 8.0);
        let poly = Complex64::new(1.0, 0.0)
            + inv_8z * 3.0
            - (inv_8z * inv_8z) * (15.0 / 2.0)
            + (inv_8z * inv_8z * inv_8z) * (105.0 / 2.0);
        prefactor * poly
    }
}

/// Evaluates K_2(z) via the exact recurrence relation:
/// K_2(z) = K_0(z) + (2 / z) * K_1(z)
pub fn bessel_k2(z: Complex64) -> Complex64 {
    bessel_k0(z) + (2.0 / z) * bessel_k1(z)
}

#[cfg(test)]
#[path = "bessel_test.rs"]
mod tests;
