import {
  Component,
  ChangeDetectionStrategy,
  inject,
  model,
  output,
  signal,
  effect,
} from '@angular/core';
import { form, FormField, min, required } from '@angular/forms/signals';
import {
  MaterialService,
  PropertyMetric,
  Axis,
  GradientForm,
  InclusionShape,
} from '../../../services/material.service';

export type StrategyKey =
  | 'DirectionalGradient'
  | 'AnalyticalExpression'
  | 'Inclusion'
  | 'VecchioCollinsConcrete'
  | 'J2Plasticity';

export interface InclusionFormModel {
  shape: InclusionShape;
  radius: number;
  e: number;
  smooth: boolean;
}

export interface ConcreteFormModel {
  fc: number;
  eps0: number;
  simStrain: number;
}

export interface PlasticityFormModel {
  sigmaY: number;
  hardening: number;
  plasticStrain: number;
}

export interface ExpressionFormModel {
  mathExpression: string;
}

@Component({
  selector: 'app-material-tab',
  standalone: true,
  imports: [FormField],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './material-tab.component.html',
  styleUrl: './material-tab.component.css',
})
export class MaterialTabComponent {
  readonly materialService = inject(MaterialService);

  readonly selectedStrategy = model.required<StrategyKey>();
  readonly activeMetric = model.required<PropertyMetric>();

  // Gradient
  readonly gradientAxis = model.required<Axis>();
  readonly gradientAlpha = model.required<number>();
  readonly gradientForm = model.required<GradientForm>();

  // Expression
  readonly mathExpression = model.required<string>();

  // Inclusion
  readonly inclusionShape = model.required<InclusionShape>();
  readonly inclusionRadius = model.required<number>();
  readonly inclusionE = model.required<number>();
  readonly inclusionSmooth = model.required<boolean>();

  // Concrete
  readonly vcFc = model.required<number>();
  readonly vcEps0 = model.required<number>();
  readonly vcSimStrain = model.required<number>();

  // Plasticity
  readonly j2SigmaY = model.required<number>();
  readonly j2Hardening = model.required<number>();
  readonly j2PlasticStrain = model.required<number>();

  readonly strategyChange = output<StrategyKey>();
  readonly metricChange = output<PropertyMetric>();
  readonly evaluateMaterial = output<void>();

  // Signal Forms with Schema Validation
  readonly expressionModel = signal<ExpressionFormModel>({
    mathExpression: 'E0 * (1 + 0.5 * x / 6.0)',
  });
  readonly expressionForm = form(this.expressionModel, (schema) => {
    required(schema.mathExpression, { message: 'Expression is required' });
  });

  readonly inclusionModel = signal<InclusionFormModel>({
    shape: 'CylinderZ',
    radius: 0.5,
    e: 50.0,
    smooth: true,
  });
  readonly inclusionForm = form(this.inclusionModel, (schema) => {
    min(schema.radius, 0.01, { message: 'Radius must be > 0' });
    min(schema.e, 0.1, { message: 'Modulus must be > 0' });
  });

  readonly concreteModel = signal<ConcreteFormModel>({
    fc: 30.0,
    eps0: 0.002,
    simStrain: 0.001,
  });
  readonly concreteForm = form(this.concreteModel, (schema) => {
    min(schema.fc, 1.0, { message: "f'c must be at least 1 MPa" });
    min(schema.eps0, 0.0001, { message: 'Strain must be positive' });
  });

  readonly plasticityModel = signal<PlasticityFormModel>({
    sigmaY: 250.0,
    hardening: 1000.0,
    plasticStrain: 0.0,
  });
  readonly plasticityForm = form(this.plasticityModel, (schema) => {
    min(schema.sigmaY, 1.0, { message: 'Yield stress must be at least 1 MPa' });
    min(schema.hardening, 0.0, { message: 'Hardening modulus cannot be negative' });
  });

  constructor() {
    // Two-way synchronization with parent model signals
    effect(() => {
      const exp = this.expressionModel();
      this.mathExpression.set(exp.mathExpression);
    });

    effect(() => {
      const inc = this.inclusionModel();
      this.inclusionShape.set(inc.shape);
      this.inclusionRadius.set(inc.radius);
      this.inclusionE.set(inc.e);
      this.inclusionSmooth.set(inc.smooth);
    });

    effect(() => {
      const c = this.concreteModel();
      this.vcFc.set(c.fc);
      this.vcEps0.set(c.eps0);
      this.vcSimStrain.set(c.simStrain);
    });

    effect(() => {
      const p = this.plasticityModel();
      this.j2SigmaY.set(p.sigmaY);
      this.j2Hardening.set(p.hardening);
      this.j2PlasticStrain.set(p.plasticStrain);
    });
  }

  onStrategyChange(event: Event): void {
    const val = (event.target as HTMLSelectElement).value as StrategyKey;
    this.selectedStrategy.set(val);
    this.strategyChange.emit(val);
  }

  onMetricChange(event: Event): void {
    const val = (event.target as HTMLSelectElement).value as PropertyMetric;
    this.activeMetric.set(val);
    this.metricChange.emit(val);
  }

  onAlphaChange(event: Event): void {
    const val = +(event.target as HTMLInputElement).value;
    this.gradientAlpha.set(val);
    this.evaluateMaterial.emit();
  }
}
