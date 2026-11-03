import { Component, ChangeDetectionStrategy, inject, signal, computed, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ModelerStore } from '../core/modeler.store';
import {
  CombinationDto,
  SeismicSettingsDto,
  DesignSpectrumPointDto,
} from './combinations.types';

@Component({
  selector: 'app-combination-manager',
  standalone: true,
  imports: [CommonModule, FormsModule],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './combination-manager.component.html',
  styleUrl: './combination-manager.component.css',
})
export class CombinationManagerComponent implements OnInit {
  readonly store: ModelerStore;

  isOpen = signal<boolean>(false);
  activeTab = signal<'loadCases' | 'combinations' | 'seismic'>('combinations');

  // Load Case form
  newLcName = signal<string>('Q1 (Live Load)');
  newLcCategory = signal<string>('Variable');

  // Seismic settings
  seismicSettings: any;
  spectrumPoints = signal<DesignSpectrumPointDto[]>([]);

  readonly combinations: any;
  readonly loadCases: any;

  constructor(store?: ModelerStore) {
    this.store = store ?? inject(ModelerStore, { optional: true }) ?? new ModelerStore();
    this.isOpen = this.store.isCombinationModalOpen;
    this.seismicSettings = this.store.seismicSettings;
    this.combinations = this.store.combinations;
    this.loadCases = computed(() => this.store.model().load_cases);
  }

  ngOnInit(): void {
    this.refreshSpectrum();
  }

  open(): void {
    this.store.isCombinationModalOpen.set(true);
    this.refreshSpectrum();
  }

  close(): void {
    this.store.isCombinationModalOpen.set(false);
  }

  private backdropMouseDown = false;

  onBackdropMouseDown(event: MouseEvent): void {
    this.backdropMouseDown = event.target === event.currentTarget;
  }

  onBackdropMouseUp(event: MouseEvent): void {
    if (this.backdropMouseDown && event.target === event.currentTarget) {
      this.close();
    }
    this.backdropMouseDown = false;
  }

  addNewLoadCase(): void {
    const cat = this.newLcCategory();
    let actionType: any = { kind: 'Permanent' };
    if (cat === 'Variable') {
      actionType = { kind: 'Variable', sub_category: 'A' };
    } else if (cat === 'Seismic') {
      actionType = { kind: 'Seismic' };
    }
    this.store.addLoadCase(this.newLcName(), actionType);
  }

  deleteCombination(id: string): void {
    this.store.deleteCombination(id);
  }

