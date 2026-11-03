import { Component, ChangeDetectionStrategy, inject, signal, effect, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ModelerStore } from '../core/modeler.store';
import { ModelerResultsService } from '../results/modeler-results.service';
import { ActiveTool, SupportKind } from '../core/modeler.types';
import { SnappingEngine } from '../core/snapping-engine';
import { DraftingViewport } from '../core/viewport';
import { I18nService } from '../../../shared/i18n.service';
import { AnalysisKind } from '../results/results.types';
import { TeachingHandoffService } from '../handoff/teaching-handoff.service';
import { DiagramExportService } from '../handoff/diagram-export.service';
import { ProjectDocument } from '../handoff/handoff.types';

@Component({
  selector: 'app-modeler-toolbar',
  standalone: true,
  imports: [CommonModule, FormsModule],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './modeler-toolbar.component.html',
  styleUrl: './modeler-toolbar.component.css',
})
export class ModelerToolbarComponent {
  readonly store = inject(ModelerStore);
  readonly resultsService = inject(ModelerResultsService);
  readonly i18n = inject(I18nService);
  readonly handoff = inject(TeachingHandoffService);
  readonly diagramExport = inject(DiagramExportService);

  coordinateInput = signal<string>('');
  isClassifierPopoverOpen = signal<boolean>(false);
  isPresetMenuOpen = signal<boolean>(false);
  isExportMenuOpen = signal<boolean>(false);

  // Expandable flyouts
  isDrawFlyoutOpen = signal<boolean>(false);
  activeDrawTool = signal<'DrawMember' | 'DrawTruss' | 'DrawCable'>('DrawMember');

  isSupportFlyoutOpen = signal<boolean>(false);
  activeSupportType = signal<'Fixed' | 'Pinned' | 'Roller' | 'Guided' | 'Elastic'>('Fixed');

  isTransformFlyoutOpen = signal<boolean>(false);
  isTransformDialogOpen = signal<boolean>(false);
  transformType = signal<'move' | 'copy' | 'mirror' | 'array'>('move');
  transformDx = signal<number>(1.0);
  transformDy = signal<number>(0.0);
  transformCount = signal<number>(3);
  mirrorAxis = signal<'Y' | 'X'>('Y');

  constructor() {
    // Re-evaluate classifier whenever model mutates
    effect(() => {
      const model = this.store.model();
      this.handoff.classifyModel(model, this.store.activeLoadCaseId());
    });
  }

  // Draw flyout controls
  toggleDrawFlyout(): void {
    this.isDrawFlyoutOpen.update(v => !v);
    this.isSupportFlyoutOpen.set(false);
    this.isTransformFlyoutOpen.set(false);
  }

  selectDrawTool(tool: 'DrawMember' | 'DrawTruss' | 'DrawCable'): void {
    this.activeDrawTool.set(tool);
    this.store.setTool(tool);
    this.isDrawFlyoutOpen.set(false);
  }

  // Support flyout controls
  toggleSupportFlyout(): void {
    this.isSupportFlyoutOpen.update(v => !v);
    this.isDrawFlyoutOpen.set(false);
    this.isTransformFlyoutOpen.set(false);
  }

  selectSupportType(type: 'Fixed' | 'Pinned' | 'Roller' | 'Guided' | 'Elastic'): void {
    this.activeSupportType.set(type);
    let kind: SupportKind;
    if (type === 'Fixed') kind = { type: 'Fixed' };
    else if (type === 'Pinned') kind = { type: 'Pinned' };
    else if (type === 'Roller') kind = { type: 'Roller', angle_deg: 0 };
    else if (type === 'Guided') kind = { type: 'Guided', angle_deg: 0 };
    else kind = { type: 'Elastic', kx: 1000, ky: 1000, ktheta: 1000 };

    this.store.selectedSupportKind.set(kind);
    this.store.setTool('PlaceSupport');
    this.isSupportFlyoutOpen.set(false);
  }

  // Transform controls
  toggleTransformFlyout(): void {
    this.isTransformFlyoutOpen.update(v => !v);
    this.isDrawFlyoutOpen.set(false);
    this.isSupportFlyoutOpen.set(false);
  }

