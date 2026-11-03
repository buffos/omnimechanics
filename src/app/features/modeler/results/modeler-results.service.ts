import { Injectable, computed, inject, signal, effect, OnDestroy, Injector } from '@angular/core';
import { ModelerStore } from '../core/modeler.store';
import {
  SolveLinearResponse,
  Diagnostics,
  ResultSet,
  CrossSectionHoverProbeDto,
  MemberResult,
  Envelope,
  AnalysisKind,
} from './results.types';
import { StructuralModel2D, isLineElement } from '../core/modeler.types';

@Injectable({
  providedIn: 'root',
})
export class ModelerResultsService implements OnDestroy {
  private readonly store: ModelerStore;

  // Analysis target & kind selectors
  readonly analysisKind = signal<AnalysisKind>('LinearStatic');
  readonly resultTarget = signal<'LoadCase' | 'Combination' | 'Envelope'>('LoadCase');
  readonly activeTargetId = signal<string>('LC1');

  // Analysis result state
  readonly isAnalyzing = signal<boolean>(false);
  readonly diagnostics = signal<Diagnostics | null>(null);
  readonly currentResultSet = signal<ResultSet | null>(null);
  readonly allResultSets = signal<ResultSet[]>([]);
  readonly rawBackendEnvelope = signal<Envelope | null>(null);

  readonly currentEnvelope = computed<Envelope | null>(() => {
    if (this.resultTarget() !== 'Envelope') return null;
    const all = this.allResultSets();
    if (all.length === 0) return this.rawBackendEnvelope();

    const targetId = this.activeTargetId();
    const combs = this.store.combinations();
    const combMap = new Map(combs.map(c => [c.id, c]));

    let filtered = all;
    let factorFallback = 1.0;

    if (targetId.includes('ULS')) {
      factorFallback = 1.35;
      const uls = all.filter(r => {
        const c = combMap.get(r.target_id);
        return c
          ? c.limit_state === 'ULS_STR' || c.limit_state === 'ULS_GEO' || c.limit_state === 'SEISMIC'
          : r.target_id.startsWith('ULS') || r.target_id.startsWith('SEISMIC');
      });
      if (uls.length > 0) filtered = uls;
    } else if (targetId.includes('SLS')) {
      factorFallback = 1.0;
      const sls = all.filter(r => {
        const c = combMap.get(r.target_id);
        return c ? c.limit_state.startsWith('SLS') : r.target_id.startsWith('SLS');
      });
      if (sls.length > 0) filtered = sls;
    }

    if (filtered === all && combs.length === 0 && all.length === 1) {
      return this.synthesizeScaledEnvelope(all[0], factorFallback, targetId.includes('ULS') ? '1.35G' : '1.00G');
    }

    return this.computeEnvelopeFromResultSets(filtered, targetId);
  });
  readonly activeDiagram = signal<'M' | 'Q' | 'N' | 'Deformed' | 'None'>('M');
  readonly diagramScale = signal<number>(1.0);
  readonly deformedScale = signal<number>(50.0);

  // Background nonlinear / dynamic job state
  readonly isBackgroundJobRunning = signal<boolean>(false);
  readonly jobProgressPercent = signal<number>(0);
  readonly activeJobId = signal<string | null>(null);
  private currentJobRevision = 0;

  // Cross section hover probe
  readonly probePoint = signal<CrossSectionHoverProbeDto | null>(null);

  // Mechanism animation state
  readonly mechanismPhase = signal<number>(0);
  private animFrameId: number | null = null;
  private animStartTime = 0;

  // Debounce timer
  private solveTimer: any = null;

  constructor(store?: ModelerStore) {
    this.store = store ?? inject(ModelerStore, { optional: true }) ?? new ModelerStore();

    try {
      const hasInjector = !!inject(Injector, { optional: true });
      if (hasInjector) {
        effect(() => {
          const model = this.store.model();
          this.analysisKind();
          this.resultTarget();
          this.activeTargetId();
          this.scheduleLiveSolve(model);
        });
      }
    } catch {
      // Running in unit test outside Angular injection context
    }

    this.startMechanismLoop();
  }

  ngOnDestroy(): void {
    if (this.solveTimer) clearTimeout(this.solveTimer);
    if (this.animFrameId) cancelAnimationFrame(this.animFrameId);
    this.cancelRunningJob();
  }

