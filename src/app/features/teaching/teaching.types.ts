export interface Node2D {
  id: number;
  x: number;
  y: number;
  label: string;
}

export type SupportCondition = 'Pinned' | 'RollerX' | 'RollerY' | 'Clamped' | 'Free';

export interface Support2D {
  node_id: number;
  condition: SupportCondition;
}

export interface Member2D {
  id: number;
  start_node: number;
  end_node: number;
  length: number;
  ea: number;
  ei: number;
  label: string;
}

export type PedagogicalPhase =
  | 'DegreeOfIndeterminacy'
  | 'SupportReactions'
  | 'MethodOfJoints'
  | 'MethodOfSections'
  | 'InternalForcesPiecewise'
  | 'CriticalExtrema'
  | 'CrossIteration'
  | 'KaniIteration'
  | 'Summary';

export interface PedagogicalStep {
  step_index: number;
  phase: PedagogicalPhase;
  title: string;
  rationale: string;
  active_entity_id: string | null;
  active_diagram: string | null;
  symbolic_equation: string;
  numerical_substitution: string;
  result_latex: string;
  solved_quantities: Record<string, number>;
}

export interface ReactionResult {
  node_id: number;
  rx: number;
  ry: number;
  rm: number;
  rx_fraction: string;
  ry_fraction: string;
  rm_fraction: string;
}

export interface InternalForceSample {
  x: number;
  n: number;
  q: number;
  m: number;
}

export interface MemberDiagramData {
  member_id: number;
  samples: InternalForceSample[];
  q_zero_locations: number[];
  m_extreme_values: number[];
  m_inflection_points: number[];
  n_expr: string;
  q_expr: string;
  m_expr: string;
}

export interface CrossCycleRow {
  cycle_index: number;
  operation: string;
  member_end_moments: Record<string, number>;
}

export interface CrossTable {
  distribution_factors: Record<string, number>;
  fixed_end_moments: Record<string, number>;
  cycles: CrossCycleRow[];
  final_moments: Record<string, number>;
}

export interface MemberPointLoad2D {
  member_id: number;
  distance_from_start: number;
  p: number;
}

export interface DistributedLoad2D {
  member_id: number;
  q_start: number;
  q_end: number;
  direction: string;
}

export interface TeachingInputConfig {
  preset_name: string;
  method: string;
  length?: number;
  height?: number;
  load_q?: number;
  load_p?: number;
  load_p_pos?: number;
  load_m?: number;
  language?: string;
}

export interface TeachingSolutionDto {
  system_type: string;
  is_isostatic: boolean;
  degree_of_indeterminacy: number;
  nodes: Node2D[];
  members: Member2D[];
  supports: Support2D[];
  reactions: ReactionResult[];
  applied_point_loads: MemberPointLoad2D[];
  applied_dist_loads: DistributedLoad2D[];
  diagrams: MemberDiagramData[];
  steps: PedagogicalStep[];
  cross_table: CrossTable | null;
}
