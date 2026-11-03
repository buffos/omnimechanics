import { describe, it, expect, beforeEach } from 'vitest';
import { TeachingHandoffService } from './teaching-handoff.service';
import { DiagramExportService } from './diagram-export.service';
import { ProjectDocument, StructuralModel } from './handoff.types';

describe('Mode 3 Teaching Classifier, Handoff & Diagram Export (Issue 026)', () => {
  let handoffService: TeachingHandoffService;
  let diagramExportService: DiagramExportService;

  beforeEach(() => {
    handoffService = new TeachingHandoffService();
    diagramExportService = new DiagramExportService();
  });

  function createSimpleBeamModel(): StructuralModel {
    return {
      id: 'model-1',
      name: 'Simply Supported Beam',
      revision: 1,
      gravity_g: 9.81,
      nodes: [
        { id: 'N1', x: 0, y: 0 },
        { id: 'N2', x: 6, y: 0 },
      ],
      materials: [
        {
          id: 'MAT1',
          name: 'Steel',
          source: { kind: 'Preset', catalog_id: 'S355' },
          class: 'Steel',
          e_gpa: 210,
          g_gpa: 81,
          density_gamma: 78.5,
          alpha_t: 1.2e-5,
        },
      ],
      sections: [
        {
          id: 'SEC1',
          name: 'IPE 300',
          source: { kind: 'Preset', catalog_id: 'IPE_300' },
          shape: 'Ipe',
          area_a: 0.00538,
          inertia_i: 0.00008356,
          depth_h: 0.30,
        },
      ],
      elements: [
        {
          id: 'E1',
          kind: 'FrameMember',
          node_i: 'N1',
          node_j: 'N2',
          section_id: 'SEC1',
          material_id: 'MAT1',
          release_i: { axial: false, shear: false, moment: false },
          release_j: { axial: false, shear: false, moment: false },
          offset_i: { dx: 0, dy: 0, frame: 'Local' },
          offset_j: { dx: 0, dy: 0, frame: 'Local' },
        },
      ],
      supports: [
        { id: 'S1', node_id: 'N1', kind: { type: 'Pinned' }, unilateral: false },
        { id: 'S2', node_id: 'N2', kind: { type: 'Roller', angle_deg: 0 }, unilateral: false },
      ],
      load_cases: [
        {
          id: 'LC1',
          name: 'G',
          action_type: { kind: 'Permanent' },
          point_loads: [],
          dist_loads: [
            {
              id: 'L1',
              element_id: 'E1',
              qx1: 0,
              qy1: -12,
              qx2: 0,
              qy2: -12,
              a_rel: 0,
              b_rel: 1,
            },
          ],
          nodal_loads: [],
          thermal_loads: [],
          settlements: [],
        },
      ],
      lumped_masses: [],
    };
  }

  it('SC-011: classifies simply supported beam as eligible for Isostatic Beams & Frames', async () => {
    const model = createSimpleBeamModel();
    const eligibility = await handoffService.classifyModel(model);

    expect(eligibility.is_stable).toBe(true);
    expect(eligibility.indeterminacy_degree).toBe(0);
    expect(eligibility.overall_eligible).toBe(true);
    expect(eligibility.dsm_walkthrough_available).toBe(true);

    const isostatic = eligibility.methods.find((m) => m.method_id === 'IsostaticBeamFrame');
    expect(isostatic?.eligible).toBe(true);

    const force = eligibility.methods.find((m) => m.method_id === 'ForceMethod');
    expect(force?.eligible).toBe(false);
  });

  it('SC-012: unconditionally marks classical methods ineligible when Cable elements exist (BR-5)', async () => {
    const model = createSimpleBeamModel();
    model.elements.push({
      id: 'E2',
      kind: 'Cable',
      node_i: 'N1',
      node_j: 'N2',
      section_id: 'SEC1',
      material_id: 'MAT1',
      prestress: { mode: 'Pretension', t0: 50 },
    });

    const eligibility = await handoffService.classifyModel(model);
    expect(eligibility.overall_eligible).toBe(false);
    expect(eligibility.dsm_walkthrough_available).toBe(true);

    for (const m of eligibility.methods) {
      expect(m.eligible).toBe(false);
      expect(m.ineligibility_reason_el).toContain('καλωδίων');
      expect(m.ineligibility_reason_en).toContain('cable');
    }

    // Classical handoff attempt fails
    await expect(handoffService.openStepByStep(model, 'IsostaticBeamFrame')).rejects.toThrow();

    // DSM walkthrough succeeds
    const dsmRes = await handoffService.openStepByStep(model, 'dsm_walkthrough');
    expect(dsmRes.selected_method).toBe('dsm_walkthrough');
  });

  it('SC-012: unconditionally marks classical methods ineligible when unilateral supports exist', async () => {
    const model = createSimpleBeamModel();
    model.supports[1].unilateral = true;

    const eligibility = await handoffService.classifyModel(model);
    expect(eligibility.overall_eligible).toBe(false);
    expect(eligibility.dsm_walkthrough_available).toBe(true);

    const isostatic = eligibility.methods.find((m) => m.method_id === 'IsostaticBeamFrame');
    expect(isostatic?.eligible).toBe(false);
    expect(isostatic?.ineligibility_reason_el).toContain('μονόπλευρες στηρίξεις');
  });

  it('SC-013: imports Mode 3 presets into Mode 2 as editable StructuralModel (revision: 1)', async () => {
    const model = await handoffService.importPreset('simply_supported_point', {
      length: 8.0,
      load_p: 40.0,
      load_p_pos: 4.0,
    });

    expect(model.revision).toBe(1);
    expect(model.nodes.length).toBe(2);
    expect(model.elements.length).toBe(1);
    expect(model.elements[0].kind).toBe('FrameMember');
    expect(model.supports.length).toBe(2);
    expect(model.load_cases.length).toBe(1);
  });

  it('SC-014: exports standalone publication-grade SVG with title block and legend', () => {
    const svgStr = diagramExportService.generateStandaloneSvg(
      { innerHtml: '<line x1="0" y1="0" x2="600" y2="0" stroke="black"/>', width: 1000, height: 600 },
      'Continuous Beam Benchmark',
      'M',
      'el'
    );

    expect(svgStr).toContain('<?xml version="1.0" encoding="UTF-8"');
    expect(svgStr).toContain('Continuous Beam Benchmark');
    expect(svgStr).toContain('Διάγραμμα Καμπτικών Ροπών M(x)');
    expect(svgStr).toContain('Ίνα Εφελκυσμού κάτω');
    expect(svgStr).toContain('publication-overlay');
  });

  it('SC-014: performs lossless JSON project document (.bem) import and export round-trip', () => {
    const model = createSimpleBeamModel();
    const doc: ProjectDocument = {
      version: '2.0.0',
      structural: model,
      combinations: [],
      seismic_settings: null,
      viewport: { panX: 50, panY: 100, scale: 80, gridStep: 0.5 },
    };

    const jsonStr = JSON.stringify(doc);
    const imported = diagramExportService.importProjectFile(jsonStr);

    expect(imported.version).toBe('2.0.0');
    expect(imported.structural.nodes.length).toBe(2);
    expect(imported.structural.elements.length).toBe(1);
    expect(imported.viewport.scale).toBe(80);
  });
});
