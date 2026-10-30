use crate::material::strategies::DirectionalGradientField;
use crate::material::types::{Axis, GradientForm, ReferenceMaterial};
use crate::mesh::generators_2d::generate_beam_2d;
use crate::mesh::types::MeshData;
use crate::solver::dynamic_solver::solve_transient_talbot;
use crate::solver::time_marching::solve_transient_newmark;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DispersionBenchmarkRecord {
    pub method: String,
    pub discretization_param: String,
    pub num_points_or_steps: usize,
    pub dt_or_nodes: f64,
    pub peak_displacement_mm: f64,
    pub period_or_phase_error_pct: f64,
    pub trajectory_l2_error: f64,
    pub cpu_time_ms: f64,
}

#[derive(Debug, Clone)]
pub struct PulseBenchmarkRecord {
    pub method: String,
    pub parameter_label: String,
    pub parameter_value: f64,
    pub peak_tip_disp_mm: f64,
    pub wavefront_arrival_time_s: f64,
    pub max_von_mises_mpa: f64,
    pub cpu_time_ms: f64,
}

#[derive(Debug, Clone)]
pub struct LongDurationBenchmarkRecord {
    pub target_time_s: f64,
    pub newmark_steps: usize,
    pub newmark_cpu_ms: f64,
    pub talbot_nodes: usize,
    pub talbot_cpu_ms: f64,
    pub speedup_talbot: f64,
    pub tip_disp_newmark_mm: f64,
    pub tip_disp_talbot_mm: f64,
    pub rel_disp_diff: f64,
}

