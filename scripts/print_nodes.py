with open('src-tauri/src/solver/tests.rs', 'r', encoding='utf-8') as f:
    text = f.read()

target = "println!(\"Freq s = {:5.2}: BEM tip node 4 uy = {:.6e}\", freq, u_b[4*2+1].re);"

replacement = """println!("Freq s = {:5.2}: BEM tip node 4 uy = {:.6e}", freq, u_b[4*2+1].re);
            println!("  BEM node 9 uy = {:.6e}", u_b[9*2+1].re);"""

assert target in text, "target not found"
text = text.replace(target, replacement, 1)

with open('src-tauri/src/solver/tests.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("Added node 9 print!")
