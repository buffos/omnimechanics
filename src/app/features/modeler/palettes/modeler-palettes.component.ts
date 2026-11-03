import { Component, ChangeDetectionStrategy, inject, signal, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ModelerStore } from '../core/modeler.store';
import { SupportKind } from '../core/modeler.types';
import { NotificationService } from '../../../shared/notification.service';

@Component({
  selector: 'app-modeler-palettes',
  standalone: true,
  imports: [CommonModule, FormsModule],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './modeler-palettes.component.html',
  styleUrl: './modeler-palettes.component.css',
})
export class ModelerPalettesComponent {
  readonly store = inject(ModelerStore);
  readonly notifications = inject(NotificationService);

  activeTab = signal<'supports' | 'loads'>('supports');

  readonly isCurrentLcSelfWeight = computed(() => {
    const lc = this.store.model().load_cases.find(l => l.id === this.store.activeLoadCaseId());
    return (lc as any)?.self_weight ?? false;
  });

  private backdropMouseDown = false;

  onBackdropMouseDown(event: MouseEvent): void {
    this.backdropMouseDown = event.target === event.currentTarget;
  }

  onBackdropMouseUp(event: MouseEvent): void {
    if (this.backdropMouseDown && event.target === event.currentTarget) {
      this.store.isLoadsModalOpen.set(false);
    }
    this.backdropMouseDown = false;
  }

  // Support inputs
  supportType = signal<'Fixed' | 'Pinned' | 'Roller' | 'Guided' | 'Elastic'>('Pinned');
  supportAngle = signal<number>(0);
  supportKx = signal<number>(1000);
  supportKy = signal<number>(1000);
  supportKtheta = signal<number>(500);
  supportUnilateral = signal<boolean>(false);

  // Load inputs
  loadKind = signal<'PointLoad' | 'DistributedLoad' | 'NodalLoad' | 'Settlement' | 'ThermalLoad'>('DistributedLoad');
  memberPointPx = signal<number>(0);
  memberPointPy = signal<number>(-10);
  memberPointA = signal<number>(1.0); // distance in meters

  distQy1 = signal<number>(-10);
  distQy2 = signal<number>(-10);
  distArel = signal<number>(0);
  distBrel = signal<number>(1.0);

  nodalFx = signal<number>(0);
  nodalFy = signal<number>(-15);
  nodalMz = signal<number>(0);

  settleDx = signal<number>(0);
  settleDy = signal<number>(-0.01);
  settleDtheta = signal<number>(0);

  thermalTop = signal<number>(20);
  thermalBottom = signal<number>(10);

  applySupportToSelection(): void {
    const sel = this.store.selection();
    if (sel.nodeIds.size === 0) {
      this.notifications.show('Επιλέξτε τουλάχιστον έναν κόμβο / Select at least one node', 'warning');
      return;
    }

    let kind: SupportKind;
    const type = this.supportType();
    if (type === 'Fixed') kind = { type: 'Fixed' };
    else if (type === 'Pinned') kind = { type: 'Pinned' };
    else if (type === 'Roller') kind = { type: 'Roller', angle_deg: this.supportAngle() };
    else if (type === 'Guided') kind = { type: 'Guided', angle_deg: this.supportAngle() };
    else kind = { type: 'Elastic', kx: this.supportKx(), ky: this.supportKy(), ktheta: this.supportKtheta() };

    for (const nodeId of sel.nodeIds) {
      this.store.assignSupport(nodeId, kind, this.supportUnilateral());
    }
    this.notifications.show('Η στήριξη εφαρμόστηκε επιτυχώς', 'success');
  }

  removeSupportFromSelection(): void {
    const sel = this.store.selection();
    for (const nodeId of sel.nodeIds) {
      this.store.removeSupport(nodeId);
    }
  }

  applyLoadToSelection(): void {
    const sel = this.store.selection();
    const kind = this.loadKind();

    try {
      if (kind === 'NodalLoad') {
        if (sel.nodeIds.size === 0) {
          this.notifications.show('Επιλέξτε κόμβο / Select a node', 'warning');
          return;
        }
        for (const nid of sel.nodeIds) {
          this.store.addNodalLoad(nid, this.nodalFx(), this.nodalFy(), this.nodalMz());
        }
      } else if (kind === 'Settlement') {
        if (sel.nodeIds.size === 0) {
          this.notifications.show('Επιλέξτε στηριγμένο κόμβο / Select a supported node', 'warning');
          return;
        }
        for (const nid of sel.nodeIds) {
          this.store.addSupportSettlement(nid, this.settleDx(), this.settleDy(), this.settleDtheta());
        }
      } else if (kind === 'PointLoad') {
        if (sel.elementIds.size === 0) {
          this.notifications.show('Επιλέξτε ράβδο / Select a member', 'warning');
          return;
        }
        for (const eid of sel.elementIds) {
          this.store.addMemberPointLoad(eid, this.memberPointPx(), this.memberPointPy(), this.memberPointA());
        }
      } else if (kind === 'DistributedLoad') {
        if (sel.elementIds.size === 0) {
          this.notifications.show('Επιλέξτε ράβδο / Select a member', 'warning');
          return;
        }
        for (const eid of sel.elementIds) {
          this.store.addMemberDistributedLoad(
            eid,
            0,
            this.distQy1(),
            0,
            this.distQy2(),
            this.distArel(),
            this.distBrel()
          );
        }
      } else if (kind === 'ThermalLoad') {
        if (sel.elementIds.size === 0) {
          this.notifications.show('Επιλέξτε ράβδο / Select a member', 'warning');
          return;
        }
        for (const eid of sel.elementIds) {
          this.store.addThermalLoad(eid, this.thermalTop(), this.thermalBottom());
        }
      }
      this.notifications.show('Το φορτίο προστέθηκε επιτυχώς', 'success');
    } catch (err: any) {
      if (err.message === 'LOAD_OUT_OF_MEMBER_BOUNDS') {
        this.notifications.show('Η απόσταση a είναι εκτός ορίων μήκους της ράβδου / Distance a out of member bounds', 'warning');
      } else {
        this.notifications.show(err.message || 'Σφάλμα προσθήκης φορτίου', 'error');
      }
    }
  }
}
