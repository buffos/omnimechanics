import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TeachingService } from '../teaching.service';
import { KatexPipe } from '../../../shared/katex.pipe';
import { I18nService } from '../../../shared/i18n.service';

@Component({
  selector: 'app-step-walker',
  standalone: true,
  imports: [CommonModule, KatexPipe],
  templateUrl: './step-walker.component.html',
  styleUrls: ['./step-walker.component.css'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class StepWalkerComponent {
  readonly teaching = inject(TeachingService);
  readonly i18n = inject(I18nService);

  getPhaseBadgeClass(phase: string | undefined): string {
    switch (phase) {
      case 'DegreeOfIndeterminacy':
        return 'badge-indeterminacy';
      case 'SupportReactions':
        return 'badge-reactions';
      case 'MethodOfJoints':
        return 'badge-joints';
      case 'MethodOfSections':
        return 'badge-ritter';
      case 'InternalForcesPiecewise':
        return 'badge-forces';
      case 'CriticalExtrema':
        return 'badge-extrema';
      case 'CrossIteration':
        return 'badge-cross';
      default:
        return 'badge-default';
    }
  }

  getPhaseLabel(phase: string | undefined): string {
    if (!phase) return this.i18n.t('phase.Default');
    const key = `phase.${phase}`;
    const translated = this.i18n.t(key);
    return translated !== key ? translated : phase;
  }
}
