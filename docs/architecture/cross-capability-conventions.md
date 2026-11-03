# Cross-Capability Conventions (Mode 2 structural stack)

- Status: binding for the 2026-10-31 spec pipeline runs
- Applies to: `cap-frame-analysis-engine`, `cap-eurocode-actions-combinations`,
  `cap-sketcher-frame-modeler`, `cap-sketcher-continuum-domain`, `cap-interactive-canvas`,
  `cap-project-persistence`, and the teaching classifier extension
- Decision source: [discovery-notes.md](cap-geometry-canvas-sketcher/discovery-notes.md)

Node-scoped artifacts may refine these conventions. They must not rename or contradict them without
updating this file.

## 1. Canonical vocabulary (owner in brackets)

| Term | Meaning |
| --- | --- |
| **StructuralModel** [frame] | Complete 2D model: nodes, elements, sections, materials, supports, loads, load cases, masses. Carries a monotonically increasing `revision` |
| **Node** [frame] | Point (x, y) in metres with a stable string id |
| **Element** [frame] | Connects nodes. Kinds: `FrameMember`, `TrussBar`, `Spring` (nodal translational/rotational), `Cable` |
| **EndRelease** [frame] | Per-end release flags for axial, shear, and moment. A node-level "internal hinge" is a modeler gesture that expands to moment releases |
| **RigidOffset** [frame] | Per-end rigid end offset of a `FrameMember` |
| **Section** / **Material** [frame] | Library or custom section (A, I, and optional depth/shape) and material (E, G, density, α_T, class). A library item is referenced by id |
| **Support** [frame] | Restraint on a node: `Fixed`, `Pinned`, `Roller{angle}`, `Guided{angle}`, `Elastic{kx,ky,kθ}`, with a `unilateral` flag for compression-only |
| **Load** [frame] | Belongs to exactly one LoadCase: `NodalLoad`, `MemberPointLoad`, `MemberDistributedLoad{q1,q2,a,b,frame: Local/Global/Projected}`, `SupportSettlement`, `ThermalLoad{ΔT_uniform, ΔT_gradient}`. Self-weight is a LoadCase flag |
| **LoadCase** [frame] | Named group of loads with an `ActionType` tag. The tag vocabulary is owned by actions |
| **ActionType** [actions] | `Permanent`, `Imposed{category A-H}`, `Snow`, `Wind`, `Temperature`, `Seismic`, `Accidental` (reserved) |
| **Combination** [actions] | Named set of factor × LoadCase terms with a `LimitState` (ULS_STR, ULS_GEO, SLS_CHAR, SLS_FREQ, SLS_QP, SEISMIC), plus an `origin` (`Generated`, `UserEdited`, `Manual`) |
| **NationalAnnexProfile** [actions] | Editable γ/ψ/seismic parameter set. Default `GR` |
| **SeismicSettings** / **DesignSpectrum** [actions] | EN 1998-1 parameters and the resulting Sd(T) |
| **AnalysisKind** [frame] | `LinearStatic`, `SecondOrderPDelta`, `LinearBuckling`, `Modal`, `ResponseSpectrum`, `LateralForce` |
| **AnalysisJob** [frame] | One execution for (model revision, kind, target set). Status: `Queued`, `Running`, `Converged`, `Failed`, `Cancelled`, `Superseded` |
| **ResultSet** [frame] | Results for one target (LoadCase or Combination): reactions, displacements, member diagrams, end forces, extrema |
| **Envelope** [frame] | max/min over a set of ResultSets, with the governing combination per sample |
| **Diagnostics** [frame] | Mechanism detection (mode shape), indeterminacy degree, warnings (slack cable, non-convergence) |
| **MethodEligibility** [teaching] | Classifier output: classical methods with eligible/ineligible status and reasons |
| **ContinuumDomain** [continuum] | Outer loop + holes + boundary tags → mesh handoff. Separate from StructuralModel |
| **ProjectDocument** [persistence] | Versioned JSON file containing workspace sections, e.g. `structural`, `continuum`, `ui` |

## 2. Shared rules

- **Units:** SI internally (kN, m, kNm, kN/m, kN/m², °C). E is entered in GPa and converted at the
  boundary.
- **Axes and signs:** global y up, counter-clockwise rotation positive. Diagram convention: M positive
  when the bottom/local −y fibre is in tension, diagrams drawn on the tension side, N positive in tension.
- **Ids:** string ids unique per model (`N1`, `E3`, `LC2`, `CO5`), stable across edits.
- **Revisions:** every model mutation increments `revision`. Results and jobs carry the revision they
  were computed for. Stale results are never shown as current.
- **Errors:** IPC errors use `{ code: SCREAMING_SNAKE, message_el, message_en, details? }`. Domain
  validation codes are owned by the node that raises them.
- **IPC naming:** Tauri commands prefixed `frame_`, `actions_`, `teaching_`, `continuum_`,
  `project_`. Background progress uses the event `analysis://progress`.
- **Language:** user-facing texts are bilingual (`el`, `en`).
- **Dependency direction:** modeler UI → frame → actions; teaching → frame; rc-design → frame + actions;
  persistence serialises the models but owns no domain rules.