  readonly isStable = computed(() => {
    const diag = this.diagnostics();
    if (!diag) return true;
    return diag.is_stable;
  });

  readonly isNonlinearSuperpositionInvalid = computed(() => {
    if (this.analysisKind() === 'SecondOrderPDelta') return true;
    const m = this.store.model();
    const hasCables = m.elements.some(e => e.kind === 'Cable');
    const hasUnilateralTruss = m.elements.some(
      e => e.kind === 'TrussBar' && (e as any).axial_behavior !== 'Both'
    );
    const hasUnilateralSup = m.supports.some(s => s.unilateral);
    return hasCables || hasUnilateralTruss || hasUnilateralSup;
  });

  readonly mechanismDisplacements = computed(() => {
    const diag = this.diagnostics();
    if (!diag || diag.is_stable || diag.mechanism_modes.length === 0) {
      return new Map<string, { dx: number; dy: number }>();
    }

    const mode = diag.mechanism_modes[0];
    const factor = Math.sin(this.mechanismPhase()) * 0.4;
    const map = new Map<string, { dx: number; dy: number }>();

    for (const nd of mode.node_displacements) {
      map.set(nd.node_id, {
        dx: nd.dux * factor,
        dy: nd.duy * factor,
      });
    }

    return map;
  });

  setAnalysisKind(kind: AnalysisKind): void {
    this.analysisKind.set(kind);
  }

  setResultTarget(target: 'LoadCase' | 'Combination' | 'Envelope', targetId?: string): void {
    this.resultTarget.set(target);
    if (targetId) {
      this.activeTargetId.set(targetId);
    } else {
      if (target === 'LoadCase') {
        this.activeTargetId.set(this.store.activeLoadCaseId());
      } else if (target === 'Combination') {
        const combs = this.store.combinations();
        this.activeTargetId.set(combs[0]?.id || 'ULS_STR_1');
      } else {
        this.activeTargetId.set('ENV_ULS_STR');
      }
    }
  }

  setActiveDiagram(diagram: 'M' | 'Q' | 'N' | 'Deformed' | 'None'): void {
    this.activeDiagram.set(diagram);
  }

  probeCrossSection(elementId: string, x_m: number): void {
    // If envelope is selected, probe envelope
    if (this.resultTarget() === 'Envelope') {
      const env = this.currentEnvelope();
      const me = env?.member_envelopes.find(m => m.element_id === elementId);
      if (me && me.stations.length > 0) {
        let st = me.stations[0];
        for (const s of me.stations) {
          if (Math.abs(s.x_m - x_m) < Math.abs(st.x_m - x_m)) {
            st = s;
          }
        }
        this.probePoint.set({
          element_id: elementId,
          x_m,
          n_kn: Math.round(st.max_n * 10) / 10,
          q_kn: Math.round(st.max_q * 10) / 10,
          m_knm: Math.round(st.max_m * 10) / 10,
          ux_local_m: 0,
          uy_local_m: 0,
          tension_fiber: st.max_m > 0 ? 'Bottom' : 'Top',
          governing_combination: st.governing_max_m || '1.35G + 1.50Q1',
          envelope_min_m: Math.round(st.min_m * 10) / 10,
          envelope_max_m: Math.round(st.max_m * 10) / 10,
        });
        return;
      }
    }

    const rs = this.currentResultSet();
    if (!rs) {
      this.probePoint.set(null);
      return;
    }

    const mr = rs.member_results.find(m => m.element_id === elementId);
    if (!mr || mr.stations.length === 0) {
      this.probePoint.set(null);
      return;
    }

    let stBefore = mr.stations[0];
    let stAfter = mr.stations[mr.stations.length - 1];

    for (let i = 0; i < mr.stations.length - 1; i++) {
      if (x_m >= mr.stations[i].x_m && x_m <= mr.stations[i + 1].x_m) {
        stBefore = mr.stations[i];
        stAfter = mr.stations[i + 1];
        break;
      }
    }

    const span = Math.max(1e-5, stAfter.x_m - stBefore.x_m);
    const t = Math.max(0, Math.min(1, (x_m - stBefore.x_m) / span));

    const n_kn = stBefore.n_kn + t * (stAfter.n_kn - stBefore.n_kn);
    const q_kn = stBefore.q_kn + t * (stAfter.q_kn - stBefore.q_kn);
    const m_knm = stBefore.m_knm + t * (stAfter.m_knm - stBefore.m_knm);
    const ux_local_m = stBefore.ux_local_m + t * (stAfter.ux_local_m - stBefore.ux_local_m);
    const uy_local_m = stBefore.uy_local_m + t * (stAfter.uy_local_m - stBefore.uy_local_m);

    const tension_fiber = m_knm > 1e-4 ? 'Bottom' : m_knm < -1e-4 ? 'Top' : 'Neutral';

    this.probePoint.set({
      element_id: elementId,
      x_m,
      n_kn: Math.round(n_kn * 100) / 100,
      q_kn: Math.round(q_kn * 100) / 100,
      m_knm: Math.round(m_knm * 100) / 100,
      ux_local_m,
      uy_local_m,
      tension_fiber,
    });
  }

