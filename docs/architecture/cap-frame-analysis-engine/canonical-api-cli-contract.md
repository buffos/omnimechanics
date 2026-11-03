# Canonical API & CLI Contract: 2D Frame Analysis Engine (`cap-frame-analysis-engine`)

- **Capability Owner:** `cap-frame-analysis-engine`
- **Upstream Artifacts:** [domain-glossary.md](domain-glossary.md), [prd.md](prd.md), [canonical-domain-model.md](canonical-domain-model.md), [canonical-use-cases.md](canonical-use-cases.md)
- **Cross-Capability Conventions:** [cross-capability-conventions.md](../cross-capability-conventions.md)

---

## 1. Transport-Neutral Contract Conventions

1. **Command Naming Prefix:** All Tauri IPC commands owned by this capability use the `frame_` prefix (`frame_upsert_model`, `frame_get_catalog`, `frame_evaluate_diagnostics`, `frame_solve_linear`, `frame_inspect_member_point`, `frame_start_analysis_job`, `frame_cancel_analysis_job`, `frame_get_analysis_job_result`, `frame_generate_dsm_walkthrough`).
2. **Units at the Contract Boundary:**
   - Lengths / coordinates / offsets / settlements: $\text{m}$ (`f64`)
   - Section area $A$, $A_v$: $\text{m}^2$ (`f64`); Second moment of area $I$: $\text{m}^4$ (`f64`); Depth $h$: $\text{m}$ (`f64`)
   - Young's modulus `e_gpa` and Shear modulus `g_gpa`: entered/serialized in $\text{GPa}$ (`f64`), converted internally to $\text{kN/m}^2$ ($\times 10^6$)
   - Forces / reactions / axial / shear: $\text{kN}$ (`f64`); Moments: $\text{kNm}$ (`f64`); Distributed loads: $\text{kN/m}$ (`f64`)
   - Translational springs: $\text{kN/m}$ (`f64`); Rotational springs: $\text{kNm/rad}$ (`f64`)
   - Temperatures: $^\circ\text{C}$ (`f64`); Angles: degrees (`f64`, counter-clockwise from $+X$); Modal masses: $\text{t}$ (`f64`)
3. **Stable String Identifiers:** Model entities use deterministic string IDs (`"N1"`, `"E1"`, `"S1"`, `"M1"`, `"SU1"`, `"LC1"`, `"CO1"`).
4. **Revision Fencing:** Every request that reads or solves the active model includes `model_revision: u64`. Every response echoes `revision: u64`.

---

## 2. Canonical Payload Shapes (Serde JSON Schema)

### 2.1 `StructuralModel` Payload

```json
{
  "id": "model-01",
  "name": "Two-Bay Portal Frame with Tie Cable",
  "revision": 12,
  "gravity_g": 9.81,
  "nodes": [
    { "id": "N1", "x": 0.0, "y": 0.0 },
    { "id": "N2", "x": 0.0, "y": 4.0 },
    { "id": "N3", "x": 6.0, "y": 4.0 },
    { "id": "N4", "x": 6.0, "y": 0.0 }
  ],
  "materials": [
    {
      "id": "M1",
      "name": "Steel S355",
      "source": { "kind": "Preset", "catalog_id": "EN1993_S355" },
      "class": "Steel",
      "e_gpa": 210.0,
      "g_gpa": 80.77,
      "density_gamma": 78.5,
      "alpha_t": 1.2e-5
    }
  ],
  "sections": [
    {
      "id": "S1",
      "name": "IPE 300",
      "source": { "kind": "Preset", "catalog_id": "IPE_300" },
      "shape": "Ipe",
      "area_a": 0.00538,
      "inertia_i": 0.00008356,
      "shear_area_av": 0.00257,
      "depth_h": 0.300
    }
  ],
  "elements": [
    {
      "id": "E1",
      "kind": "FrameMember",
      "node_i": "N1",
      "node_j": "N2",
      "section_id": "S1",
      "material_id": "M1",
      "release_i": { "axial": false, "shear": false, "moment": false },
      "release_j": { "axial": false, "shear": false, "moment": false },
      "offset_i": { "dx": 0.0, "dy": 0.0, "frame": "Local" },
      "offset_j": { "dx": 0.0, "dy": 0.0, "frame": "Local" }
    },
    {
      "id": "E2",
      "kind": "FrameMember",
      "node_i": "N2",
      "node_j": "N3",
      "section_id": "S1",
      "material_id": "M1",
      "release_i": { "axial": false, "shear": false, "moment": false },
      "release_j": { "axial": false, "shear": false, "moment": true },
      "offset_i": { "dx": 0.15, "dy": 0.0, "frame": "Local" },
      "offset_j": { "dx": -0.15, "dy": 0.0, "frame": "Local" }
    },
    {
      "id": "E3",
      "kind": "Cable",
      "node_i": "N1",
      "node_j": "N3",
      "section_id": "S2",
      "material_id": "M2",
      "prestress": { "mode": "Pretension", "t0": 25.0 }
    }
  ],
  "supports": [
    { "id": "SU1", "node_id": "N1", "kind": { "type": "Fixed" }, "unilateral": false },
    { "id": "SU2", "node_id": "N4", "kind": { "type": "Roller", "angle_deg": 15.0 }, "unilateral": true }
  ],
  "load_cases": [
    {
      "id": "LC1",
      "name": "Dead Load G",
      "action_type": { "kind": "Permanent" },
      "self_weight": true,
      "loads": [
        {
          "kind": "MemberDistributedLoad",
          "element_id": "E2",
          "qx1": 0.0,
          "qy1": -18.0,
          "qx2": 0.0,
          "qy2": -24.0,
          "a_ratio": 0.0,
          "b_ratio": 1.0,
          "frame": "Local"
        },
        {
          "kind": "ThermalLoad",
          "element_id": "E2",
          "dt_uniform": 20.0,
          "dt_gradient": 10.0
        }
      ]
    }
  ],
  "lumped_masses": [
    { "node_id": "N2", "mass_x_t": 12.5, "mass_y_t": 12.5, "inertia_theta_tm2": 0.0 }
  ]
}
```

