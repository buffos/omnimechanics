#[cfg(test)]
mod tests {
    use crate::material::evaluator::evaluate_material_field;
    use crate::material::types::*;

    #[test]
    fn test_directional_gradient_bounds() {
        let ref_mat = ReferenceMaterial::default();
        let nodes = vec![
            [0.0, 0.0, 0.0],
            [5.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
        ];

        let req = MaterialFieldRequest {
            reference: ref_mat,
            strategy: MaterialStrategyConfig::DirectionalGradient {
                axis: Axis::X,
                alpha: 0.20, // 20% gradient
                length_scale: 10.0,
                power: 1.0,
                form: GradientForm::Linear,
            },
            nodes,
            metric: PropertyMetric::YoungsModulus,
        };

        let res = evaluate_material_field(req);
        assert_eq!(res.values.len(), 3);
        // At x=0: E = 210,000 MPa
        assert!((res.values[0] - 210_000.0).abs() < 1.0);
        // At x=10: E = 210,000 * 1.20 = 252,000 MPa
        assert!((res.values[2] - 252_000.0).abs() < 1.0);
        assert_eq!(res.unit, "MPa");
    }

    #[test]
    fn test_math_expression_evaluation() {
        let ref_mat = ReferenceMaterial::default();
        let nodes = vec![
            [0.0, 0.0, 0.0],
            [5.0, 0.0, 0.0],
        ];

        let req = MaterialFieldRequest {
            reference: ref_mat,
            strategy: MaterialStrategyConfig::AnalyticalExpression {
                expression: "E0 * (1 + 0.5 * (x / 10))".to_string(),
            },
            nodes,
            metric: PropertyMetric::YoungsModulus,
        };

        let res = evaluate_material_field(req);
        assert!((res.values[0] - 210_000.0).abs() < 1.0);
        assert!((res.values[1] - (210_000.0 * 1.25)).abs() < 1.0);
    }

    #[test]
    fn test_inclusion_membership() {
        let ref_mat = ReferenceMaterial::default();
        let nodes = vec![
            [0.0, 0.0, 0.0],  // center (inside)
            [1.0, 0.0, 0.0],  // on boundary
            [5.0, 0.0, 0.0],  // far outside
        ];

        let req = MaterialFieldRequest {
            reference: ref_mat,
            strategy: MaterialStrategyConfig::Inclusion {
                shape: InclusionShape::Sphere,
                center: [0.0, 0.0, 0.0],
                radius: 1.0,
                box_half_extents: None,
                e_inclusion: 50e9, // 50 GPa inclusion in 210 GPa steel
                nu_inclusion: 0.25,
                rho_inclusion: 2500.0,
                smooth_transition: false,
            },
            nodes,
            metric: PropertyMetric::YoungsModulus,
        };

        let res = evaluate_material_field(req);
        assert!((res.values[0] - 50_000.0).abs() < 1.0); // Inside inclusion
        assert!((res.values[2] - 210_000.0).abs() < 1.0); // Outside in steel
    }

    #[test]
    fn test_vecchio_collins_softening() {
        let ref_mat = ReferenceMaterial::concrete();
        let nodes = vec![
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 2.0, 0.0],
        ];

        let req = MaterialFieldRequest {
            reference: ref_mat,
            strategy: MaterialStrategyConfig::VecchioCollinsConcrete {
                fc_prime: 30e6,
                epsilon_0: 0.002,
                ft: 2.5e6,
                rebar_ratio: 0.015,
                simulated_strain: 0.005,
            },
            nodes,
            metric: PropertyMetric::SecantModulus,
        };

        let res = evaluate_material_field(req);
        assert_eq!(res.values.len(), 3);
        // Concrete degrades in presence of high tensile cracking strain
        for val in &res.values {
            assert!(*val <= 30_000.0);
            assert!(*val > 1_000.0);
        }
    }

    #[test]
    fn test_j2_plasticity_yielding() {
        let ref_mat = ReferenceMaterial::default();
        let nodes = vec![
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
        ];

        let req = MaterialFieldRequest {
            reference: ref_mat,
            strategy: MaterialStrategyConfig::J2Plasticity {
                sigma_y0: 250e6,
                hardening_modulus: 1000e6,
                plastic_strain: 0.01,
            },
            nodes,
            metric: PropertyMetric::YieldStrength,
        };

        let res = evaluate_material_field(req);
        // At origin (highest plastic strain), yield strength hardens: sigma_y > 250 MPa
        assert!(res.values[0] >= 250.0);
        assert!(res.values[1] >= 250.0);
    }
}
