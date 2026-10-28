use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReferenceMaterial {
    /// Young's modulus E0 (Pa)
    pub e0: f64,
    /// Poisson's ratio nu0
    pub nu0: f64,
    /// Mass density rho0 (kg/m^3)
    pub rho0: f64,
}

impl Default for ReferenceMaterial {
    fn default() -> Self {
        Self {
            e0: 2.1e11, // 210 GPa (Structural steel)
            nu0: 0.30,
            rho0: 7850.0,
        }
    }
}

impl ReferenceMaterial {
    pub fn concrete() -> Self {
        Self {
            e0: 3.0e10, // 30 GPa (C30 Concrete)
            nu0: 0.20,
            rho0: 2400.0,
        }
    }

    /// Shear modulus mu0 = E0 / (2 * (1 + nu0))
    pub fn mu0(&self) -> f64 {
        self.e0 / (2.0 * (1.0 + self.nu0))
    }

    /// Lamé first parameter lambda0 = (E0 * nu0) / ((1 + nu0) * (1 - 2*nu0))
    pub fn lambda0(&self) -> f64 {
        (self.e0 * self.nu0) / ((1.0 + self.nu0) * (1.0 - 2.0 * self.nu0))
    }

    /// Bulk modulus K0 = lambda0 + 2/3 * mu0
    pub fn bulk_k0(&self) -> f64 {
        self.lambda0() + (2.0 / 3.0) * self.mu0()
    }

    /// P-wave speed cp = sqrt((lambda0 + 2*mu0) / rho0)
    pub fn cp(&self) -> f64 {
        ((self.lambda0() + 2.0 * self.mu0()) / self.rho0).sqrt()
    }

    /// S-wave speed cs = sqrt(mu0 / rho0)
    pub fn cs(&self) -> f64 {
        (self.mu0() / self.rho0).sqrt()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalMaterialProperties {
    /// Local Young's modulus E(x) (Pa)
    pub e: f64,
    /// Local Poisson's ratio nu(x)
    pub nu: f64,
    /// Local density rho(x) (kg/m^3)
    pub rho: f64,
    /// Local shear modulus mu(x) (Pa)
    pub mu: f64,
    /// Local Lamé parameter lambda(x) (Pa)
    pub lambda: f64,
    /// Optional non-linear secant modulus Es (for Vecchio-Collins concrete)
    pub secant_modulus: Option<f64>,
    /// Optional yield stress sigma_y (for J2 plasticity)
    pub yield_stress: Option<f64>,
    /// Selected scalar metric for colormap rendering
    pub scalar_metric: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropertyMetric {
    YoungsModulus,
    ShearModulus,
    PoissonRatio,
    Density,
    WaveSpeedP,
    WaveSpeedS,
    SecantModulus,
    YieldStrength,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GradientForm {
    Linear,
    Quadratic,
    Exponential,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InclusionShape {
    Sphere,
    CylinderZ,
    Box,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params")]
pub enum MaterialStrategyConfig {
    DirectionalGradient {
        axis: Axis,
        alpha: f64, // Inhomogeneity ratio, e.g. 0.20 for 20%
        length_scale: f64,
        power: f64,
        form: GradientForm,
    },
    AnalyticalExpression {
        expression: String,
    },
    Inclusion {
        shape: InclusionShape,
        center: [f64; 3],
        radius: f64,
        box_half_extents: Option<[f64; 3]>,
        e_inclusion: f64,
        nu_inclusion: f64,
        rho_inclusion: f64,
        smooth_transition: bool,
    },
    VecchioCollinsConcrete {
        fc_prime: f64,       // Compressive strength, e.g. 30e6 Pa (30 MPa)
        epsilon_0: f64,      // Peak strain, e.g. 0.002
        ft: f64,             // Tensile cracking strength, e.g. 2.5e6 Pa
        rebar_ratio: f64,    // Reinforcement ratio rho_s, e.g. 0.015 (1.5%)
        simulated_strain: f64, // Principal tensile strain epsilon_1 (e.g. 0.004)
    },
    J2Plasticity {
        sigma_y0: f64,       // Initial yield strength, e.g. 250e6 Pa (250 MPa)
        hardening_modulus: f64, // Plastic hardening modulus H (Pa)
        plastic_strain: f64, // Equivalent plastic strain epsilon_bar_p
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialFieldRequest {
    pub reference: ReferenceMaterial,
    pub strategy: MaterialStrategyConfig,
    pub nodes: Vec<[f64; 3]>,
    pub metric: PropertyMetric,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialFieldResponse {
    pub values: Vec<f64>,
    pub min_val: f64,
    pub max_val: f64,
    pub ref_val: f64,
    pub unit: String,
    pub property_name: String,
}
