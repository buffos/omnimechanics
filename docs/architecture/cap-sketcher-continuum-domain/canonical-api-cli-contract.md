# Canonical API, Tauri IPC & CLI Contract: Continuum Domain Sketcher (`cap-sketcher-continuum-domain`)

- **Owning Capability:** [`cap-sketcher-continuum-domain`](../../../.okf/capabilities/cap-sketcher-continuum-domain.md)
- **Upstream Artifacts:** [`domain-glossary.md`](domain-glossary.md), [`prd.md`](prd.md), [`canonical-domain-model.md`](canonical-domain-model.md), [`canonical-use-cases.md`](canonical-use-cases.md)
- **Shared Dependencies:** [`cap-geometry-presets`](../../../.okf/capabilities/cap-geometry-presets.md), [`cap-solver-core`](../../../.okf/capabilities/cap-solver-core.md)
- **Binding Conventions:** [`cross-capability-conventions.md`](../cross-capability-conventions.md)

---

## 1. Purpose & Contract Goals

This document defines the canonical external contract for `cap-sketcher-continuum-domain` across Tauri IPC (`continuum_*` commands), HTTP/REST equivalents, and CLI batch commands. It guarantees that any client (Angular UI, automated test runner, or CLI script) exchanges identical `ContinuumDomain` and `ConformalMeshHandoff` DTOs and observes identical bilingual error codes (`SCREAMING_SNAKE_CASE`).

---

## 2. Transport-Neutral Conventions

1. **Units:** All spatial coordinates $(x, y)$, radii $R$, dimensions $(w, h)$, and element sizes $h$ are in metres ($\text{m}$). Angles are in degrees (`deg`). Tractions in `ContinuumDomain` are in $\text{kN/m}^2$; Robin spring stiffnesses are in $\text{kN/m}^3$; inclusion Young's modulus overrides are entered in $\text{GPa}$ and converted at the solver boundary.
2. **Identifiers:** Stable per-domain string IDs:
   - Domain: `"CD1"`
   - Vertices: `"V1"`, `"V2"`, ...
   - Segments: `"S1"`, `"S2"`, ...
   - Cutouts: `"H1"`, `"H2"`, ...
   - Inclusions: `"INC1"`, `"INC2"`, ...
3. **Revision Integrity:** `ContinuumDomain.revision` is an unsigned 64-bit integer (`u64`) starting at `1`. Every generated `ConformalMeshHandoff` carries `domain_revision: u64`.
4. **Conformal `MeshData` Contract:** `ConformalMeshHandoff.mesh_data` uses the exact `crate::mesh::types::MeshData` schema consumed by `cap-solver-core` (`dimension: 2`, `nodes: Vec<[f64; 3]>`, `boundary_elements: Vec<BoundaryElement>`, `volume_elements: Vec<VolumeElement>`, `stats: MeshStats`).

---

## 3. Canonical Payload Shapes (JSON / Serde DTOs)

### 3.1 `ContinuumDomain` Request/State Payload
```json
{
  "id": "CD1",
  "revision": 3,
  "outer_boundary": {
    "id": "L_OUTER",
    "orientation": "CounterClockwise",
    "closure_state": "ClosedValid",
    "vertices": [
      { "id": "V1", "x": -5.0, "y": -5.0 },
      { "id": "V2", "x": 5.0, "y": -5.0 },
      { "id": "V3", "x": 5.0, "y": 5.0 },
      { "id": "V4", "x": -5.0, "y": 5.0 }
    ],
    "segments": [
      { "id": "S1", "start_vertex_id": "V1", "end_vertex_id": "V2", "tag": "Bottom", "local_element_size": null, "outward_normal": [0.0, -1.0, 0.0] },
      { "id": "S2", "start_vertex_id": "V2", "end_vertex_id": "V3", "tag": "Right", "local_element_size": null, "outward_normal": [1.0, 0.0, 0.0] },
      { "id": "S3", "start_vertex_id": "V3", "end_vertex_id": "V4", "tag": "Top", "local_element_size": null, "outward_normal": [0.0, 1.0, 0.0] },
      { "id": "S4", "start_vertex_id": "V4", "end_vertex_id": "V1", "tag": "Left", "local_element_size": null, "outward_normal": [-1.0, 0.0, 0.0] }
    ]
  },
  "cutouts": [
    {
      "id": "H1",
      "tag": "Hole",
      "local_element_size": 0.25,
      "shape": {
        "kind": "Circle",
        "center_x": 0.0,
        "center_y": 0.0,
        "radius": 1.5,
        "angular_divisions": 32
      }
    }
  ],
  "inclusions": [],
  "tag_conditions": {
    "Left": { "kind": "DirichletDisplacement", "fix_x": true, "fix_y": true, "ux_m": 0.0, "uy_m": 0.0 },
    "Right": { "kind": "NeumannTraction", "tx_kn_m2": 10.0, "ty_kn_m2": 0.0, "coordinate_frame": "Global" },
    "Top": { "kind": "Free" },
    "Bottom": { "kind": "Free" },
    "Hole": { "kind": "Free" }
  },
  "meshing_profile": {
    "target_element_size": 0.75,
    "hole_refinement_factor": 0.35,
    "min_angle_deg": 25.0,
    "volume_element_type": "Tri3",
    "boundary_element_type": "Line2"
  }
}
```

