import { Injectable, computed, inject, signal } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import {
  TeachingInputConfig,
  TeachingSolutionDto,
  PedagogicalStep,
} from './teaching.types';
import { I18nService } from '../../shared/i18n.service';

export interface PresetMeta {
  id: string;
  label_el: string;
  label_en: string;
  category: 'beam' | 'truss' | 'continuous_beam';
  defaultLength: number;
  defaultHeight?: number;
  defaultQ?: number;
  defaultP?: number;
  defaultPPos?: number;
  desc_el: string;
  desc_en: string;
}

export const TEACHING_PRESETS: PresetMeta[] = [
  {
    id: 'simply_supported_distributed',
    label_el: 'Αμφιέρειστη Δοκός με Ομοιόμορφο Φορτίο q',
    label_en: 'Simply Supported Beam with Uniform Load q',
    category: 'beam',
    defaultLength: 6.0,
    defaultQ: 12.0,
    desc_el: 'Υπολογισμός αντιδράσεων, διαγράμματα Q(x) & M(x), εύρεση θέσης μηδενισμού τέμνουσας και μέγιστης ροπής M_max = qL²/8.',
    desc_en: 'Support reactions, Q(x) & M(x) diagrams, zero-shear station detection and maximum bending moment M_max = qL²/8.',
  },
  {
    id: 'simply_supported_point',
    label_el: 'Αμφιέρειστη Δοκός με Συγκεντρωμένο Φορτίο P',
    label_en: 'Simply Supported Beam with Point Load P',
    category: 'beam',
    defaultLength: 8.0,
    defaultP: 40.0,
    defaultPPos: 4.0,
    desc_el: 'Αμφιέρειστη δοκός με φορτίο στο μέσο του ανοίγματος. Ασυμμετρία/συμμετρία και μέγιστη ροπή M_max = PL/4.',
    desc_en: 'Simply supported beam with center point load. Symmetry and peak bending moment M_max = PL/4.',
  },
  {
    id: 'cantilever',
    label_el: 'Πρόβολος (Πάκτωση) με Φορτία P & q',
    label_en: 'Cantilever Beam with Loads P & q',
    category: 'beam',
    defaultLength: 4.0,
    defaultP: 15.0,
    defaultPPos: 4.0,
    defaultQ: 5.0,
    desc_el: 'Πλήρης πάκτωση με κατακόρυφη δύναμη και κατανεμημένο φορτίο. Ροπή πάκτωσης M_A και διαγράμματα.',
    desc_en: 'Fixed cantilever beam under tip vertical force and distributed load. Clamping moment M_A and diagrams.',
  },
  {
    id: 'overhanging_beam',
    label_el: 'Δοκός με Πρόβολο (Overhanging Beam)',
    label_en: 'Beam with Cantilever Overhang',
    category: 'beam',
    defaultLength: 6.0,
    defaultHeight: 2.0,
    defaultQ: 10.0,
    defaultP: 20.0,
    defaultPPos: 8.0,
    desc_el: 'Κύριο άνοιγμα με υπερκρέμαση (πρόβολο). Αρνητική ροπή στη στήριξη και σημεία μηδενισμού (σημεία καμπής).',
    desc_en: 'Main span with cantilever overhang. Negative support moment and moment inflection points.',
  },
  {
    id: 'pratt_truss',
    label_el: 'Δικτύωμα Pratt (Μέθοδος Κόμβων & Ritter)',
    label_en: 'Pratt Truss (Method of Joints & Ritter)',
    category: 'truss',
    defaultLength: 12.0,
    defaultHeight: 3.0,
    defaultP: 20.0,
    desc_el: 'Ισοστατικό δικτύωμα 4 φατνωμάτων. Βήμα-προς-βήμα ανάλυση κόμβων (Method of Joints) και τομών Ritter.',
    desc_en: 'Isostatic 4-panel plane truss. Step-by-step Method of Joints and Ritter cut section equilibrium.',
  },
  {
    id: 'cross_two_span',
    label_el: 'Συνεχής Δοκός 2 Ανοιγμάτων (Μέθοδος Cross)',
    label_en: 'Continuous 2-Span Beam (Cross Method)',
    category: 'continuous_beam',
    defaultLength: 6.0,
    defaultHeight: 4.0,
    defaultQ: 15.0,
    defaultP: 30.0,
    desc_el: 'Υπερστατική συνεχής δοκός. Υπολογισμός δυσκαμψίας k, συντελεστών κατανομής DF, FEM και διαδοχικοί κύκλοι Cross.',
    desc_en: 'Hyperstatic continuous beam. Member stiffness k, distribution factors DF, FEM and iterative Cross cycles.',
  },
];

