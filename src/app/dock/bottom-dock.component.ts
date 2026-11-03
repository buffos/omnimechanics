import { Component, computed, inject, signal } from '@angular/core';
import { SolverService } from '../services/solver.service';
import { MeshService } from '../services/mesh.service';
import { MaterialService } from '../services/material.service';
import { NotificationService } from '../shared/notification.service';
import { I18nService } from '../shared/i18n.service';

@Component({
  selector: 'app-bottom-dock',
  standalone: true,
  imports: [],
  templateUrl: './bottom-dock.component.html',
  styleUrls: ['./bottom-dock.component.css'],
})
export class BottomDockComponent {
  readonly solverService: SolverService;
  readonly meshService: MeshService;
  readonly materialService: MaterialService;
  readonly notificationService: NotificationService;
  readonly i18n: I18nService;

  constructor(
    solverServiceOverride?: SolverService,
    meshServiceOverride?: MeshService,
    materialServiceOverride?: MaterialService,
    notificationServiceOverride?: NotificationService,
    i18nOverride?: I18nService
  ) {
    this.solverService =
      solverServiceOverride ?? inject(SolverService, { optional: true }) ?? new SolverService();
    this.meshService =
      meshServiceOverride ?? inject(MeshService, { optional: true }) ?? new MeshService();
    this.materialService =
      materialServiceOverride ??
      inject(MaterialService, { optional: true }) ??
      new MaterialService();
    this.notificationService =
      notificationServiceOverride ??
      inject(NotificationService, { optional: true }) ??
      new NotificationService();
    this.i18n = i18nOverride ?? inject(I18nService, { optional: true }) ?? new I18nService();
  }

  readonly isExpanded = signal<boolean>(true);
  readonly activeTab = signal<'history' | 'diagnostics' | 'sweep'>('history');
  readonly lastExportErrorCode = signal<string | null>(null);

  readonly hasExportableData = computed<boolean>(() => {
    const talbot = this.solverService.talbotTransientResult();
    const newmark = this.solverService.newmarkTransientResult();
    return Boolean(
      (talbot && talbot.frames.length > 0) || (newmark && newmark.frames.length > 0)
    );
  });

  // Hover inspection on graph
  readonly hoverPoint = signal<{ x: number; y: number; time: number; valTalbot?: number; valNewmark?: number } | null>(null);

  // Analytical Euler-Bernoulli static deflection for the 2D Cantilever Beam benchmark:
  // delta = P * L^3 / (3 * E * I)
  readonly staticEulerBernoulliMm = computed<number | null>(() => {
    const mesh = this.meshService.currentMesh();
    if (!mesh || mesh.dimension !== 2) return null;
    const ref = this.materialService.reference();
    const load = this.solverService.loadMagnitude();

    // Approximate bounding box length and height:
    let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
    for (const pt of mesh.nodes) {
      if (pt[0] < minX) minX = pt[0];
      if (pt[0] > maxX) maxX = pt[0];
      if (pt[1] < minY) minY = pt[1];
      if (pt[1] > maxY) maxY = pt[1];
    }
    const L = Math.max(1.0, maxX - minX);
    const H = Math.max(0.2, maxY - minY);
    const b = 1.0; // unit thickness for plane stress
    const I = (b * Math.pow(H, 3)) / 12.0;
    const E = ref.e0; // Pa

    const deltaMeters = (load * Math.pow(L, 3)) / (3.0 * E * I);
    return deltaMeters * 1000.0; // in mm
  });

  // Chart Dimensions
  readonly chartWidth = 900;
  readonly chartHeight = 180;
  readonly padding = { top: 25, right: 30, bottom: 30, left: 55 };

  // Plotting data bounds & coordinate transforms
  readonly plotBounds = computed(() => {
    const talbot = this.solverService.talbotTransientResult();
    const newmark = this.solverService.newmarkTransientResult();
    const staticVal = this.staticEulerBernoulliMm() ?? 0.12;

    let maxT = this.solverService.totalTime();
    let maxDispMm = staticVal * 1.5;

    if (talbot && talbot.time_points.length > 0) {
      maxT = Math.max(maxT, talbot.time_points[talbot.time_points.length - 1]);
      maxDispMm = Math.max(maxDispMm, talbot.max_overall_displacement * 1000.0 * 1.15);
    }
    if (newmark && newmark.time_points.length > 0) {
      maxT = Math.max(maxT, newmark.time_points[newmark.time_points.length - 1]);
      maxDispMm = Math.max(maxDispMm, newmark.max_overall_displacement * 1000.0 * 1.15);
    }

    if (maxDispMm <= 0) maxDispMm = 0.2;
    if (maxT <= 0) maxT = 0.5;

    return { maxT, maxDispMm };
  });

