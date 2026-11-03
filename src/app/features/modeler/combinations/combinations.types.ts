export type ActionCategory =
  | { kind: 'Permanent' }
  | { kind: 'Variable'; sub_category?: string }
  | { kind: 'Accidental' }
  | { kind: 'Seismic' };

export interface LoadCaseFactorDto {
  load_case_id: string;
  factor: number;
}

export type CombinationProvenance = 'Generated' | 'UserEdited' | 'Manual';

export interface CombinationDto {
  id: string;
  name: string;
  limit_state: 'ULS_STR' | 'ULS_GEO' | 'SLS_CHAR' | 'SLS_FREQ' | 'SLS_QP' | 'SEISMIC';
  factors: LoadCaseFactorDto[];
  provenance: CombinationProvenance;
}

export interface SeismicSettingsDto {
  zone: string;
  ag_g: number;
  ground_type: 'A' | 'B' | 'C' | 'D' | 'E';
  importance_factor: number;
  behavior_factor_q: number;
  spectrum_type: 'Type1' | 'Type2';
  direction: 'Horizontal' | 'Vertical';
}

export interface DesignSpectrumPointDto {
  period_s: number;
  sd_g: number;
  se_g: number;
}
