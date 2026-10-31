import { ChangeDetectionStrategy, Component, computed, effect, inject, signal, untracked } from '@angular/core';
import { CommonModule } from '@angular/common';
import { form, min, max } from '@angular/forms/signals';
import { TeachingService, TEACHING_PRESETS, PresetMeta } from '../teaching.service';
import { I18nService } from '../../../shared/i18n.service';

interface TeachingFormModel {
  presetId: string;
  length: number;
  height: number;
  loadQ: number;
  loadP: number;
  loadPPos: number;
}

@Component({
  selector: 'app-teaching-sidebar',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './teaching-sidebar.component.html',
  styleUrls: ['./teaching-sidebar.component.css'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class TeachingSidebarComponent {
  readonly teaching = inject(TeachingService);
  readonly i18n = inject(I18nService);
  readonly presets = TEACHING_PRESETS;

  readonly selectedPreset = signal<PresetMeta>(TEACHING_PRESETS[0]);

  // Angular 22 Signal Form
  readonly teachingModel = signal<TeachingFormModel>({
    presetId: TEACHING_PRESETS[0].id,
    length: TEACHING_PRESETS[0].defaultLength,
    height: TEACHING_PRESETS[0].defaultHeight ?? 3.0,
    loadQ: TEACHING_PRESETS[0].defaultQ ?? 12.0,
    loadP: TEACHING_PRESETS[0].defaultP ?? 40.0,
    loadPPos: TEACHING_PRESETS[0].defaultPPos ?? 3.0,
  });

  readonly totalBeamLength = computed(() => {
    const m = this.teachingModel();
    const p = this.selectedPreset();
    if (p.id === 'overhanging_beam') {
      return m.length + m.height;
    }
    return m.length;
  });

  readonly teachingForm = form(this.teachingModel, (schema) => {
    min(schema.length, 1.0);
    max(schema.length, 50.0);
    min(schema.height, 0.5);
    max(schema.height, 20.0);
    min(schema.loadPPos, 0.0);
    max(schema.loadPPos, 50.0);
  });

  constructor() {
    effect(() => {
      this.i18n.currentLang();
      untracked(() => {
        this.solveCurrent();
      });
    });
  }

  getPresetLabel(p: PresetMeta): string {
    return this.i18n.currentLang() === 'el' ? p.label_el : p.label_en;
  }

  getPresetDesc(p: PresetMeta): string {
    return this.i18n.currentLang() === 'el' ? p.desc_el : p.desc_en;
  }

  onPresetChange(newPresetId: string): void {
    const p = this.presets.find((x) => x.id === newPresetId);
    if (!p) return;
    this.selectedPreset.set(p);

    const defaultPos = p.defaultPPos ?? (p.id === 'cantilever' ? p.defaultLength : p.defaultLength / 2);

    this.teachingModel.update((m) => ({
      ...m,
      presetId: p.id,
      length: p.defaultLength,
      height: p.defaultHeight ?? m.height,
      loadQ: p.defaultQ ?? m.loadQ,
      loadP: p.defaultP ?? m.loadP,
      loadPPos: defaultPos,
    }));

    this.solveCurrent();
  }

  onLengthChange(val: string | number): void {
    const num = Math.max(1, Number(val));
    this.teachingModel.update((m) => {
      const maxLen = this.selectedPreset().id === 'overhanging_beam' ? num + m.height : num;
      return {
        ...m,
        length: num,
        loadPPos: Math.min(m.loadPPos, maxLen),
      };
    });
    this.solveCurrent();
  }

  onHeightChange(val: string | number): void {
    const num = Math.max(0.5, Number(val));
    this.teachingModel.update((m) => {
      const maxLen = this.selectedPreset().id === 'overhanging_beam' ? m.length + num : m.length;
      return {
        ...m,
        height: num,
        loadPPos: Math.min(m.loadPPos, maxLen),
      };
    });
    this.solveCurrent();
  }

  onLoadQChange(val: string | number): void {
    this.teachingModel.update((m) => ({ ...m, loadQ: Number(val) }));
    this.solveCurrent();
  }

  onLoadPChange(val: string | number): void {
    this.teachingModel.update((m) => ({ ...m, loadP: Number(val) }));
    this.solveCurrent();
  }

  onLoadPPosChange(val: string | number): void {
    const num = Number(val);
    const maxLen = this.totalBeamLength();
    const clamped = Math.min(Math.max(num, 0), maxLen);
    this.teachingModel.update((m) => ({ ...m, loadPPos: clamped }));
    this.solveCurrent();
  }

  solveCurrent(): void {
    const val = this.teachingModel();
    const maxLen = this.totalBeamLength();
    const clampedPos = Math.min(Math.max(val.loadPPos, 0), maxLen);
    if (clampedPos !== val.loadPPos) {
      this.teachingModel.update((m) => ({ ...m, loadPPos: clampedPos }));
    }
    this.teaching.solvePreset({
      preset_name: val.presetId,
      method: 'diagrams',
      length: val.length,
      height: val.height,
      load_q: val.loadQ,
      load_p: val.loadP,
      load_p_pos: clampedPos,
    });
  }
}
