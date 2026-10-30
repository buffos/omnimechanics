with open('src-tauri/src/solver/tests.rs', 'r', encoding='utf-8') as f:
    text = f.read()

target = """            println!("Talbot pt {}: s = {:?}", idx, s);
            println!("  BEM node 4: uy = {:?}", u_bem[4*2+1]);
            println!("  FEM node 4: uy = {:?}", u_fem[4*2+1]);
            println!("  Ratio FEM/BEM node 4 uy = {:?}", u_fem[4*2+1] / u_bem[4*2+1]);
            break;
        }"""

replacement = """            let r = u_fem[4*2+1] / u_bem[4*2+1];
            println!("Talbot pt {:2}: s = {:10.4} + {:10.4}i | FEM/BEM ratio = {:8.4} + {:8.4}i (|r| = {:.3})",
                idx, s.re, s.im, r.re, r.im, r.norm());
        }"""

assert target in text, "target not found"
text = text.replace(target, replacement, 1)

with open('src-tauri/src/solver/tests.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("Updated ratio loop!")
