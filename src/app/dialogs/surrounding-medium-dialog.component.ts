import { ChangeDetectionStrategy, Component, inject, signal } from '@angular/core';
import { injectDialogRef, injectDialogData } from '../shared/dialog.service';
import { NotificationService } from '../shared/notification.service';

export interface SurroundingMediumConfig {
  enabled: boolean;
  boundaryTag: string;
  eMediumMpa: number;
  nuMedium: number;
  rhoMedium: number;
}

@Component({
  selector: 'app-surrounding-medium-dialog',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="dialog-content">
      <div class="form-row">
        <label class="styled-checkbox-label">
          <input
            type="checkbox"
            [checked]="enabled()"
            (change)="enabled.set($any($event.target).checked)"
          />
          <span class="label-bold">Enable Radiation Impedance Z(s) on Interface</span>
        </label>
      </div>

      @if (enabled()) {
        <div class="form-group">
          <label>Radiating Contact Boundary</label>
          <select
            class="styled-select"
            [value]="boundaryTag()"
            (change)="boundaryTag.set($any($event.target).value)"
          >
            <option value="Bottom">Bottom Boundary (Soil Foundation Base)</option>
            <option value="Right">Right Boundary (Far-Field Radiation)</option>
            <option value="Top">Top Boundary</option>
            <option value="Hole">Cavity / Tunnel Wall</option>
          </select>
          <span class="field-hint">Robin condition t + Z(s)u = g applies ONLY to this interface. Natural boundaries remain traction-free in air.</span>
        </div>

        <div class="form-group">
          <label>Medium Preset</label>
          <div class="preset-buttons">
            <button
              type="button"
              class="btn-chip"
              [class.active]="preset() === 'soft'"
              (click)="applyPreset('soft')"
            >
              Soft Soil (50 MPa)
            </button>
            <button
              type="button"
              class="btn-chip"
              [class.active]="preset() === 'stiff'"
              (click)="applyPreset('stiff')"
            >
              Stiff Soil (150 MPa)
            </button>
            <button
              type="button"
              class="btn-chip"
              [class.active]="preset() === 'rock'"
              (click)="applyPreset('rock')"
            >
              Bedrock (20 GPa)
            </button>
          </div>
        </div>

        <div class="grid-3">
          <div class="form-group">
            <label>E_ext (MPa)</label>
            <input
              type="number"
              class="styled-input"
              [value]="eMediumMpa()"
              (input)="eMediumMpa.set(+$any($event.target).value); preset.set('custom')"
            />
          </div>
          <div class="form-group">
            <label>&nu;_ext</label>
            <input
              type="number"
              step="0.05"
              min="0.0"
              max="0.49"
              class="styled-input"
              [value]="nuMedium()"
              (input)="nuMedium.set(+$any($event.target).value); preset.set('custom')"
            />
          </div>
          <div class="form-group">
            <label>&rho;_ext (kg/m³)</label>
            <input
              type="number"
              class="styled-input"
              [value]="rhoMedium()"
              (input)="rhoMedium.set(+$any($event.target).value); preset.set('custom')"
            />
          </div>
        </div>
      }

      <div class="dialog-actions">
        <button type="button" class="btn btn-secondary" (click)="dismiss()">
          Cancel
        </button>
        <button type="button" class="btn btn-primary" (click)="save()">
          Apply Configuration
        </button>
      </div>
    </div>
  `,
  styles: [`
    .dialog-content {
      display: flex;
      flex-direction: column;
      gap: 1.25rem;
    }
    .form-row {
      display: flex;
      align-items: center;
      gap: 0.75rem;
    }
    .label-bold {
      font-weight: 600;
      color: #f1f5f9;
    }
    .form-group {
      display: flex;
      flex-direction: column;
      gap: 0.35rem;
    }
    .form-group label {
      font-size: 0.8125rem;
      color: #94a3b8;
      font-weight: 500;
    }
    .field-hint {
      font-size: 0.75rem;
      color: #64748b;
      line-height: 1.4;
      margin-top: 0.2rem;
    }
    .preset-buttons {
      display: flex;
      gap: 0.5rem;
    }
    .btn-chip {
      background: #1e293b;
      border: 1px solid #334155;
      color: #cbd5e1;
      padding: 0.35rem 0.75rem;
      border-radius: 0.5rem;
      font-size: 0.75rem;
      cursor: pointer;
      transition: all 150ms ease;
    }
    .btn-chip:hover {
      background: #334155;
      color: #fff;
    }
    .btn-chip.active {
      background: #0284c7;
      border-color: #38bdf8;
      color: #fff;
      font-weight: 600;
    }
    .grid-3 {
      display: grid;
      grid-template-columns: repeat(3, 1fr);
      gap: 0.75rem;
    }
    .styled-input, .styled-select {
      background: #1e293b;
      border: 1px solid #334155;
      border-radius: 0.375rem;
      color: #f8fafc;
      padding: 0.5rem 0.75rem;
      font-size: 0.875rem;
      width: 100%;
      box-sizing: border-box;
    }
    .styled-input:focus, .styled-select:focus {
      outline: none;
      border-color: #38bdf8;
    }
    .dialog-actions {
      display: flex;
      justify-content: flex-end;
      gap: 0.75rem;
      margin-top: 1rem;
      padding-top: 1rem;
      border-top: 1px solid #334155;
    }
    .btn {
      padding: 0.6rem 1.25rem;
      border-radius: 0.5rem;
      font-size: 0.875rem;
      font-weight: 500;
      cursor: pointer;
      border: 0;
      transition: all 150ms ease;
    }
    .btn-secondary {
      background: #334155;
      color: #f1f5f9;
    }
    .btn-secondary:hover {
      background: #475569;
    }
    .btn-primary {
      background: #0284c7;
      color: #fff;
    }
    .btn-primary:hover {
      background: #0369a1;
    }
  `],
})
export class SurroundingMediumDialogComponent {
  private readonly dialogRef = injectDialogRef<SurroundingMediumConfig>();
  private readonly data = injectDialogData<SurroundingMediumConfig>();
  private readonly notifications = inject(NotificationService);

  readonly enabled = signal<boolean>(this.data?.enabled ?? false);
  readonly boundaryTag = signal<string>(this.data?.boundaryTag ?? 'Bottom');
  readonly eMediumMpa = signal<number>(this.data?.eMediumMpa ?? 100.0);
  readonly nuMedium = signal<number>(this.data?.nuMedium ?? 0.35);
  readonly rhoMedium = signal<number>(this.data?.rhoMedium ?? 2000.0);
  readonly preset = signal<'soft' | 'stiff' | 'rock' | 'custom'>('stiff');

  applyPreset(presetKey: 'soft' | 'stiff' | 'rock'): void {
    this.preset.set(presetKey);
    if (presetKey === 'soft') {
      this.eMediumMpa.set(50.0);
      this.nuMedium.set(0.35);
      this.rhoMedium.set(1800.0);
    } else if (presetKey === 'stiff') {
      this.eMediumMpa.set(150.0);
      this.nuMedium.set(0.30);
      this.rhoMedium.set(2000.0);
    } else if (presetKey === 'rock') {
      this.eMediumMpa.set(20000.0);
      this.nuMedium.set(0.25);
      this.rhoMedium.set(2600.0);
    }
  }

  dismiss(): void {
    this.dialogRef.dismiss();
  }

  save(): void {
    const config: SurroundingMediumConfig = {
      enabled: this.enabled(),
      boundaryTag: this.boundaryTag(),
      eMediumMpa: this.eMediumMpa(),
      nuMedium: this.nuMedium(),
      rhoMedium: this.rhoMedium(),
    };
    this.dialogRef.close(config);
    if (config.enabled) {
      this.notifications.show(`Soil / Medium interaction active on [${config.boundaryTag}] (E = ${config.eMediumMpa} MPa)`, 'info');
    } else {
      this.notifications.show('Radiation boundary disabled (Free surface in vacuo)', 'info');
    }
  }
}