### 2.2 `Diagnostics`, `ResultSet` & `Envelope` Payloads

```json
{
  "revision": 12,
  "diagnostics": {
    "static_indeterminacy_degree": 1,
    "kinematic_instability_degree": 0,
    "free_dofs": 6,
    "is_stable": true,
    "mechanism_modes": [],
    "warnings": [
      {
        "code": "CABLE_SLACK_DETECTED",
        "element_id": "E3",
        "message_el": "Το καλώδιο E3 χαλάρωσε στον συνδυασμό CO2.",
        "message_en": "Cable E3 went slack in combination CO2."
      }
    ]
  },
  "result_set": {
    "revision": 12,
    "target_id": "CO1",
    "target_kind": "Combination",
    "analysis_kind": "SecondOrderPDelta",
    "superposition_valid": false,
    "nodal_displacements": [
      { "node_id": "N2", "ux_m": 0.00842, "uy_m": -0.00112, "theta_rad": -0.00215 }
    ],
    "support_reactions": [
      { "support_id": "SU1", "node_id": "N1", "rx_kn": -14.2, "ry_kn": 68.5, "mz_knm": 31.4 }
    ],
    "member_results": [
      {
        "element_id": "E2",
        "length_eff_m": 5.70,
        "end_forces_local": { "ni_kn": -14.2, "qi_kn": 54.0, "mi_knm": -31.4, "nj_kn": 14.2, "qj_kn": -65.7, "mj_knm": 0.0 },
        "stations": [
          { "x_m": 0.0, "n_kn": -14.2, "q_kn": 54.0, "m_knm": -31.4, "ux_local_m": 0.0, "uy_local_m": 0.0, "theta_rad": -0.00215 },
          { "x_m": 2.85, "n_kn": -14.2, "q_kn": 0.0, "m_knm": 45.55, "ux_local_m": -0.0001, "uy_local_m": -0.0118, "theta_rad": 0.0 }
        ],
        "extrema": {
          "max_m": { "x_m": 2.85, "value": 45.55 },
          "min_m": { "x_m": 0.0, "value": -31.40 },
          "max_q": { "x_m": 0.0, "value": 54.00 },
          "min_q": { "x_m": 5.70, "value": -65.70 },
          "max_n": { "x_m": 0.0, "value": -14.20 },
          "min_n": { "x_m": 0.0, "value": -14.20 },
          "max_abs_deflection_m": { "x_m": 2.98, "value": 0.0121 }
        }
      }
    ],
    "cable_results": [
      {
        "element_id": "E3",
        "unstressed_length_l0_m": 7.2081,
        "chord_length_m": 7.2111,
        "horizontal_tension_h_kn": 20.8,
        "vertical_reaction_vi_kn": -13.9,
        "vertical_reaction_vj_kn": 14.3,
        "max_tension_kn": 25.3,
        "mid_sag_m": 0.042,
        "is_slack": false,
        "curve_points_global": [
          { "x": 0.0, "y": 0.0 },
          { "x": 3.0, "y": 1.96 },
          { "x": 6.0, "y": 4.0 }
        ]
      }
    ]
  }
}
```

### 2.3 Background Event Payload (`analysis://progress`)

```json
{
  "job_id": "job-12-pdelta-01",
  "model_revision": 12,
  "kind": "SecondOrderPDelta",
  "status": "Running",
  "progress_percent": 65.0,
  "current_target_id": "CO3",
  "current_iteration": 4,
  "current_residual": 3.4e-6,
  "message_el": "Επίλυση 2ης τάξης P-Δ: Συνδυασμός CO3 (επανάληψη 4)...",
  "message_en": "Solving 2nd-order P-Delta: Combination CO3 (iteration 4)..."
}
```

---

## 3. Error Model & Canonical Error Codes

All IPC and CLI errors return the canonical bilingual error envelope:

