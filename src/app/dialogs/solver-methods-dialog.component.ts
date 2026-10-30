import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { injectDialogRef } from '../shared/dialog.service';

@Component({
  selector: 'app-solver-methods-dialog',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="methods-dialog-content">
      <div class="legend-pills">
        <div class="pill-item core">
          <span class="pill-dot"></span>
          <span><strong>OUR Core Invention:</strong> Operator Splitting (E₀ + K_pert)u = F on single body</span>
        </div>
        <div class="pill-item ddm">
          <span class="pill-dot"></span>
          <span><strong>Subdomain DDM:</strong> Structure (FEM) + Soil/Medium (BEM) with E₁ ≠ E₂</span>
        </div>
        <div class="pill-item benchmark">
          <span class="pill-dot"></span>
          <span><strong>Reference Baselines:</strong> Pure FEM / Pure BEM / Newmark-β for validation</span>
        </div>
      </div>

      <div class="table-scroll-container">
        <table class="methods-table">
          <thead>
            <tr>
              <th>Solver Mode</th>
              <th>Domain</th>
              <th>Classification</th>
              <th>Physical Formulation &amp; Role in Monograph</th>
            </tr>
          </thead>
          <tbody>
            <!-- Static BEMFEM -->
            <tr class="row-core">
              <td class="col-name"><strong>BEM Predictor &middot; FEM Corrector</strong></td>
              <td><span class="domain-tag static">Static</span></td>
              <td><span class="badge core">OUR METHOD (CORE)</span></td>
              <td class="col-desc">
                <strong>Our Primary Method (Sections 1–6):</strong> Single inhomogeneous body in air.
                BEM solves homogeneous linear reference <em>E₀</em> on boundary &Gamma;; variational matrix <em>S_shape</em> couples surface tractions energetically; FEM evaluates internal volume corrections <em>K_pert</em> strictly at Gauss points without global matrix inversion.
              </td>
            </tr>


            <!-- Pure FEM -->
            <tr class="row-benchmark">
              <td class="col-name"><strong>Pure FEM</strong></td>
              <td><span class="domain-tag static">Static</span></td>
              <td><span class="badge benchmark">BASELINE</span></td>
              <td class="col-desc">
                <strong>Standard Finite Elements:</strong> Solves <em>Ku = F</em> directly via PCG. Serves as ground-truth baseline to verify accuracy (<em>L₂</em> error norm) and measure speedup ratios in the paper.
              </td>
            </tr>

            <!-- Pure BEM -->
            <tr class="row-benchmark">
              <td class="col-name"><strong>Pure BEM</strong></td>
              <td><span class="domain-tag static">Static</span></td>
              <td><span class="badge benchmark">BASELINE</span></td>
              <td class="col-desc">
                <strong>Classical Boundary Elements:</strong> Solves <em>Hu = Gt</em>. Strictly restricted to homogeneous domains; cannot handle general volume inhomogeneities without cell integration (the Somigliana bottleneck).
              </td>
            </tr>

            <!-- Standard GMRES Harmonic FEM -->
            <tr class="row-benchmark">
              <td class="col-name"><strong>Harmonic Pure FEM</strong></td>
              <td><span class="domain-tag harmonic">Harmonic</span></td>
              <td><span class="badge benchmark">FEM FREQUENCY</span></td>
              <td class="col-desc">
                <strong>Standalone FEM Frequency Solver:</strong> Complex GMRES solver for dynamic operator <em>(K₀ + s²M)u = F(&omega;)</em> across all elements (2D Tri3/Quad4 and 3D Tet4/Hex8) with structural damping and exact Dirichlet degree-of-freedom filtering.
              </td>
            </tr>

            <!-- Pure BEM Harmonic -->
            <tr class="row-benchmark">
              <td class="col-name"><strong>Harmonic Pure BEM</strong></td>
              <td><span class="domain-tag harmonic">Harmonic</span></td>
              <td><span class="badge benchmark">BEM FREQUENCY</span></td>
              <td class="col-desc">
                <strong>Standalone BEM Frequency Solver:</strong> Boundary integral formulation using dynamic Kelvin fundamental solutions <em>U*(r, s)</em> and <em>T*(r, s)</em> (modified Bessel functions in 2D, spherical wave potentials in 3D) with interior displacement and stress evaluation via frequency-domain Somigliana identity.
              </td>
            </tr>

            <!-- Robin BEM-FEM Dynamic -->
            <tr class="row-ddm">
              <td class="col-name"><strong>Robin Transmission (Soil-Structure)</strong></td>
              <td><span class="domain-tag harmonic">Harmonic</span></td>
              <td><span class="badge ddm">SUBDOMAIN DDM (SSI)</span></td>
              <td class="col-desc">
                <strong>Domain Decomposition &middot; Soil-Structure Interaction:</strong> Couples structural FEM with unbounded soil/medium BEM via anisotropic dynamic impedance operator <em>Z(s) = &rho; s [c_p (n &otimes; n) + c_s (I - n &otimes; n)]</em> and boundary radiation damping matrix <em>C_Robin</em>.
              </td>
            </tr>

            <!-- Talbot Transient -->
            <tr class="row-core">
              <td class="col-name"><strong>Deformed Talbot Contour (Laplace)</strong></td>
              <td><span class="domain-tag transient">Transient</span></td>
              <td><span class="badge core">OUR DYNAMIC PARADIGM</span></td>
              <td class="col-desc">
                <strong>Laplace-Domain Method (Section 8):</strong> Bypasses step-by-step serial integration and cumulative dispersion. Solves <em>(E₀(s) + K_pert)u = F(s)</em> in parallel across 32 hyperbolic contour nodes in the complex s-plane with zero history memory.
              </td>
            </tr>

            <!-- Newmark-beta Transient -->
            <tr class="row-benchmark">
              <td class="col-name"><strong>Newmark-&beta; / HHT-&alpha; Direct Marching</strong></td>
              <td><span class="domain-tag transient">Transient</span></td>
              <td><span class="badge benchmark">BASELINE (GOLD STANDARD)</span></td>
              <td class="col-desc">
                <strong>Classical Direct Time-Marching (Section 10):</strong> Step-by-step implicit integration with pre-factored effective dynamic stiffness <em>K_eff</em>. Essential academic reference benchmark (Bathe/Chopra) to prove convergence of transformed-domain methods.
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <div class="dialog-actions">
        <button type="button" class="btn btn-primary" (click)="close()">
          Understood &middot; Close Guide
        </button>
      </div>
    </div>
  `,
  styles: [`
    .methods-dialog-content {
      display: flex;
      flex-direction: column;
      gap: 16px;
      width: 100%;
      max-width: 100%;
      font-size: 0.85rem;
      color: #cbd5e1;
    }

    .legend-pills {
      display: flex;
      flex-wrap: wrap;
      gap: 12px;
      padding: 10px 16px;
      background: rgba(15, 23, 42, 0.7);
      border: 1px solid rgba(255, 255, 255, 0.08);
      border-radius: 8px;
    }

    .pill-item {
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: 0.775rem;
    }

    .pill-dot {
      width: 8px;
      height: 8px;
      border-radius: 50%;
    }

    .pill-item.core .pill-dot { background: #38bdf8; box-shadow: 0 0 6px #38bdf8; }
    .pill-item.ddm .pill-dot { background: #a855f7; box-shadow: 0 0 6px #a855f7; }
    .pill-item.benchmark .pill-dot { background: #94a3b8; }

    .table-scroll-container {
      max-height: 52vh;
      overflow-y: auto;
      overflow-x: hidden;
      border: 1px solid rgba(255, 255, 255, 0.1);
      border-radius: 8px;
      background: rgba(15, 23, 42, 0.5);
    }

    .methods-table {
      width: 100%;
      border-collapse: collapse;
      text-align: left;
      table-layout: fixed;
    }

    .methods-table th:nth-child(1), .methods-table td:nth-child(1) { width: 22%; }
    .methods-table th:nth-child(2), .methods-table td:nth-child(2) { width: 11%; text-align: center; }
    .methods-table th:nth-child(3), .methods-table td:nth-child(3) { width: 19%; }
    .methods-table th:nth-child(4), .methods-table td:nth-child(4) { width: 48%; }

    .methods-table th {
      position: sticky;
      top: 0;
      background: #1e293b;
      color: #f1f5f9;
      font-weight: 700;
      font-size: 0.775rem;
      text-transform: uppercase;
      letter-spacing: 0.5px;
      padding: 10px 14px;
      border-bottom: 2px solid rgba(255, 255, 255, 0.12);
      z-index: 2;
    }

    .methods-table td {
      padding: 12px 14px;
      border-bottom: 1px solid rgba(255, 255, 255, 0.06);
      vertical-align: top;
      line-height: 1.45;
    }

    .row-core { background: rgba(56, 189, 248, 0.03); }
    .row-ddm { background: rgba(168, 85, 247, 0.03); }
    .row-benchmark { background: transparent; }

    .col-name {
      color: #f8fafc;
      font-weight: 600;
    }

    .col-desc {
      font-size: 0.8rem;
      color: #94a3b8;
    }

    .col-desc strong {
      color: #e2e8f0;
    }

    .badge {
      display: inline-block;
      font-size: 0.675rem;
      font-weight: 700;
      padding: 2px 8px;
      border-radius: 12px;
      white-space: nowrap;
    }

    .badge.core {
      background: rgba(56, 189, 248, 0.15);
      color: #38bdf8;
      border: 1px solid rgba(56, 189, 248, 0.3);
    }

    .badge.ddm {
      background: rgba(168, 85, 247, 0.15);
      color: #c084fc;
      border: 1px solid rgba(168, 85, 247, 0.3);
    }

    .badge.benchmark {
      background: rgba(148, 163, 184, 0.12);
      color: #94a3b8;
      border: 1px solid rgba(148, 163, 184, 0.25);
    }

    .domain-tag {
      font-size: 0.7rem;
      font-weight: 600;
      padding: 2px 6px;
      border-radius: 4px;
      text-transform: uppercase;
    }

    .domain-tag.static { background: #0369a1; color: #e0f2fe; }
    .domain-tag.harmonic { background: #4338ca; color: #e0e7ff; }
    .domain-tag.transient { background: #0f766e; color: #ccfbf1; }

    .dialog-actions {
      display: flex;
      justify-content: flex-end;
      padding-top: 8px;
    }

    .btn-primary {
      background: #0284c7;
      color: #ffffff;
      border: none;
      padding: 8px 18px;
      border-radius: 6px;
      font-weight: 600;
      cursor: pointer;
      transition: background 0.15s ease;
    }

    .btn-primary:hover {
      background: #0369a1;
    }
  `],
})
export class SolverMethodsDialogComponent {
  private readonly dialogRef = injectDialogRef<void>();

  close(): void {
    this.dialogRef.close();
  }
}