  openTransformDialog(type: 'move' | 'copy' | 'mirror' | 'array'): void {
    this.transformType.set(type);
    this.isTransformDialogOpen.set(true);
    this.isTransformFlyoutOpen.set(false);
  }

  applyTransform(): void {
    const t = this.transformType();
    if (t === 'move') {
      this.store.moveSelection(this.transformDx(), this.transformDy());
    } else if (t === 'copy') {
      this.store.copySelection(this.transformDx(), this.transformDy());
    } else if (t === 'mirror') {
      if (this.mirrorAxis() === 'Y') {
        this.store.mirrorSelection({ x: 0, y: 0 }, { x: 0, y: 1 });
      } else {
        this.store.mirrorSelection({ x: 0, y: 0 }, { x: 1, y: 0 });
      }
    } else if (t === 'array') {
      this.store.linearArraySelection(this.transformDx(), this.transformDy(), this.transformCount());
    }
    this.isTransformDialogOpen.set(false);
  }

  openLoadsModal(): void {
    this.store.isLoadsModalOpen.set(true);
  }

  setTool(tool: ActiveTool): void {
    this.store.setTool(tool);
    this.isDrawFlyoutOpen.set(false);
    this.isSupportFlyoutOpen.set(false);
    this.isTransformFlyoutOpen.set(false);
  }

  readonly currentTargetSelectValue = computed(() => {
    const target = this.resultsService.resultTarget();
    const targetId = this.resultsService.activeTargetId();
    if (target === 'LoadCase') return `LC:${targetId}`;
    if (target === 'Combination') return `COMB:${targetId}`;
    return `ENV:${targetId}`;
  });

  setGridStep(event: Event): void {
    const val = parseFloat((event.target as HTMLSelectElement).value);
    if (!isNaN(val)) {
      this.store.setGridStep(val);
    }
  }

  setAnalysisKind(event: Event): void {
    const val = (event.target as HTMLSelectElement).value as AnalysisKind;
    this.resultsService.setAnalysisKind(val);
  }

  setResultTarget(event: Event): void {
    const val = (event.target as HTMLSelectElement).value;
    if (val.startsWith('LC:')) {
      this.resultsService.setResultTarget('LoadCase', val.replace('LC:', ''));
    } else if (val.startsWith('COMB:')) {
      this.resultsService.setResultTarget('Combination', val.replace('COMB:', ''));
    } else if (val.startsWith('ENV:')) {
      this.resultsService.setResultTarget('Envelope', val.replace('ENV:', ''));
    }
  }

  openCombinationModal(): void {
    this.store.isCombinationModalOpen.set(true);
  }

  onCoordinateSubmit(): void {
    const input = this.coordinateInput().trim();
    if (!input) return;

    const basePoint = this.store.currentCursorWorld();
    const pt = SnappingEngine.parseCoordinateInput(input, basePoint);
    if (pt) {
      if (this.store.activeTool() === 'DrawMember') {
        this.store.sketchMember(basePoint, pt);
      }
      this.coordinateInput.set('');
    }
  }

  zoomExtents(): void {
    const nodes = this.store.model().nodes;
    const vp = this.store.viewport();
    if (nodes.length === 0) {
      this.store.viewport.set(new DraftingViewport());
      return;
    }

    let minX = Infinity;
    let maxX = -Infinity;
    let minY = Infinity;
    let maxY = -Infinity;

    for (const n of nodes) {
      if (n.x < minX) minX = n.x;
      if (n.x > maxX) maxX = n.x;
      if (n.y < minY) minY = n.y;
      if (n.y > maxY) maxY = n.y;
    }

    const spanX = Math.max(1.0, maxX - minX);
    const spanY = Math.max(1.0, maxY - minY);
    const margin = 1.3;

    const scaleX = (vp.width * 0.8) / (spanX * margin);
    const scaleY = (vp.height * 0.8) / (spanY * margin);
    const newScale = Math.min(scaleX, scaleY, 150);

    const midX = (minX + maxX) / 2;
    const midY = (minY + maxY) / 2;
    const newPanX = vp.width / 2 - midX * newScale;
    const newPanY = vp.height / 2 + midY * newScale;

    this.store.viewport.set(new DraftingViewport(vp.width, vp.height, newScale, newPanX, newPanY, vp.gridStep));
  }

  toggleClassifierPopover(): void {
    this.isClassifierPopoverOpen.update(v => !v);
  }