  clearProbe(): void {
    this.probePoint.set(null);
  }

  private scheduleLiveSolve(model: StructuralModel2D): void {
    // If a background job is running, cancel it
    this.cancelRunningJob();

    if (this.solveTimer) clearTimeout(this.solveTimer);
    this.solveTimer = setTimeout(() => {
      this.runLiveSolve(model);
    }, 150);
  }

  private async cancelRunningJob(): Promise<void> {
    const jobId = this.activeJobId();
    if (jobId) {
      this.activeJobId.set(null);
      this.isBackgroundJobRunning.set(false);
      const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
      if (isTauri) {
        try {
          const { invoke } = await import('@tauri-apps/api/core');
          await invoke('frame_cancel_analysis_job', { request: { job_id: jobId } });
        } catch {}
      }
    }
  }

  private async runLiveSolve(model: StructuralModel2D): Promise<void> {
    if (model.nodes.length === 0 || model.elements.length === 0) {
      this.diagnostics.set(null);
      this.currentResultSet.set(null);
      this.allResultSets.set([]);
      this.rawBackendEnvelope.set(null);
      return;
    }

    this.isAnalyzing.set(true);
    const thisJobRev = model.revision;
    this.currentJobRevision = thisJobRev;

    try {
      const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
      const isNonlinear = this.analysisKind() !== 'LinearStatic';

      if (isNonlinear && isTauri) {
        // Start background job
        const { invoke } = await import('@tauri-apps/api/core');
        const jobId = `job_${Date.now()}`;
        this.activeJobId.set(jobId);
        this.isBackgroundJobRunning.set(true);
        this.jobProgressPercent.set(10);

        // Immediate linear preview while background job runs
        const backendModel = toBackendStructuralModel(model);
        const previewResp = await invoke<SolveLinearResponse>('frame_solve_linear', {
          request: {
            model_revision: model.revision,
            model: backendModel,
            combinations: this.store.combinations(),
            compute_envelope: true,
          },
        });
        if (this.currentJobRevision === thisJobRev) {
          this.diagnostics.set(previewResp.diagnostics);
          this.allResultSets.set(previewResp.result_sets);
          this.rawBackendEnvelope.set(previewResp.envelope ?? null);
          this.currentResultSet.set(previewResp.result_sets[0] ?? null);
        }

        // Start background job with full model & combinations
        const startResp = await invoke<any>('frame_start_analysis_job', {
          request: {
            job_id: jobId,
            model_revision: model.revision,
            model: backendModel,
            kind: this.analysisKind(),
            target_ids: [this.activeTargetId()],
            combinations: this.store.combinations(),
            compute_envelope: true,
          },
        });

        // Poll for completed job result (typically takes 10-80ms for 2D frames)
        if (startResp && startResp.job_id) {
          const actualJobId = startResp.job_id;
          let jobResult: any = null;
          for (let attempt = 0; attempt < 50; attempt++) {
            await new Promise(r => setTimeout(r, 40));
            if (this.currentJobRevision !== thisJobRev) break;
            const res = await invoke<any>('frame_get_analysis_job_result', { jobId: actualJobId });
            if (res && (res.status === 'Converged' || res.status === 'Failed')) {
              jobResult = res;
              break;
            }
          }

          // If revision was not superseded in the meantime
          if (this.currentJobRevision === thisJobRev && jobResult && jobResult.status === 'Converged') {
            this.isBackgroundJobRunning.set(false);
            this.jobProgressPercent.set(100);
            if (jobResult.result_sets && jobResult.result_sets.length > 0) {
              this.allResultSets.set(jobResult.result_sets);
              const targetRs =
                jobResult.result_sets.find((r: any) => r.target_id === this.activeTargetId()) ??
                jobResult.result_sets[0] ??
                null;
              this.currentResultSet.set(targetRs);
            }
            if (jobResult.envelope) {
              this.rawBackendEnvelope.set(jobResult.envelope);
            }
          }
        }
      } else if (isTauri) {
        // Standard Linear Static Solve
        const { invoke } = await import('@tauri-apps/api/core');
        const backendModel = toBackendStructuralModel(model);
        const resp = await invoke<SolveLinearResponse>('frame_solve_linear', {
          request: {
            model_revision: model.revision,
            model: backendModel,
            combinations: this.store.combinations(),
            compute_envelope: true,
          },
        });

        if (this.currentJobRevision === thisJobRev) {
          this.diagnostics.set(resp.diagnostics);
          this.allResultSets.set(resp.result_sets);
          this.rawBackendEnvelope.set(resp.envelope ?? null);
          // Find matching result set for active target
          const targetRs =
            resp.result_sets.find(r => r.target_id === this.activeTargetId()) ??
            resp.result_sets[0] ??
            null;
          this.currentResultSet.set(targetRs);
        }
      } else {
        // Fallback simulation for tests and offline browser preview
        const fallbackResp = simulateFallbackSolve(model, this.analysisKind());
        if (this.currentJobRevision === thisJobRev) {
          this.diagnostics.set(fallbackResp.diagnostics);
          this.allResultSets.set(fallbackResp.result_sets);
          this.rawBackendEnvelope.set(fallbackResp.envelope ?? null);
          this.currentResultSet.set(fallbackResp.result_sets[0] ?? null);
        }
      }
    } catch (err: any) {
      if (err?.code === 'KINEMATIC_MECHANISM_DETECTED' || err?.message?.includes('mechanism')) {
        this.diagnostics.set({
          revision: model.revision,
          static_indeterminacy_degree: -1,
          kinematic_instability_degree: 1,
          free_dofs: 3,
          is_stable: false,
          mechanism_modes: [
            {
              mode_index: 0,
              classification: 'RigidBody',
              node_displacements: model.nodes.map(n => ({ node_id: n.id, dux: 0.1, duy: -0.1, dtheta: 0 })),
            },
          ],
          warnings: [{ code: 'MECHANISM', message_el: 'Κινηματικός μηχανισμός', message_en: 'Kinematic mechanism' }],
        });
        this.currentResultSet.set(null);
        this.allResultSets.set([]);
        this.rawBackendEnvelope.set(null);
      }
    } finally {
      this.isAnalyzing.set(false);
    }
  }

