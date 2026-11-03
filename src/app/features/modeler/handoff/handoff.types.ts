import { StructuralModel2D } from '../core/modeler.types';
import { CombinationDto, SeismicSettingsDto } from '../combinations/combinations.types';

export type StructuralModel = StructuralModel2D;

export interface MethodStatusDto {
  method_id: string;
  name_el: string;
  name_en: string;
  eligible: boolean;
  ineligibility_reason_el?: string | null;
  ineligibility_reason_en?: string | null;
}

export interface MethodEligibility {
  model_revision: number;
  methods: MethodStatusDto[];
  dsm_walkthrough_available: boolean;
  indeterminacy_degree: number;
  is_stable: boolean;
  overall_eligible: boolean;
}

export interface OpenStepByStepResponse {
  model_revision: number;
  selected_method: string;
  solution?: any;
  dsm_walkthrough?: any;
}

export interface ProjectDocument {
  version: string;
  structural: StructuralModel2D;
  combinations: CombinationDto[];
  seismic_settings?: SeismicSettingsDto | null;
  viewport?: any;
}
