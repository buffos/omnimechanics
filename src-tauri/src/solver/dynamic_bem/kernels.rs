use crate::material::types::ReferenceMaterial;
use crate::solver::bessel::{bessel_k0, bessel_k1, bessel_k2};
use num_complex::Complex64;
use std::f64::consts::PI;

/// Evaluates the 2D elastodynamic Green's displacement tensor U*(r, s)
/// in the transformed Laplace / complex frequency domain.
pub fn dynamic_kelvin_u(
    r: f64,
    dr: [f64; 2],
    s: Complex64,
    reference: &ReferenceMaterial,
) -> [[Complex64; 2]; 2] {
    let rho = reference.rho0;
    let mu = reference.mu0();
    let lambda = reference.lambda0();

    let cp = ((lambda + 2.0 * mu) / rho).sqrt();
    let cs = (mu / rho).sqrt();

    let kp = s / cp;
    let ks = s / cs;

    let zp = kp * r;
    let zs = ks * r;

    // Potential functions psi and chi
    let k2_s = bessel_k2(zs);
    let k2_p = bessel_k2(zp);
    let gamma_sq = (cs / cp) * (cs / cp);
    let chi = k2_s - k2_p * gamma_sq;

    let k0_s = bessel_k0(zs);
    let k1_s = bessel_k1(zs);
    let k1_p = bessel_k1(zp);
    let gamma = cs / cp;
    let psi = k0_s + (k1_s - k1_p * gamma) / zs;

    let prefactor = 1.0 / (2.0 * PI * rho * cs * cs);

    let mut u_star = [[Complex64::new(0.0, 0.0); 2]; 2];
    for i in 0..2 {
        for j in 0..2 {
            let delta = if i == j { 1.0 } else { 0.0 };
            u_star[i][j] = (psi * delta - chi * (dr[i] * dr[j])) * prefactor;
        }
    }

    u_star
}

/// Evaluates the 2D elastodynamic Green's traction tensor T*(r, s)
pub fn dynamic_kelvin_t(
    r: f64,
    dr: [f64; 2],
    normal: [f64; 2],
    s: Complex64,
    reference: &ReferenceMaterial,
) -> [[Complex64; 2]; 2] {
    let rho = reference.rho0;
    let mu = reference.mu0();
    let lambda = reference.lambda0();

    let cp = ((lambda + 2.0 * mu) / rho).sqrt();
    let cs = (mu / rho).sqrt();

    let kp = s / cp;
    let ks = s / cs;

    let zp = kp * r;
    let zs = ks * r;

    let k2_s = bessel_k2(zs);
    let k2_p = bessel_k2(zp);
    let gamma_sq = (cs / cp) * (cs / cp);
    let chi = k2_s - k2_p * gamma_sq;

    let k0_s = bessel_k0(zs);
    let k1_s = bessel_k1(zs);
    let k1_p = bessel_k1(zp);
    let gamma = cs / cp;
    let _psi = k0_s + (k1_s - k1_p * gamma) / zs;

    let d_psi = -k1_s * ks - chi / r;
    let d_chi = -k1_s * ks + k1_p * kp * gamma_sq - (chi * 2.0) / r;

    let dr_dn = dr[0] * normal[0] + dr[1] * normal[1];
    let prefactor = 1.0 / (2.0 * PI);

    let mut t_star = [[Complex64::new(0.0, 0.0); 2]; 2];
    for i in 0..2 {
        for k in 0..2 {
            let delta = if i == k { 1.0 } else { 0.0 };
            let term_a = (d_psi - chi / r) * (delta * dr_dn + dr[k] * normal[i]);
            let term_b = Complex64::new(lambda / mu, 0.0) * (d_psi - d_chi - chi / r) * (dr[i] * normal[k])
                - (chi * 2.0 / r) * (normal[k] * dr[i]);
            let term_c = -(d_chi * 2.0 - (chi * 4.0) / r) * (dr[i] * dr[k] * dr_dn);
            let t_val = term_a + term_b + term_c;
            t_star[i][k] = t_val * prefactor;
        }
    }

    t_star
}