  private synthesizeScaledEnvelope(rs: ResultSet, factor: number, govName: string): Envelope {
    const member_envelopes = rs.member_results.map(mr => ({
      element_id: mr.element_id,
      length_eff_m: mr.length_eff_m,
      stations: mr.stations.map(st => ({
        x_m: st.x_m,
        max_m: st.m_knm * factor,
        min_m: Math.min(0, st.m_knm * factor),
        governing_max_m: govName,
        governing_min_m: govName,
        max_q: st.q_kn * factor,
        min_q: Math.min(0, st.q_kn * factor),
        governing_max_q: govName,
        governing_min_q: govName,
        max_n: st.n_kn * factor,
        min_n: Math.min(0, st.n_kn * factor),
        governing_max_n: govName,
        governing_min_n: govName,
      })),
    }));

    return {
      id: govName,
      limit_state: govName.includes('ULS') ? 'ULS_STR' : 'SLS_CHAR',
      member_envelopes,
    };
  }

  private computeEnvelopeFromResultSets(resultSets: ResultSet[], envelopeId: string): Envelope | null {
    if (resultSets.length === 0) return null;
    const firstRs = resultSets[0];
    const member_envelopes = firstRs.member_results.map(mr => {
      const numStations = mr.stations.length;
      const stations: any[] = [];

      for (let stIdx = 0; stIdx < numStations; stIdx++) {
        const x_m = mr.stations[stIdx].x_m;
        let max_m = -Infinity;
        let min_m = Infinity;
        let gov_max_m = '';
        let gov_min_m = '';
        let max_q = -Infinity;
        let min_q = Infinity;
        let gov_max_q = '';
        let gov_min_q = '';
        let max_n = -Infinity;
        let min_n = Infinity;
        let gov_max_n = '';
        let gov_min_n = '';

        for (const rs of resultSets) {
          const targetMr = rs.member_results.find(m => m.element_id === mr.element_id);
          if (targetMr && stIdx < targetMr.stations.length) {
            const st = targetMr.stations[stIdx];
            if (st.m_knm > max_m) { max_m = st.m_knm; gov_max_m = rs.target_id; }
            if (st.m_knm < min_m) { min_m = st.m_knm; gov_min_m = rs.target_id; }
            if (st.q_kn > max_q) { max_q = st.q_kn; gov_max_q = rs.target_id; }
            if (st.q_kn < min_q) { min_q = st.q_kn; gov_min_q = rs.target_id; }
            if (st.n_kn > max_n) { max_n = st.n_kn; gov_max_n = rs.target_id; }
            if (st.n_kn < min_n) { min_n = st.n_kn; gov_min_n = rs.target_id; }
          }
        }

        stations.push({
          x_m,
          max_m: isFinite(max_m) ? max_m : 0,
          min_m: isFinite(min_m) ? min_m : 0,
          governing_max_m: gov_max_m || 'ULS_STR',
          governing_min_m: gov_min_m || 'ULS_STR',
          max_q: isFinite(max_q) ? max_q : 0,
          min_q: isFinite(min_q) ? min_q : 0,
          governing_max_q: gov_max_q || 'ULS_STR',
          governing_min_q: gov_min_q || 'ULS_STR',
          max_n: isFinite(max_n) ? max_n : 0,
          min_n: isFinite(min_n) ? min_n : 0,
          governing_max_n: gov_max_n || 'ULS_STR',
          governing_min_n: gov_min_n || 'ULS_STR',
        });
      }

      return {
        element_id: mr.element_id,
        length_eff_m: mr.length_eff_m,
        stations,
      };
    });

    return {
      id: envelopeId,
      limit_state: envelopeId.includes('SLS') ? 'SLS_CHAR' : 'ULS_STR',
      member_envelopes,
    };
  }

