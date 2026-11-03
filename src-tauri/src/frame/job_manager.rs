use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;

pub type ProgressCallback = Arc<dyn Fn(AnalysisProgressEvent) + Send + Sync + 'static>;

use super::catenary::{has_cable_elements, solve_catenary_cable};
use super::dynamics::{solve_lateral_force, solve_modal_analysis, solve_response_spectrum};
use super::envelopes::compute_envelopes;
use super::linear_solver::solve_linear_static;
use super::pdelta_buckling::{solve_linear_buckling_analysis, solve_second_order_pdelta};
use super::types::*;
use super::unilateral::{has_unilateral_elements, solve_unilateral_active_set};

#[derive(Debug, Clone)]
pub struct JobRecord {
    pub job_id: String,
    pub model_revision: u64,
    pub kind: AnalysisKind,
    pub status: JobStatus,
    pub progress: f32,
    pub current_target_id: Option<String>,
    pub current_iteration: u32,
    pub current_residual: Option<f64>,
    pub cancellation_token: Arc<AtomicBool>,
    pub result: Option<JobResultPayload>,
}

#[derive(Default)]
pub struct JobManager {
    jobs: Arc<RwLock<HashMap<String, Arc<RwLock<JobRecord>>>>>,
    job_counter: AtomicU64,
}

