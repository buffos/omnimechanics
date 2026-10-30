with open('src-tauri/src/solver/tests.rs', 'r', encoding='utf-8') as f:
    content = f.read()

target = 'println!("diff_rhs = {:.6e}", diff_rhs);'
replacement = '''println!("diff_rhs = {:.6e}", diff_rhs);
        println!("traction_y = {:?}, traction_y_stat = {}", traction_y, traction_y_stat);
        for j in 0..total_bnd_nodes {
            if bnd_node_tags[j] == "Right" {
                let col = j * dim + 1;
                println!("Right node j={}: g_stat[(1, col)] = {}, g_dyn[(1, col)] = {:?}", j, g_stat_mat[(1, col)], g_mat[(1, col)]);
            }
        }'''

if target in content:
    with open('src-tauri/src/solver/tests.rs', 'w', encoding='utf-8') as f:
        f.write(content.replace(target, replacement, 1))
    print('Replaced')
else:
    print('Target not found')
