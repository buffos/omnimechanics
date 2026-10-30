with open('src-tauri/src/solver/tests.rs', 'r', encoding='utf-8') as f:
    text = f.read()

target = """        let talbot_pts = crate::solver::talbot::generate_talbot_nodes(16, 1.0);"""

replacement = """        let talbot_pts = crate::solver::talbot::generate_talbot_nodes(16, 1.0);
        let pt0 = talbot_pts[0];
        println!("At Talbot pt 0, s = {:?}", pt0.s);"""

assert target in text, "target not found"
text = text.replace(target, replacement, 1)

with open('src-tauri/src/solver/tests.rs', 'w', encoding='utf-8') as f:
    f.write(text)
