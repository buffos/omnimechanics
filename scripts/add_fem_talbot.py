with open('src-tauri/src/solver/tests.rs', 'r', encoding='utf-8') as f:
    text = f.read()

target = """            println!("Talbot pt {}: s = {:?}, BEM node 4: ({:?}, {:?}), node 9: ({:?}, {:?})",
                idx, s, u_bem[4*2], u_bem[4*2+1], u_bem[9*2], u_bem[9*2+1]);
        }"""

replacement = """            let (k0_mat, m_mat, _) = crate::solver::fem_solver::assemble_dynamic_stiffness_and_mass(&mesh, &ref_mat, &strategy);
            let s_sq = s * s;
            let mut f_fem = vec![Complex64::new(0.0, 0.0); mesh.nodes.len() * 2];
            f_fem[4*2+1] = Complex64::new(-25_000.0, 0.0) * pulse_factor;
            f_fem[9*2+1] = Complex64::new(-25_000.0, 0.0) * pulse_factor;
            let n_dofs = mesh.nodes.len() * 2;
            let apply_a = |v: &[Complex64]| {
                let kv = crate::solver::dynamic_solver::sp_mat_mul_complex(&k0_mat, v);
                let mv = crate::solver::dynamic_solver::sp_mat_mul_complex(&m_mat, v);
                let mut out = vec![Complex64::new(0.0, 0.0); n_dofs];
                for i in 0..n_dofs {
                    out[i] = kv[i] + s_sq * mv[i];
                }
                out
            };
            let u_fem = crate::solver::krylov_solver::solve_complex_gmres(n_dofs, &f_fem, apply_a, 100, 1e-7, 100).solution;
            println!("Talbot pt {}: s = {:?}", idx, s);
            println!("  BEM node 4: uy = {:?}", u_bem[4*2+1]);
            println!("  FEM node 4: uy = {:?}", u_fem[4*2+1]);
        }"""

assert target in text, "target not found"
text = text.replace(target, replacement, 1)

with open('src-tauri/src/solver/tests.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("Added FEM Talbot inspection!")
