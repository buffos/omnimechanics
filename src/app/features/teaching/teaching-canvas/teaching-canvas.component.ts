import { ChangeDetectionStrategy, Component, computed, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TeachingService } from '../teaching.service';
import { I18nService } from '../../../shared/i18n.service';
import { buildTensionSideDiagramPath } from '../../../shared/svg-canvas-kernel';
import { Node2D, Member2D, Support2D, ReactionResult, MemberDiagramData } from '../teaching.types';

@Component({
  selector: 'app-teaching-canvas',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './teaching-canvas.component.html',
  styleUrls: ['./teaching-canvas.component.css'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class TeachingCanvasComponent {
  readonly teaching = inject(TeachingService);
  readonly i18n = inject(I18nService);

  readonly viewWidth = 900;
  readonly viewHeight = 420;
  readonly margin = 70;

  // View bounds & coordinate transformation
  readonly bounds = computed(() => {
    const sol = this.teaching.solution();
    if (!sol || sol.nodes.length === 0) {
      return { minX: 0, maxX: 10, minY: 0, maxY: 4, width: 10, height: 4 };
    }
    let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
    for (const n of sol.nodes) {
      if (n.x < minX) minX = n.x;
      if (n.x > maxX) maxX = n.x;
      if (n.y < minY) minY = n.y;
      if (n.y > maxY) maxY = n.y;
    }
    const width = Math.max(maxX - minX, 1.0);
    const height = Math.max(maxY - minY, 1.0);
    return { minX, maxX, minY, maxY, width, height };
  });

  readonly isTruss = computed(() => {
    return this.teaching.solution()?.system_type === 'truss';
  });

  readonly scaleX = computed(() => {
    const b = this.bounds();
    const margin = this.isTruss() ? 80 : this.margin;
    return (this.viewWidth - 2 * margin) / b.width;
  });

  readonly scaleY = computed(() => {
    const b = this.bounds();
    if (this.isTruss()) {
      const usableHeight = this.viewHeight - 160; // 80px margin top and bottom for labels/loads/supports
      const scaleByHeight = usableHeight / Math.max(b.height, 0.5);
      return Math.min(this.scaleX(), scaleByHeight);
    }
    // Keep aspect ratio reasonable or fit into available canvas
    const maxUsableY = this.viewHeight - 2 * this.margin - 60; // Leave space for diagrams below beam
    return Math.min(this.scaleX(), maxUsableY / b.height);
  });

  mapX(x: number): number {
    const b = this.bounds();
    if (this.isTruss()) {
      const centerX = this.viewWidth / 2;
      const structCenterX = (b.minX + b.maxX) / 2;
      return centerX + (x - structCenterX) * this.scaleY();
    }
    return this.margin + (x - b.minX) * this.scaleX();
  }

  mapY(y: number): number {
    const b = this.bounds();
    if (this.isTruss()) {
      // Center truss vertically in view, with slight upward shift for reactions below
      const centerY = this.viewHeight / 2 - 15;
      const structCenterY = (b.minY + b.maxY) / 2;
      return centerY - (y - structCenterY) * this.scaleY();
    }
    // SVG has Y down, engineering has Y up
    return this.viewHeight - this.margin - 100 - (y - b.minY) * this.scaleY();
  }

  isNodeActive(nodeId: number): boolean {
    const step = this.teaching.currentStep();
    return step?.active_entity_id === `node:${nodeId}`;
  }

  isMemberActive(memberId: number): boolean {
    const step = this.teaching.currentStep();
    return step?.active_entity_id === `member:${memberId}`;
  }

  getDistLoadTicks(memLength: number): number[] {
    const numTicks = 8;
    const ticks: number[] = [];
    const step = memLength / numTicks;
    for (let i = 0; i <= numTicks; i++) {
      ticks.push(i * step);
    }
    return ticks;
  }

  getDiagramClass(diag: MemberDiagramData, sol: any): string {
    if (sol.system_type === 'truss') {
      const force = diag.samples[0]?.n ?? 0;
      return force >= 0 ? 'diagram-truss-tension' : 'diagram-truss-compression';
    }
    const type = this.teaching.activeDiagramType();
    return type === 'M' ? 'diagram-m' : type === 'Q' ? 'diagram-q' : 'diagram-n';
  }

  getStationGlobalX(diag: MemberDiagramData, localX: number, sol: any): number {
    const mem = sol.members.find((m: Member2D) => m.id === diag.member_id);
    if (!mem) return this.mapX(localX);
    const startNode = sol.nodes.find((n: Node2D) => n.id === mem.start_node);
    const globalX = (startNode?.x ?? 0) + localX;
    return this.mapX(globalX);
  }

  getDiagramPath(diag: MemberDiagramData, sol: any): string {
    if (!diag.samples || diag.samples.length < 2) return '';
    const mem = sol.members.find((m: Member2D) => m.id === diag.member_id);
    if (!mem) return '';

    const startNode = sol.nodes.find((n: Node2D) => n.id === mem.start_node);
    const endNode = sol.nodes.find((n: Node2D) => n.id === mem.end_node);
    if (!startNode || !endNode) return '';

    const x0 = this.mapX(startNode.x);
    const y0 = this.mapY(startNode.y);
    const x1 = this.mapX(endNode.x);
    const y1 = this.mapY(endNode.y);

    if (sol.system_type === 'truss') {
      const dx = x1 - x0;
      const dy = y1 - y0;
      const len = Math.hypot(dx, dy);
      if (len < 1e-4) return '';
      // Normal vector perpendicular to member
      const nx = -dy / len;
      const ny = dx / len;

      let globalMaxN = 1e-6;
      for (const d of sol.diagrams) {
        for (const s of d.samples) {
          const v = Math.abs(s.n);
          if (v > globalMaxN) globalMaxN = v;
        }
      }

      const force = diag.samples[0]?.n ?? 0;
      if (Math.abs(force) < 1e-4) return ''; // Zero-force member

      const maxOffset = 12; // Crisp, compact envelope in px
      const offset = (Math.abs(force) / globalMaxN) * maxOffset;

      const px0 = x0 + offset * nx;
      const py0 = y0 + offset * ny;
      const px1 = x1 + offset * nx;
      const py1 = y1 + offset * ny;

      return `M ${x0} ${y0} L ${x1} ${y1} L ${px1} ${py1} L ${px0} ${py0} Z`;
    }

    const type = this.teaching.activeDiagramType();
    let maxVal = 1e-6;
    for (const d of sol.diagrams) {
      for (const s of d.samples) {
        const v = Math.abs(type === 'M' ? s.m : type === 'Q' ? s.q : s.n);
        if (v > maxVal) maxVal = v;
      }
    }

    const { path } = buildTensionSideDiagramPath(
      {
        start: { x: x0, y: y0 },
        end: { x: x1, y: y1 },
        length: mem.length,
      },
      diag.samples.map((s) => ({
        x: s.x,
        value: type === 'M' ? s.m : type === 'Q' ? s.q : s.n,
      })),
      {
        kind: type,
        maxAbsValue: maxVal,
        maxPixelOffset: 70,
        tensionSidePositiveM: true,
      }
    );
    return path;
  }
}
