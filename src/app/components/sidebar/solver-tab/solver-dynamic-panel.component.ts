import {
  Component,
  ChangeDetectionStrategy,
  model,
  output,
  signal,
  effect,
} from '@angular/core';
import { form, FormField, min } from '@angular/forms/signals';
import {
  HarmonicSolverMode,
  BemElementType,
  TransientMethod,
  TransientExcitation,
} from '../../../services/solver.service';
import { SurroundingMediumConfig } from '../../../dialogs/surrounding-medium-dialog.component';

export interface DynamicParamsFormModel {
  frequency: number;
  transientFreq: number;
  pulseDuration: number;
  totalTime: number;
  timeSteps: number;
  talbotPoints: number;
  hhtAlpha: number;
  rayleighAlpha: number;
}

@Component({
  selector: 'app-solver-dynamic-panel',
  standalone: true,
  imports: [FormField],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './solver-dynamic-panel.component.html',
  styles: `
    :host {
      display: contents;
    }
    .domain-sidebar-control {
      width: 100%;
    }
    .domain-sidebar-control button {
      flex: 1;
      text-align: center;
      padding: 6px 4px;
      font-size: 0.775rem;
    }
    .full-width {
      width: 100%;
    }
  `,
})
export class SolverDynamicPanelComponent {
  readonly activeDimension = model.required<'2D' | '3D'>();
  readonly activeDomain = model.required<'Static' | 'TimeDomain' | 'LaplaceTalbot'>();

  readonly harmonicSolverMode = model.required<HarmonicSolverMode>();
  readonly bemElementType = model.required<BemElementType>();
  readonly dynamicFrequency = model.required<number>();
  readonly mediumConfig = model.required<SurroundingMediumConfig>();

  readonly transientMethod = model.required<TransientMethod>();
  readonly transientSolverMode = model.required<'PureFEM' | 'PureBEM'>();
  readonly transientExcitation = model.required<TransientExcitation>();
  readonly transientFrequency = model.required<number>();
  readonly dynamicPulseDuration = model.required<number>();
  readonly dynamicTotalTime = model.required<number>();
  readonly dynamicTimeSteps = model.required<number>();
  readonly dynamicNumTalbotPoints = model.required<number>();
  readonly hhtAlpha = model.required<number>();
  readonly rayleighAlpha = model.required<number>();

  readonly openMediumDialog = output<void>();

  // Signal Form for dynamic & transient simulation parameters
  readonly dynamicModel = signal<DynamicParamsFormModel>({
    frequency: 5.0,
    transientFreq: 5.0,
    pulseDuration: 0.05,
    totalTime: 0.2,
    timeSteps: 100,
    talbotPoints: 24,
    hhtAlpha: 0.0,
    rayleighAlpha: 0.0,
  });

  readonly dynamicForm = form(this.dynamicModel, (schema) => {
    min(schema.frequency, 0.1, { message: 'Frequency must be > 0' });
    min(schema.transientFreq, 0.1, { message: 'Frequency must be > 0' });
    min(schema.pulseDuration, 0.001, { message: 'Pulse duration must be > 0' });
    min(schema.totalTime, 0.001, { message: 'Total time must be > 0' });
    min(schema.timeSteps, 5, { message: 'At least 5 time steps required' });
    min(schema.talbotPoints, 6, { message: 'At least 6 Talbot points required' });
  });

  constructor() {
    effect(() => {
      const d = this.dynamicModel();
      this.dynamicFrequency.set(d.frequency);
      this.transientFrequency.set(d.transientFreq);
      this.dynamicPulseDuration.set(d.pulseDuration);
      this.dynamicTotalTime.set(d.totalTime);
      this.dynamicTimeSteps.set(d.timeSteps);
      this.dynamicNumTalbotPoints.set(d.talbotPoints);
      this.hhtAlpha.set(d.hhtAlpha);
      this.rayleighAlpha.set(d.rayleighAlpha);
    });
  }

  onHarmonicSolverModeChange(event: Event): void {
    const val = (event.target as HTMLSelectElement).value as HarmonicSolverMode;
    this.harmonicSolverMode.set(val);
  }
}
