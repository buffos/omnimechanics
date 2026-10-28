use crate::material::traits::MaterialFieldStrategy;
use crate::material::types::{InclusionShape, LocalMaterialProperties, ReferenceMaterial};

#[derive(Debug, Clone)]
pub struct InclusionField {
    pub shape: InclusionShape,
    pub center: [f64; 3],
    pub radius: f64,
    pub box_half_extents: Option<[f64; 3]>,
    pub e_inclusion: f64,
    pub nu_inclusion: f64,
    pub rho_inclusion: f64,
    pub smooth_transition: bool,
}

impl InclusionField {
    pub fn new(
        shape: InclusionShape,
        center: [f64; 3],
        radius: f64,
        box_half_extents: Option<[f64; 3]>,
        e_inclusion: f64,
        nu_inclusion: f64,
        rho_inclusion: f64,
        smooth_transition: bool,
    ) -> Self {
        Self {
            shape,
            center,
            radius: radius.max(1e-4),
            box_half_extents,
            e_inclusion,
            nu_inclusion,
            rho_inclusion,
            smooth_transition,
        }
    }

    /// Returns inclusion membership factor s in [0, 1]
    /// 1.0 = fully inside inclusion, 0.0 = fully in matrix
    pub fn membership(&self, point: [f64; 3]) -> f64 {
        let dx = point[0] - self.center[0];
        let dy = point[1] - self.center[1];
        let dz = point[2] - self.center[2];

        let dist_val = match self.shape {
            InclusionShape::Sphere => (dx * dx + dy * dy + dz * dz).sqrt() - self.radius,
            InclusionShape::CylinderZ => (dx * dx + dy * dy).sqrt() - self.radius,
            InclusionShape::Box => {
                let half = self.box_half_extents.unwrap_or([self.radius, self.radius, self.radius]);
                let qx = dx.abs() - half[0];
                let qy = dy.abs() - half[1];
                let qz = dz.abs() - half[2];
                qx.max(qy).max(qz)
            }
        };

        if self.smooth_transition {
            // Sigmoid smoothing: transition width = 10% of radius
            let w = (self.radius * 0.12).max(1e-3);
            let s = 1.0 / (1.0 + (dist_val / w).exp());
            s.clamp(0.0, 1.0)
        } else {
            if dist_val <= 0.0 {
                1.0
            } else {
                0.0
            }
        }
    }
}

impl MaterialFieldStrategy for InclusionField {
    fn evaluate(&self, point: [f64; 3], reference: &ReferenceMaterial) -> LocalMaterialProperties {
        let s = self.membership(point);

        // Interpolate between baseline matrix and inclusion
        let e = (1.0 - s) * reference.e0 + s * self.e_inclusion;
        let nu = (1.0 - s) * reference.nu0 + s * self.nu_inclusion;
        let rho = (1.0 - s) * reference.rho0 + s * self.rho_inclusion;

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
