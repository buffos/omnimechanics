use crate::material::traits::MaterialFieldStrategy;
use crate::material::types::{LocalMaterialProperties, ReferenceMaterial};

/// J2 von Mises Elastoplasticity representation.
/// Models yield surface evolution, isotropic hardening, and effective elastoplastic stiffness.
#[derive(Debug, Clone)]
pub struct J2PlasticityField {
    /// Initial yield strength sigma_y0 (Pa), e.g. 250e6 Pa (250 MPa for structural steel)
    pub sigma_y0: f64,
    /// Plastic hardening modulus H (Pa), e.g. 1000e6 Pa (1 GPa)
    pub hardening_modulus: f64,
    /// Maximum accumulated equivalent plastic strain in the domain
    pub plastic_strain: f64,
}

impl J2PlasticityField {
    pub fn new(sigma_y0: f64, hardening_modulus: f64, plastic_strain: f64) -> Self {
        Self {
            sigma_y0: sigma_y0.max(1e6),
            hardening_modulus: hardening_modulus.max(0.0),
            plastic_strain: plastic_strain.max(0.0),
        }
    }

    /// Current yield stress sigma_y(ep) = sigma_y0 + H * ep
    pub fn current_yield_stress(&self, ep: f64) -> f64 {
        self.sigma_y0 + self.hardening_modulus * ep
    }

    /// 1D Elastoplastic tangent modulus E_ep = (E * H) / (E + H)
    pub fn elastoplastic_tangent(&self, e_elastic: f64) -> f64 {
        if self.hardening_modulus <= 0.0 {
            0.01 * e_elastic // perfectly plastic residual
        } else {
            (e_elastic * self.hardening_modulus) / (e_elastic + self.hardening_modulus)
        }
    }
}

impl MaterialFieldStrategy for J2PlasticityField {
    fn evaluate(&self, point: [f64; 3], reference: &ReferenceMaterial) -> LocalMaterialProperties {
        // Spatial distribution of plastic yielding (e.g. plastic hinge / notch concentration)
        // High plastic strain near root / center x=0 or y=0
        let dist = (point[0].powi(2) + point[1].powi(2)).sqrt();
        let local_ep = self.plastic_strain * (-dist * 0.3).exp();

        let current_sy = self.current_yield_stress(local_ep);

        // Effective degraded stiffness in yielded zone
        let is_yielded = local_ep > 1e-5;
        let e = if is_yielded {
            let e_ep = self.elastoplastic_tangent(reference.e0);
            (reference.e0 * (1.0 - 0.7 * (local_ep / (local_ep + 0.005)))).max(e_ep)
        } else {
            reference.e0
        };

        let nu = if is_yielded {
            // Plastic incompressibility drives nu toward 0.49
            (reference.nu0 + 0.15 * (local_ep / (local_ep + 0.005))).min(0.48)
        } else {
            reference.nu0
        };

        let rho = reference.rho0;
        let mu = e / (2.0 * (1.0 + nu));
        let lambda = (e * nu) / ((1.0 + nu) * (1.0 - 2.0 * nu));

        LocalMaterialProperties {
            e,
            nu,
            rho,
            mu,
            lambda,
            secant_modulus: Some(e),
            yield_stress: Some(current_sy),
            scalar_metric: current_sy,
        }
    }
}