impl JobManager {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            job_counter: AtomicU64::new(1),
        }
    }

    pub fn supersede_jobs_older_than(&self, new_revision: u64) {
        if let Ok(jobs) = self.jobs.read() {
            for record_lock in jobs.values() {
                if let Ok(mut record) = record_lock.write() {
                    if record.model_revision < new_revision
                        && (record.status == JobStatus::Queued || record.status == JobStatus::Running)
                    {
                        record.status = JobStatus::Superseded;
                        record.cancellation_token.store(true, Ordering::SeqCst);
                    }
                }
            }
        }
    }

    pub fn cancel_job(&self, job_id: &str) -> Result<JobStatus, CanonicalError> {
        let jobs = self.jobs.read().map_err(|e| {
            CanonicalError::new("LOCK_ERROR", e.to_string(), e.to_string())
        })?;

        let record_lock = jobs.get(job_id).ok_or_else(|| {
            CanonicalError::new(
                "JOB_NOT_FOUND",
                format!("Η εργασία '{}' δεν βρέθηκε.", job_id),
                format!("Job '{}' was not found.", job_id),
            )
        })?;

        let mut record = record_lock.write().map_err(|e| {
            CanonicalError::new("LOCK_ERROR", e.to_string(), e.to_string())
        })?;

        record.cancellation_token.store(true, Ordering::SeqCst);
        if record.status == JobStatus::Queued || record.status == JobStatus::Running {
            record.status = JobStatus::Cancelled;
        }

        Ok(record.status)
    }

    pub fn get_job_result(&self, job_id: &str) -> Result<Option<JobResultPayload>, CanonicalError> {
        let jobs = self.jobs.read().map_err(|e| {
            CanonicalError::new("LOCK_ERROR", e.to_string(), e.to_string())
        })?;

        let record_lock = jobs.get(job_id).ok_or_else(|| {
            CanonicalError::new(
                "JOB_NOT_FOUND",
                format!("Η εργασία '{}' δεν βρέθηκε.", job_id),
                format!("Job '{}' was not found.", job_id),
            )
        })?;

        let record = record_lock.read().map_err(|e| {
            CanonicalError::new("LOCK_ERROR", e.to_string(), e.to_string())
        })?;

        if let Some(res) = &record.result {
            return Ok(Some(res.clone()));
        }

        // If not completed yet or cancelled without saved payload
        Ok(Some(JobResultPayload {
            job_id: record.job_id.clone(),
            model_revision: record.model_revision,
            kind: record.kind,
            status: record.status,
            result_sets: Vec::new(),
            envelope: None,
            error: None,
        }))
    }

    pub fn start_job(
        &self,
        model: &StructuralModel,
        request: StartJobRequest,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<StartJobResponse, CanonicalError> {
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

        let job_id = format!(
            "job-{}-{}",
            model.revision,
            self.job_counter.fetch_add(1, Ordering::SeqCst)
        );
        let cancellation_token = Arc::new(AtomicBool::new(false));

        let record = Arc::new(RwLock::new(JobRecord {
            job_id: job_id.clone(),
            model_revision: model.revision,
            kind: request.kind,
            status: JobStatus::Queued,
            progress: 0.0,
            current_target_id: None,
            current_iteration: 0,
            current_residual: None,
            cancellation_token: Arc::clone(&cancellation_token),
            result: None,
        }));

        {
            let mut jobs = self.jobs.write().map_err(|e| {
                CanonicalError::new("LOCK_ERROR", e.to_string(), e.to_string())
            })?;
            jobs.insert(job_id.clone(), Arc::clone(&record));
        }

        let model_clone = model.clone();
        let record_clone = Arc::clone(&record);
        let job_id_clone = job_id.clone();
        let req_clone = request.clone();

        thread::spawn(move || {
            // Update status to Running
            {
                if let Ok(mut rec) = record_clone.write() {
                    if rec.status == JobStatus::Superseded || rec.status == JobStatus::Cancelled {
                        return;
                    }
                    rec.status = JobStatus::Running;
                    rec.progress = 10.0;
                }
            }

            if let Some(ref cb) = progress_callback {
                cb(AnalysisProgressEvent {
                    job_id: job_id_clone.clone(),
                    model_revision: model_clone.revision,
                    kind: req_clone.kind,
                    status: JobStatus::Running,
                    progress_percent: 10.0,
                    current_target_id: None,
                    current_iteration: 0,
                    current_residual: None,
                    message_el: "Έναρξη υπολογισμού...".to_string(),
                    message_en: "Starting analysis...".to_string(),
                });
            }

            // Check cancellation
            if cancellation_token.load(Ordering::SeqCst) {
                return;
            }

            // Execute according to kind
            let solve_result: Result<Vec<ResultSet>, CanonicalError> = match req_clone.kind {
                AnalysisKind::LinearStatic => {
                    if has_cable_elements(&model_clone) {
                        solve_catenary_cable(&model_clone, &req_clone.combinations)
                    } else if has_unilateral_elements(&model_clone) {
                        solve_unilateral_active_set(&model_clone, &req_clone.combinations)
                    } else {
                        solve_linear_static(&model_clone, &req_clone.combinations)
                    }
                }
                AnalysisKind::SecondOrderPDelta => {
                    solve_second_order_pdelta(&model_clone, &req_clone.combinations)
                }
                AnalysisKind::LinearBuckling => {
                    solve_linear_buckling_analysis(&model_clone, &req_clone.combinations)
                }
                AnalysisKind::Modal => {
                    solve_modal_analysis(&model_clone, 0.90).map(|dyn_data| {
                        vec![ResultSet {
                            revision: model_clone.revision,
                            target_id: "MODAL".to_string(),
                            target_kind: "Modal".to_string(),
                            analysis_kind: AnalysisKind::Modal,
                            superposition_valid: false,
                            nodal_displacements: Vec::new(),
                            support_reactions: Vec::new(),
                            member_results: Vec::new(),
                            cable_results: Vec::new(),
                            buckling_modes: Vec::new(),
                            dynamic_data: Some(dyn_data),
                        }]
                    })
                }
                AnalysisKind::LateralForce => {
                    solve_lateral_force(&model_clone, 0.24 * 9.81, "B", 1.5).map(|rs| vec![rs])
                }
                AnalysisKind::ResponseSpectrum => {
                    let use_cqc = req_clone.combination_rule.as_deref() != Some("SRSS");
                    let damping = req_clone.damping_ratio.unwrap_or(0.05);
                    solve_response_spectrum(&model_clone, 0.24 * 9.81, "B", 1.5, damping, use_cqc).map(|rs| vec![rs])
                }
            };

            // Check cancellation before saving
            if cancellation_token.load(Ordering::SeqCst) {
                return;
            }

            match solve_result {
                Ok(result_sets) => {
                    let envelope = if req_clone.compute_envelope && !result_sets.is_empty() {
                        compute_envelopes(model_clone.revision, &result_sets)
                    } else {
                        None
                    };

                    let payload = JobResultPayload {
                        job_id: job_id_clone.clone(),
                        model_revision: model_clone.revision,
                        kind: req_clone.kind,
                        status: JobStatus::Converged,
                        result_sets,
                        envelope,
                        error: None,
                    };

                    if let Ok(mut rec) = record_clone.write() {
                        if rec.status != JobStatus::Superseded && rec.status != JobStatus::Cancelled {
                            rec.status = JobStatus::Converged;
                            rec.progress = 100.0;
                            rec.result = Some(payload);
                        }
                    }

                    if let Some(ref cb) = progress_callback {
                        cb(AnalysisProgressEvent {
                            job_id: job_id_clone,
                            model_revision: model_clone.revision,
                            kind: req_clone.kind,
                            status: JobStatus::Converged,
                            progress_percent: 100.0,
                            current_target_id: None,
                            current_iteration: 1,
                            current_residual: Some(0.0),
                            message_el: "Η επίλυση ολοκληρώθηκε επιτυχώς.".to_string(),
                            message_en: "Analysis completed successfully.".to_string(),
                        });
                    }
                }
                Err(err) => {
                    let payload = JobResultPayload {
                        job_id: job_id_clone.clone(),
                        model_revision: model_clone.revision,
                        kind: req_clone.kind,
                        status: JobStatus::Failed,
                        result_sets: Vec::new(),
                        envelope: None,
                        error: Some(err.clone()),
                    };

                    if let Ok(mut rec) = record_clone.write() {
                        if rec.status != JobStatus::Superseded && rec.status != JobStatus::Cancelled {
                            rec.status = JobStatus::Failed;
                            rec.result = Some(payload);
                        }
                    }

                    if let Some(ref cb) = progress_callback {
                        cb(AnalysisProgressEvent {
                            job_id: job_id_clone,
                            model_revision: model_clone.revision,
                            kind: req_clone.kind,
                            status: JobStatus::Failed,
                            progress_percent: 100.0,
                            current_target_id: None,
                            current_iteration: 0,
                            current_residual: None,
                            message_el: err.message_el,
                            message_en: err.message_en,
                        });
                    }
                }
            }
        });

        Ok(StartJobResponse {
            job_id,
            model_revision: model.revision,
            status: JobStatus::Queued,
        })
    }
}
