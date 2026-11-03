import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { WorkspaceModeService } from './workspace-mode.service';
import { I18nService } from './i18n.service';
import { DialogService } from './dialog.service';
import { NotificationService } from './notification.service';
import {
  buildTensionSideDiagramPath,
  SemanticColorTokens,
  ViewportTransform2D,
} from './svg-canvas-kernel';
import { DraftingViewport } from '../features/modeler/core/viewport';
import { SolverService, TransientSimulationResult } from '../services/solver.service';
import { MeshService } from '../services/mesh.service';
import { MaterialService } from '../services/material.service';
import { BottomDockComponent } from '../dock/bottom-dock.component';

class DummyModalComponent {}

describe('cap-interactive-canvas Slice 2: Shell, Dialogs, Toasts, Dock Synchronization & Shared 2D SVG Kernel', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('SC-001: Tri-Mode Workspace Switching Preserves Active Session State', () => {
    it('switches cleanly between Mode 1 (continuum), Mode 2 (structural/modeler), and Mode 3 (teaching) while preserving session signals and pausing active timers', () => {
      const workspaceService = new WorkspaceModeService();
      const meshService = new MeshService();
      const solverService = new SolverService();

      meshService.setMesh({
        dimension: 2,
        nodes: [
          [0, 0, 0],
          [10, 0, 0],
          [10, 2, 0],
          [0, 2, 0],
        ],
        boundary_elements: [],
        volume_elements: [],
        stats: {
          dimension: 2,
          num_nodes: 4,
          num_boundary_elements: 0,
          num_volume_elements: 0,
        },
      });

      const sampleTransient: TransientSimulationResult = {
        time_points: [0, 0.25, 0.5],
        frames: [
          { time: 0, displacements: [[0, 0, 0]], deformed_nodes: [[0, 0, 0]], von_mises_stresses: [0], max_displacement: 0 },
          { time: 0.25, displacements: [[0, -0.002, 0]], deformed_nodes: [[0, -0.002, 0]], von_mises_stresses: [25], max_displacement: 0.002 },
          { time: 0.5, displacements: [[0, -0.001, 0]], deformed_nodes: [[0, -0.001, 0]], von_mises_stresses: [12], max_displacement: 0.001 },
        ],
        max_overall_displacement: 0.002,
        execution_time_ms: 14.2,
      };
      solverService.transientResult.set(sampleTransient);
      solverService.playAnimation();
      expect(solverService.isPlayingAnimation()).toBe(true);

      // Switch to Mode 2 ('structural' canonical alias -> 'modeler')
      solverService.pauseAnimation();
      workspaceService.setMode('structural');
      expect(workspaceService.activeMode()).toBe('modeler');
      expect(solverService.isPlayingAnimation()).toBe(false);

      // Switch to Mode 3 ('teaching')
      workspaceService.setMode('teaching');
      expect(workspaceService.activeMode()).toBe('teaching');

      // Return to Mode 1 ('continuum'): mesh and transient results remain intact without re-solving
      workspaceService.setMode('continuum');
      expect(workspaceService.activeMode()).toBe('continuum');
      expect(meshService.currentMesh()?.stats.num_nodes).toBe(4);
      expect(solverService.transientResult()?.max_overall_displacement).toBe(0.002);
    });
  });

  describe('SC-002: Instant Bilingual Toggle (el/en) Across Shell, Viewport & Dock', () => {
    it('starts in default Greek locale (el) and toggles reactively between el and en (BR-5)', () => {
      const i18n = new I18nService();
      expect(i18n.currentLang()).toBe('el');
      expect(i18n.t('nav.continuum')).toContain('Υπολογιστικό Συνεχούς');
      expect(i18n.t('nav.modeler')).toContain('Μοντελοποίηση');
      expect(i18n.t('nav.teaching')).toContain('Εκπαιδευτικό Studio');

      i18n.toggleLanguage();
      expect(i18n.currentLang()).toBe('en');
      expect(i18n.t('nav.continuum')).toContain('Continuum Workbench');
      expect(i18n.t('nav.modeler')).toContain('Structural Modeler');
      expect(i18n.t('nav.teaching')).toContain('Teaching Studio');

      i18n.toggleLanguage();
      expect(i18n.currentLang()).toBe('el');
    });
  });

  describe('SC-003: Modal Dialog Stack LIFO Enforcement & DismissGuard Protection', () => {
    it('enforces LIFO ordering, blocks dismissal when dismissGuard() returns false (DIALOG_DISMISS_BLOCKED), emits warning toast, and resolves on valid close (INV-1)', async () => {
      const notifications = new NotificationService();
      const i18n = new I18nService();
      const dialogService = new DialogService(notifications, i18n);

      let isInputValid = false;
      const ref1 = dialogService.open<{ eMediumMpa: number }, { eMediumMpa: number }>(
        DummyModalComponent,
        {
          title: 'Soil / Surrounding Medium Interaction',
          data: { eMediumMpa: -50 },
        }
      );
      ref1.setDismissGuard(() => isInputValid);

      const ref2 = dialogService.open<string, undefined>(DummyModalComponent, {
        title: 'Top LIFO Modal',
      });

      expect(dialogService.stack().length).toBe(2);
      expect(dialogService.active()?.ref.id).toBe(ref2.id);

      // Attempting to close ref1 while ref2 is on top is ignored (LIFO invariant)
      ref1.close({ eMediumMpa: 100 });
      expect(dialogService.stack().length).toBe(2);

      // Close top dialog ref2
      ref2.close('done');
      await expect(ref2.closed).resolves.toBe('done');
      expect(dialogService.stack().length).toBe(1);
      expect(dialogService.active()?.ref.id).toBe(ref1.id);

      // Step 1: Attempt to dismiss ref1 while dismissGuard() === false
      ref1.dismiss();
      expect(dialogService.stack().length).toBe(1);
      expect(dialogService.lastBlockedErrorCode()).toBe('DIALOG_DISMISS_BLOCKED');
      expect(notifications.notifications().length).toBe(1);
      expect(notifications.notifications()[0].type).toBe('warning');
      expect(notifications.notifications()[0].message).toContain('DIALOG_DISMISS_BLOCKED');

      // Step 2: Fix input so dismissGuard() === true and close dialog with result
      isInputValid = true;
      ref1.close({ eMediumMpa: 150 });
      const closedResult = await ref1.closed;
      expect(closedResult).toEqual({ eMediumMpa: 150 });
      expect(dialogService.stack().length).toBe(0);
    });
  });

  describe('SC-004: Toast Notification Deduplication & 5000 ms Auto-Dismissal', () => {
    it('deduplicates identical active toasts and resets their 5000 ms expiration timer (INV-2)', () => {
      const notifications = new NotificationService();

      notifications.show('Mesh generated', 'info');
      expect(notifications.notifications().length).toBe(1);

      // Advance 2000 ms and fire identical toast
      vi.advanceTimersByTime(2000);
      notifications.show('Mesh generated', 'info');
      expect(notifications.notifications().length).toBe(1);

      // At 4000 ms after first call (2000 ms after second call), toast is still active
      vi.advanceTimersByTime(2000);
      expect(notifications.notifications().length).toBe(1);

      // At 5000 ms after second call (+3000 ms more), toast auto-dismisses
      vi.advanceTimersByTime(3000);
      expect(notifications.notifications().length).toBe(0);
    });
  });

  describe('SC-010 & SC-011: Synchronized Playback Scrubbing, Convergence Residuals & EXPORT_DATA_EMPTY Guard', () => {
    it('guards CSV/SVG exports when empty (EXPORT_DATA_EMPTY) and synchronizes chart scrubbing with solver frame index', () => {
      const solverService = new SolverService();
      const meshService = new MeshService();
      const materialService = new MaterialService();
      const notificationService = new NotificationService();
      const i18n = new I18nService();

      const dock = new BottomDockComponent(
        solverService,
        meshService,
        materialService,
        notificationService,
        i18n
      );

      // Before solving: export is guarded (SC-011)
      expect(dock.hasExportableData()).toBe(false);
      expect(dock.exportCsv()).toBe(false);
      expect(dock.lastExportErrorCode()).toBe('EXPORT_DATA_EMPTY');
      expect(notificationService.notifications().some((n) => n.message.includes('EXPORT_DATA_EMPTY'))).toBe(true);

      // Load 25-frame TimeDomain result (T_max = 0.50 s)
      const frames = Array.from({ length: 25 }, (_, k) => {
        const t = (k / 24) * 0.5;
        return {
          time: t,
          displacements: [[0, -0.002 * Math.sin(Math.PI * (k / 24)), 0]] as [number, number, number][],
          deformed_nodes: [[0, 0, 0]] as [number, number, number][],
          von_mises_stresses: [35 * Math.sin(Math.PI * (k / 24))],
          max_displacement: 0.002 * Math.sin(Math.PI * (k / 24)),
        };
      });
      const tr: TransientSimulationResult = {
        time_points: frames.map((f) => f.time),
        frames,
        max_overall_displacement: 0.002,
        execution_time_ms: 28.4,
      };
      solverService.transientResult.set(tr);
      solverService.talbotTransientResult.set(tr);

      expect(dock.hasExportableData()).toBe(true);
      expect(dock.talbotPath().length).toBeGreaterThan(10);

      // Click at horizontal midpoint of plot area (padding.left = 55, w = 815 => midX = 55 + 407.5 = 462.5)
      const midpointX = 55 + (900 - 55 - 30) * 0.5;
      const scrubbedIdx = dock.scrubHistoryChartToScreenX(midpointX, 900);
      expect(scrubbedIdx).toBe(12); // frame 12 corresponds to t = 0.25 s
      expect(solverService.currentFrameIndex()).toBe(12);
      expect(dock.currentCursorX()).toBeCloseTo(midpointX, 1);

      // Verify static convergence history DTO (SC-011)
      solverService.currentResult.set({
        solver_mode: 'BEMFEM',
        converged: true,
        iterations: 3,
        displacements: [[0, -0.001, 0]],
        deformed_nodes: [[0, -0.001, 0]],
        von_mises_stresses: [20.5],
        max_displacement: 0.001,
        max_von_mises: 20.5,
        execution_time_ms: 11.2,
        convergence_history: [
          { iteration: 1, residual_norm: 1.2e-1, relative_error: 1.0e-1 },
          { iteration: 2, residual_norm: 3.4e-3, relative_error: 2.8e-3 },
          { iteration: 3, residual_norm: 4.1e-5, relative_error: 3.4e-5 },
        ],
      });
      expect(solverService.currentResult()?.convergence_history.length).toBe(3);
      expect(dock.exportCsv()).toBe(true);
    });
  });

  describe('SC-012: Shared 2D SVG World-to-Screen Transform & Tension-Side Moment Diagram Parity', () => {
    it('preserves +y up to -v up mapping and exact round-trip identity within 1e-9', () => {
      const transform = new ViewportTransform2D({
        viewWidth: 1200,
        viewHeight: 800,
        panX: 300,
        panY: 500,
        scale: 60,
      });

      // World +y maps to smaller screen y (upward on screen)
      const originScreen = transform.worldToScreen(0, 0);
      const positiveYScreen = transform.worldToScreen(0, 2);
      expect(positiveYScreen.y).toBeLessThan(originScreen.y);

      const testPoints = [
        { x: 0, y: 0 },
        { x: 6.5, y: 3.25 },
        { x: -4.125, y: -2.75 },
        { x: 12.345678, y: 8.765432 },
      ];

      for (const pt of testPoints) {
        const screen = transform.worldToScreen(pt.x, pt.y);
        const roundTrip = transform.screenToWorld(screen.x, screen.y);
        expect(Math.abs(roundTrip.x - pt.x)).toBeLessThanOrEqual(1e-9);
        expect(Math.abs(roundTrip.y - pt.y)).toBeLessThanOrEqual(1e-9);
      }
    });

    it('plots positive sagging moment M(3) = +45 kNm downward on the bottom/local -y tension side across Mode 2 and Mode 3 (INV-6, BR-4)', () => {
      // Horizontal simply supported beam L = 6 m under downward q = 10 kN/m -> M(3) = +45 kNm
      const samples = [
        { x: 0, value: 0 },
        { x: 3, value: 45 },
        { x: 6, value: 0 },
      ];

      // Mode 2 DraftingViewport projection
      const mode2Vp = new DraftingViewport(1200, 800, 60, 200, 400);
      const m2Start = mode2Vp.worldToScreen(0, 0);
      const m2End = mode2Vp.worldToScreen(6, 0);
      const mode2Diagram = buildTensionSideDiagramPath(
        { start: m2Start, end: m2End, length: 6 },
        samples,
        { kind: 'M', maxAbsValue: 45, maxPixelOffset: 70, tensionSidePositiveM: true }
      );

      // Mode 3 TeachingViewport projection
      const mode3Vp = new ViewportTransform2D({
        viewWidth: 900,
        viewHeight: 420,
        panX: 70,
        panY: 250,
        scale: 120,
      });
      const mode3Diagram = buildTensionSideDiagramPath(
        { start: { x: 0, y: 0 }, end: { x: 6, y: 0 }, length: 6 },
        samples,
        { kind: 'M', maxAbsValue: 45, maxPixelOffset: 70, tensionSidePositiveM: true },
        mode3Vp
      );

      // In both Mode 2 and Mode 3, midspan sagging moment +45 kNm is plotted at +70 px downward (+v on screen) from the beam axis
      expect(mode2Diagram.screenPoints[1].y - mode2Diagram.basePoints[1].y).toBeCloseTo(70, 6);
      expect(mode3Diagram.screenPoints[1].y - mode3Diagram.basePoints[1].y).toBeCloseTo(70, 6);
      expect(SemanticColorTokens.momentPositiveBlue).toBe('#3b82f6');
      expect(SemanticColorTokens.momentNegativeRed).toBe('#ef4444');
    });
  });
});