### 3.2 `CutoutShape` Discriminated Union (`#[serde(tag = "kind")]`)
- **`Circle`:** `{ "kind": "Circle", "center_x": 0.0, "center_y": 0.0, "radius": 1.5, "angular_divisions": 32 }`
- **`Rectangle`:** `{ "kind": "Rectangle", "center_x": 1.0, "center_y": 1.0, "width": 2.0, "height": 1.0, "rotation_deg": 0.0 }`
- **`Polygon`:** `{ "kind": "Polygon", "vertices": [ { "id": "HV1", "x": -1.0, "y": -1.0 }, ... ] }`

### 3.3 `ConformalMeshHandoff` Response Payload
```json
{
  "domain_id": "CD1",
  "domain_revision": 3,
  "mesh_data": {
    "dimension": 2,
    "nodes": [[-5.0, -5.0, 0.0], [5.0, -5.0, 0.0]],
    "boundary_elements": [
      {
        "id": 0,
        "elem_type": "Line2",
        "nodes": [0, 1],
        "tag": "Bottom",
        "normal": [0.0, -1.0, 0.0]
      }
    ],
    "volume_elements": [
      {
        "id": 0,
        "elem_type": "Tri3",
        "nodes": [0, 1, 14]
      }
    ],
    "stats": {
      "num_nodes": 312,
      "num_boundary_elements": 88,
      "num_volume_elements": 536,
      "dimension": 2
    }
  },
  "quality_report": {
    "num_nodes": 312,
    "num_boundary_elements": 88,
    "num_volume_elements": 536,
    "min_angle_deg": 26.4,
    "max_angle_deg": 118.2,
    "min_aspect_ratio": 1.02,
    "max_aspect_ratio": 2.45,
    "is_conformal": true,
    "outward_normals_verified": true
  },
  "tag_element_counts": {
    "Bottom": 14,
    "Right": 14,
    "Top": 14,
    "Left": 14,
    "Hole": 32
  }
}
```

---

## 4. Canonical Error Model (`cross-capability-conventions.md` §2)

All Tauri IPC and API errors conform to:
```json
{
  "code": "SELF_INTERSECTING_LOOP",
  "message_el": "Το περίγραμμα τέμνει τον εαυτό του μεταξύ των τμημάτων S2 και S4.",
  "message_en": "Boundary loop self-intersects between segments S2 and S4.",
  "details": {
    "loop_id": "L_OUTER",
    "segment_a": "S2",
    "segment_b": "S4",
    "intersection_point": [1.25, 0.80]
  }
}
```

### Canonical Error Codes Owned by `cap-sketcher-continuum-domain`