  async openTeachingStepByStep(): Promise<void> {
    this.isClassifierPopoverOpen.set(false);
    await this.handoff.openStepByStep(this.store.model(), undefined, this.store.activeLoadCaseId());
  }

  async openDsmWalkthrough(): Promise<void> {
    this.isClassifierPopoverOpen.set(false);
    await this.handoff.openStepByStep(this.store.model(), 'dsm_walkthrough', this.store.activeLoadCaseId());
  }

  togglePresetMenu(): void {
    this.isPresetMenuOpen.update(v => !v);
  }

  async importPreset(presetId: string): Promise<void> {
    this.isPresetMenuOpen.set(false);
    const model = await this.handoff.importPreset(presetId);
    this.store.loadModel(model);
    this.zoomExtents();
  }

  toggleExportMenu(): void {
    this.isExportMenuOpen.update(v => !v);
  }

  private getActiveDiagramType(): 'M' | 'Q' | 'N' {
    const d = this.resultsService.activeDiagram();
    return d === 'Q' ? 'Q' : d === 'N' ? 'N' : 'M';
  }

  exportSvg(): void {
    this.isExportMenuOpen.set(false);
    const svg = document.querySelector('svg.modeler-svg') as SVGSVGElement | null;
    if (svg) {
      this.diagramExport.downloadSvg(
        svg,
        'structural-diagram.svg',
        this.getActiveDiagramType(),
        this.i18n.currentLang() as 'el' | 'en'
      );
    }
  }

  async exportPng(): Promise<void> {
    this.isExportMenuOpen.set(false);
    const svg = document.querySelector('svg.modeler-svg') as SVGSVGElement | null;
    if (svg) {
      await this.diagramExport.downloadPng(
        svg,
        'structural-diagram.png',
        2.0,
        this.getActiveDiagramType(),
        this.i18n.currentLang() as 'el' | 'en'
      );
    }
  }

  saveProject(): void {
    this.isExportMenuOpen.set(false);
    const doc: ProjectDocument = {
      version: '2.0.0',
      structural: this.store.model(),
      combinations: this.store.combinations(),
      seismic_settings: this.store.seismicSettings(),
      viewport: {
        panX: this.store.viewport().panX,
        panY: this.store.viewport().panY,
        scale: this.store.viewport().scale,
        gridStep: this.store.viewport().gridStep,
      },
    };
    this.diagramExport.exportProjectFile(doc, 'project.bem');
  }

  private transformBackdropMouseDown = false;

  onTransformBackdropMouseDown(event: MouseEvent): void {
    this.transformBackdropMouseDown = event.target === event.currentTarget;
  }

  onTransformBackdropMouseUp(event: MouseEvent): void {
    if (this.transformBackdropMouseDown && event.target === event.currentTarget) {
      this.isTransformDialogOpen.set(false);
    }
    this.transformBackdropMouseDown = false;
  }

  triggerFileInput(fileInput: HTMLInputElement): void {
    this.isExportMenuOpen.set(false);
    fileInput.click();
  }

  onFileSelected(event: Event): void {
    const input = event.target as HTMLInputElement;
    if (!input.files || input.files.length === 0) return;
    const file = input.files[0];
    const reader = new FileReader();
    reader.onload = () => {
      try {
        const doc = this.diagramExport.importProjectFile(reader.result as string);
        this.store.loadModel(doc.structural);
        if (doc.combinations) {
          this.store.setCombinations(doc.combinations);
        }
        if (doc.seismic_settings) {
          this.store.setSeismicSettings(doc.seismic_settings);
        }
        if (doc.viewport) {
          const vp = this.store.viewport();
          vp.panX = doc.viewport.panX ?? vp.panX;
          vp.panY = doc.viewport.panY ?? vp.panY;
          vp.scale = doc.viewport.scale ?? vp.scale;
          vp.gridStep = doc.viewport.gridStep ?? vp.gridStep;
          this.store.viewport.set(new DraftingViewport(vp.width, vp.height, vp.scale, vp.panX, vp.panY, vp.gridStep));
        }
      } catch (err: any) {
        alert('Failed to load .bem project: ' + (err?.message || err));
      }
    };
    reader.readAsText(file);
    input.value = '';
  }
}
