with open('src-tauri/src/solver/tests.rs', 'r', encoding='utf-8') as f:
    text = f.read()

target = "println!(\"Static Pure BEM max disp = {:.6e} m\", static_bem.max_displacement);"

replacement = """println!("Static Pure BEM max disp = {:.6e} m", static_bem.max_displacement);
        for (idx, bnd) in mesh.boundary_elements.iter().enumerate() {
            println!("  bnd elem {}: tag={}, nodes={:?}, p0={:?}, p1={:?}",
                idx, bnd.tag, bnd.nodes, mesh.nodes[bnd.nodes[0]], mesh.nodes[bnd.nodes[1]]);
        }
        for (i, p) in mesh.nodes.iter().enumerate() {
            println!("  node {}: coord={:?}", i, p);
        }"""

assert target in text, "target not found"
text = text.replace(target, replacement, 1)

with open('src-tauri/src/solver/tests.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("Added mesh boundary print!")
