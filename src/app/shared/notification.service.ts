import { Injectable, signal } from '@angular/core';

export type NotificationType = 'success' | 'error' | 'warning' | 'info';

export interface Notification {
  id: number;
  message: string;
  type: NotificationType;
}

@Injectable({ providedIn: 'root' })
export class NotificationService {
  private readonly state = signal<Notification[]>([]);
  private readonly dismissTimers = new Map<number, ReturnType<typeof window.setTimeout>>();
  private nextID = 0;

  readonly notifications = this.state.asReadonly();

  show(message: string, type: NotificationType = 'info'): void {
    const existing = this.state().find((item) => item.message === message && item.type === type);
    if (existing !== undefined) {
      this.scheduleDismissal(existing.id);
      return;
    }
    const notification = { id: ++this.nextID, message, type };
    this.state.update((items) => [...items, notification]);
    this.scheduleDismissal(notification.id);
  }

  dismiss(id: number): void {
    const timer = this.dismissTimers.get(id);
    if (timer !== undefined) window.clearTimeout(timer);
    this.dismissTimers.delete(id);
    this.state.update((items) => items.filter((item) => item.id !== id));
  }

  private scheduleDismissal(id: number): void {
    const existingTimer = this.dismissTimers.get(id);
    if (existingTimer !== undefined) window.clearTimeout(existingTimer);
    this.dismissTimers.set(id, window.setTimeout(() => this.dismiss(id), 5000));
  }
}
