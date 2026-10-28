pub mod material;
pub mod mesh;

use material::types::{MaterialFieldRequest, MaterialFieldResponse};
use mesh::types::{MeshData, MeshGeneratorRequest};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn generate_mesh(request: MeshGeneratorRequest) -> Result<MeshData, String> {
    Ok(mesh::generate_mesh(request))
}

#[tauri::command]
fn evaluate_material_field(
    request: MaterialFieldRequest,
) -> Result<MaterialFieldResponse, String> {
    Ok(material::evaluate_material_field(request))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            generate_mesh,
            evaluate_material_field
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
