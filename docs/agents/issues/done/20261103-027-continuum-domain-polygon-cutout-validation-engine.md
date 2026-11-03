## Issue Metadata

- Issue number: `027`
- Owning capability node: `/.okf/capabilities/cap-sketcher-continuum-domain.md`
- Artifact root: `docs/architecture/cap-sketcher-continuum-domain/`
- Issue file: `docs/agents/issues/done/20261103-027-continuum-domain-polygon-cutout-validation-engine.md`
- Category: `feature`
- Execution type: `AFK`
- Review gate: `none`
- Suggested state: `done`

## Parent Artifacts

- `docs/prd.md`
- `docs/architecture/application-architecture-summary.md`
- `docs/architecture/cross-capability-conventions.md`
- `.okf/capabilities/cap-sketcher-continuum-domain.md`
- `docs/architecture/cap-sketcher-continuum-domain/prd.md`
- `docs/architecture/cap-sketcher-continuum-domain/canonical-domain-model.md`
- `docs/architecture/cap-sketcher-continuum-domain/canonical-use-cases.md`
- `docs/architecture/cap-sketcher-continuum-domain/canonical-api-cli-contract.md`
- `docs/architecture/cap-sketcher-continuum-domain/acceptance-scenarios.md`
- `docs/architecture/cap-sketcher-continuum-domain/readiness-review.md`

## What to build

Implement **Vertical Slice 1** of `cap-sketcher-continuum-domain`: the authoritative Rust `ContinuumDomain` geometry model, loop winding normalization (`CCW` outer loop, `CW` cutout loops), real-time self-intersection preview (`continuum_preview_segment`), and strict topological & clearance validation (`continuum_validate_domain`, `continuum_close_and_normalize_loop`) under `src-tauri/src/continuum/`.

### 1. Canonical Domain Types & Serde DTOs (`src-tauri/src/continuum/types.rs`):
- Define `ContinuumDomain`, `BoundaryLoop`, `Vertex2D`, `BoundarySegment`, `LoopOrientation` (`CounterClockwise`, `Clockwise`), `LoopClosureState` (`Open`, `ClosedValid`, `ClosedInvalid`), `Cutout` (`CutoutShape::Circle`, `Rectangle`, `Polygon`), `InclusionRegion`, `BoundaryConditionSpec` (`DirichletDisplacement`, `NeumannTraction`, `RobinImpedance`, `Free`), `MeshingProfile`, `CandidateSegmentPreview`, and `DomainValidationResponse`.
- Enforce monotonic `revision: u64` starting at `1`.

### 2. Loop Closure, Winding Normalization & Outward Unit Normals (`src-tauri/src/continuum/validation.rs`):
- Implement signed shoelace area computation $A = \frac{1}{2}\sum (x_i y_{i+1} - x_{i+1} y_i)$.
- Implement `continuum_close_and_normalize_loop` (`BR-CONT-01`, `BR-CONT-05`, `SC-005`):
  - Require $N \ge 3$ vertices (`OPEN_BOUNDARY_LOOP`), minimum edge length $\ell_{\min} = 10^{-4}\text{ m}$ (`ZERO_LENGTH_SEGMENT`), no $180^\circ$ collinear folds, and $|A| \ge 10^{-6}\text{ m}^2$ (`DEGENERATE_POLYGON_AREA`).
  - Automatically reverse outer boundary vertices to `CounterClockwise` ($A > 0$) with outward unit normals pointing away from the interior, and reverse hole/cutout vertices to `Clockwise` ($A < 0$) with outward normals pointing into the hole cavity.
  - Auto-assign default semantic tags (`Bottom`, `Right`, `Top`, `Left` for axis-aligned outer segments, or `Outer_i` / `Hole`).

