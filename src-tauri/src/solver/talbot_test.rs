use super::*;

#[test]
    fn test_talbot_inversion_analytical() {
        // Invert f(t) = exp(-2*t) * sin(3*t)
        // Analytical Laplace transform: F(s) = 3 / ((s + 2)^2 + 9)
        let t_target: f64 = 1.0;
        let analytical = (-2.0 * t_target).exp() * (3.0 * t_target).sin();

        let num_points = 24;
        let nodes = generate_talbot_nodes(num_points, t_target);

        let u_hat_list: Vec<Vec<Complex64>> = nodes
            .iter()
            .map(|pt| {
                let s = pt.s;
                let val = 3.0 / ((s + 2.0) * (s + 2.0) + 9.0);
                vec![val]
            })
            .collect();

        let res = invert_laplace_talbot(&nodes, &u_hat_list, t_target);
        let numerical = res[0];

        assert!((numerical - analytical).abs() < 1e-4, "Numerical: {}, Analytical: {}", numerical, analytical);
    }
