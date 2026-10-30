use super::*;

#[test]
    fn test_anderson_linear_system() {
        // Solve (I - T) x = b with spectral radius close to 1:
        // T = [[0.9, 0.05], [0.05, 0.85]], b = [1.0, 2.0]
        // Exact solution: x* = (I - T)^(-1) b
        let mut acc = AndersonAccelerator::new(3, 1.0);
        let mut x = vec![0.0f64, 0.0f64];

        for _iter in 0..15 {
            // g(x) = T x + b
            let gx0 = 0.9 * x[0] + 0.05 * x[1] + 1.0;
            let gx1 = 0.05 * x[0] + 0.85 * x[1] + 2.0;
            let f = vec![gx0 - x[0], gx1 - x[1]];

            let res_norm = (f[0] * f[0] + f[1] * f[1]).sqrt();
            if res_norm < 1e-8 {
                break;
            }

            x = acc.compute_update(&x, &f);
        }

        let gx0 = 0.9 * x[0] + 0.05 * x[1] + 1.0;
        let gx1 = 0.05 * x[0] + 0.85 * x[1] + 2.0;
        let err = ((gx0 - x[0]).powi(2) + (gx1 - x[1]).powi(2)).sqrt();
        assert!(err < 1e-7, "Anderson should converge rapidly: err = {}", err);
    }
