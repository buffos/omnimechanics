with open('src-tauri/src/solver/tests.rs', 'r', encoding='utf-8') as f:
    text = f.read()

# Let's inspect how self-element G is assembled in dynamic_bem.rs
print("Checking dynamic_bem.rs self-element G")
