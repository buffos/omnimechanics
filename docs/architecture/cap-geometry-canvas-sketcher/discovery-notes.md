# Discovery Notes: 2D Structural Modeling & Analysis (Mode 2 sketcher + general frame solver)

- Mode: clear fog from code (brownfield) on `cap-geometry-canvas-sketcher`, plus a proposed new
  frame-analysis capability
- Trigger: issue 015 pre-implementation review found material gaps
  (see [requirements-gap-analysis.md](requirements-gap-analysis.md))
- Status: interrogation in progress (batches of 10 questions with recommended answers)

## User goal (user-confirmed target behavior)

The user can enter **any 2D structure** (beams, trusses, springs, cables, etc.) and see its solution.
When a step-by-step educational solution is possible, it is offered. Otherwise the model goes to a full
solver with real-time updates (adding loads etc.).

## Observed in code (2026-10-31)

- `src-tauri/src/teaching/`: an isostatic straight beam solver, a pin-jointed truss solver
  (joints/Ritter), and a Cross continuous-beam solver. There is no general frame solver.
- Mode switcher in the header: Mode 1 (Continuum Workbench) and Mode 3 (Teaching Studio). Mode 2 does
  not exist.
- No frontend test harness (0 `*.spec.ts`).
- Planning-map hygiene: several nodes use `state: in-progress`, which is not a profile state, and
  `.okf/index.md` disagrees with them (e.g. sketcher is `in-progress` in the node and `specified` in the
  index). The root `verification` block uses `policy: required` instead of the profile's `mode:` key.

## Batch 1 decisions (user-confirmed, 2026-10-31)

| # | Topic | Decision |
| --- | --- | --- |
| Q1 | Element types | Euler-Bernoulli frame members with end releases, truss bars, linear translational/rotational springs (nodal and support), **true elastic catenary cables from the start** (plus tension-only behaviour) |
| Q2 | Analysis type | Linear static analysis, **plus 2nd-order P-Δ now**, plus the tension/compression-only nonlinearity |
| Q3 | Solver ownership | **New top-level capability** "2D Frame Analysis Engine" (Direct Stiffness Method, Rust), shared by Mode 2, Mode 3, and Eurocode |
| Q4 | Educational vs solver routing | Automatic classifier. DSM results are always shown. "Step-by-step" is enabled when the structure matches a supported classical method; otherwise the UI explains why it is unavailable |
| Q5 | DSM walkthrough | A step-by-step DSM walkthrough is a generic educational method for any structure: local k, transformation T, assembly, boundary conditions, solution, end forces. Full matrices are shown for small models (e.g. ≤ 6 nodes) |
| Q6 | Real time | Live DSM re-solve on every edit (~150 ms debounce) that updates diagrams and the deformed shape. The step-by-step solution is regenerated on demand |
| Q7 | Results | Reactions, M/Q/N diagrams, scaled deformed shape, nodal displacements, member end-force table, extrema, hover inspection |
| Q8 | Loads | Nodal forces/moments, member point loads, uniform/trapezoidal loads (local, global, projected), self-weight toggle, support settlements, thermal loads (uniform ΔT and gradient) |
| Q9 | Load cases & combinations | **Full EN 1990 combinations now** |
| Q10 | Units & signs | SI: kN, m, kNm, kN/m, E in GPa. y axis up. M positive when the bottom fibre is in tension, diagrams drawn on the tension side (Greek practice) |

## Batch 2 decisions (user-confirmed, 2026-10-31)

| # | Topic | Decision |
| --- | --- | --- |
| Q11 | Catenary cables | Exact elastic catenary element (Irvine) with self-weight. Input is unstressed length or pretension. Newton-Raphson solution with slack detection |
| Q12 | Real time vs nonlinear | The linear solution appears immediately. The nonlinear solve (P-Δ, cables) runs in the background with a progress indicator and replaces the linear result when it converges. A new edit cancels stale jobs |
| Q13 | 2nd order | Geometric stiffness kG (P-Δ and P-δ) per combination, solved iteratively, **plus linear buckling analysis** (α_cr and buckling modes) |
| Q14 | Superposition | Linear models superpose load cases. Nonlinear models solve each combination separately, and the UI indicates that superposition does not apply |
| Q15 | EN 1990 scope | ULS STR/GEO (6.10 or 6.10a/b selectable), SLS characteristic/frequent/quasi-permanent, envelopes, Greek National Annex defaults, **plus EN 1998 seismic (lateral force method) now**. Accidental combinations come later |
| Q16 | Combination generation | Automatic generation from load cases tagged by action type (G, Q categories A-H, wind, snow, temperature), with every leading-variable permutation. The user can edit them and add manual combinations |
| Q17 | Supports | Fixed, pinned, roller in any direction (inclined), guided (roller with fixed rotation), elastic supports (kx, ky, kθ), unilateral (compression-only) supports |
| Q18 | Sections & materials | Material library (concrete C, steel S235/275/355, timber C/GL, cable steel) and section library (rectangular, circular, IPE/HEA/HEB, hollow), plus custom (E, A, I) |
| Q19 | Instability | Detect kinematic instability before solving (rank / singular matrix), animate the mechanism on the canvas, explain it in Greek/English, and show the degree of indeterminacy |
| Q20 | Size & speed | Up to ~500 nodes / 1000 members. Linear solve < 50 ms, nonlinear < 2 s per combination |

## Batch 3 decisions (user-confirmed, 2026-10-31)

