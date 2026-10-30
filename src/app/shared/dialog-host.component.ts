import { NgComponentOutlet } from '@angular/common';
import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { DialogShellComponent } from './dialog-shell.component';
import { DialogService } from './dialog.service';
import type { ActiveDialog } from './dialog.service';

@Component({
  selector: 'app-dialog-host',
  standalone: true,
  imports: [DialogShellComponent, NgComponentOutlet],
  template: `
    @for (dialog of dialogs.stack(); track dialog.ref.id) {
      <app-dialog-shell [dialog]="dialog" [active]="isActive(dialog)">
        <ng-container *ngComponentOutlet="dialog.component; injector: dialog.injector" />
      </app-dialog-shell>
    }
  `,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DialogHostComponent {
  protected readonly dialogs = inject(DialogService);

  protected isActive(dialog: ActiveDialog): boolean {
    const active = this.dialogs.active();
    return active !== null && active.ref.id === dialog.ref.id;
  }
}
