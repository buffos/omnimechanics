import { MeshData } from '../../services/mesh.service';

export type LoopOrientation = 'CounterClockwise' | 'Clockwise';
export type LoopClosureState = 'OpenDraft' | 'ClosedValid' | 'ClosedInvalid';

export interface Vertex2D {
  id: string;
  x: number;
  y: number;
}

export interface BoundarySegment {
  id: string;
  start_vertex_id: string;
  end_vertex_id: string;
  tag: string;
  local_element_size?: number;
  outward_normal: [number, number, number];
}

export interface BoundaryLoop {
  id: string;
  orientation: LoopOrientation;
  closure_state: LoopClosureState;
  vertices: Vertex2D[];
  segments: BoundarySegment[];
}

export type CutoutShape =
  | {
      kind: 'Circle';
      center_x: number;
      center_y: number;
      radius: number;
      angular_divisions: number;
    }
  | {
      kind: 'Rectangle';
      center_x: number;
      center_y: number;
      width: number;
      height: number;
      rotation_deg: number;
    }
  | {
      kind: 'Polygon';
      vertices: Vertex2D[];
    };

export interface Cutout {
  id: string;
  shape: CutoutShape;
  tag: string;
  local_element_size?: number;
}

export interface InclusionRegion {
  id: string;
  shape: CutoutShape;
  material_label: string;
  youngs_modulus_override_gpa?: number;
  poisson_ratio_override?: number;
  local_element_size?: number;
}

export type TractionFrame = 'Global' | 'LocalNormalTangential';

export type BoundaryConditionSpec =
  | { kind: 'Free' }
  | {
      kind: 'DirichletDisplacement';
      fix_x: boolean;
      fix_y: boolean;
      ux_m: number;
      uy_m: number;
    }
  | {
      kind: 'NeumannTraction';
      tx_kn_m2: number;
      ty_kn_m2: number;
      coordinate_frame: TractionFrame;
    }
  | {
      kind: 'RobinImpedance';
      kn_kn_m3: number;
      ks_kn_m3: number;
      cn_kn_s_m3: number;
      cs_kn_s_m3: number;
    };

export type VolumeElementKind = 'Tri3' | 'Quad4';
export type BoundaryElementKind = 'Line2' | 'Line3';

export interface MeshingProfile {
  target_element_size: number;
  hole_refinement_factor: number;
  min_angle_deg: number;
  volume_element_type: VolumeElementKind;
  boundary_element_type: BoundaryElementKind;
}

export interface ContinuumDomain {
  id: string;
  revision: number;
  outer_boundary: BoundaryLoop;
  cutouts: Cutout[];
  inclusions: InclusionRegion[];
  tag_conditions: Record<string, BoundaryConditionSpec>;
  meshing_profile: MeshingProfile;
}

export interface CandidateSegmentPreview {
  is_valid: boolean;
  intersecting_segment_indices: number[];
  snap_closure_ready: boolean;
}

export interface DomainValidationDiagnostic {
  code: string;
  message_el: string;
  message_en: string;
  details?: Record<string, unknown>;
}

export interface DomainValidationResponse {
  is_valid_for_meshing: boolean;
  diagnostics: DomainValidationDiagnostic[];
}

export interface MeshQualityReport {
  num_nodes: number;
  num_boundary_elements: number;
  num_volume_elements: number;
  min_angle_deg: number;
  max_angle_deg: number;
  min_aspect_ratio: number;
  max_aspect_ratio: number;
  is_conformal: boolean;
  outward_normals_verified: boolean;
}

export interface ConformalMeshHandoff {
  domain_id: string;
  domain_revision: number;
  mesh_data: MeshData;
  quality_report: MeshQualityReport;
  tag_element_counts: Record<string, number>;
}

export type SketcherTool =
  | 'Select'
  | 'OuterPolygon'
  | 'QuickRectangle'
  | 'QuickPlateWithHole'
  | 'AddCircularCutout'
  | 'AddRectangularCutout'
  | 'AddCircularInclusion';
