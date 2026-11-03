pub mod actions;
pub mod benchmarks;
pub mod frame;
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

#[tauri::command]
fn frame_upsert_model(
    state: tauri::State<'_, frame::FrameState>,
    request: frame::UpsertModelRequest,
) -> Result<frame::StructuralModel, frame::CanonicalError> {
    frame::upsert_model(&state, request.model)
}

#[tauri::command]
fn frame_get_catalog() -> frame::catalog::CatalogResponse {
    frame::get_catalog()
}

#[tauri::command]
fn frame_evaluate_diagnostics(
    state: tauri::State<'_, frame::FrameState>,
    model: Option<frame::StructuralModel>,
) -> Result<frame::Diagnostics, frame::CanonicalError> {
    frame::evaluate_diagnostics(&state, model)
}

#[tauri::command]
fn frame_solve_linear(
    state: tauri::State<'_, frame::FrameState>,
    request: frame::SolveLinearRequest,
) -> Result<frame::SolveLinearResponse, frame::CanonicalError> {
    frame::solve_linear(&state, request)
}

#[tauri::command]
fn frame_inspect_member_point(
    state: tauri::State<'_, frame::FrameState>,
    request: frame::InspectMemberPointRequest,
) -> Result<frame::InspectMemberPointResponse, frame::CanonicalError> {
    frame::inspect_member_point(&state, request)
}

#[tauri::command]
fn frame_generate_dsm_walkthrough(
    state: tauri::State<'_, frame::FrameState>,
    request: frame::WalkthroughRequest,
) -> Result<frame::DsmWalkthroughData, frame::CanonicalError> {
    frame::generate_dsm_walkthrough(&state, request)
}

#[tauri::command]
fn frame_start_analysis_job(
    state: tauri::State<'_, frame::FrameState>,
    app_handle: tauri::AppHandle,
    request: frame::StartJobRequest,
) -> Result<frame::StartJobResponse, frame::CanonicalError> {
    use tauri::Emitter;
    let handle = app_handle.clone();
    let cb = std::sync::Arc::new(move |event: frame::types::AnalysisProgressEvent| {
        let _ = handle.emit("analysis://progress", event);
    });
    frame::start_analysis_job(&state, request, Some(cb))
}

#[tauri::command]
fn frame_cancel_analysis_job(
    state: tauri::State<'_, frame::FrameState>,
    job_id: String,
) -> Result<frame::JobStatus, frame::CanonicalError> {
    frame::cancel_analysis_job(&state, &job_id)
}

#[tauri::command]
fn frame_get_analysis_job_result(
    state: tauri::State<'_, frame::FrameState>,
    job_id: String,
) -> Result<Option<frame::JobResultPayload>, frame::CanonicalError> {
    frame::get_analysis_job_result(&state, &job_id)
}

#[tauri::command]
fn actions_get_default_na_profile(
    code: Option<String>,
) -> Result<actions::GetNaProfileResponse, frame::CanonicalError> {
    actions::get_default_na_profile(code.as_deref())
}

#[tauri::command]
fn actions_validate_na_profile(
    profile: actions::NationalAnnexProfile,
) -> Result<(), frame::CanonicalError> {
    actions::validate_na_profile(&profile)
}

#[tauri::command]
fn actions_generate_combinations(
    request: actions::GenerateCombinationsRequest,
) -> Result<actions::CombinationSet, frame::CanonicalError> {
    actions::generate_combinations(&request)
}

#[tauri::command]
fn actions_save_custom_combination(
    request: actions::SaveCustomCombinationRequest,
) -> Result<actions::CombinationSet, frame::CanonicalError> {
    actions::save_custom_combination(&request)
}

#[tauri::command]
fn actions_delete_or_reset_combination(
    request: actions::DeleteOrResetCombinationRequest,
) -> Result<actions::CombinationSet, frame::CanonicalError> {
    actions::delete_or_reset_combination(&request)
}

#[tauri::command]
fn actions_compute_design_spectrum(
    request: actions::ComputeDesignSpectrumRequest,
) -> Result<actions::ComputeDesignSpectrumResponse, frame::CanonicalError> {
    actions::compute_design_spectrum(&request)
}

#[tauri::command]
fn actions_compute_seismic_mass_rule(
    request: actions::ComputeSeismicMassRuleRequest,
) -> Result<actions::SeismicMassCombinationRule, frame::CanonicalError> {
    actions::compute_seismic_mass_rule(&request)
}

#[tauri::command]
fn actions_compute_lateral_forces(
    request: actions::ComputeLateralForcesRequest,
) -> Result<actions::LateralForceDistributionResult, frame::CanonicalError> {
    actions::compute_lateral_forces(&request)
}

#[tauri::command]
fn actions_compute_cqc_matrix(
    request: actions::ComputeCqcMatrixRequest,
) -> Result<actions::ModalCorrelationMatrix, frame::CanonicalError> {
    actions::compute_cqc_matrix(&request)
}

#[tauri::command]
fn teaching_classify_model(
    request: teaching::ClassifyModelRequest,
) -> Result<teaching::MethodEligibility, frame::CanonicalError> {
    teaching::classify_model(&request.model, request.load_case_id.as_deref())
}

#[tauri::command]
fn teaching_open_step_by_step(
    request: teaching::OpenStepByStepRequest,
) -> Result<teaching::OpenStepByStepResponse, frame::CanonicalError> {
    teaching::open_step_by_step(&request)
}

#[tauri::command]
fn teaching_preset_to_structural_model(
    request: teaching::PresetToModelRequest,
) -> Result<teaching::PresetToModelResponse, frame::CanonicalError> {
    teaching::preset_to_structural_model(&request)
}

#[tauri::command]
fn project_save_workspace(
    request: teaching::SaveWorkspaceRequest,
) -> Result<String, frame::CanonicalError> {
    teaching::save_workspace(&request)
}

#[tauri::command]
fn project_load_workspace(
    request: teaching::LoadWorkspaceRequest,
) -> Result<teaching::ProjectDocument, frame::CanonicalError> {
    teaching::load_workspace(&request)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(frame::FrameState::default())
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
            solve_teaching_system,
            frame_upsert_model,
            frame_get_catalog,
            frame_evaluate_diagnostics,
            frame_solve_linear,
            frame_inspect_member_point,
            frame_generate_dsm_walkthrough,
            frame_start_analysis_job,
            frame_cancel_analysis_job,
            frame_get_analysis_job_result,
            actions_get_default_na_profile,
            actions_validate_na_profile,
            actions_generate_combinations,
            actions_save_custom_combination,
            actions_delete_or_reset_combination,
            actions_compute_design_spectrum,
            actions_compute_seismic_mass_rule,
            actions_compute_lateral_forces,
            actions_compute_cqc_matrix,
            teaching_classify_model,
            teaching_open_step_by_step,
            teaching_preset_to_structural_model,
            project_save_workspace,
            project_load_workspace
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
