pub mod catalog;
pub mod catenary;
pub mod diagrams;
pub mod diagnostics;
pub mod dynamics;
pub mod element_stiffness;
pub mod envelopes;
pub mod job_manager;
pub mod linear_solver;
pub mod pdelta_buckling;
pub mod types;
pub mod unilateral;
pub mod validation;
pub mod walkthrough;

#[cfg(test)]
pub mod benchmarks_test;
#[cfg(test)]
pub mod catalog_test;
#[cfg(test)]
pub mod catenary_test;
#[cfg(test)]
pub mod diagnostics_test;
#[cfg(test)]
pub mod dynamics_test;
#[cfg(test)]
pub mod linear_test;
#[cfg(test)]
pub mod nonlinear_test;
#[cfg(test)]
pub mod types_test;
#[cfg(test)]
pub mod validation_test;

use std::sync::{Arc, RwLock};
pub use types::*;

pub struct FrameState {
    pub model: RwLock<Option<StructuralModel>>,
    pub job_manager: Arc<job_manager::JobManager>,
}

impl Default for FrameState {
    fn default() -> Self {
        Self {
            model: RwLock::new(None),
            job_manager: Arc::new(job_manager::JobManager::new()),
        }
    }
}

pub fn upsert_model(
    state: &FrameState,
    model: StructuralModel,
) -> Result<StructuralModel, CanonicalError> {
    validation::validate_model(&model)?;
    let mut lock = state
        .model
        .write()
        .map_err(|e| CanonicalError::new("LOCK_ERROR", e.to_string(), e.to_string()))?;
    
    let old_rev = lock.as_ref().map(|m| m.revision).unwrap_or(0);
    if model.revision > old_rev {
        state.job_manager.supersede_jobs_older_than(model.revision);
    }

    *lock = Some(model.clone());
    Ok(model)
}

pub fn get_catalog() -> catalog::CatalogResponse {
    catalog::get_default_catalog()
}

pub fn evaluate_diagnostics(
    state: &FrameState,
    maybe_model: Option<StructuralModel>,
) -> Result<Diagnostics, CanonicalError> {
    let model = match maybe_model {
        Some(m) => m,
        None => {
            let lock = state
                .model
                .read()
                .map_err(|e| CanonicalError::new("LOCK_ERROR", e.to_string(), e.to_string()))?;
            lock.clone().ok_or_else(|| {
                CanonicalError::new(
                    "NO_ACTIVE_MODEL",
                    "Δεν έχει οριστεί ενεργό μοντέλο.",
                    "No active structural model has been set.",
                )
            })?
        }
    };

    diagnostics::evaluate_diagnostics(&model)
}

pub fn solve_linear(
    state: &FrameState,
    request: SolveLinearRequest,
) -> Result<SolveLinearResponse, CanonicalError> {
    let model = match request.model {
        Some(m) => m,
        None => {
            let lock = state
                .model
                .read()
                .map_err(|e| CanonicalError::new("LOCK_ERROR", e.to_string(), e.to_string()))?;
            lock.clone().ok_or_else(|| {
                CanonicalError::new(
                    "NO_ACTIVE_MODEL",
                    "Δεν έχει οριστεί ενεργό μοντέλο.",
                    "No active structural model has been set.",
                )
            })?
        }
    };

    if request.model_revision != model.revision {
        return Err(CanonicalError::new(
            "STALE_MODEL_REVISION",
            format!(
                "Η αναθεώρηση αιτήματος {} δεν ταιριάζει με την τρέχουσα {}.",
                request.model_revision, model.revision
            ),
            format!(
                "Requested revision {} does not match active revision {}.",
                request.model_revision, model.revision
            ),
        ));
    }

    let diagnostics = diagnostics::evaluate_diagnostics(&model)?;
    let result_sets = if catenary::has_cable_elements(&model) {
        catenary::solve_catenary_cable(&model, &request.combinations)?
    } else if unilateral::has_unilateral_elements(&model) {
        unilateral::solve_unilateral_active_set(&model, &request.combinations)?
    } else {
        linear_solver::solve_linear_static(&model, &request.combinations)?
    };
    let envelope = if request.compute_envelope && !result_sets.is_empty() {
        envelopes::compute_envelopes(model.revision, &result_sets)
    } else {
        None
    };

    Ok(SolveLinearResponse {
        revision: model.revision,
        diagnostics,
        result_sets,
        envelope,
    })
}