/// Runs Sweep A: Dispersion and Frequency Response under Harmonic Sine Loading
pub fn run_harmonic_dispersion_sweep(
    mesh: &MeshData,
    ref_mat: &ReferenceMaterial,
    total_time: f64,
    freq_hz: f64,
    load_mag: f64,
) -> Vec<DispersionBenchmarkRecord> {
    let mut records = Vec::new();
    let strategy = DirectionalGradientField::new(Axis::Y, 0.0, 2.0, 1.0, GradientForm::Linear);

    // 1. High-resolution reference solve using Newmark with 2000 steps (dt = 0.00025 s)
    let ref_steps = 2000;
    let ref_newmark = solve_transient_newmark(
        mesh,
        ref_mat,
        &strategy,
        total_time,
        ref_steps,
        load_mag,
        0.05,
        Some("HarmonicSine"),
        Some(freq_hz),
        Some(0.0),
        Some(0.0), // Undamped for pure dispersion study
        Some(0.0),
    );
    let ref_peak = ref_newmark.max_overall_displacement;

    // Helper to compute trajectory error against reference at 10 sampled times
    let sample_times = [0.05, 0.10, 0.15, 0.20, 0.25, 0.30, 0.35, 0.40, 0.45, 0.50];
    let mut ref_samples = Vec::new();
    for &st in &sample_times {
        let best_idx = ref_newmark
            .frames
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| (a.time - st).abs().partial_cmp(&(b.time - st).abs()).unwrap())
            .map(|(idx, _)| idx)
            .unwrap_or(0);
        ref_samples.push(ref_newmark.frames[best_idx].max_displacement);
    }

    // 2. Sweep Newmark time steps: dt = T/10, T/20, T/40, T/80, T/160
    // Period T = 1 / freq_hz (for 3 Hz, T = 0.333 s)
    let step_counts = [10, 25, 50, 100, 200, 500];
    for &steps in &step_counts {
        let dt = total_time / steps as f64;
        let start = std::time::Instant::now();
        let res = solve_transient_newmark(
            mesh,
            ref_mat,
            &strategy,
            total_time,
            steps,
            load_mag,
            0.05,
            Some("HarmonicSine"),
            Some(freq_hz),
            Some(0.0),
            Some(0.0),
            Some(0.0),
        );
        let cpu_time_ms = start.elapsed().as_secs_f64() * 1000.0;
        let peak = res.max_overall_displacement;
        let peak_err = ((peak - ref_peak).abs() / ref_peak) * 100.0;

        let mut diff_sq = 0.0;
        let mut base_sq = 0.0;
        for (i, &st) in sample_times.iter().enumerate() {
            let best_idx = res
                .frames
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| (a.time - st).abs().partial_cmp(&(b.time - st).abs()).unwrap())
                .map(|(idx, _)| idx)
                .unwrap_or(0);
            let disp = res.frames[best_idx].max_displacement;
            let diff = disp - ref_samples[i];
            diff_sq += diff * diff;
            base_sq += ref_samples[i] * ref_samples[i];
        }
        let l2_err = (diff_sq / base_sq.max(1e-14)).sqrt();

        records.push(DispersionBenchmarkRecord {
            method: "Newmark-Beta (Average Accel)".to_string(),
            discretization_param: "Time Steps".to_string(),
            num_points_or_steps: steps,
            dt_or_nodes: dt,
            peak_displacement_mm: peak * 1000.0,
            period_or_phase_error_pct: peak_err,
            trajectory_l2_error: l2_err,
            cpu_time_ms,
        });
    }

    // 3. Sweep Talbot quadrature nodes: N = 16, 24, 32, 48, 64
    let talbot_nodes_sweep = [16, 24, 32, 48, 64];
    for &nodes in &talbot_nodes_sweep {
        let start = std::time::Instant::now();
        let res = solve_transient_talbot(
            mesh,
            ref_mat,
            &strategy,
            total_time,
            25,
            nodes,
            load_mag,
            0.05,
            Some("HarmonicSine"),
            Some(freq_hz),
        );
        let cpu_time_ms = start.elapsed().as_secs_f64() * 1000.0;
        let peak = res.max_overall_displacement;
        let peak_err = ((peak - ref_peak).abs() / ref_peak) * 100.0;

        let mut diff_sq = 0.0;
        let mut base_sq = 0.0;
        for (i, &st) in sample_times.iter().enumerate() {
            let best_idx = res
                .frames
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| (a.time - st).abs().partial_cmp(&(b.time - st).abs()).unwrap())
                .map(|(idx, _)| idx)
                .unwrap_or(0);
            let disp = res.frames[best_idx].max_displacement;
            let diff = disp - ref_samples[i];
            diff_sq += diff * diff;
            base_sq += ref_samples[i] * ref_samples[i];
        }
        let l2_err = (diff_sq / base_sq.max(1e-14)).sqrt();

        records.push(DispersionBenchmarkRecord {
            method: "Deformed Talbot Contour".to_string(),
            discretization_param: "Contour Nodes".to_string(),
            num_points_or_steps: nodes,
            dt_or_nodes: nodes as f64,
            peak_displacement_mm: peak * 1000.0,
            period_or_phase_error_pct: peak_err,
            trajectory_l2_error: l2_err,
            cpu_time_ms,
        });
    }

    records
}

