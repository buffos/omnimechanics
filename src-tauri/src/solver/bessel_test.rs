use super::*;

use num_complex::Complex64;


    

    #[test]
    fn test_bessel_real_values() {
        // Known benchmark values for real arguments:
        // K_0(1.0) approx 0.4210244382
        // K_1(1.0) approx 0.6019072302
        let z1 = Complex64::new(1.0, 0.0);
        let k0_val = bessel_k0(z1);
        let k1_val = bessel_k1(z1);

        assert!((k0_val.re - 0.421024).abs() < 1e-4);
        assert!((k1_val.re - 0.601907).abs() < 1e-4);
        assert!(k0_val.im.abs() < 1e-10);
    }

    #[test]
    fn test_bessel_complex_decay() {
        // For Re(z) > 0, K_0 should exponentially decay
        let z_far = Complex64::new(5.0, 2.0);
        let val = bessel_k0(z_far);
        assert!(val.norm() < 0.01);
    }
