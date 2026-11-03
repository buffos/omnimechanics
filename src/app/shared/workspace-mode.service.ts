import { Injectable, signal } from '@angular/core';

export type WorkspaceMode = 'continuum' | 'modeler' | 'structural' | 'teaching';

@Injectable({
  providedIn: 'root',
})
export class WorkspaceModeService {
  readonly activeMode = signal<'continuum' | 'modeler' | 'teaching'>('continuum');

  setMode(mode: WorkspaceMode): void {
    const normalized = mode === 'structural' ? 'modeler' : mode;
    this.activeMode.set(normalized);
  }
}