pub fn inspect_member_point(
    state: &FrameState,
    request: InspectMemberPointRequest,
) -> Result<InspectMemberPointResponse, CanonicalError> {
    let solve_resp = solve_linear(
        state,
        SolveLinearRequest {
            model_revision: request.model_revision,
            model: request.model,
            combinations: Vec::new(),
            compute_envelope: false,
        },
    )?;

    let rs = solve_resp
        .result_sets
        .iter()
        .find(|r| r.target_id == request.target_id)
        .or_else(|| solve_resp.result_sets.first())
        .ok_or_else(|| {
            CanonicalError::new(
                "TARGET_NOT_FOUND",
                format!("Δεν βρέθηκαν αποτελέσματα για {}", request.target_id),
                format!("No results found for target {}", request.target_id),
            )
        })?;

    let mr = rs
        .member_results
        .iter()
        .find(|m| m.element_id == request.element_id)
        .ok_or_else(|| {
            CanonicalError::new(
                "ELEMENT_NOT_FOUND",
                format!("Το στοιχείο '{}' δεν βρέθηκε.", request.element_id),
                format!("Element '{}' not found in results.", request.element_id),
            )
        })?;

    // Interpolate or find closest station
    let mut closest_st = &mr.stations[0];
    let mut min_diff = f64::INFINITY;
    for st in &mr.stations {
        let diff = (st.x_m - request.x_m).abs();
        if diff < min_diff {
            min_diff = diff;
            closest_st = st;
        }
    }

    Ok(InspectMemberPointResponse {
        revision: solve_resp.revision,
        element_id: request.element_id,
        x_m: request.x_m,
        n_kn: closest_st.n_kn,
        q_kn: closest_st.q_kn,
        m_knm: closest_st.m_knm,
        ux_local_m: closest_st.ux_local_m,
        uy_local_m: closest_st.uy_local_m,
    })
}

pub fn generate_dsm_walkthrough(
    state: &FrameState,
    request: WalkthroughRequest,
) -> Result<DsmWalkthroughData, CanonicalError> {
    let model = match request.model {
        Some(m) => m,
        None => {
            let lock = state
                .model
                .read()
                .map_err(|e| CanonicalError::new("LOCK_ERROR", e.to_string(), e.to_string()))?;
            lock.clone().ok_or_else(|| {
                CanonicalError::new(
                    "NO_ACTIVE_MODEL",
                    "Δεν έχει οριστεί ενεργό μοντέλο.",
                    "No active structural model has been set.",
                )
            })?
        }
    };

    if request.model_revision != model.revision {
        return Err(CanonicalError::new(
            "STALE_MODEL_REVISION",
            format!(
                "Η αναθεώρηση αιτήματος {} δεν ταιριάζει με την τρέχουσα {}.",
                request.model_revision, model.revision
            ),
            format!(
                "Requested revision {} does not match active revision {}.",
                request.model_revision, model.revision
            ),
        ));
    }

    walkthrough::generate_dsm_walkthrough(&model, &request.target_id)
}

pub fn start_analysis_job(
    state: &FrameState,
    request: StartJobRequest,
    progress_callback: Option<job_manager::ProgressCallback>,
) -> Result<StartJobResponse, CanonicalError> {
    let model = match request.model.clone() {
        Some(m) => m,
        None => {
            let lock = state
                .model
                .read()
                .map_err(|e| CanonicalError::new("LOCK_ERROR", e.to_string(), e.to_string()))?;
            lock.clone().ok_or_else(|| {
                CanonicalError::new(
                    "NO_ACTIVE_MODEL",
                    "Δεν έχει οριστεί ενεργό μοντέλο.",
                    "No active structural model has been set.",
                )
            })?
        }
    };

    state.job_manager.start_job(&model, request, progress_callback)
}

pub fn cancel_analysis_job(
    state: &FrameState,
    job_id: &str,
) -> Result<JobStatus, CanonicalError> {
    state.job_manager.cancel_job(job_id)
}

pub fn get_analysis_job_result(
    state: &FrameState,
    job_id: &str,
) -> Result<Option<JobResultPayload>, CanonicalError> {
    state.job_manager.get_job_result(job_id)
}

