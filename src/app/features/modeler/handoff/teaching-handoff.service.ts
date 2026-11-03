import { Injectable, inject, signal } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { StructuralModel2D } from '../core/modeler.types';
import { MethodEligibility, OpenStepByStepResponse, StructuralModel } from './handoff.types';
import { WorkspaceModeService } from '../../../shared/workspace-mode.service';

@Injectable({
  providedIn: 'root',
})
export class TeachingHandoffService {
  private readonly workspaceMode?: WorkspaceModeService;

  readonly eligibility = signal<MethodEligibility | null>(null);
  readonly isClassifying = signal<boolean>(false);
  readonly lastError = signal<string | null>(null);

  constructor(workspaceMode?: WorkspaceModeService) {
    if (workspaceMode) {
      this.workspaceMode = workspaceMode;
    } else {
      try {
        this.workspaceMode = inject(WorkspaceModeService, { optional: true }) ?? undefined;
      } catch {
        this.workspaceMode = undefined;
      }
    }
  }

  /**
   * Evaluates method eligibility for a structural model.
   */
  async classifyModel(model: StructuralModel, loadCaseId?: string): Promise<MethodEligibility> {
    this.isClassifying.set(true);
    this.lastError.set(null);
    try {
      if (typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__) {
        const res = await invoke<MethodEligibility>('teaching_classify_model', {
          request: { model, load_case_id: loadCaseId ?? null },
        });
        this.eligibility.set(res);
        return res;
      }
    } catch (err: any) {
      console.warn('Tauri IPC teaching_classify_model unavailable, using TypeScript fallback:', err);
    } finally {
      this.isClassifying.set(false);
    }

    // Pure TypeScript deterministic fallback
    const res = this.classifyModelTsFallback(model);
    this.eligibility.set(res);
    return res;
  }

  /**
   * Opens the structure in Mode 3 Teaching Studio.
   */
  async openStepByStep(
    model: StructuralModel,
    methodId?: string,
    loadCaseId?: string
  ): Promise<OpenStepByStepResponse> {
    this.lastError.set(null);
    try {
      if (typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__) {
        const res = await invoke<OpenStepByStepResponse>('teaching_open_step_by_step', {
          request: {
            model,
            load_case_id: loadCaseId ?? null,
            method_id: methodId ?? null,
          },
        });
        this.workspaceMode?.setMode('teaching');
        return res;
      }
    } catch (err: any) {
      const msg = err?.message_el || err?.message || String(err);
      this.lastError.set(msg);
      throw err;
    }

    // Fallback response for browser/Vitest
    const elig = this.classifyModelTsFallback(model);
    if (!elig.overall_eligible && methodId !== 'dsm_walkthrough') {
      const err = new Error('Structure is ineligible for classical analytical methods.');
      this.lastError.set('Ο φορέας δεν είναι επιλέξιμος για κλασική αναλυτική μέθοδο.');
      throw err;
    }

    this.workspaceMode?.setMode('teaching');
    return {
      model_revision: model.revision,
      selected_method: methodId ?? 'IsostaticBeamFrame',
    };
  }

  /**
   * Imports a Mode 3 preset into Mode 2 as an editable StructuralModel.
   */
  async importPreset(
    presetId: string,
    params?: { length?: number; height?: number; load_q?: number; load_p?: number; load_p_pos?: number }
  ): Promise<StructuralModel> {
    try {
      if (typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__) {
        const res = await invoke<{ model: StructuralModel }>('teaching_preset_to_structural_model', {
          request: {
            preset_id: presetId,
            length: params?.length ?? null,
            height: params?.height ?? null,
            load_q: params?.load_q ?? null,
            load_p: params?.load_p ?? null,
            load_p_pos: params?.load_p_pos ?? null,
          },
        });
        return res.model;
      }
    } catch (err) {
      console.warn('Tauri IPC teaching_preset_to_structural_model fallback:', err);
    }

    // Pure TS preset generator fallback
    return this.createPresetModelTsFallback(presetId, params);
  }