/// Evaluates the 3D elastodynamic Green's displacement tensor U*(r, s)
/// in the transformed Laplace / complex frequency domain using exact analytical spherical wave potentials.
pub fn dynamic_kelvin_3d_u(
    r: f64,
    dr: [f64; 3],
    s: Complex64,
    reference: &ReferenceMaterial,
) -> [[Complex64; 3]; 3] {
    let rho = reference.rho0;
    let mu = reference.mu0();
    let lambda = reference.lambda0();

    let cp = ((lambda + 2.0 * mu) / rho).sqrt();
    let cs = (mu / rho).sqrt();

    let kp = s / cp;
    let ks = s / cs;

    let zp = kp * r;
    let zs = ks * r;

    let gamma = cs / cp;
    let gamma_sq = gamma * gamma;

    let (psi_raw, chi_raw) = if zs.norm() < 0.1 {
        let psi = Complex64::new(0.5 * (1.0 + gamma_sq), 0.0)
            - (zs * 2.0 + zp * gamma_sq) / 3.0
            + (zs * zs * 3.0 + zp * zp * gamma_sq) / 8.0
            - (zs * zs * zs * 4.0 + zp * zp * zp * gamma_sq) / 30.0;
        let chi = Complex64::new(-0.5 * (1.0 - gamma_sq), 0.0)
            + (zs * zs - zp * zp * gamma_sq) / 8.0
            - (zs * zs * zs - zp * zp * zp * gamma_sq) / 15.0;
        (psi, chi)
    } else {
        let exp_s = (-zs).exp();
        let exp_p = (-zp).exp();
        let psi = (Complex64::new(1.0, 0.0) + Complex64::new(1.0, 0.0) / zs + Complex64::new(1.0, 0.0) / (zs * zs)) * exp_s
            - (Complex64::new(1.0, 0.0) / zp + Complex64::new(1.0, 0.0) / (zp * zp)) * (gamma_sq * exp_p);
        let chi = (Complex64::new(1.0, 0.0) + Complex64::new(3.0, 0.0) / zs + Complex64::new(3.0, 0.0) / (zs * zs)) * exp_s
            - (Complex64::new(1.0, 0.0) + Complex64::new(3.0, 0.0) / zp + Complex64::new(3.0, 0.0) / (zp * zp)) * (gamma_sq * exp_p);
        (psi, chi)
    };

    let factor = Complex64::new(1.0 / (4.0 * PI * mu * r), 0.0);

    let mut u_star = [[Complex64::new(0.0, 0.0); 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            let delta = if i == j { 1.0 } else { 0.0 };
            u_star[i][j] = factor * (psi_raw * delta - chi_raw * (dr[i] * dr[j]));
        }
    }

    u_star
}