/// Runs Sweep B: Shock Pulse and Wavefront Transmission
pub fn run_shock_pulse_sweep(
    mesh: &MeshData,
    ref_mat: &ReferenceMaterial,
    total_time: f64,
    pulse_duration: f64,
    load_mag: f64,
) -> Vec<PulseBenchmarkRecord> {
    let mut records = Vec::new();
    let strategy = DirectionalGradientField::new(Axis::Y, 0.0, 2.0, 1.0, GradientForm::Linear);

    // 1. Newmark with varying damping: undamped, damped (Rayleigh), and HHT-alpha (-0.1)
    let damping_configs = [
        ("Newmark (Undamped)", 0.0, 0.0, 0.0),
        ("Newmark (Rayleigh Damped)", 0.0, 0.05, 0.0002),
        ("HHT-Alpha (Dissipative a=-0.05)", -0.05, 0.0, 0.0),
        ("HHT-Alpha (Dissipative a=-0.10)", -0.10, 0.0, 0.0),
    ];

    for (label, hht_a, r_alpha, r_beta) in damping_configs {
        let start = std::time::Instant::now();
        let res = solve_transient_newmark(
            mesh,
            ref_mat,
            &strategy,
            total_time,
            100,
            load_mag,
            pulse_duration,
            Some("HalfSine"),
            None,
            Some(hht_a),
            Some(r_alpha),
            Some(r_beta),
        );
        let cpu_time_ms = start.elapsed().as_secs_f64() * 1000.0;

        // Detect wavefront arrival time when displacement exceeds 1% of peak
        let threshold = res.max_overall_displacement * 0.01;
        let arrival_time = res
            .frames
            .iter()
            .find(|f| f.max_displacement >= threshold)
            .map(|f| f.time)
            .unwrap_or(0.0);

        let max_vm = res
            .frames
            .iter()
            .flat_map(|f| f.von_mises_stresses.iter().cloned())
            .fold(0.0_f64, f64::max);

        records.push(PulseBenchmarkRecord {
            method: label.to_string(),
            parameter_label: "Alpha_HHT / Damping".to_string(),
            parameter_value: hht_a,
            peak_tip_disp_mm: res.max_overall_displacement * 1000.0,
            wavefront_arrival_time_s: arrival_time,
            max_von_mises_mpa: max_vm / 1e6,
            cpu_time_ms,
        });
    }

    // 2. Deformed Talbot Contour with 32 and 64 nodes
    for &nodes in &[32, 64] {
        let start = std::time::Instant::now();
        let res = solve_transient_talbot(
            mesh,
            ref_mat,
            &strategy,
            total_time,
            50,
            nodes,
            load_mag,
            pulse_duration,
            Some("HalfSine"),
            None,
        );
        let cpu_time_ms = start.elapsed().as_secs_f64() * 1000.0;

        let threshold = res.max_overall_displacement * 0.01;
        let arrival_time = res
            .frames
            .iter()
            .find(|f| f.max_displacement >= threshold)
            .map(|f| f.time)
            .unwrap_or(0.0);

        let max_vm = res
            .frames
            .iter()
            .flat_map(|f| f.von_mises_stresses.iter().cloned())
            .fold(0.0_f64, f64::max);

        records.push(PulseBenchmarkRecord {
            method: format!("Deformed Talbot ({} nodes)", nodes),
            parameter_label: "Contour Nodes".to_string(),
            parameter_value: nodes as f64,
            peak_tip_disp_mm: res.max_overall_displacement * 1000.0,
            wavefront_arrival_time_s: arrival_time,
            max_von_mises_mpa: max_vm / 1e6,
            cpu_time_ms,
        });
    }

    records
}

