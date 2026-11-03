# Canonical API & CLI Contract: Project Persistence & Self-Contained Workspace Storage (`cap-project-persistence`)

- **Owning Capability:** [`cap-project-persistence`](../../../.okf/capabilities/cap-project-persistence.md)
- **Canonical Domain Model:** [`canonical-domain-model.md`](canonical-domain-model.md)
- **Canonical Use Cases:** [`canonical-use-cases.md`](canonical-use-cases.md)
- **Conventions Reference:** [`cross-capability-conventions.md`](../cross-capability-conventions.md)
- **Status:** Canonical External Contract (2026-11-01)

---

## 1. Purpose & Transport-Neutral Conventions

This document defines the canonical Tauri IPC command surface (`project_*` namespace), `.bem` JSON file wire schema, CLI batch subcommands, and bilingual error contracts for `cap-project-persistence`.

### 1.1 Transport-Neutral Rules
- **IPC Command Prefix:** All Tauri commands owned by this capability use the `project_` prefix (`cross-capability-conventions.md` §2).
- **Units & Coordinates:** SI units internally (`kN`, `m`, `kNm`, `kN/m`, `kN/m²`, `°C`, `Pa`/`GPa` per domain owner). Global `y` up, counter-clockwise rotation positive.
- **Canonical JSON Wire Rules:**
  - Encoding: UTF-8 without BOM.
  - Line endings: `\n`.
  - Indentation: 2 spaces.
  - Key ordering: Lexicographical sort on all JSON object maps before computing `integrity.payload_sha256` and writing to disk.
  - Numbers: Standard IEEE 754 finite numbers only (`NaN`, `Infinity`, `-Infinity` are strictly forbidden).
- **Timestamps:** RFC 3339 UTC strings (`"2026-11-01T19:00:00Z"`).

---

## 2. Canonical `.bem` `ProjectDocument` JSON Schema (`v1.0`)

```json
{
  "schema_version": "1.0",
  "metadata": {
    "project_id": "550e8400-e29b-41d4-a716-446655440000",
    "title": "Two-Bay Portal Frame (EN 1990 / EC8)",
    "author": "Dr. E. Papadopoulos",
    "description": "2D seismic frame benchmark with Greek NA",
    "created_at": "2026-11-01T10:15:00Z",
    "updated_at": "2026-11-01T11:42:10Z",
    "app_version": "0.2.0",
    "dimensions": "2D"
  },
  "integrity": {
    "algorithm": "SHA-256",
    "payload_sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
  },
  "active_mode": "StructuralModeler",
  "continuum": {
    "preset_id": "plate_with_circular_hole_2d",
    "geometry_params": { "width": 4.0, "height": 4.0, "radius": 0.5 },
    "mesh": null,
    "material_field": {
      "kind": "FunctionalGradation",
      "e0_gpa": 210.0,
      "nu0": 0.3,
      "rho0": 7850.0,
      "gradation_axis": "y",
      "alpha": 0.25
    },
    "boundary_conditions": {
      "regime": { "kind": "Static" },
      "prescribed_tractions": [{ "boundary_tag": "top", "tx": 0.0, "ty": 100.0 }],
      "prescribed_displacements": [{ "boundary_tag": "bottom", "ux": 0.0, "uy": 0.0 }]
    }
  },
  "continuum_sketch": {
    "domain": {
      "outer_loop": [{ "x": 0.0, "y": 0.0 }, { "x": 4.0, "y": 0.0 }, { "x": 4.0, "y": 3.0 }, { "x": 0.0, "y": 3.0 }],
      "holes": [],
      "boundary_tags": [{ "segment_index": 0, "tag": "bottom_fixed" }],
      "max_element_area": 0.05
    }
  },
  "structural": {
    "model": {
      "revision": 12,
      "nodes": [
        { "id": "N1", "x": 0.0, "y": 0.0 },
        { "id": "N2", "x": 0.0, "y": 4.0 },
        { "id": "N3", "x": 6.0, "y": 4.0 },
        { "id": "N4", "x": 6.0, "y": 0.0 }
      ],
      "materials": [
        { "id": "M1", "name": "S355", "e_gpa": 210.0, "g_gpa": 81.0, "density_kn_m3": 78.5, "alpha_t": 1.2e-5, "class": "Steel" }
      ],
      "sections": [
        { "id": "S1", "name": "IPE 300", "area_m2": 0.00538, "inertia_m4": 0.00008356, "depth_m": 0.3, "shape": "IProfile" }
      ],
      "elements": [
        {
          "id": "E1",
          "kind": "FrameMember",
          "node_i": "N1",
          "node_j": "N2",
          "material_id": "M1",
          "section_id": "S1",
          "end_release": { "axial_i": false, "shear_i": false, "moment_i": false, "axial_j": false, "shear_j": false, "moment_j": false },
          "rigid_offset": { "dx_i": 0.0, "dy_i": 0.0, "dx_j": 0.0, "dy_j": 0.0 }
        }
      ],
      "supports": [
        { "id": "SUP1", "node_id": "N1", "kind": { "type": "Fixed" }, "unilateral": false }
      ],
      "load_cases": [
        { "id": "LC1", "name": "Dead Load G", "action_type": { "kind": "Permanent" }, "include_self_weight": true },
        { "id": "LC2", "name": "Office Live Q", "action_type": { "kind": "Imposed", "category": "B" }, "include_self_weight": false }
      ],
      "loads": [
        {
          "id": "L1",
          "load_case_id": "LC1",
          "kind": { "type": "MemberDistributedLoad", "element_id": "E1", "q1": -15.0, "q2": -15.0, "a": 0.0, "b": 1.0, "frame": "Global" }
        }
      ],
      "masses": []
    },
    "combinations": [
      {
        "id": "CO1",
        "name": "1.35G + 1.50Q (ULS STR)",
        "limit_state": "ULS_STR",
        "origin": "Generated",
        "terms": [
          { "load_case_id": "LC1", "factor": 1.35 },
          { "load_case_id": "LC2", "factor": 1.50 }
        ]
      }
    ],
    "national_annex": {
      "code": "GR",
      "gamma_g_sup": 1.35,
      "gamma_g_inf": 1.00,
      "gamma_q": 1.50,
      "uls_equation_policy": "Eq6_10"
    },
    "seismic_settings": {
      "zone": "II",
      "ag_over_g": 0.24,
      "ground_type": "B",
      "importance_class": "II",
      "gamma_i": 1.0,
      "behaviour_factor_q": 3.5,
      "damping_ratio": 0.05,
      "modal_combination": "CQC"
    }
  },
  "solution_cache": null,
  "ui_state": {
    "language": "el",
    "viewport": { "pan_x": 120.0, "pan_y": 80.0, "zoom": 1.25 },
    "grid_snap": {
      "grid_step": 0.5,
      "snap_grid": true,
      "snap_node": true,
      "snap_midpoint": true,
      "snap_intersection": true,
      "ortho": false,
      "angle_snap_deg": 15.0
    },
    "diagram_scales": { "moment_scale": 1.0, "shear_scale": 1.0, "axial_scale": 1.0, "deformed_scale": 50.0 },
    "active_target": { "kind": "Combination", "id": "CO1" }
  }
}
```

