---
type: capability
title: Eurocode Actions, Load Combinations & Seismic Loading (EN 1990 / EN 1998)
description: Shared normative layer that classifies load cases into actions, generates EN 1990 ULS/SLS combinations and envelopes, and defines EN 1998 seismic loading (lateral force method and response spectrum) with Greek National Annex defaults.
tags: [eurocode, en1990, en1998, actions, load-combinations, psi-factors, envelopes, seismic, response-spectrum, greek-national-annex]
timestamp: 2026-11-01T19:20:00Z
state: implemented
state_changed: 2026-11-02T19:55:00Z
project: /project.md
parent: /project.md
shared_with:
  - /capabilities/cap-frame-analysis-engine.md
  - /capabilities/cap-eurocode-rc-design.md
artifact_root: docs/architecture/cap-eurocode-actions-combinations
discovery_notes: docs/architecture/cap-geometry-canvas-sketcher/discovery-notes.md
orchestration_status: docs/architecture/cap-eurocode-actions-combinations/orchestration-status.md
prd: docs/architecture/cap-eurocode-actions-combinations/prd.md
glossary: docs/architecture/cap-eurocode-actions-combinations/domain-glossary.md
domain_model: docs/architecture/cap-eurocode-actions-combinations/canonical-domain-model.md
use_cases: docs/architecture/cap-eurocode-actions-combinations/canonical-use-cases.md
contract: docs/architecture/cap-eurocode-actions-combinations/canonical-api-cli-contract.md
scenarios: docs/architecture/cap-eurocode-actions-combinations/acceptance-scenarios.md
readiness_review: docs/architecture/cap-eurocode-actions-combinations/readiness-review.md
issues:
  - docs/agents/issues/done/20261102-021-eurocode-na-profile-en1990-combination-engine.md
  - docs/agents/issues/done/20261102-022-en1998-seismic-spectra-mass-lateral-cqc.md
---

# Intent

Give every analysis and design workflow one shared, normative way to turn load cases into Eurocode
combinations and seismic actions, so the frame analysis engine and the RC design engine use the same
factors, combinations, and governing envelopes.

# Scope

### Included
- **Action tagging:** load cases tagged as G, Q (categories A-H), wind, snow, temperature, seismic.
- **EN 1990 combinations:** ULS STR/GEO (6.10 or 6.10a/b selectable), SLS characteristic, frequent, and
  quasi-permanent. Automatic generation over every leading-variable permutation. The user can edit
  generated combinations and add manual ones.
- **Greek National Annex defaults** (γ, ψ factors), all editable.
- **EN 1998-1 seismic:**
  - Parameters: zones I/II/III (a_g 0.16g/0.24g/0.36g), ground type A-E, importance class I-IV
    (γ_I 0.8/1.0/1.2/1.4), user-entered behaviour factor q with guidance per structural system, 5% damping.
  - Type 1 design spectrum, horizontal component plus optional vertical.
  - Lateral force method (T₁ from modal or Rayleigh analysis, force distribution by height).
  - Modal response spectrum analysis with SRSS or CQC.
  - Seismic combination with masses from G + ψ₂Q.
- **Envelope definitions:** max/min per result quantity with the governing combination identified.

### Excluded
- Accidental combinations (later).
- Solving the structure (owned by `cap-frame-analysis-engine`).
- Member design checks (owned by `cap-eurocode-rc-design` and future steel/timber design).

# Relationships

- Parent: [OmniMechanics Platform](/project.md)
- Shared with: [2D Frame Analysis Engine](/capabilities/cap-frame-analysis-engine.md)
- Shared with: [Eurocode RC Design](/capabilities/cap-eurocode-rc-design.md)

# Notes

Decisions are recorded in the discovery notes (batches 2-4, 2026-10-31).