/// Runs Sweep C: Point-in-Time Random Access & Long-Duration Computational Scaling
pub fn run_long_duration_scaling_sweep(
    mesh: &MeshData,
    ref_mat: &ReferenceMaterial,
    durations: &[f64],
    dt_target: f64,
    talbot_nodes: usize,
    load_mag: f64,
) -> Vec<LongDurationBenchmarkRecord> {
    let mut records = Vec::new();
    let strategy = DirectionalGradientField::new(Axis::Y, 0.0, 2.0, 1.0, GradientForm::Linear);

    for &t_end in durations {
        let n_steps = (t_end / dt_target).round() as usize;

        // 1. Newmark time-marching
        let start_newmark = std::time::Instant::now();
        let res_newmark = solve_transient_newmark(
            mesh,
            ref_mat,
            &strategy,
            t_end,
            n_steps,
            load_mag,
            0.05,
            Some("HarmonicSine"),
            Some(3.0),
            Some(0.0),
            Some(0.02),
            Some(0.0001),
        );
        let cpu_newmark = start_newmark.elapsed().as_secs_f64() * 1000.0;
        let disp_newmark = res_newmark.frames.last().map(|f| f.max_displacement).unwrap_or(0.0);

        // 2. Talbot contour evaluated directly at t_end with only 5 frames
        let start_talbot = std::time::Instant::now();
        let res_talbot = solve_transient_talbot(
            mesh,
            ref_mat,
            &strategy,
            t_end,
            5,
            talbot_nodes,
            load_mag,
            0.05,
            Some("HarmonicSine"),
            Some(3.0),
        );
        let cpu_talbot = start_talbot.elapsed().as_secs_f64() * 1000.0;
        let disp_talbot = res_talbot.frames.last().map(|f| f.max_displacement).unwrap_or(0.0);

        let speedup = cpu_newmark / cpu_talbot.max(0.01);
        let diff = (disp_newmark - disp_talbot).abs() / disp_newmark.max(1e-12);

        records.push(LongDurationBenchmarkRecord {
            target_time_s: t_end,
            newmark_steps: n_steps,
            newmark_cpu_ms: cpu_newmark,
            talbot_nodes,
            talbot_cpu_ms: cpu_talbot,
            speedup_talbot: speedup,
            tip_disp_newmark_mm: disp_newmark * 1000.0,
            tip_disp_talbot_mm: disp_talbot * 1000.0,
            rel_disp_diff: diff,
        });
    }

    records
}

/// Exports Sweep A records to CSV
pub fn export_dispersion_to_csv(records: &[DispersionBenchmarkRecord], path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    let mut file = File::create(path)?;
    writeln!(
        file,
        "method,discretization_param,num_points_or_steps,dt_or_nodes,peak_displacement_mm,period_or_phase_error_pct,trajectory_l2_error,cpu_time_ms"
    )?;
    for r in records {
        writeln!(
            file,
            "{},{},{},{:.6},{:.6},{:.4},{:.6e},{:.2}",
            r.method,
            r.discretization_param,
            r.num_points_or_steps,
            r.dt_or_nodes,
            r.peak_displacement_mm,
            r.period_or_phase_error_pct,
            r.trajectory_l2_error,
            r.cpu_time_ms
        )?;
    }
    Ok(())
}

/// Exports Sweep B records to CSV
pub fn export_pulse_to_csv(records: &[PulseBenchmarkRecord], path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    let mut file = File::create(path)?;
    writeln!(
        file,
        "method,parameter_label,parameter_value,peak_tip_disp_mm,wavefront_arrival_time_s,max_von_mises_mpa,cpu_time_ms"
    )?;
    for r in records {
        writeln!(
            file,
            "{},{},{:.4},{:.6},{:.4},{:.4},{:.2}",
            r.method,
            r.parameter_label,
            r.parameter_value,
            r.peak_tip_disp_mm,
            r.wavefront_arrival_time_s,
            r.max_von_mises_mpa,
            r.cpu_time_ms
        )?;
    }
    Ok(())
}

/// Exports Sweep C records to CSV
pub fn export_long_duration_to_csv(records: &[LongDurationBenchmarkRecord], path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    let mut file = File::create(path)?;
    writeln!(
        file,
        "target_time_s,newmark_steps,newmark_cpu_ms,talbot_nodes,talbot_cpu_ms,speedup_talbot,tip_disp_newmark_mm,tip_disp_talbot_mm,rel_disp_diff"
    )?;
    for r in records {
        writeln!(
            file,
            "{:.2},{},{:.2},{},{:.2},{:.2},{:.6},{:.6},{:.6e}",
            r.target_time_s,
            r.newmark_steps,
            r.newmark_cpu_ms,
            r.talbot_nodes,
            r.talbot_cpu_ms,
            r.speedup_talbot,
            r.tip_disp_newmark_mm,
            r.tip_disp_talbot_mm,
            r.rel_disp_diff
        )?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "transient_elastodynamics_test.rs"]
mod tests;
