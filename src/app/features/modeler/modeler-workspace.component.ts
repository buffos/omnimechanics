import { Component, ChangeDetectionStrategy, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ModelerToolbarComponent } from './ui/modeler-toolbar.component';
import { ModelerCanvasComponent } from './ui/modeler-canvas.component';
import { ModelerPalettesComponent } from './palettes/modeler-palettes.component';
import { PropertyInspectorComponent } from './inspector/property-inspector.component';
import { EndForceTableComponent } from './results/end-force-table.component';
import { CombinationManagerComponent } from './combinations/combination-manager.component';
import { ModelerStore } from './core/modeler.store';

@Component({
  selector: 'app-modeler-workspace',
  standalone: true,
  imports: [
    CommonModule,
    ModelerToolbarComponent,
    ModelerCanvasComponent,
    ModelerPalettesComponent,
    PropertyInspectorComponent,
    EndForceTableComponent,
    CombinationManagerComponent,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './modeler-workspace.component.html',
  styleUrl: './modeler-workspace.component.css',
})
export class ModelerWorkspaceComponent {
  readonly store = inject(ModelerStore);
}
