use super::*;

use num_complex::Complex64;


    

    #[test]
    fn test_gmres_identity_system() {
        let n = 4;
        let b = vec![
            Complex64::new(1.0, 0.5),
            Complex64::new(-2.0, 1.0),
            Complex64::new(3.0, -1.0),
            Complex64::new(0.5, 2.0),
        ];

        let apply_a = |v: &[Complex64]| v.to_vec();

        let res = solve_complex_gmres(n, &b, apply_a, 20, 1e-8, 10);
        assert!(res.converged);
        for i in 0..n {
            assert!((res.solution[i] - b[i]).norm() < 1e-6);
        }
    }

    #[test]
    fn test_gmres_diagonal_scaled_system() {
        let n = 3;
        let b = vec![
            Complex64::new(2.0, 0.0),
            Complex64::new(4.0, 1.0),
            Complex64::new(-1.0, 3.0),
        ];

        let diag = [
            Complex64::new(2.0, 1.0),
            Complex64::new(3.0, -0.5),
            Complex64::new(1.5, 2.0),
        ];

        let apply_a = move |v: &[Complex64]| {
            let mut out = vec![Complex64::new(0.0, 0.0); n];
            for i in 0..n {
                out[i] = diag[i] * v[i];
            }
            out
        };

        let res = solve_complex_gmres(n, &b, apply_a, 30, 1e-9, 10);
        assert!(res.converged);

        for i in 0..n {
            let expected = b[i] / diag[i];
            assert!((res.solution[i] - expected).norm() < 1e-6);
        }
    }
