export interface NodalDisplacementResult {
  node_id: string;
  ux_m: number;
  uy_m: number;
  theta_rad: number;
}

export interface SupportReactionResult {
  support_id: string;
  node_id: string;
  rx_kn: number;
  ry_kn: number;
  mz_knm: number;
}

export interface MemberEndForces {
  ni_kn: number;
  qi_kn: number;
  mi_knm: number;
  nj_kn: number;
  qj_kn: number;
  mj_knm: number;
}

export interface StationResult {
  x_m: number;
  n_kn: number;
  q_kn: number;
  m_knm: number;
  ux_local_m: number;
  uy_local_m: number;
  theta_rad: number;
}

export interface ExtremaPoint {
  x_m: number;
  value: number;
}

export interface MemberExtrema {
  max_m: ExtremaPoint;
  min_m: ExtremaPoint;
  max_q: ExtremaPoint;
  min_q: ExtremaPoint;
  max_n: ExtremaPoint;
  min_n: ExtremaPoint;
  max_abs_deflection_m: ExtremaPoint;
}

export interface CableResult {
  element_id: string;
  unstressed_length_l0_m: number;
  chord_length_m: number;
  horizontal_tension_h_kn: number;
  max_tension_kn: number;
  mid_sag_m: number;
  is_slack: boolean;
  curve_points_global: Array<{ x: number; y: number }>;
}

export interface MemberResult {
  element_id: string;
  length_eff_m: number;
  end_forces_local: MemberEndForces;
  stations: StationResult[];
  extrema: MemberExtrema;
}

export interface NodeDisplacement {
  node_id: string;
  dux: number;
  duy: number;
  dtheta: number;
}

export interface MechanismMode {
  mode_index: number;
  classification: string;
  node_displacements: NodeDisplacement[];
}

export interface DiagnosticMessage {
  code: string;
  element_id?: string;
  node_id?: string;
  message_el: string;
  message_en: string;
}

export interface Diagnostics {
  revision: number;
  static_indeterminacy_degree: number;
  kinematic_instability_degree: number;
  free_dofs: number;
  is_stable: boolean;
  mechanism_modes: MechanismMode[];
  warnings: DiagnosticMessage[];
}

export type AnalysisKind =
  | 'LinearStatic'
  | 'SecondOrderPDelta'
  | 'LinearBuckling'
  | 'Modal'
  | 'ResponseSpectrum'
  | 'LateralForce';

export interface EnvelopeStation {
  x_m: number;
  max_m: number;
  governing_max_m: string;
  min_m: number;
  governing_min_m: string;
  max_q: number;
  governing_max_q: string;
  min_q: number;
  governing_min_q: string;
  max_n: number;
  governing_max_n: string;
  min_n: number;
  governing_min_n: string;
}

export interface MemberEnvelope {
  element_id: string;
  length_eff_m: number;
  stations: EnvelopeStation[];
}

export interface Envelope {
  id: string;
  limit_state: string;
  member_envelopes: MemberEnvelope[];
}

export interface ResultSet {
  revision: number;
  target_id: string;
  target_kind: string;
  analysis_kind: AnalysisKind | string;
  superposition_valid: boolean;
  nodal_displacements: NodalDisplacementResult[];
  support_reactions: SupportReactionResult[];
  member_results: MemberResult[];
  cable_results?: CableResult[];
  buckling_modes?: any[];
  dynamic_data?: any;
}

export interface SolveLinearResponse {
  revision: number;
  diagnostics: Diagnostics;
  result_sets: ResultSet[];
  envelope?: Envelope;
}

export type JobStatus =
  | 'Queued'
  | 'Running'
  | 'Converged'
  | 'Failed'
  | 'Cancelled'
  | 'Superseded';

export interface AnalysisProgressEvent {
  job_id: string;
  model_revision: number;
  kind: AnalysisKind;
  status: JobStatus;
  progress_percent: number;
  current_iteration?: number;
  current_residual?: number;
  message_el: string;
  message_en: string;
}

export interface CrossSectionHoverProbeDto {
  element_id: string;
  x_m: number;
  n_kn: number;
  q_kn: number;
  m_knm: number;
  ux_local_m: number;
  uy_local_m: number;
  tension_fiber: 'Bottom' | 'Top' | 'Neutral';
  governing_combination?: string;
  envelope_min_m?: number;
  envelope_max_m?: number;
}