  readonly talbotPath = computed<string>(() => {
    const talbot = this.solverService.talbotTransientResult();
    if (!talbot || talbot.frames.length === 0) return '';
    const { maxT, maxDispMm } = this.plotBounds();

    const w = this.chartWidth - this.padding.left - this.padding.right;
    const h = this.chartHeight - this.padding.top - this.padding.bottom;

    const coords: [number, number][] = [[this.padding.left, this.padding.top + h]]; // start at (0,0)

    for (const frame of talbot.frames) {
      const x = this.padding.left + (frame.time / maxT) * w;
      const y = this.padding.top + h - (frame.max_displacement * 1000.0 / maxDispMm) * h;
      coords.push([x, y]);
    }

    return coords.reduce((acc, pt, idx) => `${acc} ${idx === 0 ? 'M' : 'L'} ${pt[0].toFixed(1)},${pt[1].toFixed(1)}`, '');
  });

  readonly newmarkPath = computed<string>(() => {
    const newmark = this.solverService.newmarkTransientResult();
    if (!newmark || newmark.frames.length === 0) return '';
    const { maxT, maxDispMm } = this.plotBounds();

    const w = this.chartWidth - this.padding.left - this.padding.right;
    const h = this.chartHeight - this.padding.top - this.padding.bottom;

    const coords: [number, number][] = [[this.padding.left, this.padding.top + h]];

    for (const frame of newmark.frames) {
      const x = this.padding.left + (frame.time / maxT) * w;
      const y = this.padding.top + h - (frame.max_displacement * 1000.0 / maxDispMm) * h;
      coords.push([x, y]);
    }

    return coords.reduce((acc, pt, idx) => `${acc} ${idx === 0 ? 'M' : 'L'} ${pt[0].toFixed(1)},${pt[1].toFixed(1)}`, '');
  });

  readonly newmarkPoints = computed<{ cx: number; cy: number; time: number; valMm: number }[]>(() => {
    const newmark = this.solverService.newmarkTransientResult();
    if (!newmark || newmark.frames.length === 0) return [];
    const { maxT, maxDispMm } = this.plotBounds();

    const w = this.chartWidth - this.padding.left - this.padding.right;
    const h = this.chartHeight - this.padding.top - this.padding.bottom;

    return newmark.frames.map((f) => {
      const cx = this.padding.left + (f.time / maxT) * w;
      const cy = this.padding.top + h - (f.max_displacement * 1000.0 / maxDispMm) * h;
      return { cx, cy, time: f.time, valMm: f.max_displacement * 1000.0 };
    });
  });

  readonly staticLineY = computed<number | null>(() => {
    const staticVal = this.staticEulerBernoulliMm();
    if (staticVal === null) return null;
    const { maxDispMm } = this.plotBounds();
    const h = this.chartHeight - this.padding.top - this.padding.bottom;
    return this.padding.top + h - (staticVal / maxDispMm) * h;
  });

  // Current animation scrubbing indicator X position
  readonly currentCursorX = computed<number | null>(() => {
    const res = this.solverService.transientResult();
    if (!res || res.frames.length === 0) return null;
    const idx = this.solverService.currentFrameIndex();
    if (idx >= res.frames.length) return null;
    const frame = res.frames[idx];

    const { maxT } = this.plotBounds();
    const w = this.chartWidth - this.padding.left - this.padding.right;
    return this.padding.left + (frame.time / maxT) * w;
  });

  toggleExpand(): void {
    this.isExpanded.update((v) => !v);
  }

  onChartClick(event: MouseEvent): void {
    const res = this.solverService.transientResult();
    if (!res || res.frames.length === 0) return;

    const svgEl = event.currentTarget as HTMLElement;
    const svg = svgEl.getBoundingClientRect();
    const clickX = event.clientX - svg.left;
    this.scrubHistoryChartToScreenX(clickX, svg.width || this.chartWidth);
  }

