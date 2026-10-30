with open('src-tauri/src/solver/dynamic_bem.rs', 'r', encoding='utf-8') as f:
    text = f.read()

lines = text.splitlines(True)
# Find line with "let num_nodes = mesh.nodes.len();" inside solve_laplace_pure_bem_quadratic_2d
start_idx = None
end_idx = None

for i in range(1190, len(lines)):
    if "let num_nodes = mesh.nodes.len();" in lines[i]:
        start_idx = i
        break

for i in range(start_idx, len(lines)):
    if "u_vec\n" in lines[i] or "u_vec\r\n" in lines[i]:
        end_idx = i
        break

print("Found block from", start_idx, "to", end_idx)
print("".join(lines[start_idx:start_idx+5]))
print("...")
print("".join(lines[end_idx-5:end_idx+2]))