  private classifyModelTsFallback(model: StructuralModel): MethodEligibility {
    const hasCables = model.elements.some((e: any) => e.kind === 'Cable');
    const hasUnilateral = model.supports.some((s: any) => s.unilateral);

    // Compute basic static determinacy ns = r + m - 2j or similar
    // For 2D frame: total restrained DOFs r
    let r = 0;
    for (const sup of model.supports) {
      if (sup.kind.type === 'Fixed') r += 3;
      else if (sup.kind.type === 'Pinned') r += 2;
      else if (sup.kind.type === 'Roller') r += 1;
      else if (sup.kind.type === 'Guided') r += 2;
    }
    const isBeam = model.elements.length >= 1 && model.elements.every((e: any) => e.kind === 'FrameMember');
    const isPureTruss = model.elements.length >= 1 && model.elements.every((e: any) => e.kind === 'TrussBar');

    const ns = Math.max(0, r - 3);
    const isStable = r >= 3;

    if (hasCables) {
      const reasonEl = 'Ο φορέας περιέχει μη-γραμμικά στοιχεία καλωδίων (BR-5).';
      const reasonEn = 'The structure contains nonlinear catenary cable elements (BR-5).';
      return {
        model_revision: model.revision,
        methods: [
          { method_id: 'IsostaticBeamFrame', name_el: 'Ισοστατικοί Φορείς & Δοκοί', name_en: 'Isostatic Beams & Frames', eligible: false, ineligibility_reason_el: reasonEl, ineligibility_reason_en: reasonEn },
          { method_id: 'TrussJointsRitter', name_el: 'Δικτυώματα - Μέθοδος Κόμβων & Ritter', name_en: 'Trusses - Method of Joints & Ritter', eligible: false, ineligibility_reason_el: reasonEl, ineligibility_reason_en: reasonEn },
          { method_id: 'ContinuousBeamClapeyronCross', name_el: 'Συνεχείς Δοκοί - Clapeyron & Cross', name_en: 'Continuous Beams - Clapeyron & Cross', eligible: false, ineligibility_reason_el: reasonEl, ineligibility_reason_en: reasonEn },
          { method_id: 'ForceMethod', name_el: 'Μέθοδος Δυνάμεων', name_en: 'Force Method', eligible: false, ineligibility_reason_el: reasonEl, ineligibility_reason_en: reasonEn },
        ],
        dsm_walkthrough_available: true,
        indeterminacy_degree: ns,
        is_stable: true,
        overall_eligible: false,
      };
    }

    if (hasUnilateral) {
      const reasonEl = 'Ο φορέας περιέχει μη-γραμμικές μονόπλευρες στηρίξεις.';
      const reasonEn = 'The structure contains nonlinear unilateral supports.';
      return {
        model_revision: model.revision,
        methods: [
          { method_id: 'IsostaticBeamFrame', name_el: 'Ισοστατικοί Φορείς & Δοκοί', name_en: 'Isostatic Beams & Frames', eligible: false, ineligibility_reason_el: reasonEl, ineligibility_reason_en: reasonEn },
          { method_id: 'TrussJointsRitter', name_el: 'Δικτυώματα - Μέθοδος Κόμβων & Ritter', name_en: 'Trusses - Method of Joints & Ritter', eligible: false, ineligibility_reason_el: reasonEl, ineligibility_reason_en: reasonEn },
          { method_id: 'ContinuousBeamClapeyronCross', name_el: 'Συνεχείς Δοκοί - Clapeyron & Cross', name_en: 'Continuous Beams - Clapeyron & Cross', eligible: false, ineligibility_reason_el: reasonEl, ineligibility_reason_en: reasonEn },
          { method_id: 'ForceMethod', name_el: 'Μέθοδος Δυνάμεων', name_en: 'Force Method', eligible: false, ineligibility_reason_el: reasonEl, ineligibility_reason_en: reasonEn },
        ],
        dsm_walkthrough_available: true,
        indeterminacy_degree: ns,
        is_stable: true,
        overall_eligible: false,
      };
    }

    const isostaticEligible = isStable && ns === 0 && !isPureTruss;
    const trussEligible = isStable && ns === 0 && isPureTruss;
    const continuousEligible = isStable && isBeam && ns >= 1 && ns <= 3;
    const forceEligible = isStable && ns >= 1 && ns <= 3;

    const methods = [
      {
        method_id: 'IsostaticBeamFrame',
        name_el: 'Ισοστατικοί Φορείς & Δοκοί',
        name_en: 'Isostatic Beams & Frames',
        eligible: isostaticEligible,
        ineligibility_reason_el: isostaticEligible ? null : `Απαιτείται ns = 0 (τρέχων: ${ns}).`,
        ineligibility_reason_en: isostaticEligible ? null : `Requires ns = 0 (current: ${ns}).`,
      },
      {
        method_id: 'TrussJointsRitter',
        name_el: 'Δικτυώματα - Μέθοδος Κόμβων & Ritter',
        name_en: 'Trusses - Method of Joints & Ritter',
        eligible: trussEligible,
        ineligibility_reason_el: trussEligible ? null : 'Απαιτείται αμιγές ισοστατικό δικτύωμα (TrussBar).',
        ineligibility_reason_en: trussEligible ? null : 'Requires a pure isostatic truss (TrussBar elements only).',
      },
      {
        method_id: 'ContinuousBeamClapeyronCross',
        name_el: 'Συνεχείς Δοκοί - Clapeyron & Cross',
        name_en: 'Continuous Beams - Clapeyron & Cross',
        eligible: continuousEligible,
        ineligibility_reason_el: continuousEligible ? null : 'Εφαρμόζεται σε οριζόντιες συνεχείς δοκούς 2 ή περισσότερων ανοιγμάτων.',
        ineligibility_reason_en: continuousEligible ? null : 'Applies to continuous beams across multiple spans.',
      },
      {
        method_id: 'ForceMethod',
        name_el: 'Μέθοδος Δυνάμεων',
        name_en: 'Force Method',
        eligible: forceEligible,
        ineligibility_reason_el: forceEligible ? null : `Η μέθοδος δυνάμεων εφαρμόζεται για 1 <= ns <= 3 (τρέχων: ${ns}).`,
        ineligibility_reason_en: forceEligible ? null : `Force method applies for 1 <= ns <= 3 (current: ${ns}).`,
      },
    ];

    return {
      model_revision: model.revision,
      methods,
      dsm_walkthrough_available: isStable,
      indeterminacy_degree: ns,
      is_stable: isStable,
      overall_eligible: methods.some((m) => m.eligible),
    };
  }

