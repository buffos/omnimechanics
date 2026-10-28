use crate::material::traits::MaterialFieldStrategy;
use crate::material::types::{LocalMaterialProperties, ReferenceMaterial};

/// Vecchio-Collins Modified Compression Field Theory (MCFT, 1986).
/// Captures non-linear concrete behavior: compression softening due to transverse cracking,
/// tensile cracking, and tension-stiffening from bonded reinforcement.
#[derive(Debug, Clone)]
pub struct VecchioCollinsField {
    /// Compressive cylinder strength f'c (Pa), e.g. 30e6 Pa (30 MPa)
    pub fc_prime: f64,
    /// Uniaxial compressive peak strain epsilon_0 (typically 0.002)
    pub epsilon_0: f64,
    /// Tensile cracking strength ft (Pa), typically 0.33 * sqrt(f'c)
    pub ft: f64,
    /// Longitudinal / shear reinforcement ratio rho_s (e.g. 0.015 for 1.5%)
    pub rebar_ratio: f64,
    /// Simulated or current maximum principal tensile strain epsilon_1
    pub simulated_strain: f64,
}

impl VecchioCollinsField {
    pub fn new(
        fc_prime: f64,
        epsilon_0: f64,
        ft: f64,
        rebar_ratio: f64,
        simulated_strain: f64,
    ) -> Self {
        let fc = fc_prime.abs().max(1e6);
        let ep0 = if epsilon_0.abs() < 1e-5 { 0.002 } else { epsilon_0.abs() };
        let ft_val = if ft <= 0.0 { 0.33 * (fc * 1e-6).sqrt() * 1e6 } else { ft };

        Self {
            fc_prime: fc,
            epsilon_0: ep0,
            ft: ft_val,
            rebar_ratio: rebar_ratio.max(0.0),
            simulated_strain: simulated_strain.max(0.0),
        }
    }

    /// Initial uncracked concrete modulus Ec = 2 * f'c / epsilon_0
    pub fn initial_modulus(&self) -> f64 {
        2.0 * self.fc_prime / self.epsilon_0
    }

    /// Vecchio-Collins softening coefficient beta = 1 / (0.8 + 0.34 * epsilon_1 / epsilon_0) <= 1.0
    pub fn softening_beta(&self, eps1: f64) -> f64 {
        let ratio = eps1.max(0.0) / self.epsilon_0;
        let denom = 0.8 + 0.34 * ratio;
        (1.0 / denom).min(1.0)
    }

    /// Post-cracking tension stiffening stress fc1
    pub fn tensile_stress(&self, eps1: f64) -> f64 {
        let ec = self.initial_modulus();
        let eps_cr = self.ft / ec;
        if eps1 <= eps_cr {
            ec * eps1
        } else {
            // Classical Vecchio-Collins tension stiffening formulation
            self.ft / (1.0 + (500.0 * eps1).sqrt())
        }
    }

    /// Softened compressive stress-strain curve fc2(eps2)
    pub fn compressive_stress(&self, eps2: f64, eps1: f64) -> f64 {
        let beta = self.softening_beta(eps1);
        let fc2_max = beta * self.fc_prime;
        let eps2_p = beta * self.epsilon_0;

        let eta = eps2 / eps2_p;
        if eta <= 1.0 {
            fc2_max * (2.0 * eta - eta * eta)
        } else {
            // Softening descending branch
            fc2_max * (1.0 - ((eta - 1.0) / 2.0).powi(2)).max(0.1 * fc2_max)
        }
    }

    /// Secant elastic modulus Es evaluated at current principal strains
    pub fn secant_modulus(&self, eps1: f64, eps2: f64) -> f64 {
        let fc2 = self.compressive_stress(eps2, eps1);
        let es2 = fc2 / eps2.max(1e-6);
        es2.clamp(0.05 * self.initial_modulus(), self.initial_modulus())
    }
}

impl MaterialFieldStrategy for VecchioCollinsField {
    fn evaluate(&self, point: [f64; 3], reference: &ReferenceMaterial) -> LocalMaterialProperties {
        // Vary simulated strain spatially to reflect structural stress concentration (e.g. bending/shear)
        // High tensile strain at bottom / sides, low at top / neutral axis
        let normalized_y = (point[1].abs() / 2.0).min(1.0);
        let local_eps1 = self.simulated_strain * (0.2 + 0.8 * normalized_y);
        let local_eps2 = self.epsilon_0 * (0.5 + 0.5 * (1.0 - normalized_y));

        let es = self.secant_modulus(local_eps1, local_eps2);
        let nu = 0.15; // Cracked concrete Poisson contraction
        let rho = reference.rho0;
        let mu = es / (2.0 * (1.0 + nu));
        let lambda = (es * nu) / ((1.0 + nu) * (1.0 - 2.0 * nu));

        LocalMaterialProperties {
            e: es,
            nu,
            rho,
            mu,
            lambda,
            secant_modulus: Some(es),
            yield_stress: Some(self.fc_prime),
            scalar_metric: es,
        }
    }
}
