with open('src-tauri/src/solver/dynamic_bem.rs', 'r', encoding='utf-8') as f:
    content = f.read()

target = '''    let mut t_star = [[Complex64::new(0.0, 0.0); 2]; 2];
    for i in 0..2 {
        for j in 0..2 {
            let delta = if i == j { 1.0 } else { 0.0 };
            let term_a = (d_psi - chi / r) * (delta * dr_dn + dr[j] * normal[i]);
            let term_b = (d_chi * (-2.0) + (chi * 4.0) / r) * (dr[i] * dr[j] * dr_dn);
            let term_c = Complex64::new(lambda / mu, 0.0) * (d_psi - d_chi - (chi * 2.0) / r) * (dr[i] * normal[j]);
            let term_d = -(chi * 2.0 / r) * (normal[j] * dr[i]);
            let t_val = term_a + term_b + term_c + term_d;
            t_star[i][j] = t_val * prefactor;
        }
    }'''

replacement = '''    let mut t_star = [[Complex64::new(0.0, 0.0); 2]; 2];
    for i in 0..2 {
        for k in 0..2 {
            let delta = if i == k { 1.0 } else { 0.0 };
            let term_a = (d_psi - chi / r) * (delta * dr_dn + dr[k] * normal[i]);
            let term_b = Complex64::new(lambda / mu, 0.0) * (d_psi - d_chi - chi / r) * (dr[i] * normal[k])
                - (chi * 2.0 / r) * (normal[k] * dr[i]);
            let term_c = -(d_chi * 2.0 - (chi * 4.0) / r) * (dr[i] * dr[k] * dr_dn);
            let t_val = term_a + term_b + term_c;
            t_star[i][k] = t_val * prefactor;
        }
    }'''

if target in content:
    with open('src-tauri/src/solver/dynamic_bem.rs', 'w', encoding='utf-8') as f:
        f.write(content.replace(target, replacement, 1))
    print('Replaced dynamic_kelvin_t with exact Hooke law formula')
else:
    print('Target not found')
