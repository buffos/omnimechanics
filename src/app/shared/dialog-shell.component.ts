import { afterNextRender, ChangeDetectionStrategy, Component, ElementRef, inject, input } from '@angular/core';
import type { ActiveDialog } from './dialog.service';

@Component({
  selector: 'app-dialog-shell',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './dialog-shell.component.html',
  styleUrl: './dialog-shell.component.css',
})
export class DialogShellComponent {
  private readonly host = inject<ElementRef<HTMLElement>>(ElementRef);

  readonly dialog = input.required<ActiveDialog>();
  readonly active = input(true);

  constructor() {
    afterNextRender(() => this.initialFocusTarget()?.focus());
  }

  protected dismiss(): void {
    this.dialog().ref.dismiss();
  }

  protected trapFocus(event: KeyboardEvent): void {
    if (event.key !== 'Tab') {
      return;
    }
    const focusable = this.focusableElements();
    if (focusable.length === 0) {
      event.preventDefault();
      return;
    }
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  protected titleID(): string {
    return `bem-dialog-title-${this.dialog().ref.id}`;
  }

  private initialFocusTarget(): HTMLElement | null {
    return (
      this.host.nativeElement.querySelector<HTMLElement>('[data-dialog-autofocus]') ??
      this.focusableElements()[0] ??
      this.host.nativeElement.querySelector<HTMLElement>('.bem-dialog')
    );
  }

  private focusableElements(): HTMLElement[] {
    return Array.from(
      this.host.nativeElement.querySelectorAll<HTMLElement>(
        'button:not(:disabled):not([tabindex="-1"]), input:not(:disabled):not([tabindex="-1"]), select:not(:disabled):not([tabindex="-1"]), textarea:not(:disabled):not([tabindex="-1"]), a[href]:not([tabindex="-1"]), [tabindex]:not([tabindex="-1"])'
      )
    );
  }
}