/// Evaluates the 3D elastodynamic Green's traction tensor T*(r, s)
pub fn dynamic_kelvin_3d_t(
    r: f64,
    dr: [f64; 3],
    normal: [f64; 3],
    s: Complex64,
    reference: &ReferenceMaterial,
) -> [[Complex64; 3]; 3] {
    let rho = reference.rho0;
    let mu = reference.mu0();
    let lambda = reference.lambda0();

    let cp = ((lambda + 2.0 * mu) / rho).sqrt();
    let cs = (mu / rho).sqrt();

    let kp = s / cp;
    let ks = s / cs;

    let zp = kp * r;
    let zs = ks * r;

    let gamma = cs / cp;
    let gamma_sq = gamma * gamma;

    let (psi_raw, chi_raw, d_psi_raw, d_chi_raw) = if zs.norm() < 0.1 {
        let psi = Complex64::new(0.5 * (1.0 + gamma_sq), 0.0)
            - (zs * 2.0 + zp * gamma_sq) / 3.0
            + (zs * zs * 3.0 + zp * zp * gamma_sq) / 8.0
            - (zs * zs * zs * 4.0 + zp * zp * zp * gamma_sq) / 30.0;
        let chi = Complex64::new(-0.5 * (1.0 - gamma_sq), 0.0)
            + (zs * zs - zp * zp * gamma_sq) / 8.0
            - (zs * zs * zs - zp * zp * zp * gamma_sq) / 15.0;
        let d_psi = ks * (
            Complex64::new(-(2.0 + gamma * gamma_sq) / 3.0, 0.0)
            + (zs * 3.0 + zp * (gamma * gamma_sq)) / 4.0
            - (zs * zs * 4.0 + zp * zp * (gamma * gamma_sq)) / 10.0
        );
        let d_chi = ks * (
            (zs - zp * (gamma * gamma_sq)) / 4.0
            - (zs * zs - zp * zp * (gamma * gamma_sq)) / 5.0
        );
        (psi, chi, d_psi, d_chi)
    } else {
        let exp_s = (-zs).exp();
        let exp_p = (-zp).exp();
        let psi = (Complex64::new(1.0, 0.0) + Complex64::new(1.0, 0.0) / zs + Complex64::new(1.0, 0.0) / (zs * zs)) * exp_s
            - (Complex64::new(1.0, 0.0) / zp + Complex64::new(1.0, 0.0) / (zp * zp)) * (gamma_sq * exp_p);
        let chi = (Complex64::new(1.0, 0.0) + Complex64::new(3.0, 0.0) / zs + Complex64::new(3.0, 0.0) / (zs * zs)) * exp_s
            - (Complex64::new(1.0, 0.0) + Complex64::new(3.0, 0.0) / zp + Complex64::new(3.0, 0.0) / (zp * zp)) * (gamma_sq * exp_p);
        let d_psi = -ks * (Complex64::new(1.0, 0.0) + Complex64::new(1.0, 0.0) / zs + Complex64::new(2.0, 0.0) / (zs * zs) + Complex64::new(2.0, 0.0) / (zs * zs * zs)) * exp_s
            + kp * (Complex64::new(1.0, 0.0) / zp + Complex64::new(2.0, 0.0) / (zp * zp) + Complex64::new(2.0, 0.0) / (zp * zp * zp)) * (gamma_sq * exp_p);
        let d_chi = -ks * (Complex64::new(1.0, 0.0) + Complex64::new(3.0, 0.0) / zs + Complex64::new(6.0, 0.0) / (zs * zs) + Complex64::new(6.0, 0.0) / (zs * zs * zs)) * exp_s
            + kp * (Complex64::new(1.0, 0.0) + Complex64::new(3.0, 0.0) / zp + Complex64::new(6.0, 0.0) / (zp * zp) + Complex64::new(6.0, 0.0) / (zp * zp * zp)) * (gamma_sq * exp_p);
        (psi, chi, d_psi, d_chi)
    };

    let psi_d = psi_raw / r;
    let chi_d = chi_raw / r;
    let d_psi_d = d_psi_raw / r - psi_raw / (r * r);
    let d_chi_d = d_chi_raw / r - chi_raw / (r * r);

    let dr_dn = dr[0] * normal[0] + dr[1] * normal[1] + dr[2] * normal[2];
    let prefactor = 1.0 / (4.0 * PI);
    let lam_over_mu = lambda / mu;

    let mut t_star = [[Complex64::new(0.0, 0.0); 3]; 3];
    for i in 0..3 {
        for k in 0..3 {
            let delta = if i == k { 1.0 } else { 0.0 };
            let term1 = (d_psi_d - chi_d / r) * (delta * dr_dn + dr[k] * normal[i]);
            let term2 = - (chi_d * 2.0 / r) * (normal[k] * dr[i] - dr[i] * dr[k] * 2.0 * dr_dn);
            let term3 = - d_chi_d * 2.0 * dr[i] * dr[k] * dr_dn;
            let term4 = Complex64::new(lam_over_mu, 0.0) * (d_psi_d - d_chi_d - chi_d * 2.0 / r) * (dr[i] * normal[k]);
            t_star[i][k] = (term1 + term2 + term3 + term4) * prefactor;
        }
    }

    t_star
}
