use crate::material::traits::MaterialFieldStrategy;
use crate::material::types::{
    Axis, GradientForm, LocalMaterialProperties, ReferenceMaterial,
};

#[derive(Debug, Clone)]
pub struct DirectionalGradientField {
    pub axis: Axis,
    pub alpha: f64,
    pub length_scale: f64,
    pub power: f64,
    pub form: GradientForm,
}

impl DirectionalGradientField {
    pub fn new(
        axis: Axis,
        alpha: f64,
        length_scale: f64,
        power: f64,
        form: GradientForm,
    ) -> Self {
        Self {
            axis,
            alpha,
            length_scale: if length_scale <= 0.0 { 1.0 } else { length_scale },
            power,
            form,
        }
    }
}

impl MaterialFieldStrategy for DirectionalGradientField {
    fn evaluate(&self, point: [f64; 3], reference: &ReferenceMaterial) -> LocalMaterialProperties {
        let coord = match self.axis {
            Axis::X => point[0],
            Axis::Y => point[1],
            Axis::Z => point[2],
        };

        // Normalized coordinate xi in [0, 1]
        let xi = coord / self.length_scale;

        // Perturbation factor factor(xi)
        let factor = match self.form {
            GradientForm::Linear => (1.0 + self.alpha * xi).max(0.01),
            GradientForm::Quadratic => {
                let term = (1.0 + self.alpha * xi).max(0.01);
                term.powf(self.power)
            }
            GradientForm::Exponential => (self.alpha * xi).exp(),
        };

        let e = reference.e0 * factor;
        let nu = reference.nu0;
        let rho = reference.rho0 * (1.0 + 0.5 * (factor - 1.0)); // subtle density variation
        let mu = e / (2.0 * (1.0 + nu));
        let lambda = (e * nu) / ((1.0 + nu) * (1.0 - 2.0 * nu));

        LocalMaterialProperties {
            e,
            nu,
            rho,
            mu,
            lambda,
            secant_modulus: None,
            yield_stress: None,
            scalar_metric: e,
        }
    }
}