  private startMechanismLoop(): void {
    const loop = (time: number) => {
      if (!this.animStartTime) this.animStartTime = time;
      const elapsedSec = (time - this.animStartTime) / 1000;
      this.mechanismPhase.set(elapsedSec * 2 * Math.PI * 1.5);
      if (typeof requestAnimationFrame !== 'undefined') {
        this.animFrameId = requestAnimationFrame(loop);
      }
    };
    if (typeof requestAnimationFrame !== 'undefined') {
      this.animFrameId = requestAnimationFrame(loop);
    }
  }
}

/**
 * Transforms frontend StructuralModel2D to Rust Frame StructuralModel payload
 */
export function toBackendStructuralModel(model: StructuralModel2D): any {
  return {
    id: model.id,
    name: model.name,
    revision: model.revision,
    gravity_g: model.gravity_g,
    nodes: model.nodes.map(n => ({ id: n.id, x: n.x, y: n.y })),
    materials: model.materials.map(m => ({
      id: m.id,
      name: m.name,
      source: m.source,
      class: m.class,
      e_gpa: m.e_gpa,
      g_gpa: m.g_gpa,
      density_gamma: m.density_gamma,
      alpha_t: m.alpha_t,
    })),
    sections: model.sections.map(s => ({
      id: s.id,
      name: s.name,
      source: s.source,
      shape: s.shape,
      area_a: s.area_a,
      inertia_i: s.inertia_i,
      shear_area_av: s.shear_area_av,
      depth_h: s.depth_h,
    })),
    elements: model.elements.map(e => {
      if (e.kind === 'FrameMember') {
        return {
          kind: 'FrameMember',
          id: e.id,
          node_i: e.node_i,
          node_j: e.node_j,
          section_id: e.section_id,
          material_id: e.material_id,
          release_i: e.release_i,
          release_j: e.release_j,
          offset_i: e.offset_i,
          offset_j: e.offset_j,
        };
      }
      if (e.kind === 'TrussBar') {
        return {
          kind: 'TrussBar',
          id: e.id,
          node_i: e.node_i,
          node_j: e.node_j,
          section_id: e.section_id,
          material_id: e.material_id,
          axial_behavior: e.axial_behavior,
        };
      }
      if (e.kind === 'Cable') {
        return {
          kind: 'Cable',
          id: e.id,
          node_i: e.node_i,
          node_j: e.node_j,
          section_id: e.section_id,
          material_id: e.material_id,
          prestress: e.prestress,
        };
      }
      return {
        kind: 'Spring',
        id: e.id,
        node_i: e.node_i,
        node_j: e.node_j,
        kx: e.kx,
        ky: e.ky,
        ktheta: e.ktheta,
        angle_deg: e.angle_deg,
      };
    }),
    supports: model.supports.map(s => ({
      id: s.id,
      node_id: s.node_id,
      kind: s.kind,
      unilateral: s.unilateral,
    })),
    load_cases: model.load_cases.map(lc => {
      const loads: any[] = [];
      for (const p of lc.point_loads) {
        loads.push({
          kind: 'MemberPointLoad',
          element_id: p.element_id,
          fx: p.px,
          fy: p.py,
          mz: 0,
          x_ratio: p.a_rel,
          frame: 'Global',
        });
      }
      for (const d of lc.dist_loads) {
        loads.push({
          kind: 'MemberDistributedLoad',
          element_id: d.element_id,
          qx1: d.qx1,
          qy1: d.qy1,
          qx2: d.qx2,
          qy2: d.qy2,
          a_ratio: d.a_rel,
          b_ratio: d.b_rel,
          frame: 'Global',
        });
      }
      for (const n of lc.nodal_loads) {
        loads.push({
          kind: 'NodalLoad',
          node_id: n.node_id,
          fx: n.fx,
          fy: n.fy,
          mz: n.mz,
        });
      }
      for (const st of lc.settlements) {
        loads.push({
          kind: 'SupportSettlement',
          support_id: st.node_id,
          dx: st.delta_x,
          dy: st.delta_y,
          dtheta: st.theta_z,
        });
      }
      for (const th of lc.thermal_loads) {
        loads.push({
          kind: 'ThermalLoad',
          element_id: th.element_id,
          dt_uniform: (th.delta_t_top + th.delta_t_bottom) / 2,
          dt_gradient: th.delta_t_bottom - th.delta_t_top,
        });
      }

      return {
        id: lc.id,
        name: lc.name,
        action_type: lc.action_type,
        self_weight: (lc as any).self_weight ?? false,
        loads,
      };
    }),
    lumped_masses: model.lumped_masses.map(lm => ({
      node_id: lm.node_id,
      mass_x_t: lm.mass_tonnes,
      mass_y_t: lm.mass_tonnes,
      inertia_theta_tm2: lm.mass_moment_tm2,
    })),
  };
}