| Error Code | Trigger Condition | HTTP Status | CLI Exit Code |
| --- | --- | --- | --- |
| `OPEN_BOUNDARY_LOOP` | Outer boundary or polygonal cutout has fewer than 3 vertices or is not closed. | `422` | `2` |
| `SELF_INTERSECTING_LOOP` | Two non-adjacent segments of a loop intersect or a vertex lies on a non-incident edge. | `422` | `2` |
| `ZERO_LENGTH_SEGMENT` | Any boundary segment length $\ell < 10^{-4}\text{ m}$ or adjacent segments fold at $180^\circ$. | `422` | `2` |
| `DEGENERATE_POLYGON_AREA` | Absolute signed polygon area $|A| < 10^{-6}\text{ m}^2$. | `422` | `2` |
| `HOLE_OUTSIDE_OUTER_BOUNDARY` | Any point or edge of a `Cutout` or `InclusionRegion` lies outside or touches the `outer_boundary`. | `422` | `2` |
| `HOLE_OVERLAPS_EXISTING_FEATURE` | Two cutouts or inclusions intersect, or a feature is nested inside a cutout void. | `422` | `2` |
| `CLEARANCE_BELOW_MINIMUM` | Minimum distance between any cutout/inclusion and the outer boundary (or another cutout) is $< d_{\min}$. | `422` | `2` |
| `INVALID_BOUNDARY_CONDITION` | Non-finite traction/displacement values or negative Robin impedance ($k_n < 0$, $c_n < 0$). | `400` | `2` |
| `INVALID_MESHING_PROFILE` | $h \le 0$, $\alpha_{\text{hole}} \notin [0.1, 1.0]$, or $\theta_{\min} \notin [15.0, 32.0]$. | `400` | `2` |
| `CDT_CONVERGENCE_FAILURE` | Constrained Delaunay refinement exceeded maximum Steiner point limit ($50{,}000$ nodes) or failed conformity check. | `500` | `3` |
| `STALE_MESH_HANDOFF` | Attempted to hand off a `ConformalMeshHandoff` whose `domain_revision != domain.revision`. | `409` | `4` |

---

## 5. Canonical Tauri IPC Commands (`continuum_*`)

Per `cross-capability-conventions.md` §2, all Tauri commands for this capability use the `continuum_` prefix:

| Tauri Command Name | Maps to Use Case | Request Arguments | Response Type |
| --- | --- | --- | --- |
| `continuum_validate_domain` | `UC-QRY-01`, `UC-CMD-01..06` | `{ domain: ContinuumDomain }` | `Result<DomainValidationResponse, IpcError>` |
| `continuum_preview_segment` | `UC-QRY-02` | `{ existing_vertices: Vec<Vertex2D>, candidate_point: Vertex2D, closing_loop: bool }` | `Result<CandidateSegmentPreview, IpcError>` |
| `continuum_close_and_normalize_loop` | `UC-CMD-01` | `{ loop_id: String, vertices: Vec<Vertex2D>, is_hole: bool }` | `Result<BoundaryLoop, IpcError>` |
| `continuum_generate_mesh` | `UC-CMD-07` | `{ domain: ContinuumDomain }` | `Result<ConformalMeshHandoff, IpcError>` |

### Integration with Mode 1 Solver Commands (`cap-solver-core`)
Once `continuum_generate_mesh` returns `ConformalMeshHandoff`, its `mesh_data: MeshData` field is passed directly as `request.mesh` into the existing Mode 1 Tauri commands:
- `run_static_simulation(request: SimulationRequest)`
- `run_harmonic_simulation(request: HarmonicSimulationRequest)`
- `run_transient_simulation(request: TransientSimulationRequest)`

---

## 6. Canonical HTTP & CLI Mapping

### 6.1 HTTP Mapping (Headless / Web Runner Equivalence)
- `POST /api/v1/continuum/validate` $\to$ `continuum_validate_domain`
- `POST /api/v1/continuum/loops/close` $\to$ `continuum_close_and_normalize_loop`
- `POST /api/v1/continuum/mesh` $\to$ `continuum_generate_mesh`

### 6.2 CLI Mapping (Headless Verification & Batch Benchmarking)
- `omnimech continuum validate --input <domain.json> [--lang el|en]`
  - Validates `ContinuumDomain` JSON; prints `DomainValidationResponse` JSON and exits `0` if `ValidForMeshing`, `2` on domain validation error.
- `omnimech continuum mesh --input <domain.json> --output <mesh-handoff.json> [--h <float>]`
  - Validates and runs CDT; writes `ConformalMeshHandoff` JSON and exits `0` if conformal, `2` on invalid geometry, `3` on meshing error.

---

## 7. Parity & Testing Contract

1. **Deterministic CDT Output:** Given identical `ContinuumDomain` JSON, `continuum_generate_mesh` must return identical node coordinates, element connectivity, outward normals, and `MeshQualityReport` across Tauri IPC and Rust unit tests.
2. **Preset Equivalence Check:** A $10 \times 10\text{ m}$ square `ContinuumDomain` with a central $R = 1.5\text{ m}$ `CircularCutout` and standard semantic tags (`Left`, `Right`, `Top`, `Bottom`, `Hole`) must produce a `MeshData` payload that solves in `run_static_simulation` (`PureFEM`, `PureBEM`, `BEMFEM`) with the same boundary tag semantics as `MeshGeneratorRequest::PlateWithHole2D`.
