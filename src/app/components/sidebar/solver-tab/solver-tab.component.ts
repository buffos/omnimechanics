import {
  Component,
  ChangeDetectionStrategy,
  inject,
  input,
  model,
  output,
  signal,
  effect,
} from '@angular/core';
import { form, FormField, min } from '@angular/forms/signals';
import {
  SolverService,
  SolverMode,
  HarmonicSolverMode,
  BemElementType,
  StressState2D,
  TransientMethod,
  TransientExcitation,
} from '../../../services/solver.service';
import { SurroundingMediumConfig } from '../../../dialogs/surrounding-medium-dialog.component';
import { SolverDynamicPanelComponent } from './solver-dynamic-panel.component';

export interface SolverFormModel {
  loadMagnitude: number;
  maxIterations: number;
  solverTolerance: number;
}

@Component({
  selector: 'app-solver-tab',
  standalone: true,
  imports: [FormField, SolverDynamicPanelComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './solver-tab.component.html',
  styleUrl: './solver-tab.component.css',
})
export class SolverTabComponent {
  readonly solverService = inject(SolverService);

  readonly hasMesh = input.required<boolean>();

  readonly activeDimension = model.required<'2D' | '3D'>();
  readonly activeDomain = model.required<'Static' | 'TimeDomain' | 'LaplaceTalbot'>();

  readonly solverMode = model.required<SolverMode>();
  readonly stressState = model.required<StressState2D>();
  readonly bemElementType = model.required<BemElementType>();

  readonly harmonicSolverMode = model.required<HarmonicSolverMode>();
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

  readonly loadMagnitude = model.required<number>();
  readonly maxIterations = model.required<number>();
  readonly solverTolerance = model.required<number>();

  readonly domainChange = output<'Static' | 'TimeDomain' | 'LaplaceTalbot'>();
  readonly solve = output<void>();
  readonly openMediumDialog = output<void>();
  readonly openGuide = output<void>();

  // Signal Form for numerical solver parameters
  readonly solverModel = signal<SolverFormModel>({
    loadMagnitude: 10000.0,
    maxIterations: 50,
    solverTolerance: 1e-6,
  });

  readonly solverForm = form(this.solverModel, (schema) => {
    min(schema.loadMagnitude, 0.0, { message: 'Load magnitude must be positive' });
    min(schema.maxIterations, 1, { message: 'At least 1 iteration required' });
    min(schema.solverTolerance, 1e-12, { message: 'Tolerance must be positive' });
  });

  constructor() {
    // Two-way synchronization with model inputs
    effect(() => {
      const s = this.solverModel();
      this.loadMagnitude.set(s.loadMagnitude);
      this.maxIterations.set(s.maxIterations);
      this.solverTolerance.set(s.solverTolerance);
    });
  }

  onSolverModeChange(event: Event): void {
    const val = (event.target as HTMLSelectElement).value as SolverMode;
    this.solverMode.set(val);
  }
}