  private createPresetModelTsFallback(
    presetId: string,
    params?: { length?: number; height?: number; load_q?: number; load_p?: number; load_p_pos?: number }
  ): StructuralModel {
    const l = params?.length ?? 6.0;
    const q = params?.load_q ?? 12.0;

    return {
      id: `model-${presetId}`,
      name: `Preset ${presetId}`,
      revision: 1,
      gravity_g: 9.81,
      nodes: [
        { id: 'N1', x: 0.0, y: 0.0 },
        { id: 'N2', x: l, y: 0.0 },
      ],
      materials: [
        {
          id: 'MAT1',
          name: 'S355 Structural Steel',
          source: { kind: 'Preset', catalog_id: 'S355' },
          class: 'Steel',
          e_gpa: 210.0,
          g_gpa: 81.0,
          density_gamma: 78.5,
          alpha_t: 1.2e-5,
        },
      ],
      sections: [
        {
          id: 'SEC1',
          name: 'IPE 300',
          source: { kind: 'Preset', catalog_id: 'IPE_300' },
          shape: 'Ipe',
          area_a: 0.00538,
          inertia_i: 0.00008356,
          depth_h: 0.30,
        },
      ],
      elements: [
        {
          id: 'E1',
          kind: 'FrameMember',
          node_i: 'N1',
          node_j: 'N2',
          section_id: 'SEC1',
          material_id: 'MAT1',
          release_i: { axial: false, shear: false, moment: false },
          release_j: { axial: false, shear: false, moment: false },
          offset_i: { dx: 0, dy: 0, frame: 'Local' },
          offset_j: { dx: 0, dy: 0, frame: 'Local' },
        },
      ],
      supports: [
        { id: 'S1', node_id: 'N1', kind: { type: 'Pinned' }, unilateral: false },
        { id: 'S2', node_id: 'N2', kind: { type: 'Roller', angle_deg: 0 }, unilateral: false },
      ],
      load_cases: [
        {
          id: 'LC1',
          name: 'Permanent Loads G',
          action_type: { kind: 'Permanent' },
          point_loads: [],
          dist_loads: [
            {
              id: 'L1',
              element_id: 'E1',
              qx1: 0,
              qy1: -q,
              qx2: 0,
              qy2: -q,
              a_rel: 0,
              b_rel: 1,
            },
          ],
          nodal_loads: [],
          thermal_loads: [],
          settlements: [],
        },
      ],
      lumped_masses: [],
    };
  }
}