@Injectable({
  providedIn: 'root',
})
export class TeachingService {
  private readonly i18n = inject(I18nService);

  readonly solution = signal<TeachingSolutionDto | null>(null);
  readonly currentStepIndex = signal<number>(0);
  readonly isPlaying = signal<boolean>(false);
  readonly activeDiagramType = signal<'M' | 'Q' | 'N'>('M');
  readonly isLoading = signal<boolean>(false);
  readonly errorMessage = signal<string | null>(null);

  private playIntervalId: any = null;

  readonly currentStep = computed<PedagogicalStep | null>(() => {
    const sol = this.solution();
    if (!sol || sol.steps.length === 0) return null;
    const idx = this.currentStepIndex();
    return sol.steps[Math.min(idx, sol.steps.length - 1)] ?? null;
  });

  readonly totalSteps = computed<number>(() => {
    return this.solution()?.steps.length ?? 0;
  });

  async solvePreset(config: TeachingInputConfig): Promise<void> {
    this.isLoading.set(true);
    this.errorMessage.set(null);
    this.stopPlay();
    try {
      const payload: TeachingInputConfig = {
        ...config,
        language: config.language ?? this.i18n.currentLang(),
      };
      const res = await invoke<TeachingSolutionDto>('solve_teaching_system', { config: payload });
      this.solution.set(res);
      this.currentStepIndex.set(0);
      if (res.system_type === 'truss') {
        this.activeDiagramType.set('N');
      } else {
        this.activeDiagramType.set('M');
      }
    } catch (err: any) {
      console.error('Failed to solve teaching preset:', err);
      this.errorMessage.set(typeof err === 'string' ? err : err?.message || 'Calculation error');
    } finally {
      this.isLoading.set(false);
    }
  }

  nextStep(): void {
    const total = this.totalSteps();
    if (total === 0) return;
    this.currentStepIndex.update((curr) => {
      if (curr < total - 1) {
        return curr + 1;
      }
      this.stopPlay();
      return curr;
    });
  }

  prevStep(): void {
    this.stopPlay();
    this.currentStepIndex.update((curr) => Math.max(0, curr - 1));
  }

  firstStep(): void {
    this.stopPlay();
    this.currentStepIndex.set(0);
  }

  lastStep(): void {
    this.stopPlay();
    const total = this.totalSteps();
    if (total > 0) {
      this.currentStepIndex.set(total - 1);
    }
  }

  togglePlay(): void {
    if (this.isPlaying()) {
      this.stopPlay();
    } else {
      this.startPlay();
    }
  }

  private startPlay(): void {
    const total = this.totalSteps();
    if (total === 0) return;
    if (this.currentStepIndex() >= total - 1) {
      this.currentStepIndex.set(0);
    }
    this.isPlaying.set(true);
    this.playIntervalId = setInterval(() => {
      this.nextStep();
    }, 2500);
  }

  private stopPlay(): void {
    this.isPlaying.set(false);
    if (this.playIntervalId) {
      clearInterval(this.playIntervalId);
      this.playIntervalId = null;
    }
  }

  setActiveDiagramType(type: 'M' | 'Q' | 'N'): void {
    this.activeDiagramType.set(type);
  }
}
