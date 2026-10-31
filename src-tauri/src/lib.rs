pub mod benchmarks;
pub mod material;
pub mod mesh;
pub mod solver;
pub mod teaching;

use material::types::{MaterialFieldRequest, MaterialFieldResponse};
use mesh::types::{MeshData, MeshGeneratorRequest};
use solver::types::{
    HarmonicSimulationRequest, HarmonicSimulationResult, SimulationRequest, SimulationResult,
    TransientSimulationRequest, TransientSimulationResult,
};
use teaching::types::{TeachingInputConfig, TeachingSolutionDto};
use tauri::Manager;

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

#[tauri::command]
fn run_simulation(request: SimulationRequest) -> Result<SimulationResult, String> {
    Ok(solver::run_simulation(request))
}

#[tauri::command]
fn run_harmonic_simulation(
    request: HarmonicSimulationRequest,
) -> Result<HarmonicSimulationResult, String> {
    Ok(solver::run_harmonic_simulation(request))
}

#[tauri::command]
fn run_transient_simulation(
    request: TransientSimulationRequest,
) -> Result<TransientSimulationResult, String> {
    Ok(solver::run_transient_simulation(request))
}

#[tauri::command]
fn solve_teaching_system(
    config: TeachingInputConfig,
) -> Result<TeachingSolutionDto, String> {
    teaching::solve_teaching_system(config)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                window.app_handle().exit(0);
            }
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            generate_mesh,
            evaluate_material_field,
            run_simulation,
            run_harmonic_simulation,
            run_transient_simulation,
            solve_teaching_system
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
