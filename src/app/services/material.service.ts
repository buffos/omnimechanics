import { Injectable, signal } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';

export interface ReferenceMaterial {
  e0: number; // Pa
  nu0: number;
  rho0: number; // kg/m^3
}

export type Axis = 'X' | 'Y' | 'Z';
export type GradientForm = 'Linear' | 'Quadratic' | 'Exponential';
export type InclusionShape = 'Sphere' | 'CylinderZ' | 'Box';

export type PropertyMetric =
  | 'YoungsModulus'
  | 'ShearModulus'
  | 'PoissonRatio'
  | 'Density'
  | 'WaveSpeedP'
  | 'WaveSpeedS'
  | 'SecantModulus'
  | 'YieldStrength';

export type MaterialStrategyConfig =
  | {
      type: 'DirectionalGradient';
      params: {
        axis: Axis;
        alpha: number;
        length_scale: number;
        power: number;
        form: GradientForm;
      };
    }
  | {
      type: 'AnalyticalExpression';
      params: {
        expression: String;
      };
    }
  | {
      type: 'Inclusion';
      params: {
        shape: InclusionShape;
        center: [number, number, number];
        radius: number;
        box_half_extents: [number, number, number] | null;
        e_inclusion: number;
        nu_inclusion: number;
        rho_inclusion: number;
        smooth_transition: boolean;
      };
    }
  | {
      type: 'VecchioCollinsConcrete';
      params: {
        fc_prime: number;
        epsilon_0: number;
        ft: number;
        rebar_ratio: number;
        simulated_strain: number;
      };
    }
  | {
      type: 'J2Plasticity';
      params: {
        sigma_y0: number;
        hardening_modulus: number;
        plastic_strain: number;
      };
    };

export interface MaterialFieldRequest {
  reference: ReferenceMaterial;
  strategy: MaterialStrategyConfig;
  nodes: [number, number, number][];
  metric: PropertyMetric;
}

export interface MaterialFieldResponse {
  values: number[];
  min_val: number;
  max_val: number;
  ref_val: number;
  unit: string;
  property_name: string;
}

@Injectable({
  providedIn: 'root',
})
export class MaterialService {
  readonly reference = signal<ReferenceMaterial>({
    e0: 2.1e11, // 210 GPa (Steel)
    nu0: 0.30,
    rho0: 7850.0,
  });

  readonly activeMetric = signal<PropertyMetric>('YoungsModulus');
  readonly fieldResult = signal<MaterialFieldResponse | null>(null);
  readonly isEvaluating = signal<boolean>(false);
  readonly evaluationError = signal<string | null>(null);

  async evaluateField(
    strategy: MaterialStrategyConfig,
    nodes: [number, number, number][],
    metric: PropertyMetric = this.activeMetric(),
    reference: ReferenceMaterial = this.reference()
  ): Promise<MaterialFieldResponse> {
    if (nodes.length === 0) {
      const empty: MaterialFieldResponse = {
        values: [],
        min_val: 0,
        max_val: 0,
        ref_val: reference.e0 * 1e-6,
        unit: 'MPa',
        property_name: "Young's Modulus E",
      };
      this.fieldResult.set(empty);
      return empty;
    }

    this.isEvaluating.set(true);
    this.evaluationError.set(null);

    const request: MaterialFieldRequest = {
      reference,
      strategy,
      nodes,
      metric,
    };

    try {
      const res = await invoke<MaterialFieldResponse>('evaluate_material_field', { request });
      this.fieldResult.set(res);
      return res;
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      this.evaluationError.set(msg);
      throw err;
    } finally {
      this.isEvaluating.set(false);
    }
  }
}
