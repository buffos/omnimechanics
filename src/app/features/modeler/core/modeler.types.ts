export type SnapType =
  | 'Node'
  | 'Intersection'
  | 'Midpoint'
  | 'Ortho'
  | 'AngularGuide'
  | 'Grid';

export interface SnapCandidateDto {
  type: SnapType;
  point: { x: number; y: number };
  screenPoint: { x: number; y: number };
  distancePx: number;
  referenceId?: string;
  guideAngleDeg?: number;
}

export interface Node2D {
  id: string;
  x: number;
  y: number;
}

export interface EndRelease {
  axial: boolean;
  shear: boolean;
  moment: boolean;
}

export interface RigidOffset {
  dx: number;
  dy: number;
  frame: 'Local' | 'Global';
}

export type AxialBehavior = 'Both' | 'TensionOnly' | 'CompressionOnly';

export type CablePrestress =
  | { mode: 'UnstressedLength'; l0: number }
  | { mode: 'SagRatio'; sag_to_span: number }
  | { mode: 'Pretension'; t0: number };

export type ElementKind = 'FrameMember' | 'TrussBar' | 'Spring' | 'Cable';

export interface BaseElement {
  id: string;
  node_i: string;
  node_j: string;
  section_id: string;
  material_id: string;
}

export interface FrameMemberElement extends BaseElement {
  kind: 'FrameMember';
  release_i: EndRelease;
  release_j: EndRelease;
  offset_i: RigidOffset;
  offset_j: RigidOffset;
}

export interface TrussBarElement extends BaseElement {
  kind: 'TrussBar';
  axial_behavior: AxialBehavior;
}

export interface SpringElement {
  kind: 'Spring';
  id: string;
  node_i: string;
  node_j?: string;
  kx: number;
  ky: number;
  ktheta: number;
  angle_deg: number;
}

export interface CableElement extends BaseElement {
  kind: 'Cable';
  prestress: CablePrestress;
}

export type Element2D = FrameMemberElement | TrussBarElement | SpringElement | CableElement;
export type LineElement2D = FrameMemberElement | TrussBarElement | CableElement;

export function isLineElement(e: Element2D): e is LineElement2D {
  return e.kind !== 'Spring';
}

export type SupportKind =
  | { type: 'Fixed' }
  | { type: 'Pinned' }
  | { type: 'Roller'; angle_deg: number }
  | { type: 'Guided'; angle_deg: number }
  | { type: 'Elastic'; kx: number; ky: number; ktheta: number };

export interface Support2D {
  id: string;
  node_id: string;
  kind: SupportKind;
  unilateral: boolean;
}

export interface Material2D {
  id: string;
  name: string;
  source: { kind: 'Preset'; catalog_id: string } | { kind: 'Custom' };
  class: 'Concrete' | 'Steel' | 'Timber' | 'CableSteel' | 'Custom';
  e_gpa: number;
  g_gpa: number;
  density_gamma: number;
  alpha_t: number;
}

export interface Section2D {
  id: string;
  name: string;
  source: { kind: 'Preset'; catalog_id: string } | { kind: 'Custom' };
  shape: 'Ipe' | 'Hea' | 'Heb' | 'HollowRect' | 'HollowCirc' | 'Rectangular' | 'Circular' | 'Generic';
  area_a: number;
  inertia_i: number;
  shear_area_av?: number;
  depth_h?: number;
}

export interface PointLoad2D {
  id: string;
  element_id: string;
  px: number;
  py: number;
  a_rel: number;
}

export interface DistributedLoad2D {
  id: string;
  element_id: string;
  qx1: number;
  qy1: number;
  qx2: number;
  qy2: number;
  a_rel: number;
  b_rel: number;
}

export interface NodalLoad2D {
  id: string;
  node_id: string;
  fx: number;
  fy: number;
  mz: number;
}

export interface ThermalLoad2D {
  id: string;
  element_id: string;
  delta_t_top: number;
  delta_t_bottom: number;
}

export interface Settlement2D {
  id: string;
  node_id: string;
  delta_x: number;
  delta_y: number;
  theta_z: number;
}

export type ActionCategory =
  | { kind: 'Permanent' }
  | { kind: 'Variable'; sub_category?: string }
  | { kind: 'Accidental' }
  | { kind: 'Seismic' };

export interface LoadCase2D {
  id: string;
  name: string;
  action_type: ActionCategory;
  self_weight?: boolean;
  point_loads: PointLoad2D[];
  dist_loads: DistributedLoad2D[];
  nodal_loads: NodalLoad2D[];
  thermal_loads: ThermalLoad2D[];
  settlements: Settlement2D[];
}

export interface StructuralModel2D {
  id: string;
  name: string;
  revision: number;
  gravity_g: number;
  nodes: Node2D[];
  materials: Material2D[];
  sections: Section2D[];
  elements: Element2D[];
  supports: Support2D[];
  load_cases: LoadCase2D[];
  lumped_masses: Array<{ id: string; node_id: string; mass_tonnes: number; mass_moment_tm2: number }>;
}

export type ActiveTool =
  | 'Select'
  | 'DrawMember'
  | 'DrawTruss'
  | 'DrawCable'
  | 'PlaceSupport'
  | 'PlaceLoad'
  | 'Move'
  | 'Measure';

export interface SelectionState {
  nodeIds: Set<string>;
  elementIds: Set<string>;
}
