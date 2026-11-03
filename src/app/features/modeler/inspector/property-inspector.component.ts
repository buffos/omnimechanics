import { Component, ChangeDetectionStrategy, inject, computed, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ModelerStore } from '../core/modeler.store';
import { NotificationService } from '../../../shared/notification.service';
import {
  FrameMemberElement,
  TrussBarElement,
  CableElement,
  SpringElement,
  Element2D,
  Node2D,
  EndRelease,
  RigidOffset,
  isLineElement,
} from '../core/modeler.types';

@Component({
  selector: 'app-property-inspector',
  standalone: true,
  imports: [CommonModule, FormsModule],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './property-inspector.component.html',
  styleUrl: './property-inspector.component.css',
})
export class PropertyInspectorComponent {
  readonly store = inject(ModelerStore);
  readonly notifications = inject(NotificationService);

  readonly selection = this.store.selection;
  readonly model = this.store.model;

  readonly selectedNode = computed<Node2D | null>(() => {
    const sel = this.selection();
    if (sel.nodeIds.size !== 1) return null;
    const nodeId = Array.from(sel.nodeIds)[0];
    return this.model().nodes.find(n => n.id === nodeId) ?? null;
  });

  readonly selectedElement = computed<Element2D | null>(() => {
    const sel = this.selection();
    if (sel.elementIds.size !== 1) return null;
    const elemId = Array.from(sel.elementIds)[0];
    return this.model().elements.find(e => e.id === elemId) ?? null;
  });

  readonly isMultiSelection = computed(() => {
    const sel = this.selection();
    return sel.nodeIds.size + sel.elementIds.size > 1;
  });

  readonly selectedElementGeometry = computed(() => {
    const elem = this.selectedElement();
    if (!elem || !isLineElement(elem)) return null;
    const model = this.model();
    const ni = model.nodes.find(n => n.id === elem.node_i);
    const nj = model.nodes.find(n => n.id === elem.node_j);
    if (!ni || !nj) return null;

    const dx = nj.x - ni.x;
    const dy = nj.y - ni.y;
    const length = Math.hypot(dx, dy);
    let angleDeg = (Math.atan2(dy, dx) * 180) / Math.PI;
    if (angleDeg < 0) angleDeg += 360;

    return {
      lengthM: length.toFixed(3),
      angleDeg: angleDeg.toFixed(1),
      nodeI: { id: ni.id, x: ni.x.toFixed(3), y: ni.y.toFixed(3) },
      nodeJ: { id: nj.id, x: nj.x.toFixed(3), y: nj.y.toFixed(3) },
    };
  });

  readonly nodeSupport = computed(() => {
    const node = this.selectedNode();
    if (!node) return null;
    return this.model().supports.find(s => s.node_id === node.id) ?? null;
  });

  // Node actions
  updateNodeCoord(coord: 'x' | 'y', valStr: string): void {
    const node = this.selectedNode();
    if (!node) return;
    const val = parseFloat(valStr);
    if (isNaN(val)) return;

    if (coord === 'x') {
      this.store.updateNodePosition(node.id, val, node.y);
    } else {
      this.store.updateNodePosition(node.id, node.x, val);
    }
  }

  applyInternalHinge(): void {
    const node = this.selectedNode();
    if (!node) return;
    try {
      this.store.applyInternalHinge(node.id);
      this.notifications.show(`Εφαρμόστηκε εσωτερική άρθρωση στον κόμβο ${node.id}`, 'success');
    } catch (err: any) {
      if (err.message === 'INVALID_HINGE_TARGET') {
        this.notifications.show('Απαιτούνται τουλάχιστον 2 συνδεδεμένες ράβδοι για εσωτερική άρθρωση', 'warning');
      } else {
        this.notifications.show(err.message || 'Σφάλμα εφαρμογής άρθρωσης', 'error');
      }
    }
  }

  removeSupport(): void {
    const node = this.selectedNode();
    if (!node) return;
    this.store.removeSupport(node.id);
  }

  // Element actions
  updateElementSection(sectionId: string): void {
    const elem = this.selectedElement();
    if (!elem) return;
    this.store.updateElementProperties(elem.id, { section_id: sectionId } as any);
  }

  updateElementMaterial(materialId: string): void {
    const elem = this.selectedElement();
    if (!elem) return;
    this.store.updateElementProperties(elem.id, { material_id: materialId } as any);
  }

  toggleRelease(end: 'i' | 'j', dof: keyof EndRelease): void {
    const elem = this.selectedElement();
    if (!elem || elem.kind !== 'FrameMember') return;
    const frameElem = elem as FrameMemberElement;
    const currentRel = end === 'i' ? frameElem.release_i : frameElem.release_j;
    const updatedRel = { ...currentRel, [dof]: !currentRel[dof] };

    this.store.updateElementProperties(elem.id, {
      [end === 'i' ? 'release_i' : 'release_j']: updatedRel,
    } as any);
  }

  updateOffset(end: 'i' | 'j', axis: 'dx' | 'dy', valStr: string): void {
    const elem = this.selectedElement();
    if (!elem || elem.kind !== 'FrameMember') return;
    const val = parseFloat(valStr);
    if (isNaN(val)) return;

    const frameElem = elem as FrameMemberElement;
    const currentOffset = end === 'i' ? frameElem.offset_i : frameElem.offset_j;
    const updatedOffset = { ...currentOffset, [axis]: val };

    this.store.updateElementProperties(elem.id, {
      [end === 'i' ? 'offset_i' : 'offset_j']: updatedOffset,
    } as any);
  }

  updateCablePrestress(mode: 'UnstressedLength' | 'SagRatio' | 'Pretension', valStr: string): void {
    const elem = this.selectedElement();
    if (!elem || elem.kind !== 'Cable') return;
    const val = parseFloat(valStr);

    let prestressSpec: any;
    if (mode === 'UnstressedLength') prestressSpec = { mode, l0: val };
    else if (mode === 'SagRatio') prestressSpec = { mode, sag_to_span: val };
    else prestressSpec = { mode, t0: val };

    try {
      this.store.updateElementProperties(elem.id, { prestress: prestressSpec } as any);
    } catch (err: any) {
      if (err.message === 'INVALID_CABLE_SPEC') {
        this.notifications.show('Μη έγκυρη προένταση τένοντα (> 0) / Invalid cable spec (> 0)', 'warning');
      }
    }
  }

  updateTrussAxial(behavior: 'Both' | 'TensionOnly' | 'CompressionOnly'): void {
    const elem = this.selectedElement();
    if (!elem || elem.kind !== 'TrussBar') return;
    this.store.updateElementProperties(elem.id, { axial_behavior: behavior } as any);
  }
}
