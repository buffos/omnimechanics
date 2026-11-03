import '@angular/compiler';
import { describe, it, expect, beforeEach } from 'vitest';
import { ModelerStore } from '../core/modeler.store';
import { ModelerResultsService } from '../results/modeler-results.service';
import { CombinationManagerComponent } from './combination-manager.component';

describe('Issue 025: Multi-LoadCase, Eurocode Combinations, Envelopes & Async Jobs', () => {
  let store: ModelerStore;
  let resultsService: ModelerResultsService;

  beforeEach(() => {
    store = new ModelerStore();
    resultsService = new ModelerResultsService(store);
  });

  describe('SC-010: Multi-LoadCase Management & Eurocode Combinations', () => {
    it('creates multiple load cases, switches active case, and deletes case safely', () => {
      expect(store.model().load_cases.length).toBe(1);
      expect(store.activeLoadCaseId()).toBe('LC1');

      // Add Q1 Live Load
      store.addLoadCase('Q1 (Live)', { kind: 'Variable', sub_category: 'A' });
      expect(store.model().load_cases.length).toBe(2);
      expect(store.activeLoadCaseId()).toBe('LC2');

      // Add W1 Wind Load
      store.addLoadCase('W1 (Wind)', { kind: 'Variable', sub_category: 'Wind' });
      expect(store.model().load_cases.length).toBe(3);
      expect(store.activeLoadCaseId()).toBe('LC3');

      // Delete LC2
      store.deleteLoadCase('LC2');
      expect(store.model().load_cases.length).toBe(2);
      expect(store.model().load_cases.some(c => c.id === 'LC2')).toBe(false);

      // Cannot delete last remaining loadcase
      store.deleteLoadCase('LC3');
      expect(store.model().load_cases.length).toBe(1);
      store.deleteLoadCase(store.model().load_cases[0].id);
      expect(store.model().load_cases.length).toBe(1);
    });

    it('generates ULS_STR, SLS_CHAR and SEISMIC combinations with Greek National Annex defaults', async () => {
      store.addLoadCase('Q1 (Imposed)', { kind: 'Variable', sub_category: 'A' });
      const comp = new CombinationManagerComponent(store);

      await comp.generateEurocodeCombinations();
      const combs = store.combinations();
      expect(combs.length).toBeGreaterThanOrEqual(3);

      const uls = combs.find(c => c.limit_state === 'ULS_STR');
      expect(uls).toBeDefined();
      expect(uls!.factors.find(f => f.load_case_id === 'LC1')?.factor).toBe(1.35);
      expect(uls!.factors.find(f => f.load_case_id === 'LC2')?.factor).toBe(1.50);

      const sls = combs.find(c => c.limit_state === 'SLS_CHAR');
      expect(sls).toBeDefined();
      expect(sls!.factors.find(f => f.load_case_id === 'LC1')?.factor).toBe(1.0);

      const seismic = combs.find(c => c.limit_state === 'SEISMIC');
      expect(seismic).toBeDefined();
      expect(seismic!.factors.find(f => f.load_case_id === 'LC1')?.factor).toBe(1.0);

      // Deleting a combination (e.g. SEISMIC_1) via UI or store
      comp.deleteCombination('SEISMIC_1');
      expect(store.combinations().some(c => c.id === 'SEISMIC_1')).toBe(false);
      expect(store.combinations().length).toBe(combs.length - 1);
    });

    it('computes EN 1998-1 Type 1 design spectrum curve Sd(T) and updates preview points', async () => {
      const comp = new CombinationManagerComponent(store);

      await comp.refreshSpectrum();
      const pts = comp.spectrumPoints();
      expect(pts.length).toBeGreaterThan(10);
      expect(pts[0].period_s).toBe(0);
      expect(pts[0].sd_g).toBeGreaterThan(0);

      const svgPath = comp.spectrumSvgPath();
      expect(svgPath).toContain('M');
      expect(svgPath).toContain('L');
    });

    it('probes governing envelope at stations with governing combination readout', async () => {
      store.sketchMember({ x: 0, y: 0 }, { x: 6, y: 0 });
      const n1 = store.model().nodes[0];
      const n2 = store.model().nodes[1];
      store.assignSupport(n1.id, { type: 'Pinned' });
      store.assignSupport(n2.id, { type: 'Roller', angle_deg: 0 });

      // Run live solve to generate envelope
      await (resultsService as any).runLiveSolve(store.model());

      // Select Envelope mode
      resultsService.setResultTarget('Envelope', 'ENV_ULS_STR');
      expect(resultsService.resultTarget()).toBe('Envelope');

      const elem = store.model().elements[0];
      resultsService.probeCrossSection(elem.id, 3.0); // Midspan
      const probe = resultsService.probePoint();

      expect(probe).toBeDefined();
      expect(probe?.element_id).toBe(elem.id);
      expect(probe?.x_m).toBe(3.0);
      expect(probe?.governing_combination).toBeDefined();
      expect(probe?.governing_combination).toContain('G');
    });
  });

  describe('SC-008: Non-Superposition Notice & Background Job Cancellation', () => {
    it('flags non-superposition when P-Delta or nonlinear elements (cable, unilateral) are active', () => {
      // Linear model without nonlinear elements
      store.sketchMember({ x: 0, y: 0 }, { x: 6, y: 0 });
      (resultsService as any).store = store;
      resultsService.setAnalysisKind('LinearStatic');
      expect(resultsService.isNonlinearSuperpositionInvalid()).toBe(false);

      // 1. Enabling SecondOrderPDelta triggers notice
      resultsService.setAnalysisKind('SecondOrderPDelta');
      expect(resultsService.isNonlinearSuperpositionInvalid()).toBe(true);

      // 2. Switch back to LinearStatic, add Cable element
      resultsService.setAnalysisKind('LinearStatic');
      const elem = store.model().elements[0];
      store.updateElementProperties(elem.id, {
        kind: 'Cable',
        prestress: { mode: 'Pretension', t0: 50.0 },
      } as any);
      expect(resultsService.isNonlinearSuperpositionInvalid()).toBe(true);

      // 3. Unilateral support triggers notice
      const n1 = store.model().nodes[0];
      store.assignSupport(n1.id, { type: 'Roller', angle_deg: 0 }, true);
      expect(resultsService.isNonlinearSuperpositionInvalid()).toBe(true);
    });

    it('cancels active job session on rapid model mutation', () => {
      (resultsService as any).activeJobId.set('job_test_123');
      (resultsService as any).isBackgroundJobRunning.set(true);

      expect(resultsService.isBackgroundJobRunning()).toBe(true);

      // Mutating model triggers live solve schedule which cancels previous job
      store.sketchMember({ x: 0, y: 0 }, { x: 0, y: 4 });
      (resultsService as any).scheduleLiveSolve(store.model());

      expect(resultsService.activeJobId()).toBeNull();
      expect(resultsService.isBackgroundJobRunning()).toBe(false);
    });
  });
});
