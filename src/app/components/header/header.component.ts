import { Component, ChangeDetectionStrategy, inject, input, output } from '@angular/core';
import { I18nService } from '../../shared/i18n.service';

@Component({
  selector: 'app-header',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './header.component.html',
  styleUrl: './header.component.css',
})
export class HeaderComponent {
  readonly i18n = inject(I18nService);

  readonly activeDimension = input.required<'2D' | '3D'>();
  readonly activeDomain = input.required<'Static' | 'TimeDomain' | 'LaplaceTalbot'>();
  readonly backendStatus = input.required<string>();
  readonly activeWorkspaceMode = input<'continuum' | 'teaching'>('continuum');

  readonly dimensionChange = output<'2D' | '3D'>();
  readonly domainChange = output<'Static' | 'TimeDomain' | 'LaplaceTalbot'>();
  readonly modeChange = output<'continuum' | 'teaching'>();
  readonly openGuide = output<void>();
}
