with open('src-tauri/src/solver/tests.rs', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i, line in enumerate(lines):
    if i in [636, 641, 642, 2328, 2329]:
        lines[i] = line.replace(".max_displacement", ".max_overall_displacement")

with open('src-tauri/src/solver/tests.rs', 'w', encoding='utf-8') as f:
    f.writelines(lines)
print("Updated lines in tests.rs")
