import {
  computed,
  createEnvironmentInjector,
  DestroyRef,
  EnvironmentInjector,
  Injectable,
  InjectionToken,
  inject,
  signal,
} from '@angular/core';
import type { Type } from '@angular/core';
import { I18nService } from './i18n.service';
import { NotificationService } from './notification.service';

export interface DialogRef<TResult> {
  readonly id: number;
  readonly closed: Promise<TResult | undefined>;
  close(result?: TResult): void;
  dismiss(): void;
  setDismissGuard(guard: () => boolean): void;
}

export interface DialogHandle<TResult> extends DialogRef<TResult> {
  dispose(): void;
}

export interface DialogContent<TResult, TData = undefined> {
  readonly dialogRef: DialogRef<TResult>;
  readonly dialogData: TData;
}

export interface ActiveDialog<TResult = unknown> {
  readonly component: Type<unknown>;
  readonly config: DialogConfig<unknown>;
  readonly injector: EnvironmentInjector;
  readonly ref: DialogRef<TResult>;
}

export interface DialogConfig<TData> {
  readonly title: string;
  readonly eyebrow?: string;
  readonly description?: string;
  readonly data?: TData;
  readonly dismissible?: boolean;
  readonly size?: 'standard' | 'wide';
}

export const DIALOG_REF = new InjectionToken<DialogRef<unknown>>('DIALOG_REF');
export const DIALOG_DATA = new InjectionToken<unknown>('DIALOG_DATA');

export function injectDialogRef<TResult>(): DialogRef<TResult> {
  return inject(DIALOG_REF) as DialogRef<TResult>;
}

export function injectDialogData<TData>(): TData {
  return inject(DIALOG_DATA) as TData;
}

interface DialogEntry extends ActiveDialog {
  readonly resolve: (result: unknown | undefined) => void;
  readonly trigger: HTMLElement | null;
  readonly dismissGuard: () => boolean;
}

@Injectable({ providedIn: 'root' })
export class DialogService {
  private readonly environmentInjector: EnvironmentInjector | null;
  private readonly destroyRef: DestroyRef | null;
  private readonly notifications: NotificationService;
  private readonly i18n: I18nService;
  private readonly entries = signal<DialogEntry[]>([]);
  private nextID = 0;
  private previousBodyOverflow: string | null = null;
  readonly stack = this.entries.asReadonly();
  readonly active = computed(() => this.entries().at(-1) ?? null);
  readonly lastBlockedErrorCode = signal<string | null>(null);

  constructor(notificationsOverride?: NotificationService, i18nOverride?: I18nService) {
    let envInj: EnvironmentInjector | null = null;
    let dRef: DestroyRef | null = null;
    try {
      envInj = inject(EnvironmentInjector, { optional: true });
      dRef = inject(DestroyRef, { optional: true });
    } catch {
      envInj = null;
      dRef = null;
    }
    this.environmentInjector = envInj;
    this.destroyRef = dRef;
    this.notifications =
      notificationsOverride ?? this.tryInject(NotificationService) ?? new NotificationService();
    this.i18n = i18nOverride ?? this.tryInject(I18nService) ?? new I18nService();
    this.destroyRef?.onDestroy(() => this.unlockPageScroll());
  }

  private tryInject<T>(token: Type<T>): T | null {
    try {
      return inject(token, { optional: true });
    } catch {
      return null;
    }
  }

  open<TResult, TData = undefined>(
    component: Type<any>,
    config: DialogConfig<TData>,
  ): DialogHandle<TResult> {
    const id = ++this.nextID;
    const trigger =
      typeof document === 'undefined' ? null : (document.activeElement as HTMLElement | null);
    let resolveClosed!: (result: TResult | undefined) => void;
    const closed = new Promise<TResult | undefined>((resolve) => {
      resolveClosed = resolve;
    });
    const ref: DialogHandle<TResult> = {
      id,
      closed,
      close: (result) => this.close(id, result),
      dismiss: () => this.dismiss(id),
      dispose: () => this.dispose(id),
      setDismissGuard: (guard) => this.setDismissGuard(id, guard),
    };
    const injector = this.environmentInjector
      ? createEnvironmentInjector(
          [
            { provide: DIALOG_REF, useValue: ref },
            { provide: DIALOG_DATA, useValue: config.data },
          ],
          this.environmentInjector,
        )
      : ({ destroy: () => {} } as EnvironmentInjector);
    const entry: DialogEntry = {
      component,
      config,
      injector,
      ref: ref as DialogRef<unknown>,
      resolve: (result) => resolveClosed(result as TResult | undefined),
      trigger,
      dismissGuard: () => true,
    };
    if (this.entries().length === 0) this.lockPageScroll();
    this.entries.update((items) => [...items, entry]);
    return ref;
  }

  private close(id: number, result: unknown | undefined): void {
    const entry = this.active();
    if (entry === null || entry.ref.id !== id) {
      return;
    }
    const remaining = this.entries().slice(0, -1);
    this.entries.set(remaining);
    if (remaining.length === 0) this.unlockPageScroll();
    entry.injector.destroy();
    entry.resolve(result);
    queueMicrotask(() => entry.trigger?.focus());
  }

  private dismiss(id: number): void {
    const entry = this.active();
    if (entry === null || entry.ref.id !== id) {
      return;
    }
    if (entry.config.dismissible === false || !entry.dismissGuard()) {
      this.lastBlockedErrorCode.set('DIALOG_DISMISS_BLOCKED');
      const msg =
        this.i18n.currentLang() === 'el'
          ? 'DIALOG_DISMISS_BLOCKED: Μη έγκυρες ή μη αποθηκευμένες αλλαγές στο παράθυρο διαλόγου.'
          : 'DIALOG_DISMISS_BLOCKED: Unsaved or invalid configuration in modal dialog.';
      this.notifications.show(msg, 'warning');
      return;
    }
    this.lastBlockedErrorCode.set(null);
    this.close(id, undefined);
  }

  private dispose(id: number): void {
    const entries = this.entries();
    const index = entries.findIndex((entry) => entry.ref.id === id);
    if (index < 0) return;
    const removed = entries.slice(index);
    const remaining = entries.slice(0, index);
    this.entries.set(remaining);
    for (const entry of removed.reverse()) {
      entry.injector.destroy();
      entry.resolve(undefined);
    }
    if (remaining.length === 0) this.unlockPageScroll();
  }

  private setDismissGuard(id: number, guard: () => boolean): void {
    this.entries.update((items) =>
      items.map((entry) => (entry.ref.id === id ? { ...entry, dismissGuard: guard } : entry)),
    );
  }

  private lockPageScroll(): void {
    if (typeof document === 'undefined' || this.previousBodyOverflow !== null) return;
    this.previousBodyOverflow = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
  }

  private unlockPageScroll(): void {
    if (typeof document === 'undefined' || this.previousBodyOverflow === null) return;
    document.body.style.overflow = this.previousBodyOverflow;
    this.previousBodyOverflow = null;
  }
}