  /**
   * Maps an SVG click X coordinate to normalized time t in [0, T_max], finds the nearest frame index k,
   * and synchronizes the playback frame (`SC-010`).
   */
  scrubHistoryChartToScreenX(clickScreenX: number, svgClientWidth = this.chartWidth): number {
    const res = this.solverService.transientResult();
    if (!res || res.frames.length === 0) return 0;

    const scaleRatio = svgClientWidth > 0 ? this.chartWidth / svgClientWidth : 1.0;
    const viewBoxX = clickScreenX * scaleRatio;
    const w = this.chartWidth - this.padding.left - this.padding.right;
    const normX = Math.max(0, Math.min(1, (viewBoxX - this.padding.left) / w));

    const targetTime = normX * this.plotBounds().maxT;
    let closestIdx = 0;
    let minDiff = Infinity;
    res.frames.forEach((f, idx) => {
      const diff = Math.abs(f.time - targetTime);
      if (diff < minDiff) {
        minDiff = diff;
        closestIdx = idx;
      }
    });

    this.solverService.setFrame(closestIdx);
    return closestIdx;
  }

  exportCsv(): boolean {
    if (!this.hasExportableData()) {
      this.lastExportErrorCode.set('EXPORT_DATA_EMPTY');
      const msg =
        this.i18n.currentLang() === 'el'
          ? 'EXPORT_DATA_EMPTY: Δεν υπάρχουν διαθέσιμα δεδομένα χρονοϊστορίας για εξαγωγή.'
          : 'EXPORT_DATA_EMPTY: No transient or sweep series available to export.';
      this.notificationService.show(msg, 'info');
      return false;
    }
    this.lastExportErrorCode.set(null);

    const talbot = this.solverService.talbotTransientResult();
    const newmark = this.solverService.newmarkTransientResult();
    const staticVal = this.staticEulerBernoulliMm() ?? 0.0;

    let csvContent = 'data:text/csv;charset=utf-8,';
    csvContent += 'time_s,talbot_displacement_mm,newmark_displacement_mm,static_euler_bernoulli_mm\n';

    const maxSteps = Math.max(
      talbot?.frames.length ?? 0,
      newmark?.frames.length ?? 0
    );

    for (let i = 0; i < maxSteps; i++) {
      const t = talbot?.frames[i]?.time ?? newmark?.frames[i]?.time ?? i * 0.02;
      const dTal = talbot?.frames[i] ? (talbot.frames[i].max_displacement * 1000.0).toFixed(6) : '';
      const dNew = newmark?.frames[i] ? (newmark.frames[i].max_displacement * 1000.0).toFixed(6) : '';
      csvContent += `${t.toFixed(4)},${dTal},${dNew},${staticVal.toFixed(6)}\n`;
    }

    if (typeof document !== 'undefined') {
      const encodedUri = encodeURI(csvContent);
      const link = document.createElement('a');
      link.setAttribute('href', encodedUri);
      link.setAttribute('download', `transient_response_history_${Date.now()}.csv`);
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
    }
    return true;
  }

  exportSvg(): boolean {
    if (!this.hasExportableData()) {
      this.lastExportErrorCode.set('EXPORT_DATA_EMPTY');
      const msg =
        this.i18n.currentLang() === 'el'
          ? 'EXPORT_DATA_EMPTY: Δεν υπάρχουν διαθέσιμα δεδομένα γραφήματος SVG για εξαγωγή.'
          : 'EXPORT_DATA_EMPTY: No transient or sweep series available to export.';
      this.notificationService.show(msg, 'info');
      return false;
    }
    this.lastExportErrorCode.set(null);

    if (typeof document === 'undefined') return true;
    const svgElem = document.getElementById('transient-response-svg');
    if (!svgElem) return false;
    const serializer = new XMLSerializer();
    const source = serializer.serializeToString(svgElem);
    const svgBlob = new Blob([source], { type: 'image/svg+xml;charset=utf-8' });
    const url = URL.createObjectURL(svgBlob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `response_history_curve_${Date.now()}.svg`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    return true;
  }
}
