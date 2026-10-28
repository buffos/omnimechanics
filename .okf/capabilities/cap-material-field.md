---
type: capability
title: Material Modeling & Open-Closed Spatial Inhomogeneity Engine
description: Definition of reference homogeneous elasticity and extensible spatial perturbation fields (gradients, mathematical expressions, localized inclusions) conforming to the Open-Closed Principle.
tags: [materials, fgm, inhomogeneity, open-closed, elasticity, stiffness-field]
timestamp: 2026-10-28T05:18:00Z
state: implemented
project: /project.md
parent: /project.md
issues:
  - docs/agents/issues/done/20261028-003-material-inhomogeneity-field.md
---

# Intent

Provide a rigorous, modular material definition framework where the baseline homogeneous reference parameters $(\mathbf{C}_0, \rho_0)$ and spatial perturbation fields $(\Delta \mathbf{C}(\mathbf{x}), \Delta \rho(\mathbf{x}))$ are decoupled and easily extensible without modifying the underlying solver code (Open-Closed Principle).

# Scope

### Included:
- **Reference Material Specification:**
  - Homogeneous baseline parameters: Young's modulus $E_0$, Poisson's ratio $\nu_0$, shear modulus $\mu_0$, Lamé parameters $\lambda_0$, and mass density $\rho_0$.
- **Inhomogeneity Strategy Interface (`MaterialInhomogeneityField`):**
  - **Type 1 (Directional Gradient / FGM):** Continuous linear, quadratic, or exponential gradient along coordinate axes (e.g. $\mu(y) = \mu_0 (1 + \alpha y/L)^2$).
  - **Type 2 (Math Parser Expression):** User-defined mathematical expressions evaluated at Gauss points (e.g., $E(x,y) = E_0 (1 + 0.2 \sin(\pi x/L))$).
  - **Type 3 (Localized Inclusions / Geometric Defects):** Subdomains (circles, spheres, cylinders) possessing distinct moduli $(E_1, \nu_1, \rho_1)$ representing inclusions, voids, or localized damage.
  - **Type 4 (Non-Linear Vecchio-Collins Concrete / MCFT):** Modified Compression Field Theory modeling non-linear concrete in compression, tension stiffening, and cracked shear response based on Vecchio & Collins (1986).
  - **Type 5 (J2 von Mises Elastoplasticity):** Metal plasticity with yield surface, isotropic/kinematic hardening, and radial return-mapping algorithm.
  - **Extensibility:** Interface permits new material strategies without modifying solver loops.
- **Evaluation & Field Sampling:**
  - Fast point-evaluation at numerical quadrature points (Gauss points of FEM volume elements).
  - Pre-calculation of scalar heatmap field for interactive 3D rendering.

### Excluded:
- Anisotropic monoclinic/triclinic crystals (focus is isotropic with spatially varying scalar moduli).

# Relationships

- Parent: [BemApp Platform](/project.md)
- Shared with: [Solver Core Engine](/capabilities/cap-solver-core.md)
- Shared with: [Interactive 3D Canvas](/capabilities/cap-interactive-canvas.md)
