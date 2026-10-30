import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { NotificationService } from './notification.service';
import type { Notification, NotificationType } from './notification.service';

@Component({
  selector: 'app-notification-host',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <section class="bem-notifications" aria-live="polite" aria-relevant="additions">
      @for (item of notifications.notifications(); track item.id) {
        <div class="bem-toast bem-toast--{{ item.type }}" role="status">
          <span class="bem-toast-icon" aria-hidden="true">{{ iconFor(item) }}</span>
          <span class="bem-toast-message">{{ item.message }}</span>
          <button type="button" (click)="notifications.dismiss(item.id)" aria-label="Dismiss notification">×</button>
        </div>
      }
    </section>
  `,
  styles: [`
    .bem-notifications {
      bottom: 1.5rem;
      display: flex;
      flex-direction: column-reverse;
      gap: 0.5rem;
      max-width: min(25rem, calc(100vw - 3rem));
      pointer-events: none;
      position: fixed;
      right: 1.5rem;
      z-index: 1000;
    }
    .bem-toast {
      align-items: center;
      animation: bem-toast-slide-in 180ms ease-out both;
      background: #1e293b;
      border: 1px solid #334155;
      border-radius: 0.625rem;
      box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.5);
      color: #f8fafc;
      display: flex;
      font-size: 0.875rem;
      gap: 0.75rem;
      line-height: 1.4;
      min-width: min(17.5rem, calc(100vw - 3rem));
      overflow: hidden;
      padding: 0.75rem 1rem 0.75rem 1.25rem;
      pointer-events: auto;
      position: relative;
    }
    .bem-toast::before {
      background: #38bdf8;
      border-radius: 0.625rem 0 0 0.625rem;
      content: '';
      inset: 0 auto 0 0;
      position: absolute;
      width: 0.35rem;
    }
    .bem-toast-icon {
      align-items: center;
      border-radius: 50%;
      color: #38bdf8;
      display: inline-flex;
      flex: 0 0 1.25rem;
      font-size: 0.875rem;
      font-weight: 800;
      height: 1.25rem;
      justify-content: center;
    }
    .bem-toast-message {
      flex: 1;
      min-width: 0;
    }
    .bem-toast button {
      align-items: center;
      background: transparent;
      border: 0;
      border-radius: 0.25rem;
      color: rgb(255 255 255 / 72%);
      cursor: pointer;
      display: inline-flex;
      flex: 0 0 1.5rem;
      font-size: 1.25rem;
      height: 1.5rem;
      justify-content: center;
      line-height: 1;
      padding: 0;
    }
    .bem-toast button:hover {
      background: rgb(255 255 255 / 14%);
      color: #fff;
    }
    .bem-toast.bem-toast--success::before,
    .bem-toast.bem-toast--success .bem-toast-icon {
      color: #4ade80;
      background: #4ade80;
    }
    .bem-toast.bem-toast--error::before,
    .bem-toast.bem-toast--error .bem-toast-icon {
      color: #f87171;
      background: #f87171;
    }
    .bem-toast.bem-toast--warning::before,
    .bem-toast.bem-toast--warning .bem-toast-icon {
      color: #fbbf24;
      background: #fbbf24;
    }
    .bem-toast.bem-toast--info::before,
    .bem-toast.bem-toast--info .bem-toast-icon {
      color: #38bdf8;
      background: #38bdf8;
    }
    @keyframes bem-toast-slide-in {
      from { opacity: 0; transform: translateX(1rem); }
      to { opacity: 1; transform: translateX(0); }
    }
  `],
})
export class NotificationHostComponent {
  protected readonly notifications = inject(NotificationService);

  protected iconFor(notification: Notification): string {
    const icons: Record<NotificationType, string> = {
      success: '✓',
      error: '!',
      warning: '!',
      info: 'i',
    };
    return icons[notification.type];
  }
}