---

## 3. Canonical Tauri IPC Command Surface (`project_*`)

| IPC Command Name | Maps to Use Case | Request Payload | Response Payload |
| :--- | :--- | :--- | :--- |
| `project_new` | `UC-C01 NewProject` | `{ active_mode: WorkspaceMode, title?: string, dimensions?: "2D" \| "3D", confirm_discard: boolean }` | `{ document: ProjectDocument, session: ProjectSessionState }` |
| `project_open` | `UC-C02 OpenProject` | `{ file_path?: string, confirm_discard: boolean }` | `{ status: "Loaded" \| "Cancelled", document?: ProjectDocument, session: ProjectSessionState, migration_report?: MigrationReport, stale_cache_stripped: boolean }` |
| `project_save` | `UC-C03 SaveProject` | `{ document_payload: ProjectPayloadInput, include_solution_cache: boolean }` | `{ status: "Saved" \| "Cancelled", session: ProjectSessionState, metadata?: ProjectMetadata, integrity?: DocumentIntegrity, bytes_written?: number }` |
| `project_save_as` | `UC-C04 SaveProjectAs` | `{ target_path?: string, document_payload: ProjectPayloadInput, include_solution_cache: boolean }` | `{ status: "Saved" \| "Cancelled", session: ProjectSessionState, metadata?: ProjectMetadata, integrity?: DocumentIntegrity, bytes_written?: number }` |
| `project_record_revision` | `UC-C05 RecordWorkspaceRevision` | `{ active_revision: number }` | `ProjectSessionState` |
| `project_get_session_state` | `UC-Q01 GetSessionState` | `{}` | `ProjectSessionState` |
| `project_list_recent` | `UC-Q02 ListRecentProjects` | `{}` | `{ entries: RecentProjectEntryView[] }` (includes `exists_on_disk: boolean`) |
| `project_manage_recent` | `UC-C06 ManageRecentProjects` | `{ action: "Pin" \| "Unpin" \| "Remove" \| "ClearUnpinned", file_path?: string }` | `{ entries: RecentProjectEntryView[] }` |
| `project_inspect_header` | `UC-Q03 InspectProjectHeader` | `{ file_path: string }` | `{ schema_version: string, metadata: ProjectMetadata, checksum_valid: boolean, migratable: boolean }` |
| `project_export_diagram` | `UC-C07 ExportDiagram` | `DiagramExportSpec` | `{ status: "Exported" \| "Cancelled", output_path?: string, format?: "Svg" \| "Png", bytes_written?: number }` |