```json
{
  "code": "KINEMATIC_INSTABILITY",
  "message_el": "Ο φορέας είναι κινηματικά ασταθής (μηχανισμός βαθμού nk = 1 στον κόμβο N3).",
  "message_en": "The structure is kinematically unstable (mechanism of degree nk = 1 at node N3).",
  "details": {
    "revision": 12,
    "diagnostics": {
      "static_indeterminacy_degree": -1,
      "kinematic_instability_degree": 1,
      "is_stable": false,
      "mechanism_modes": [
        {
          "mode_index": 1,
          "classification": "GlobalSwayOrHingeChain",
          "node_displacements": [
            { "node_id": "N2", "dux": 1.0, "duy": 0.0, "dtheta": -0.25 },
            { "node_id": "N3", "dux": 1.0, "duy": 0.0, "dtheta": -0.25 }
          ]
        }
      ]
    }
  }
}
```

| Error Code | Category | HTTP Equivalent | CLI Exit Code | Description |
| --- | --- | --- | --- | --- |
| `INVALID_ELEMENT_GEOMETRY` | Validation | `422 Unprocessable Entity` | `2` | Zero length, self-loop, or invalid rigid offsets. |
| `MISSING_MODEL_REFERENCE` | Validation | `422 Unprocessable Entity` | `2` | Unknown `NodeId`, `SectionId`, `MaterialId`, or `SupportId`. |
| `DUPLICATE_NODE_SUPPORT` | Validation | `422 Unprocessable Entity` | `2` | Multiple supports assigned to the same node. |
| `INVALID_LOAD_ASSIGNMENT` | Validation | `422 Unprocessable Entity` | `2` | Transverse load on cable or thermal gradient without `depth_h`. |
| `KINEMATIC_INSTABILITY` | Domain Rule | `409 Conflict` | `3` | Mechanism detected ($n_k > 0$); includes `mechanism_modes` in `details`. |
| `NONLINEAR_NOT_CONVERGED` | Numerical | `409 Conflict` | `4` | Active-set, $P\text{-}\Delta$, or catenary NR failed to converge. |
| `PDELTA_BUCKLING_DIVERGENCE` | Numerical | `409 Conflict` | `4` | Structure buckled under applied compressive loads ($\det(\mathbf{K}_E + \mathbf{K}_G) \le 0$). |
| `CABLE_COMPLETE_SLACK_MECHANISM` | Numerical | `409 Conflict` | `4` | Unrestrained node lost all stiffness due to slack cables. |
| `STALE_MODEL_REVISION` | Concurrency | `409 Conflict` | `5` | Requested `model_revision` does not match active `StructuralModel.revision`. |
| `JOB_NOT_FOUND` | Not Found | `404 Not Found` | `6` | Unknown `job_id`. |

---

## 4. Tauri IPC Command & HTTP/CLI Mapping

| Use Case | Tauri IPC Command | CLI Subcommand (`omnimech frame ...`) | Execution Mode |
| --- | --- | --- | --- |
| `UpsertStructuralModel` | `frame_upsert_model` | `frame validate --model <file.json>` | Synchronous |
| `ListMaterialAndSectionCatalog` | `frame_get_catalog` | `frame catalog [--json]` | Synchronous |
| `EvaluateKinematicDiagnostics` | `frame_evaluate_diagnostics` | `frame diagnostics --model <file.json>` | Synchronous |
| `SolveLinearStatic` | `frame_solve_linear` | `frame solve --kind linear --model <file.json>` | Synchronous (< 50 ms) |
| `InspectMemberPoint` | `frame_inspect_member_point` | `frame inspect --model <file.json> --element E1 --x 2.5` | Synchronous |
| `StartAnalysisJob` | `frame_start_analysis_job` | `frame solve --kind <pdelta\|buckling\|modal\|rsa\|lateral> --model <file.json>` | Async Job (IPC) / Blocking with progress (CLI) |
| `CancelAnalysisJob` | `frame_cancel_analysis_job` | *(SIGINT / Ctrl-C in CLI)* | Synchronous |
| `GetAnalysisJobResult` | `frame_get_analysis_job_result` | *(Emitted to stdout/file on CLI completion)* | Synchronous |
| `GenerateDsmWalkthrough` | `frame_generate_dsm_walkthrough` | `frame walkthrough --model <file.json> --target LC1` | Synchronous |

---

## 5. Parity & Verification Contract

1. **Greek Sign Convention Parity:** All endpoints (`frame_solve_linear`, `frame_get_analysis_job_result`, `frame_inspect_member_point`, `frame_generate_dsm_walkthrough`) must return $M(x) > 0$ when the bottom/local $-y$ fibre is in tension, $N(x) > 0$ in tension, and $Q(x) = dM/dx$.
2. **Closed-Form & OpenSeesPy Verification Contract:**
   - Companion Rust tests (`src-tauri/src/frame/*_test.rs`) verify the exact same DTO structures returned by the IPC layer against closed-form analytical solutions (relative tolerance $\le 10^{-6}$ linear, $\le 10^{-3}$ nonlinear) and offline OpenSeesPy CSV benchmarks in `docs/benchmarks/frame/`.
