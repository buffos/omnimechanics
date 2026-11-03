## Issue Metadata

- Issue number: `036`
- Owning capability node: `/.okf/capabilities/cap-geometry-cad-import.md`
- Artifact root: `.okf/capabilities/cap-geometry-cad-import.md`
- Issue file: `docs/agents/issues/pending/20261103-036-cad-2d-dxf-svg-parser-polyline-healing-handoff.md`
- Category: `feature`
- Execution type: `AFK`
- Review gate: `visual-review`
- Suggested state: `ready-for-agent`

## Parent Artifacts

- `docs/prd.md`
- `docs/architecture/application-architecture-summary.md`
- `docs/architecture/cross-capability-conventions.md`
- `.okf/capabilities/cap-geometry-cad-import.md`
- `.okf/capabilities/cap-geometry-meshing.md`

## What to build

Implement **Vertical Slice 1** of `cap-geometry-cad-import`: the 2D vector CAD ingestion and automated polyline healing engine in Rust (`src-tauri/src/cad/`) for **DXF** (`LINE`, `LWPOLYLINE`, `POLYLINE`, `CIRCLE`, `ARC` with layer filtering) and **SVG** paths, snapping micro-gaps within configurable tolerance $\epsilon$, consolidating duplicate vertices, and handing off healed 2D frame wireframes into Mode 2 (`StructuralModel`) / Mode 3 (`Scenario 1`) and closed 2D contours into the Mode 1 Continuum Sketcher / CDT mesher (`Scenario 2`).

### 1. 2D DXF & SVG Parser with Layer Filtering (`src-tauri/src/cad/dxf_svg_parser.rs`):
- Parse ASCII DXF (`ENTITIES` section: `LINE`, `LWPOLYLINE`, `POLYLINE`, `CIRCLE`, `ARC`) and SVG `<path>`, `<polyline>`, `<polygon>`, `<line>`, `<rect>`, `<circle>` elements.
- Extract `CadLayerSummary` records so users can filter structural layers (e.g., `"BEAMS"`, `"WALLS"`) and ignore annotation/dimension layers (`"DIMENSIONS"`, `"TEXT"`) (`Scenario 1`).
- Discretize circular arcs and curves via configurable chordal tolerance.

### 2. Automated Polyline Healing & Topological Classification (`src-tauri/src/cad/healing_2d.rs`):
- Snap non-touching segment endpoints within configurable tolerance $\epsilon$ (default $0.5\text{ mm}$ / $0.0005\text{ m}$), consolidate coincident vertices, chain collinear segments, and detect member-to-member joint intersections (`Scenario 1`, `Scenario 2`).
- Classify closed loops into outer boundaries (`is_outer_boundary: true`, CCW) and internal holes (`is_outer_boundary: false`, CW) using winding numbers (`HealedContour2D`).
- Register Tauri IPC commands `cad_inspect_layers`, `cad_import_2d`, and `cad_handoff_to_frame_or_continuum`.

### 3. CAD Import Modal & Handoff UI (`src/app/features/cad-import/`):
- Provide a **CAD Import Dialog** accessible from both Mode 2's toolbar (`Import DXF/SVG Frame`) and Mode 1's Geometry tab (`Import DXF/SVG Continuum Contour`).
- Allow the user to preview imported layers, toggle active layers, adjust healing tolerance $\epsilon$, inspect healed joint/loop counts and warnings, and click **"Import into Mode 2 Frame Modeler"** or **"Import into Mode 1 Continuum Mesher"**.

## Acceptance criteria

- [ ] Importing a 2D DXF file with centerlines on layer `"BEAMS"` and annotations on `"DIMENSIONS"` filters out `"DIMENSIONS"`, chains collinear segments, detects joint intersections, and populates an editable `StructuralModel` in Mode 2 / Mode 3 (`Scenario 1`).
- [ ] Importing a 2D DXF/SVG rectangle with a $0.2\text{ mm}$ corner micro-gap under healing tolerance $\epsilon = 0.5\text{ mm}$ snaps the gap closed, marks the contour `is_outer_boundary: true`, and meshes cleanly in the Constrained Delaunay triangulator without boundary leak errors (`Scenario 2`).
- [ ] Rust unit tests (`cad_2d_test.rs`), Vitest tests (`cad-import.spec.ts`), `npx ng build`, and `npm run check:limits` pass with 0 errors.

## Artifact sync required

- Application PRD: `none - already covered in docs/prd.md`
- Application architecture summary: `none - matches docs/architecture/application-architecture-summary.md`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-geometry-cad-import.md`
- Issue registry: `required`; node `issues:` reference: `required: .okf/capabilities/cap-geometry-cad-import.md`
- Reason/no-impact decision: `Delivers 2D DXF/SVG ingestion, layer filtering, and polyline healing for Mode 1 and Mode 2/3.`

## Human review gate

`visual-review`: User opens the CAD Import dialog, loads a 2D DXF/SVG file, selects layer `"BEAMS"`, previews healed micro-gaps and intersections, and imports the geometry into Mode 2 (as frame members) and Mode 1 (as a closed continuum contour).

## Blocked by

- Blocked by `docs/agents/issues/pending/20261103-028-conformal-cdt-mesher-boundary-tagging-inclusions.md`

## Artifact anchors

- Capability spec (`.okf/capabilities/cap-geometry-cad-import.md`): Scope §1 (2D Vector CAD Ingestion & Automated Polyline Healing), §3 (Handoff Pipeline), `CadImportResult`, `HealedContour2D`

## Acceptance scenarios addressed

- `Scenario 1`: Importing 2D DXF Floor Beam Layout
- `Scenario 2`: Polyline Healing with Micro-Gaps

## Verification obligations

- Policy source: `/.okf/project.md`

| Scenario | Backend boundary | Frontend integration | End-to-end journey |
|---|---|---|---|
| `Scenario 1` | `planned` | `planned` | `catalog-only` |
| `Scenario 2` | `planned` | `planned` | `catalog-only` |
