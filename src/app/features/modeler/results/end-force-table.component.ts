import { Component, ChangeDetectionStrategy, inject, signal, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ModelerResultsService } from './modeler-results.service';
import { ModelerStore } from '../core/modeler.store';
import { isLineElement } from '../core/modeler.types';

@Component({
  selector: 'app-end-force-table',
  standalone: true,
  imports: [CommonModule],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './end-force-table.component.html',
  styleUrl: './end-force-table.component.css',
})
export class EndForceTableComponent {
  readonly resultsService = inject(ModelerResultsService);
  readonly store = inject(ModelerStore);

  isCollapsed = signal<boolean>(false);

  readonly rows = computed(() => {
    const isEnvelope = this.resultsService.resultTarget() === 'Envelope';
    const model = this.store.model();
    const elemMap = new Map(model.elements.map(e => [e.id, e]));
    const nodeMap = new Map(model.nodes.map(n => [n.id, n]));

    if (isEnvelope) {
      const env = this.resultsService.currentEnvelope();
      if (!env) return [];
      return env.member_envelopes.map(me => {
        const elem = elemMap.get(me.element_id);
        const nodeI = elem && isLineElement(elem) ? elem.node_i : '-';
        const nodeJ = elem && isLineElement(elem) ? elem.node_j : '-';
        const ni = elem && isLineElement(elem) ? nodeMap.get(elem.node_i) : null;
        const nj = elem && isLineElement(elem) ? nodeMap.get(elem.node_j) : null;
        const calcLen = ni && nj ? Math.hypot(nj.x - ni.x, nj.y - ni.y) : (me.stations[me.stations.length - 1]?.x_m ?? 0);
        const st0 = me.stations[0];
        const stEnd = me.stations[me.stations.length - 1];
        return {
          elementId: me.element_id,
          nodeI,
          nodeJ,
          lengthM: Math.round((me.length_eff_m || calcLen) * 100) / 100,
          ni: Math.round((st0?.max_n ?? 0) * 10) / 10,
          qi: Math.round((st0?.max_q ?? 0) * 10) / 10,
          mi: Math.round((st0?.max_m ?? 0) * 10) / 10,
          nj: Math.round((stEnd?.max_n ?? 0) * 10) / 10,
          qj: Math.round((stEnd?.max_q ?? 0) * 10) / 10,
          mj: Math.round((stEnd?.max_m ?? 0) * 10) / 10,
          governing: st0?.governing_max_m || '1.35G + 1.50Q1',
        };
      });
    }

    const rs = this.resultsService.currentResultSet();
    if (!rs) return [];

    return rs.member_results.map(mr => {
      const elem = elemMap.get(mr.element_id);
      const nodeI = elem && isLineElement(elem) ? elem.node_i : '-';
      const nodeJ = elem && isLineElement(elem) ? elem.node_j : '-';
      const ni = elem && isLineElement(elem) ? nodeMap.get(elem.node_i) : null;
      const nj = elem && isLineElement(elem) ? nodeMap.get(elem.node_j) : null;
      const calcLen = ni && nj ? Math.hypot(nj.x - ni.x, nj.y - ni.y) : 0;
      return {
        elementId: mr.element_id,
        nodeI,
        nodeJ,
        lengthM: Math.round((mr.length_eff_m || calcLen) * 100) / 100,
        ni: Math.round(mr.end_forces_local.ni_kn * 10) / 10,
        qi: Math.round(mr.end_forces_local.qi_kn * 10) / 10,
        mi: Math.round(mr.end_forces_local.mi_knm * 10) / 10,
        nj: Math.round(mr.end_forces_local.nj_kn * 10) / 10,
        qj: Math.round(mr.end_forces_local.qj_kn * 10) / 10,
        mj: Math.round(mr.end_forces_local.mj_knm * 10) / 10,
        governing: '-',
      };
    });
  });

  toggleCollapse(): void {
    this.isCollapsed.update(c => !c);
  }
}
