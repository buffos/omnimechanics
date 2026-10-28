use super::strategies::{
    DirectionalGradientField, InclusionField, J2PlasticityField, MathExpressionField,
    VecchioCollinsField,
};
use super::traits::MaterialFieldStrategy;
use super::types::{
    MaterialFieldRequest, MaterialFieldResponse, MaterialStrategyConfig, PropertyMetric,
};
use rayon::prelude::*;

pub fn evaluate_material_field(req: MaterialFieldRequest) -> MaterialFieldResponse {
    let strategy: Box<dyn MaterialFieldStrategy> = match req.strategy {
        MaterialStrategyConfig::DirectionalGradient {
            axis,
            alpha,
            length_scale,
            power,
            form,
        } => Box::new(DirectionalGradientField::new(
            axis,
            alpha,
            length_scale,
            power,
            form,
        )),

        MaterialStrategyConfig::AnalyticalExpression { expression } => {
            Box::new(MathExpressionField::new(expression))
        }

        MaterialStrategyConfig::Inclusion {
            shape,
            center,
            radius,
            box_half_extents,
            e_inclusion,
            nu_inclusion,
            rho_inclusion,
            smooth_transition,
        } => Box::new(InclusionField::new(
            shape,
            center,
            radius,
            box_half_extents,
            e_inclusion,
            nu_inclusion,
            rho_inclusion,
            smooth_transition,
        )),

        MaterialStrategyConfig::VecchioCollinsConcrete {
            fc_prime,
            epsilon_0,
            ft,
            rebar_ratio,
            simulated_strain,
        } => Box::new(VecchioCollinsField::new(
            fc_prime,
            epsilon_0,
            ft,
            rebar_ratio,
            simulated_strain,
        )),

        MaterialStrategyConfig::J2Plasticity {
            sigma_y0,
            hardening_modulus,
            plastic_strain,
        } => Box::new(J2PlasticityField::new(
            sigma_y0,
            hardening_modulus,
            plastic_strain,
        )),
    };

    let ref_mat = &req.reference;
    let metric = req.metric;

    // Unit conversion and property extraction
    let (scale, unit, property_name, ref_val) = match metric {
        PropertyMetric::YoungsModulus => (1e-6, "MPa".to_string(), "Young's Modulus E".to_string(), ref_mat.e0 * 1e-6),
        PropertyMetric::ShearModulus => (1e-6, "MPa".to_string(), "Shear Modulus μ".to_string(), ref_mat.mu0() * 1e-6),
        PropertyMetric::PoissonRatio => (1.0, "-".to_string(), "Poisson's Ratio ν".to_string(), ref_mat.nu0),
        PropertyMetric::Density => (1.0, "kg/m³".to_string(), "Density ρ".to_string(), ref_mat.rho0),
        PropertyMetric::WaveSpeedP => (1.0, "m/s".to_string(), "P-Wave Speed cp".to_string(), ref_mat.cp()),
        PropertyMetric::WaveSpeedS => (1.0, "m/s".to_string(), "S-Wave Speed cs".to_string(), ref_mat.cs()),
        PropertyMetric::SecantModulus => (1e-6, "MPa".to_string(), "Secant Modulus Es".to_string(), ref_mat.e0 * 1e-6),
        PropertyMetric::YieldStrength => (1e-6, "MPa".to_string(), "Yield Strength σy".to_string(), 250.0),
    };

    // Parallel evaluation using rayon for optimal performance on large meshes
    let values: Vec<f64> = req
        .nodes
        .par_iter()
        .map(|node| {
            let props = strategy.evaluate(*node, ref_mat);
            let raw_val = match metric {
                PropertyMetric::YoungsModulus => props.e,
                PropertyMetric::ShearModulus => props.mu,
                PropertyMetric::PoissonRatio => props.nu,
                PropertyMetric::Density => props.rho,
                PropertyMetric::WaveSpeedP => ((props.lambda + 2.0 * props.mu) / props.rho).sqrt(),
                PropertyMetric::WaveSpeedS => (props.mu / props.rho).sqrt(),
                PropertyMetric::SecantModulus => props.secant_modulus.unwrap_or(props.e),
                PropertyMetric::YieldStrength => props.yield_stress.unwrap_or(250e6),
            };
            raw_val * scale
        })
        .collect();

    let mut min_val = f64::INFINITY;
    let mut max_val = f64::NEG_INFINITY;

    for &v in &values {
        if v < min_val {
            min_val = v;
        }
        if v > max_val {
            max_val = v;
        }
    }

    if min_val.is_infinite() || max_val.is_infinite() {
        min_val = ref_val;
        max_val = ref_val;
    }

    MaterialFieldResponse {
        values,
        min_val,
        max_val,
        ref_val,
        unit,
        property_name,
    }
}
