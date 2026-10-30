import { Component, ChangeDetectionStrategy, input, output } from '@angular/core';

@Component({
  selector: 'app-header',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './header.component.html',
  styleUrl: './header.component.css',
})
export class HeaderComponent {
  readonly activeDimension = input.required<'2D' | '3D'>();
  readonly activeDomain = input.required<'Static' | 'TimeDomain' | 'LaplaceTalbot'>();
  readonly backendStatus = input.required<string>();

  readonly dimensionChange = output<'2D' | '3D'>();
  readonly domainChange = output<'Static' | 'TimeDomain' | 'LaplaceTalbot'>();
  readonly openGuide = output<void>();
}
