with open('src-tauri/src/solver/dynamic_bem.rs', 'r', encoding='utf-8') as f:
    text = f.read()

print("factor_h_sing in text:", "factor_h_sing" in text)
print("Rigid Body in text:", "Apply Rigid Body Translation condition to H" in text)
print("int_cauchy in text:", "int_cauchy" in text)