---

## 4. Canonical Error Model

All IPC errors conform to [`cross-capability-conventions.md`](../cross-capability-conventions.md) §2:
```ts
export interface ProjectIpcError {
  code:
    | 'PROJECT_SCHEMA_UNSUPPORTED'
    | 'PROJECT_CORRUPT_JSON'
    | 'PROJECT_IO_ERROR'
    | 'PROJECT_UNSAVED_CHANGES'
    | 'PROJECT_EXPORT_INVALID_SPEC';
  message_el: string;
  message_en: string;
  details?: Record<string, unknown>;
}
```

| Error Code | Trigger Condition | Bilingual Message Template (`el` / `en`) | `details` Payload |
| :--- | :--- | :--- | :--- |
| `PROJECT_SCHEMA_UNSUPPORTED` | `schema_version` has unsupported major version (`!= 1`) or future minor version (`> 0`). | **EL:** Μη υποστηριζόμενη έκδοση αρχείου έργου (`.bem`).<br>**EN:** Unsupported project file (`.bem`) schema version. | `{ found_version: string, supported_range: "1.0..=1.0", file_path: string }` |
| `PROJECT_CORRUPT_JSON` | Malformed UTF-8/JSON, missing required section, `NaN`/`Infinity` float, or `payload_sha256` checksum mismatch. | **EL:** Το αρχείο έργου (`.bem`) είναι κατεστραμμένο ή αλλοιωμένο.<br>**EN:** The project file (`.bem`) is corrupted or failed integrity verification. | `{ reason: "SyntaxError" \| "ChecksumMismatch" \| "NonFiniteNumber" \| "SchemaViolation", line?: number, column?: number, expected_sha256?: string, actual_sha256?: string }` |
| `PROJECT_IO_ERROR` | File not found, permission denied, directory missing, locked file, or disk full during read/write/rename. | **EL:** Σφάλμα ανάγνωσης ή εγγραφής αρχείου στο σύστημα αρχείων.<br>**EN:** Filesystem I/O error while reading or writing project file. | `{ operation: "Read" \| "WriteTemp" \| "AtomicRename" \| "Export", file_path: string, os_error_kind: string }` |
| `PROJECT_UNSAVED_CHANGES` | `project_new` or `project_open` invoked while `is_dirty == true` and `confirm_discard == false`. | **EL:** Υπάρχουν μη αποθηκευμένες αλλαγές στο ενεργό έργο.<br>**EN:** The active project has unsaved changes. | `{ saved_revision: number, active_revision: number, file_path: string \| null }` |
| `PROJECT_EXPORT_INVALID_SPEC` | `project_export_diagram` invoked with invalid dimensions (`< 100` or `> 16384`), invalid DPI scale, or malformed SVG markup. | **EL:** Μη έγκυρες παράμετροι εξαγωγής διαγράμματος.<br>**EN:** Invalid diagram export specification. | `{ field: string, value: unknown }` |

---

## 5. Canonical CLI Mapping (Headless Verification & Batch Tooling)

For offline verification suites and batch grading, the Rust binary exposes equivalent subcommands under `omnimechanics project`:
- `omnimechanics project validate <path.bem> [--json]` $\to$ Runs `InspectProjectHeader` + full schema & SHA-256 verification. Exit code `0` on valid, `2` on `PROJECT_SCHEMA_UNSUPPORTED`, `3` on `PROJECT_CORRUPT_JSON`, `4` on `PROJECT_IO_ERROR`.
- `omnimechanics project migrate <in.bem> --out <out.bem>` $\to$ Upgrades an older migratable `.bem` file to current `schema_version` (`"1.0"`) with canonical formatting and updated SHA-256.
- `omnimechanics project export-diagram <in.bem> --target <LC_OR_CO_ID> --format <svg|png> --out <path>` $\to$ Headless diagram export.

---

## 6. Parity & Testing Contract

1. **Round-Trip Identity Rule:** `deserialize(serialize(doc)) == doc` across all workspace sections (`continuum`, `continuum_sketch`, `structural`, `solution_cache`, `ui_state`).
2. **Canonical Digest Determinism:** Serializing the same `ProjectDocument` in Rust on Windows, macOS, or Linux must produce the exact same UTF-8 byte sequence and `payload_sha256`.
3. **Frontend/Backend Revision Parity:** `ProjectSessionState` returned by `project_record_revision` is the single source of truth for the header dirty indicator (`*`) and the `UnsavedChangesGuard`.
