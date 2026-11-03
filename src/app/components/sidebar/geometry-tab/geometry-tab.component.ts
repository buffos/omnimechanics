import {
  Component,
  ChangeDetectionStrategy,
  inject,
  model,
  output,
  signal,
  effect,
} from '@angular/core';
import { form, FormField, min, max, required } from '@angular/forms/signals';
import { MeshService } from '../../../services/mesh.service';

export type PresetKey =
  | 'Beam2D'
  | 'PlateWithHole2D'
  | 'Beam3D'
  | 'BlockWithCavity3D';

export interface BeamFormModel {
  length: number;
  height: number;
  nx: number;
  ny: number;
  useTriangles: boolean;
}

export interface PlateFormModel {
  width: number;
  height: number;
  radius: number;
  radialDivs: number;
  angularDivs: number;
}

export interface Beam3DFormModel {
  length: number;
  height: number;
  depth: number;
  nx: number;
  ny: number;
  nz: number;
  useTets: boolean;
}

@Component({
  selector: 'app-geometry-tab',
  standalone: true,
  imports: [FormField],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './geometry-tab.component.html',
  styleUrl: './geometry-tab.component.css',
})
export class GeometryTabComponent {
  readonly meshService = inject(MeshService);

  readonly selectedPreset = model.required<PresetKey>();

  readonly beamLength = model.required<number>();
  readonly beamHeight = model.required<number>();
  readonly beamNx = model.required<number>();
  readonly beamNy = model.required<number>();
  readonly beamUseTriangles = model.required<boolean>();

  readonly plateWidth = model.required<number>();
  readonly plateHeight = model.required<number>();
  readonly plateRadius = model.required<number>();
  readonly plateRadialDivs = model.required<number>();
  readonly plateAngularDivs = model.required<number>();

  readonly beam3dLength = model.required<number>();
  readonly beam3dHeight = model.required<number>();
  readonly beam3dDepth = model.required<number>();
  readonly beam3dNx = model.required<number>();
  readonly beam3dNy = model.required<number>();
  readonly beam3dNz = model.required<number>();
  readonly beam3dUseTets = model.required<boolean>();

  readonly presetChange = output<PresetKey>();
  readonly generateMesh = output<void>();
  readonly openSketcher = output<void>();

  // 1. Signal Form for 2D Beam Preset with Schema Validation
  readonly beamModel = signal<BeamFormModel>({
    length: 6.0,
    height: 2.0,
    nx: 20,
    ny: 10,
    useTriangles: false,
  });

  readonly beamForm = form(this.beamModel, (schema) => {
    min(schema.length, 0.1, { message: 'Length must be > 0' });
    min(schema.height, 0.1, { message: 'Height must be > 0' });
    min(schema.nx, 2, { message: 'Min 2 divisions' });
    max(schema.nx, 100, { message: 'Max 100 divisions' });
    min(schema.ny, 1, { message: 'Min 1 division' });
    max(schema.ny, 50, { message: 'Max 50 divisions' });
  });

  // 2. Signal Form for 2D Plate with Hole Preset
  readonly plateModel = signal<PlateFormModel>({
    width: 10.0,
    height: 10.0,
    radius: 1.5,
    radialDivs: 8,
    angularDivs: 16,
  });

  readonly plateForm = form(this.plateModel, (schema) => {
    min(schema.width, 0.1, { message: 'Width must be > 0' });
    min(schema.height, 0.1, { message: 'Height must be > 0' });
    min(schema.radius, 0.01, { message: 'Radius must be > 0' });
    min(schema.radialDivs, 2, { message: 'Min 2 radial divisions' });
    min(schema.angularDivs, 4, { message: 'Min 4 angular divisions' });
  });

  // 3. Signal Form for 3D Beam Preset
  readonly beam3dModel = signal<Beam3DFormModel>({
    length: 6.0,
    height: 2.0,
    depth: 2.0,
    nx: 12,
    ny: 4,
    nz: 4,
    useTets: false,
  });

  readonly beam3dForm = form(this.beam3dModel, (schema) => {
    min(schema.length, 0.1, { message: 'Length must be > 0' });
    min(schema.height, 0.1, { message: 'Height must be > 0' });
    min(schema.depth, 0.1, { message: 'Depth must be > 0' });
    min(schema.nx, 2, { message: 'Min 2 divisions' });
    min(schema.ny, 1, { message: 'Min 1 division' });
    min(schema.nz, 1, { message: 'Min 1 division' });
  });

  constructor() {
    // Synchronize Signal Forms -> Parent Component Inputs
    effect(() => {
      const b = this.beamModel();
      this.beamLength.set(b.length);
      this.beamHeight.set(b.height);
      this.beamNx.set(b.nx);
      this.beamNy.set(b.ny);
      this.beamUseTriangles.set(b.useTriangles);
    });

    effect(() => {
      const p = this.plateModel();
      this.plateWidth.set(p.width);
      this.plateHeight.set(p.height);
      this.plateRadius.set(p.radius);
      this.plateRadialDivs.set(p.radialDivs);
      this.plateAngularDivs.set(p.angularDivs);
    });

    effect(() => {
      const b3 = this.beam3dModel();
      this.beam3dLength.set(b3.length);
      this.beam3dHeight.set(b3.height);
      this.beam3dDepth.set(b3.depth);
      this.beam3dNx.set(b3.nx);
      this.beam3dNy.set(b3.ny);
      this.beam3dNz.set(b3.nz);
      this.beam3dUseTets.set(b3.useTets);
    });
  }

  onPresetChange(event: Event): void {
    const val = (event.target as HTMLSelectElement).value as PresetKey;
    this.selectedPreset.set(val);
    this.presetChange.emit(val);
  }

  generateCurrentMesh(): void {
    this.generateMesh.emit();
  }
}