  async generateEurocodeCombinations(): Promise<void> {
    const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
    const lcs = this.loadCases();

    if (isTauri) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const resp = await invoke<any>('actions_generate_combinations', {
          request: {
            profile: null,
            load_cases: lcs.map((lc: any) => ({
              id: lc.id,
              name: lc.name,
              category: (lc.action_type as any).kind || 'Permanent',
              sub_category: (lc.action_type as any).sub_category,
            })),
            approach: 'Single6_10',
            include_sls: true,
            include_seismic: true,
            seismic_settings: this.seismicSettings(),
            existing_combinations: this.combinations(),
          },
        });
        if (resp && resp.combinations) {
          this.store.setCombinations(resp.combinations);
          return;
        }
      } catch (e) {
        console.warn('Tauri generate combinations failed, using fallback', e);
      }
    }

    // Fallback combination generator
    const fallbackCombs: CombinationDto[] = [];
    const gCase = lcs.find((c: any) => (c.action_type as any).kind === 'Permanent') || lcs[0];
    const qCases = lcs.filter((c: any) => c.id !== gCase.id);

    // ULS 1: 1.35G + 1.50Q1
    if (qCases.length > 0) {
      fallbackCombs.push({
        id: 'ULS_STR_1',
        name: '1.35G + 1.50' + qCases[0].name,
        limit_state: 'ULS_STR',
        provenance: 'Generated',
        factors: [
          { load_case_id: gCase.id, factor: 1.35 },
          { load_case_id: qCases[0].id, factor: 1.5 },
        ],
      });
    } else {
      fallbackCombs.push({
        id: 'ULS_STR_1',
        name: '1.35G',
        limit_state: 'ULS_STR',
        provenance: 'Generated',
        factors: [{ load_case_id: gCase.id, factor: 1.35 }],
      });
    }

    // SLS CHAR: 1.0G + 1.0Q1
    if (qCases.length > 0) {
      fallbackCombs.push({
        id: 'SLS_CHAR_1',
        name: '1.00G + 1.00' + qCases[0].name,
        limit_state: 'SLS_CHAR',
        provenance: 'Generated',
        factors: [
          { load_case_id: gCase.id, factor: 1.0 },
          { load_case_id: qCases[0].id, factor: 1.0 },
        ],
      });
    }

    // SEISMIC: 1.0G + 0.3Q + 1.0Ex
    fallbackCombs.push({
      id: 'SEISMIC_1',
      name: '1.00G + 0.30Q + 1.00Ex',
      limit_state: 'SEISMIC',
      provenance: 'Generated',
      factors: [
        { load_case_id: gCase.id, factor: 1.0 },
        ...(qCases[0] ? [{ load_case_id: qCases[0].id, factor: 0.3 }] : []),
      ],
    });

    this.store.setCombinations(fallbackCombs);
  }

  async refreshSpectrum(): Promise<void> {
    const s = this.seismicSettings();
    const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

    if (isTauri) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const resp = await invoke<any>('actions_compute_design_spectrum', {
          request: {
            settings: s,
            t_max: 3.0,
            num_points: 60,
          },
        });
        if (resp && resp.points) {
          this.spectrumPoints.set(resp.points);
          return;
        }
      } catch (e) {
        console.warn('Spectrum IPC error, using analytical formula', e);
      }
    }

    // Analytical fallback EN 1998-1 Type 1 spectrum
    const points: DesignSpectrumPointDto[] = [];
    const ag = s.ag_g;
    const S = s.ground_type === 'A' ? 1.0 : s.ground_type === 'B' ? 1.2 : 1.15;
    const Tb = 0.15;
    const Tc = 0.50;
    const Td = 2.0;
    const q = s.behavior_factor_q;

    for (let k = 0; k <= 60; k++) {
      const T = (k / 60) * 3.0;
      let sd = 0;
      if (T <= Tb) {
        sd = ag * S * (2 / 3 + (T / Tb) * (2.5 / q - 2 / 3));
      } else if (T <= Tc) {
        sd = (ag * S * 2.5) / q;
      } else if (T <= Td) {
        sd = ((ag * S * 2.5) / q) * (Tc / T);
      } else {
        sd = ((ag * S * 2.5) / q) * ((Tc * Td) / (T * T));
      }
      points.push({ period_s: T, sd_g: sd, se_g: sd * q });
    }
    this.spectrumPoints.set(points);
  }

  updateSeismicParam(key: keyof SeismicSettingsDto, val: any): void {
    this.store.seismicSettings.update(cur => {
      const next = { ...cur, [key]: val };
      if (key === 'zone') {
        if (val.includes('0.16')) next.ag_g = 0.16;
        else if (val.includes('0.24')) next.ag_g = 0.24;
        else if (val.includes('0.36')) next.ag_g = 0.36;
      }
      return next;
    });
    this.refreshSpectrum();
  }

  readonly spectrumSvgPath = computed(() => {
    const pts = this.spectrumPoints();
    if (pts.length < 2) return '';
    const width = 280;
    const height = 120;
    const maxT = 3.0;
    let maxSd = 0.01;
    for (const p of pts) {
      if (p.sd_g > maxSd) maxSd = p.sd_g;
    }

    let d = '';
    for (let i = 0; i < pts.length; i++) {
      const x = (pts[i].period_s / maxT) * width;
      const y = height - (pts[i].sd_g / maxSd) * (height - 15);
      d += i === 0 ? `M ${x} ${y} ` : `L ${x} ${y} `;
    }
    return d;
  });
}