### 3. Self-Intersection & Cutout Containment/Clearance Engine (`src-tauri/src/continuum/validation.rs`):
- Implement exact segment-segment intersection and point-on-segment checks (`BR-CONT-02`, `SC-002`) returning `SELF_INTERSECTING_LOOP` with `{ loop_id, segment_a, segment_b, intersection_point }`.
- Implement `continuum_preview_segment` (`UC-QRY-02`, `SC-002`) returning `CandidateSegmentPreview { is_valid, intersecting_segment_indices, snap_closure_ready }`.
- Implement strict cutout and inclusion containment & minimum clearance checks (`BR-CONT-03`, `SC-003`, `SC-004`):
  - Minimum clearance $d_{\min} = \max(10^{-3}\text{ m}, 0.1 \cdot h \cdot \alpha_{\text{hole}})$.
  - Reject features outside or touching the outer loop with `HOLE_OUTSIDE_OUTER_BOUNDARY`.
  - Reject features closer than $d_{\min}$ to the outer boundary or another cutout with `CLEARANCE_BELOW_MINIMUM` (reporting actual vs. required distance).
  - Reject overlapping or nested cutouts/inclusions with `HOLE_OVERLAPS_EXISTING_FEATURE`.
  - Validate `BoundaryConditionSpec` (`INVALID_BOUNDARY_CONDITION`) and `MeshingProfile` (`INVALID_MESHING_PROFILE`).
- Register Tauri IPC commands `continuum_validate_domain`, `continuum_preview_segment`, and `continuum_close_and_normalize_loop` in `src-tauri/src/lib.rs`.

## Acceptance criteria

- [x] `continuum_close_and_normalize_loop` reverses CW outer boundaries to CCW ($A > 0$) and CCW hole polygons to CW ($A < 0$), computing outward unit normals pointing away from the solid material (`SC-005`).
- [x] `continuum_preview_segment` and `continuum_validate_domain` reject a figure-8 / bowtie polygon $(0,0)\to(4,4)\to(4,0)\to(0,4)$ with `SELF_INTERSECTING_LOOP` and `details.intersection_point == [2.0, 2.0]` (`SC-002`).
- [x] Cutouts extending outside the $10 \times 10\text{ m}$ outer square fail with `HOLE_OUTSIDE_OUTER_BOUNDARY`, and cutouts within $d < d_{\min}$ fail with `CLEARANCE_BELOW_MINIMUM` (`SC-003`).
- [x] Overlapping interior cutouts fail with `HOLE_OVERLAPS_EXISTING_FEATURE` referencing both cutout IDs (`SC-004`).
- [x] Companion unit tests in `src-tauri/src/continuum/validation_test.rs` pass 100% (`cargo test --lib`), and `npm run check:limits` passes with 0 errors.

## Artifact sync required

- Application PRD: `none - already covered in docs/prd.md`
- Application architecture summary: `none - matches docs/architecture/application-architecture-summary.md`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-sketcher-continuum-domain.md`
- Issue registry: `required`; node `issues:` reference: `required: .okf/capabilities/cap-sketcher-continuum-domain.md`
- Reason/no-impact decision: `Delivers backend domain validation and loop normalization for cap-sketcher-continuum-domain without altering cross-capability contracts.`

## Human review gate

`none`: Pure backend geometric validation and IPC boundary slice; UI exposure is delivered in Issue `029`.

## Blocked by

- None - can start immediately

## Artifact anchors

- PRD: `WF-1`, `WF-2`, `BR-CONT-01`, `BR-CONT-02`, `BR-CONT-03`, `BR-CONT-05`, `FR-02`, `FR-03`
- Domain model: `ContinuumDomain`, `BoundaryLoop`, `Cutout`, `InclusionRegion`, `INV-CONT-01..03`
- Use cases: `UC-CMD-01`, `UC-CMD-02`, `UC-QRY-01`, `UC-QRY-02`
- Contract: `continuum_validate_domain`, `continuum_preview_segment`, `continuum_close_and_normalize_loop`

## Acceptance scenarios addressed

- `SC-002`: Self-intersecting (bowtie) polygon boundary is rejected on loop closure and validation
- `SC-003`: Cutout breaching outer boundary or violating minimum clearance $d_{\min}$ is rejected
- `SC-004`: Overlapping or touching interior cutouts are rejected with `HOLE_OVERLAPS_EXISTING_FEATURE`
- `SC-005`: Clockwise-drawn outer boundary is automatically reversed to CCW and hole loops orient CW with outward normals away from material

## Verification obligations

- Policy source: `/.okf/project.md`

| Scenario | Backend boundary | Frontend integration | End-to-end journey |
|---|---|---|---|
| `SC-002` | `planned` | `deferred: Issue 029` | `catalog-only` |
| `SC-003` | `planned` | `deferred: Issue 029` | `catalog-only` |
| `SC-004` | `planned` | `deferred: Issue 029` | `catalog-only` |
| `SC-005` | `planned` | `deferred: Issue 029` | `not-applicable` |
