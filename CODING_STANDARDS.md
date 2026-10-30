# OmniMechanics Coding Standards & Architectural Guidelines

## 1. Purpose and Scope

These standards apply to the **OmniMechanics** desktop application: an Angular 22 frontend, a Tauri 2.0 desktop boundary, and high-performance Rust simulation services (BEM, FEM, Dynamic Laplace-Talbot, Robin-Schwarz Coupling, Eurocode RC Design, Structural Mechanics).

The application is local-first:
- **Frontend (Angular):** Presents state, user intent, interactive 2D/3D visualizers, educational modules, and CAD canvas.
- **Backend (Rust):** Owns intensive numerical computation, meshing, solvers, native system integration, and memory management.
- **Tauri IPC:** Connects the two through strictly typed commands, DTOs, and event channels.

---

## 2. SOLID Architectural Principles

Every module, component, and service must strictly adhere to SOLID:

1. **Single Responsibility Principle (SRP):**
   - Each component/class/module has exactly one reason to change.
   - HTML templates must never exceed **500 lines**.
   - Code files (Rust, TypeScript, JavaScript) must never exceed **1,000 lines**.
   - UI components do not perform direct IPC or mathematical orchestration; they delegate to dedicated services and facades.

2. **Open/Closed Principle (OCP):**
   - Solvers, mesh generators, and visualizers must be extensible without modifying existing core loops (e.g. trait/interface polymorphism for Solvers and Renderers).

3. **Liskov Substitution Principle (LSP):**
   - Solver engines (PureFEM, PureBEM, BEM-FEM, DynamicTalbot) satisfy uniform solver contracts and exchange compatible state representations.

4. **Interface Segregation Principle (ISP):**
   - Clients should not depend on interfaces they do not use. Prefer small, focused interfaces and input/output contracts over monolithic god-objects.

5. **Dependency Inversion Principle (DIP):**
   - High-level modules do not depend on low-level details. Both depend on abstractions (traits in Rust, injectable tokens/interfaces in Angular).

---

## 3. Angular Frontend Standards

- **Standalone Components & OnPush:**
  - All components use standalone APIs with `changeDetection: ChangeDetectionStrategy.OnPush`.
  - Zoneless reactivity via Signals (`signal()`, `computed()`, `linkedSignal()`, `model()`).
- **Signal Forms & Input Validation:**
  - **All forms and parameter controls must be Signal Forms** using the modern Angular Signal Forms API (`@angular/forms/signals` with `form()`, `FormField`, and schema validators).
  - **Zero legacy forms**: Strictly prohibited to use legacy Reactive Forms (`FormGroup`, `FormControl`, `FormBuilder`) or legacy Template-driven Forms (`[(ngModel)]`, `ngForm`).
  - **Schema Validation**: Define schema validators (`min`, `max`, `required`, `validate`) directly on signal models to enforce physical invariants (e.g. Young's modulus $E > 0$, Poisson ratio $0 < \nu < 0.5$, mesh divisions $\ge 2$, load magnitude finite).
  - Two-way component parameter flows between parents and child tab components use Angular's signal `model()` / `model.required()`.
- **Control Flow:**
  - Strict use of native `@if`, `@for`, `@switch`. Every `@for` loop must have a unique `@for (...; track item.id)` track expression.
- **File Organization & Folder Structure:**
  ```text
  src/app/
  ├── core/               # App-wide singleton services, Tauri IPC adapters, models
  │   ├── adapters/       # Tauri command invoke wrappers
  │   ├── models/         # TypeScript interfaces & DTO contracts
  │   └── state/          # Global application state signals
  ├── shared/             # Reusable UI primitives, widgets, pipes, directives
  │   ├── button/
  │   ├── dropdown/
  │   └── modal/
  ├── features/           # Organized by capability / workspace mode
  │   ├── workbench/      # Mode 1: Benchmark & Verification Workbench
  │   │   ├── components/ # Toolbar, Inspector, Parameter docks
  │   │   └── services/   # Workbench facade
  │   ├── cad-studio/     # Mode 2: CAD Sketcher & Meshing Studio
  │   ├── teaching/       # Mode 3: Structural Mechanics & Eurocode Studio
  │   └── viewport/       # 2D/3D Rendering Engine & Three.js Canvas
  │       ├── renderers/  # Modular 2D Canvas & Three.js sub-renderers
  │       └── controls/   # Camera, pan, zoom, selection interactions
  └── app.component.*     # Clean composition shell (< 500 lines HTML)
  ```
- **Strict Size Limits:**
  - HTML templates: `<= 500 lines`.
  - TypeScript classes/files: `<= 1,000 lines`.

---

## 4. Rust Backend Standards

- **Separation of Tests:**
  - **No inline tests in production files.** Production files in `src-tauri/src/` must not contain `#[cfg(test)] mod tests { ... }`.
  - Unit tests belong in dedicated companion files (e.g. `talbot_test.rs`) or integration tests in `src-tauri/tests/`.
- **Modular Solvers:**
  - Mathematical solvers (`bem_solver.rs`, `dynamic_bem.rs`, `fem_solver.rs`, `oraiopoulos_bem_fem.rs`) must be decomposed into focused submodules (Assembly, Kernels, Boundary Conditions, Post-processing) each `<= 1,000 lines`.
- **Safety & Error Handling:**
  - No `unwrap()` or `expect()` in production pathways; return typed `Result<T, SolverError>`.
  - Keep Tauri command handlers thin: validate inputs, invoke domain services, and return typed serializable DTOs.

---

## 5. Quality Gates & Pre-Commit Enforcement

Before every commit, the Git hook automatically executes `node scripts/check-code-limits.mjs`.
A commit is **strictly blocked** if:
1. Any `.rs`, `.ts`, or `.js` file exceeds 1,000 lines.
2. Any `.html` file exceeds 500 lines.
3. Any production Rust file contains inline `#[cfg(test)]` blocks.