| # | Topic | Decision |
| --- | --- | --- |
| Q21 | Seismic | EN 1998-1 lateral force method **plus modal response spectrum analysis (SRSS/CQC) now**. Greek NA zones, ground type A-E, importance class, q, design spectrum, masses from G + ψ₂Q |
| Q22 | Mode 2 vs Mode 3 | Mode 2 = "Structural Modeling & Analysis", with sketching and live DSM results on the same canvas. Mode 3 stays the Teaching Studio and receives the model via "Open step-by-step". Mode 3 presets also open as editable sketches in Mode 2 |
| Q23 | Drawing tools | Grid snap (selectable step), node snap, midpoint and intersection snap (auto-split members), ortho, angle snap, numeric entry (x,y or L,θ), copy/mirror/array, undo/redo, box selection, inspector property editing |
| Q24 | Persistence & export | Save/load a JSON project (model, load cases, combinations) via `cap-project-persistence`. SVG/PNG export of diagrams. PDF/LaTeX report and DXF import come later |
| Q25 | Classical methods in the classifier | In order: isostatic beams and frames (inclined members, three-hinged, compound with hinges), trusses (joints/Ritter), continuous beams (Clapeyron/Cross), force method for degree ≤ 3. The DSM walkthrough is always available. Cross/Kani on frames come later |
| Q26 | Language | Bilingual el/en for UI, messages, and step texts |
| Q27 | Verification | Corrected verification block: backend-boundary required (Rust tests), frontend-integration required with a new Vitest harness for pure TS logic, end-to-end catalog-only with visual review |
| Q28 | Solver validation | Closed-form reference suite (tolerance 1e-6 linear, 1e-3 nonlinear) **plus comparison with OpenSees or other software** |
| Q29 | Topology | (a) New top-level `cap-frame-analysis-engine` (future children: linear DSM; nonlinear/buckling/cables). (b) New top-level shared `cap-eurocode-actions-combinations` (EN 1990/1998), shared with RC design. (c) Split the sketcher into a frame modeler (now) and a continuum polygon sketcher (later) |
| Q30 | Delivery order | 1) Mode 2 modeler with live linear DSM (beams, bars, springs, all supports, one load case) + M/Q/N/deformed shape; 2) load cases/EN 1990; 3) classifier + step-by-step + DSM walkthrough; 4) P-Δ + buckling; 5) catenary cables; 6) seismic |

## Batch 4 decisions (user-confirmed, 2026-10-31)

| # | Topic | Decision |
| --- | --- | --- |
| Q31 | Modal RSA | Lumped masses from G + ψ₂Q (consistent mass optional). Modes up to ≥ 90% effective mass, Type 1 spectrum, 5% damping, SRSS or CQC selectable, horizontal component plus optional vertical |
| Q32 | Seismic parameters | Greek NA: zones I/II/III (a_g = 0.16g/0.24g/0.36g), ground type A-E, importance class I-IV (γ_I 0.8/1.0/1.2/1.4), user-entered q with guidance per structural system. All values editable |
| Q33 | OpenSees comparison | Offline OpenSeesPy script (not installed yet) produces committed reference CSVs in `docs/benchmarks/frame/`. Rust tests compare against them with no runtime dependency |
| Q34 | Sketcher split | `cap-geometry-canvas-sketcher` becomes a roll-up with children `cap-sketcher-frame-modeler` (Mode 2, now) and `cap-sketcher-continuum-domain` (polygons/holes/CDT, later). Issue 015 is superseded and replaced by new issues after specification |
| Q35 | Model ownership | The structural model (nodes, members, sections, supports, loads, load cases) belongs to `cap-frame-analysis-engine`. The modeler is the UI that edits it, teaching reads it via the classifier, and RC design reads the results |
| Q36 | Member options | End releases (moment, axial, shear), internal node hinge = release on all ends but one, loads at any position without internal nodes, **plus rigid end offsets now** |
| Q37 | Result presentation | Selector for load case / combination / envelope (max/min), with envelope diagrams and the governing combination shown at every point |
| Q38 | Self-weight & masses | Automatic self-weight (density × A) as a G load case. Extra permanent loads entered as member loads in G. Seismic masses derived automatically from the same load cases, with optional extra lumped masses |
| Q39 | Cables | Drawn between two nodes. Input: section/material plus one of L₀, sag ratio, or T₀. Shows the true curve and end tensions. Structures with cables are not eligible for classical step-by-step (solver results only) |
| Q40 | Application synthesis | After topology approval, write `docs/prd.md` and `docs/architecture/application-architecture-summary.md` covering all 3 modes, and repair the invalid `in-progress` states and the index. The user reviews them |

## Fog-clearing exit assessment

- Purpose, sibling boundaries, child territories, actors, and inputs/outputs are clear for the frame
  modeler, the frame analysis engine, and the actions & combinations node. The remaining uncertainty is
  inside those nodes (exact contracts and scenarios), so they move to `bounded`.
- Next route: architecture spec pipeline (`bounded-capability-spec-orchestrator`) per node, starting
  with `cap-frame-analysis-engine` (linear core) and `cap-sketcher-frame-modeler` for delivery slice 1.

### Consequences surfaced by batch 1

- Catenary cables and P-Δ make the solver nonlinear. Superposition is invalid for those models, so
  combinations must be solved per combination.
- Real-time behaviour must be reconciled with iterative nonlinear solves.
- Full EN 1990 requires action categories, ψ factors, the National Annex, and the leading-variable
  permutations.
