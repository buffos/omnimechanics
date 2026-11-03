import { Injectable, signal } from '@angular/core';

export type WorkspaceMode = 'continuum' | 'modeler' | 'teaching';

@Injectable({
  providedIn: 'root',
})
export class WorkspaceModeService {
  readonly activeMode = signal<WorkspaceMode>('continuum');

  setMode(mode: WorkspaceMode): void {
    this.activeMode.set(mode);
  }
}