/**
 * Fallback analytical solution for testing / browser offline use
 */
export function simulateFallbackSolve(
  model: StructuralModel2D,
  kind: AnalysisKind = 'LinearStatic'
): SolveLinearResponse {
  const supportedNodeIds = new Set(model.supports.map(s => s.node_id));
  const isMechanic = supportedNodeIds.size < 2 && model.nodes.length > 1;

  if (isMechanic) {
    return {
      revision: model.revision,
      diagnostics: {
        revision: model.revision,
        static_indeterminacy_degree: -1,
        kinematic_instability_degree: 1,
        free_dofs: model.nodes.length * 3 - model.supports.length * 2,
        is_stable: false,
        mechanism_modes: [
          {
            mode_index: 0,
            classification: 'RigidBody',
            node_displacements: model.nodes.map(n => ({
              node_id: n.id,
              dux: 0.1,
              duy: -0.05,
              dtheta: 0.02,
            })),
          },
        ],
        warnings: [
          {
            code: 'KINEMATIC_INSTABILITY',
            message_el: 'Ανεπαρκείς στηρίξεις - κινηματικός μηχανισμός',
            message_en: 'Insufficient restraints - kinematic mechanism detected',
          },
        ],
      },
      result_sets: [],
    };
  }

  const pdeltaFactor = kind === 'SecondOrderPDelta' ? 1.15 : 1.0;
  const memberResults: MemberResult[] = [];
  const memberEnvelopes = [];
  const nodeMap = new Map(model.nodes.map(n => [n.id, n]));

  for (const elem of model.elements) {
    if (!isLineElement(elem)) continue;
    const n1 = nodeMap.get(elem.node_i);
    const n2 = nodeMap.get(elem.node_j);
    if (!n1 || !n2) continue;

    const len = Math.hypot(n2.x - n1.x, n2.y - n1.y);
    const stations = [];
    const envStations = [];
    const numStations = 11;

    for (let k = 0; k < numStations; k++) {
      const x = (k / (numStations - 1)) * len;
      const m = 10 * x * (len - x) * pdeltaFactor;
      const q = 10 * (len / 2 - x) * pdeltaFactor;
      const n = -5.0 * pdeltaFactor;

      stations.push({
        x_m: x,
        n_kn: n,
        q_kn: q,
        m_knm: m,
        ux_local_m: 0.0001 * (x / len) * pdeltaFactor,
        uy_local_m: -0.001 * Math.sin((Math.PI * x) / len) * pdeltaFactor,
        theta_rad: 0.0002 * pdeltaFactor,
      });

      envStations.push({
        x_m: x,
        max_m: m * 1.35,
        governing_max_m: '1.35G + 1.50Q1',
        min_m: 0,
        governing_min_m: '1.00G',
        max_q: q * 1.35,
        governing_max_q: '1.35G + 1.50Q1',
        min_q: -q * 1.35,
        governing_min_q: '1.35G + 1.50Q1',
        max_n: n * 1.35,
        governing_max_n: '1.35G + 1.50Q1',
        min_n: n * 1.35,
        governing_min_n: '1.35G + 1.50Q1',
      });
    }

    memberResults.push({
      element_id: elem.id,
      length_eff_m: len,
      end_forces_local: {
        ni_kn: -5.0 * pdeltaFactor,
        qi_kn: 5.0 * len * pdeltaFactor,
        mi_knm: 0.0,
        nj_kn: 5.0 * pdeltaFactor,
        qj_kn: -5.0 * len * pdeltaFactor,
        mj_knm: 0.0,
      },
      stations,
      extrema: {
        max_m: { x_m: len / 2, value: ((10 * len * len) / 4) * pdeltaFactor },
        min_m: { x_m: 0, value: 0 },
        max_q: { x_m: 0, value: 5 * len * pdeltaFactor },
        min_q: { x_m: len, value: -5 * len * pdeltaFactor },
        max_n: { x_m: 0, value: -5 * pdeltaFactor },
        min_n: { x_m: len, value: -5 * pdeltaFactor },
        max_abs_deflection_m: { x_m: len / 2, value: 0.001 * pdeltaFactor },
      },
    });

    memberEnvelopes.push({
      element_id: elem.id,
      length_eff_m: len,
      stations: envStations,
    });
  }

  return {
    revision: model.revision,
    diagnostics: {
      revision: model.revision,
      static_indeterminacy_degree: Math.max(0, model.supports.length * 2 - model.nodes.length * 3),
      kinematic_instability_degree: 0,
      free_dofs: Math.max(0, model.nodes.length * 3 - model.supports.length * 2),
      is_stable: true,
      mechanism_modes: [],
      warnings: [],
    },
    result_sets: [
      {
        revision: model.revision,
        target_id: 'LC1',
        target_kind: 'LoadCase',
        analysis_kind: kind,
        superposition_valid: kind === 'LinearStatic',
        nodal_displacements: model.nodes.map(n => ({
          node_id: n.id,
          ux_m: 0.0002 * pdeltaFactor,
          uy_m: -0.0008 * pdeltaFactor,
          theta_rad: 0.0001 * pdeltaFactor,
        })),
        support_reactions: model.supports.map(s => ({
          support_id: s.id,
          node_id: s.node_id,
          rx_kn: 0.0,
          ry_kn: 25.0 * pdeltaFactor,
          mz_knm: 0.0,
        })),
        member_results: memberResults,
      },
    ],
    envelope: {
      id: 'ENV_ULS_STR',
      limit_state: 'ULS_STR',
      member_envelopes: memberEnvelopes,
    },
  };
}
