---
type: capability
title: Project Persistence & Self-Contained Workspace Storage
description: Serialization and deserialization of complete simulation states into self-contained .bem files (JSON/binary schema containing geometry, mesh, material fields, boundary conditions, and solution datasets).
tags: [persistence, file-io, serialization, project-management, bem-file]
timestamp: 2026-11-01T19:20:00Z
state: specified
state_changed: 2026-11-01T19:20:00Z
project: /project.md
parent: /project.md
artifact_root: docs/architecture/cap-project-persistence
orchestration_status: docs/architecture/cap-project-persistence/orchestration-status.md
prd: docs/architecture/cap-project-persistence/prd.md
glossary: docs/architecture/cap-project-persistence/domain-glossary.md
domain_model: docs/architecture/cap-project-persistence/canonical-domain-model.md
use_cases: docs/architecture/cap-project-persistence/canonical-use-cases.md
contract: docs/architecture/cap-project-persistence/canonical-api-cli-contract.md
scenarios: docs/architecture/cap-project-persistence/acceptance-scenarios.md
readiness_review: docs/architecture/cap-project-persistence/readiness-review.md
issues:
  - docs/agents/issues/pending/20261103-032-canonical-bem-schema-sha256-migration-atomic-io.md
  - docs/agents/issues/pending/20261103-033-session-dirty-tracking-unsaved-guard-mru-header-menu.md
---

# Intent

Enable full reproducibility and session portability by saving and restoring the entire application state into a single, compact, and versioned `.bem` project file.

# Scope

### Included:
- **File Format Specification (`.bem`):**
  - Metadata: Creation timestamp, application version, project dimensions (2D vs 3D).
  - Geometry Definition: Parameters or embedded CAD boundary representation.
  - Mesh Topology: Nodal coordinates, boundary face indices, volume element connectivity.
  - Material State: Homogeneous reference moduli and parameters of active inhomogeneity field.
  - Boundary Conditions & Loads: Mappings of face/node sets to prescribed values.
  - Solution Cache (Optional): Displacements, tractions, stress fields, and convergence history.
- **Tauri Native File Dialogs:**
  - `New Project`, `Save Project (.bem)`, `Save As...`, `Open Project (.bem)`.
  - Recent projects list and automatic dirty-state tracking.

### Excluded:
- Cloud database syncing (projects are stored locally on the user's filesystem).

# Relationships

- Parent: [BemApp Platform](/project.md)
- Interacts with all capabilities across the system.
