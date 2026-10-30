with open('src-tauri/src/solver/tests.rs', 'r', encoding='utf-8') as f:
    content = f.read()

test_code = """
    #[test]
    fn test_debug_beam_transient_fem_vs_bem() {
        use crate::material::types::ReferenceMaterial;
        use crate::solver::bessel::*;
        use num_complex::Complex64;

        let ref_mat = ReferenceMaterial::default();
        let mu = ref_mat.mu0();
        let nu = ref_mat.nu0;
        let rho = ref_mat.rho0;
        let lambda = ref_mat.lambda0();

        let cp = ((lambda + 2.0 * mu) / rho).sqrt();
        let cs = (mu / rho).sqrt();

        // Let's test for r = 0.1, 0.01, 0.001 with s = 1.0
        let s = Complex64::new(1.0, 0.0);
        for &r in &[1.0, 0.1, 0.01, 0.001, 0.0001] {
            let kp = s / cp;
            let ks = s / cs;
            let zp = kp * r;
            let zs = ks * r;

            let k0_s = bessel_k0(zs);
            let k1_s = bessel_k1(zs);
            let k1_p = bessel_k1(zp);
            let k2_s = bessel_k2(zs);
            let k2_p = bessel_k2(zp);

            let gamma = cs / cp;
            let gamma_sq = gamma * gamma;

            let chi = k2_s - k2_p * gamma_sq;
            let psi = k0_s + (k1_s - k1_p * gamma) / zs;

            let prefactor = 1.0 / (2.0 * std::f64::consts::PI * mu);

            // Let's look at U_xx for dr = [1, 0]:
            let u_dyn_xx = (psi - chi) * prefactor;

            // Static Kelvin:
            let c1 = 1.0 / (8.0 * std::f64::consts::PI * mu * (1.0 - nu));
            let kappa = 3.0 - 4.0 * nu;
            let u_stat_xx = c1 * (kappa * (1.0 / r).ln() + 1.0);

            println!("r = {:<7}: Dyn U_xx = {:<25?}, Stat U_xx = {:<12.6e}, Ratio = {:.4}",
                r, u_dyn_xx.re, u_stat_xx, u_dyn_xx.re / u_stat_xx);
        }
    }
"""

start_idx = content.find("fn test_debug_beam_transient_fem_vs_bem()")
if start_idx != -1:
    prev_idx = content.rfind("#[test]", 0, start_idx)
    new_content = content[:prev_idx] + test_code.strip() + "\n}\n"
    with open('src-tauri/src/solver/tests.rs', 'w', encoding='utf-8') as f:
        f.write(new_content)
    print("Test script written")
