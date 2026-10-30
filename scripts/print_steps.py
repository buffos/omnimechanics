with open('src-tauri/src/solver/tests.rs', 'r', encoding='utf-8') as f:
    text = f.read()

target = """        println!("TRANSIENT STEP FEM max_disp = {:.6} mm", res_fem.max_overall_displacement * 1000.0);
        println!("TRANSIENT STEP BEM max_disp = {:.6} mm", res_bem.max_overall_displacement * 1000.0);"""

replacement = """        println!("TRANSIENT STEP FEM max_disp = {:.6} mm", res_fem.max_overall_displacement * 1000.0);
        println!("TRANSIENT STEP BEM max_disp = {:.6} mm", res_bem.max_overall_displacement * 1000.0);
        for m in 0..res_fem.frames.len() {
            let t = res_fem.time_points[m];
            let d_fem = res_fem.frames[m].max_displacement * 1000.0;
            let d_bem = res_bem.frames[m].max_displacement * 1000.0;
            println!("  t = {:.3} s: FEM = {:.6} mm, BEM = {:.6} mm", t, d_fem, d_bem);
        }"""

assert target in text, "target not found"
text = text.replace(target, replacement, 1)

with open('src-tauri/src/solver/tests.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("Added time step printing!")
