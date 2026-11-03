import {
  Component,
  ChangeDetectionStrategy,
  ElementRef,
  viewChild,
  effect,
  OnDestroy,
  signal,
  input,
  output,
  computed,
} from "@angular/core";
import * as THREE from "three";
import { MeshData } from "../../services/mesh.service";
import { MaterialFieldResponse } from "../../services/material.service";
import {
  HarmonicSimulationResult,
  SimulationResult,
  TransientSimulationResult,
} from "../../services/solver.service";
import {
  BoundaryConditionAssignment,
  BoundaryFacetPickEvent,
  BoundaryTag,
  CameraProjectionMode,
  ClippingPlaneConfig,
  ColormapPalette,
  ColormapType,
  computeAutoDeformationScale,
  derivePrincipalStressChannel,
  evaluateColormap,
  getColormapGradientCss,
  normalizeScalarFieldValues,
} from "./colormaps";
import {
  buildBoundaryElementGroup,
  buildBoundaryVectorGlyphs,
  buildVolumeAndEdgeGeometries,
  clampSphericalCamera,
  computeAutoFitCameraState,
  computeOrthographicFrustum,
  computeWorldClippingPlane,
  createDefaultBoundaryAssignments,
  disposeThreeObject,
  normalizeBoundaryTag,
  resolveBoundaryHitFromIntersections,
  TAG_COLORS,
} from "./viewport-scene.helpers";

export type {
  BoundaryConditionAssignment,
  BoundaryFacetPickEvent,
  BoundaryTag,
  CameraProjectionMode,
  ClippingPlaneConfig,
  ColormapPalette,
  ColormapType,
};

@Component({
  selector: "app-viewport",
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: "./viewport.component.html",
  styleUrl: "./viewport.component.css",
})
export class ViewportComponent implements OnDestroy {
  canvasContainer = viewChild.required<ElementRef<HTMLDivElement>>("canvasContainer");

  // Inputs
  activeDomain = input<"Static" | "TimeDomain" | "LaplaceTalbot">("Static");
  mesh = input<MeshData | null>(null);
  materialField = input<MaterialFieldResponse | null>(null);
  simulationResult = input<SimulationResult | null>(null);
  harmonicResult = input<HarmonicSimulationResult | null>(null);
  transientResult = input<TransientSimulationResult | null>(null);
  currentFrameIndex = input<number>(0);
  isPlayingAnimation = input<boolean>(false);

  // Outputs
  playAnimation = output<void>();
  pauseAnimation = output<void>();
  setFrameIndex = output<number>();
  boundaryFacetPicked = output<BoundaryFacetPickEvent>();

  toggleAnimationPlayback(): void {
    if (this.isPlayingAnimation()) {
      this.pauseAnimation.emit();
    } else {
      this.playAnimation.emit();
    }
  }

  onTimelineInput(event: Event): void {
    const val = +(event.target as HTMLInputElement).value;
    this.setFrameIndex.emit(val);
  }

  // Harmonic oscillation state
  harmonicPhase = signal<number>(0);
  isPlayingHarmonic = signal<boolean>(true);
  readonly harmonicPhaseDegrees = computed(() => Math.round((this.harmonicPhase() * 180) / Math.PI));
  private harmonicTimer: ReturnType<typeof setInterval> | null = null;

  toggleHarmonicPlayback(): void {
    if (this.isPlayingHarmonic()) {
      this.pauseHarmonicAnimation();
    } else {
      this.playHarmonicAnimation();
    }
  }

  playHarmonicAnimation(): void {
    this.isPlayingHarmonic.set(true);
    if (this.harmonicTimer) clearInterval(this.harmonicTimer);
    this.harmonicTimer = setInterval(() => {
      const next = (this.harmonicPhase() + 0.12) % (2 * Math.PI);
      this.harmonicPhase.set(next);
    }, 40);
  }

  pauseHarmonicAnimation(): void {
    this.isPlayingHarmonic.set(false);
    if (this.harmonicTimer) {
      clearInterval(this.harmonicTimer);
      this.harmonicTimer = null;
    }
  }

  onHarmonicPhaseInput(event: Event): void {
    this.pauseHarmonicAnimation();
    const deg = +(event.target as HTMLInputElement).value;
    this.harmonicPhase.set((deg * Math.PI) / 180);
  }

  // Camera Projection & Layer Visibility States
  cameraProjection = signal<CameraProjectionMode>("Perspective");
  showColormap = signal<boolean>(true);
  showWireframe = signal<boolean>(true);
  showBoundaries = signal<boolean>(true);
  showNodes = signal<boolean>(false);
  showVectorGlyphs = signal<boolean>(true);

  // Colormap Channel, Palette & Deformation States
  activeColormapType = signal<ColormapType>("Material");
  activePalette = signal<ColormapPalette>("Turbo");
  deformationScale = signal<number>(50.0);

  // 3D Dynamic Clipping Plane State (`SC-006`)
  clippingPlane = signal<ClippingPlaneConfig>({
    enabled: false,
    axis: "Z",
    normalizedOffset: 0.0,
    invertNormal: false,
  });

  // Interactive Raycast Boundary Facet Picking & BC Assignment State (`SC-007`)
  pickedBoundaryTag = signal<BoundaryTag | null>(null);
  lastPickedEvent = signal<BoundaryFacetPickEvent | null>(null);
  boundaryAssignments = signal<Map<BoundaryTag, BoundaryConditionAssignment>>(new Map());

  readonly pickedBoundaryAssignment = computed<BoundaryConditionAssignment | null>(() => {
    const tag = this.pickedBoundaryTag();
    if (!tag) return null;
    return (
      this.boundaryAssignments().get(tag) ?? {
        tag,
        kind: "Neumann",
        prescribedTraction: [0, 0, 0],
      }
    );
  });

  readonly activeSimData = computed(() => {
    const domain = this.activeDomain();
    const staticRes = this.simulationResult();
    const harmRes = this.harmonicResult();
    const transRes = this.transientResult();

    if (domain === "TimeDomain" && transRes && transRes.frames.length > 0) {
      const idx = Math.min(this.currentFrameIndex(), transRes.frames.length - 1);
      const frame = transRes.frames[idx];
      return {
        type: "transient" as const,
        time: frame.time,
        totalTime: transRes.time_points[transRes.time_points.length - 1],
        displacements: frame.displacements,
        von_mises_stresses: frame.von_mises_stresses,
        max_displacement: frame.max_displacement,
        max_von_mises: Math.max(...frame.von_mises_stresses, 0.0),
        converged: true,
        iterations: transRes.frames.length,
        execution_time_ms: transRes.execution_time_ms,
        mode: "Transient Talbot Wave",
        history: [] as { iteration: number; relative_error: number }[],
      };
    } else if (domain === "LaplaceTalbot" && harmRes) {
      const theta = this.harmonicPhase();
      const numNodes = harmRes.amplitudes.length;
      const instantaneousDisp: [number, number, number][] = new Array(numNodes);
      for (let i = 0; i < numNodes; i++) {
        const [ax, ay, az] = harmRes.amplitudes[i];
        const [px, py, pz] = harmRes.phase_lags[i];
        instantaneousDisp[i] = [
          ax * Math.cos(theta - px),
          ay * Math.cos(theta - py),
          az * Math.cos(theta - pz),
        ];
      }
      const cosThetaFactor = Math.abs(Math.cos(theta));
      const instantaneousStress = harmRes.von_mises_stresses.map(
        (s) => s * (0.2 + 0.8 * cosThetaFactor)
      );

      return {
        type: "harmonic" as const,
        time: (theta / (2 * Math.PI)) * (1 / harmRes.frequency_hz),
        totalTime: 1 / harmRes.frequency_hz,
        displacements: instantaneousDisp,
        von_mises_stresses: instantaneousStress,
        max_displacement: harmRes.max_amplitude,
        max_von_mises: harmRes.max_von_mises,
        converged: harmRes.converged,
        iterations: harmRes.iterations,
        execution_time_ms: harmRes.execution_time_ms,
        mode: `Harmonic ${harmRes.frequency_hz.toFixed(1)} Hz (GMRES)`,
        history: [{ iteration: harmRes.iterations, relative_error: harmRes.relative_residual }],
      };
    } else if (staticRes) {
      return {
        type: "static" as const,
        time: 0,
        totalTime: 0,
        displacements: staticRes.displacements,
        von_mises_stresses: staticRes.von_mises_stresses,
        max_displacement: staticRes.max_displacement,
        max_von_mises: staticRes.max_von_mises,
        converged: staticRes.converged,
        iterations: staticRes.iterations,
        execution_time_ms: staticRes.execution_time_ms,
        mode: staticRes.solver_mode as string,
        history: staticRes.convergence_history,
      };
    }
    return null;
  });

  // Computed Boundary Tag Legend directly from mesh data + active BC assignments
  readonly detectedTags = computed(() => {
    const m = this.mesh();
    if (!m) return [];
    const assignments = this.boundaryAssignments();
    const picked = this.pickedBoundaryTag();
    const tagMap = new Map<BoundaryTag, number>();
    for (const bnd of m.boundary_elements) {
      const tag = normalizeBoundaryTag(bnd.tag);
      tagMap.set(tag, (tagMap.get(tag) ?? 0) + 1);
    }
    return Array.from(tagMap.entries()).map(([name, count]) => {
      const colorVal = TAG_COLORS[name] ?? TAG_COLORS["Default"];
      const hex = `#${colorVal.toString(16).padStart(6, "0")}`;
      const assignedCondition = assignments.get(name);
      return {
        name,
        hex,
        count,
        isPicked: picked === name,
        assignedCondition,
      };
    });
  });

  // Computed Colorbar Information (`SC-008`, `SC-009`)
  readonly activeColormapInfo = computed(() => {
    const sim = this.activeSimData();
    const type = this.activeColormapType();
    const palette = this.activePalette();
    const paletteGradientCss = getColormapGradientCss(palette);

    if (type === "VonMises" && sim) {
      return {
        title: "von Mises Stress \u03c3_vM",
        min: 0.0,
        max: sim.max_von_mises > 0 ? sim.max_von_mises : 1.0,
        unit: "MPa",
        paletteGradientCss,
      };
    } else if (type === "PrincipalStress" && sim) {
      const pStresses = derivePrincipalStressChannel(sim.von_mises_stresses, sim.displacements);
      const norm = normalizeScalarFieldValues(pStresses);
      return {
        title: "Principal Stress \u03c3_1",
        min: norm.min,
        max: norm.max,
        unit: "MPa",
        paletteGradientCss,
      };
    } else if (type === "Displacement" && sim) {
      return {
        title: "Displacement ||u||",
        min: 0.0,
        max: sim.max_displacement * 1000.0 > 0 ? sim.max_displacement * 1000.0 : 1.0,
        unit: "mm",
        paletteGradientCss,
      };
    } else {
      const field = this.materialField();
      if (!field) return null;
      return {
        title: field.property_name,
        min: field.min_val,
        max: field.max_val,
        unit: field.unit,
        paletteGradientCss,
      };
    }
  });

  private scene!: THREE.Scene;
  private perspectiveCamera!: THREE.PerspectiveCamera;
  private orthographicCamera!: THREE.OrthographicCamera;
  private activeCamera!: THREE.PerspectiveCamera | THREE.OrthographicCamera;
  private renderer: THREE.WebGLRenderer | null = null;
  private animFrameId: number | null = null;
  private resizeObserver: ResizeObserver | null = null;
  private lastFittedMesh: MeshData | null = null;

  // Active mesh 3D root group
  private meshGroup: THREE.Group | null = null;
  private volumeMesh: THREE.Mesh | null = null;
  private wireframeMesh: THREE.LineSegments | null = null;
  private boundaryGroup: THREE.Group | null = null;
  private nodePoints: THREE.Points | null = null;
  private glyphGroup: THREE.Group | null = null;
  private threeClippingPlanes: THREE.Plane[] = [];

  // Orbit & Raycast interaction state
  private isDragging = false;
  private isRightDragging = false;
  private pointerDownPos = { x: 0, y: 0 };
  private previousMousePosition = { x: 0, y: 0 };
  private spherical = { radius: 15, theta: 0, phi: Math.PI / 2.05 };
  private target = new THREE.Vector3(0, 0, 0);

  constructor() {
    effect(() => {
      const containerRef = this.canvasContainer();
      if (containerRef && !this.renderer) {
        this.initThree(containerRef.nativeElement);
      }
    });

    // Reset boundary assignments and selection when mesh changes (`BR-3`)
    effect(() => {
      const currentMesh = this.mesh();
      if (currentMesh && currentMesh !== this.lastFittedMesh) {
        this.pickedBoundaryTag.set(null);
        this.lastPickedEvent.set(null);
        this.boundaryAssignments.set(createDefaultBoundaryAssignments(currentMesh));
      }
    });

    // Re-render mesh whenever geometry, field, simulation, palette, clipping, or picking changes
    effect(() => {
      const currentMesh = this.mesh();
      const field = this.materialField();
      const sim = this.activeSimData();
      const scale = this.deformationScale();
      const colormapType = this.activeColormapType();
      const palette = this.activePalette();
      const colormapActive = this.showColormap();
      const clipCfg = this.clippingPlane();
      const pickedTag = this.pickedBoundaryTag();
      const assignments = this.boundaryAssignments();

      if (this.scene && currentMesh) {
        this.renderMesh(
          currentMesh,
          field,
          sim,
          scale,
          colormapType,
          palette,
          colormapActive,
          clipCfg,
          pickedTag,
          assignments
        );
      }
    });

    effect(() => {
      const wf = this.showWireframe();
      if (this.wireframeMesh) this.wireframeMesh.visible = wf;
    });

    effect(() => {
      const bnd = this.showBoundaries();
      if (this.boundaryGroup) this.boundaryGroup.visible = bnd;
    });

    effect(() => {
      const nds = this.showNodes();
      if (this.nodePoints) this.nodePoints.visible = nds;
    });

    effect(() => {
      const glyphs = this.showVectorGlyphs();
      if (this.glyphGroup) this.glyphGroup.visible = glyphs;
    });

    // Manage harmonic vibration timer based on active domain & harmonicResult
    effect(() => {
      const harm = this.harmonicResult();
      const domain = this.activeDomain();
      if (domain === "LaplaceTalbot" && harm) {
        if (!this.harmonicTimer && this.isPlayingHarmonic()) {
          this.playHarmonicAnimation();
        }
      } else {
        this.pauseHarmonicAnimation();
      }
    });

    // Auto-scale deformation on simulation completion (`POL-1`, `SC-009`)
    effect(() => {
      const sim = this.activeSimData();
      const m = this.mesh();
      if (sim && sim.max_displacement > 1e-12 && m && m.nodes.length > 0) {
        let minX = Infinity, maxX = -Infinity;
        for (const [x] of m.nodes) {
          if (x < minX) minX = x;
          if (x > maxX) maxX = x;
        }
        const L = Math.max(maxX - minX, 1.0);
        const visibleDeflection = sim.max_displacement * this.deformationScale();
        if (visibleDeflection < 0.02 * L || visibleDeflection > 0.4 * L) {
          this.autoScale();
        }
      }
    });
  }

  autoScale(): void {
    const sim = this.activeSimData();
    const m = this.mesh();
    if (!sim || !m || sim.max_displacement <= 1e-12) return;
    this.deformationScale.set(computeAutoDeformationScale(m.nodes, sim.max_displacement));
  }

  setColormapType(type: ColormapType): void {
    this.activeColormapType.set(type);
  }

  setPalette(palette: ColormapPalette): void {
    this.activePalette.set(palette);
  }

  setCameraProjection(mode: CameraProjectionMode): void {
    this.cameraProjection.set(mode);
    if (this.perspectiveCamera && this.orthographicCamera) {
      this.activeCamera =
        mode === "Orthographic" ? this.orthographicCamera : this.perspectiveCamera;
      this.updateCameraFromSpherical();
    }
  }

  toggleCameraProjection(): void {
    const next: CameraProjectionMode =
      this.cameraProjection() === "Perspective" ? "Orthographic" : "Perspective";
    this.setCameraProjection(next);
  }

  toggleClippingPlane(): void {
    this.clippingPlane.update((cfg) => ({ ...cfg, enabled: !cfg.enabled }));
  }

  setClippingAxis(axis: "X" | "Y" | "Z"): void {
    this.clippingPlane.update((cfg) => ({ ...cfg, axis }));
  }

  onClippingOffsetInput(event: Event): void {
    const val = +(event.target as HTMLInputElement).value;
    this.clippingPlane.update((cfg) => ({
      ...cfg,
      normalizedOffset: Math.max(-1, Math.min(1, val)),
    }));
  }

  toggleClippingInvert(): void {
    this.clippingPlane.update((cfg) => ({ ...cfg, invertNormal: !cfg.invertNormal }));
  }

  selectBoundaryTagFromLegend(tag: BoundaryTag): void {
    this.pickedBoundaryTag.update((cur) => (cur === tag ? null : tag));
  }

  clearPickedBoundaryTag(): void {
    this.pickedBoundaryTag.set(null);
  }

  assignBoundaryConditionKind(kind: "Dirichlet" | "Neumann" | "Robin"): void {
    const tag = this.pickedBoundaryTag();
    if (!tag) return;
    const nextMap = new Map(this.boundaryAssignments());
    if (kind === "Dirichlet") {
      nextMap.set(tag, { tag, kind: "Dirichlet", prescribedDisplacement: [0, 0, 0] });
    } else if (kind === "Neumann") {
      nextMap.set(tag, { tag, kind: "Neumann", prescribedTraction: [0, -50, 0] });
    } else {
      nextMap.set(tag, {
        tag,
        kind: "Robin",
        robinImpedance: { eMediumMpa: 150, nuMedium: 0.3, rhoMedium: 2000 },
      });
    }
    this.boundaryAssignments.set(nextMap);
  }

  toggleColormap(): void {
    this.showColormap.update((v) => !v);
  }

  toggleWireframe(): void {
    this.showWireframe.update((v) => !v);
  }

  toggleBoundaries(): void {
    this.showBoundaries.update((v) => !v);
  }

  toggleNodes(): void {
    this.showNodes.update((v) => !v);
  }

  toggleVectorGlyphs(): void {
    this.showVectorGlyphs.update((v) => !v);
  }

  resetView(): void {
    if (!this.mesh()) {
      this.spherical = { radius: 12, theta: 0, phi: Math.PI / 2.05 };
      this.target.set(0, 0, 0);
      this.updateCameraFromSpherical();
      return;
    }
    this.fitCameraToMesh(this.mesh()!);
  }

  ngOnDestroy(): void {
    this.pauseHarmonicAnimation();
    if (this.animFrameId !== null) {
      cancelAnimationFrame(this.animFrameId);
    }
    if (this.resizeObserver) {
      this.resizeObserver.disconnect();
    }
    if (this.meshGroup) {
      disposeThreeObject(this.meshGroup);
      this.meshGroup = null;
    }
    if (this.renderer) {
      this.renderer.dispose();
      this.renderer = null;
    }
  }

  private initThree(container: HTMLDivElement): void {
    if (this.renderer) return;

    const width = container.clientWidth || 800;
    const height = container.clientHeight || 600;
    const aspect = width / height;

    this.scene = new THREE.Scene();
    this.scene.background = new THREE.Color("#090d16");

    this.perspectiveCamera = new THREE.PerspectiveCamera(45, aspect, 0.05, 2000);
    const frustum = computeOrthographicFrustum(this.spherical.radius, aspect);
    this.orthographicCamera = new THREE.OrthographicCamera(
      frustum.left,
      frustum.right,
      frustum.top,
      frustum.bottom,
      -500,
      2000
    );
    this.activeCamera =
      this.cameraProjection() === "Orthographic"
        ? this.orthographicCamera
        : this.perspectiveCamera;
    this.updateCameraFromSpherical();

    this.renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
    this.renderer.setSize(width, height);
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    this.renderer.shadowMap.enabled = true;
    this.renderer.localClippingEnabled = true;
    container.appendChild(this.renderer.domElement);

    const ambientLight = new THREE.AmbientLight(0xffffff, 0.7);
    this.scene.add(ambientLight);

    const dirLight1 = new THREE.DirectionalLight(0xffffff, 0.85);
    dirLight1.position.set(20, 40, 30);
    dirLight1.castShadow = true;
    this.scene.add(dirLight1);

    const dirLight2 = new THREE.DirectionalLight(0x38bdf8, 0.35);
    dirLight2.position.set(-20, -10, -20);
    this.scene.add(dirLight2);

    const grid = new THREE.GridHelper(40, 40, 0x334155, 0x1e293b);
    grid.position.y = -0.01;
    this.scene.add(grid);

    this.setupInteraction(container);

    this.resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const { width: w, height: h } = entry.contentRect;
        if (w > 0 && h > 0) {
          this.perspectiveCamera.aspect = w / h;
          this.perspectiveCamera.updateProjectionMatrix();
          this.syncOrthographicFrustum(w / h);
          this.renderer?.setSize(w, h);
        }
      }
    });
    this.resizeObserver.observe(container);

    const animate = () => {
      this.animFrameId = requestAnimationFrame(animate);
      if (this.renderer && this.scene && this.activeCamera) {
        this.renderer.render(this.scene, this.activeCamera);
      }
    };
    animate();

    if (this.mesh()) {
      this.renderMesh(
        this.mesh()!,
        this.materialField(),
        this.activeSimData(),
        this.deformationScale(),
        this.activeColormapType(),
        this.activePalette(),
        this.showColormap(),
        this.clippingPlane(),
        this.pickedBoundaryTag(),
        this.boundaryAssignments()
      );
    }
  }

  private syncOrthographicFrustum(aspectInput?: number): void {
    if (!this.orthographicCamera || !this.perspectiveCamera) return;
    const aspect = aspectInput ?? this.perspectiveCamera.aspect ?? 1.0;
    const frustum = computeOrthographicFrustum(this.spherical.radius, aspect);
    this.orthographicCamera.left = frustum.left;
    this.orthographicCamera.right = frustum.right;
    this.orthographicCamera.top = frustum.top;
    this.orthographicCamera.bottom = frustum.bottom;
    this.orthographicCamera.updateProjectionMatrix();
  }

  private setupInteraction(container: HTMLDivElement): void {
    container.addEventListener("contextmenu", (e) => e.preventDefault());

    container.addEventListener("mousedown", (e: MouseEvent) => {
      if (e.button === 0) {
        this.isDragging = true;
      } else if (e.button === 2) {
        this.isRightDragging = true;
      }
      this.pointerDownPos = { x: e.clientX, y: e.clientY };
      this.previousMousePosition = { x: e.clientX, y: e.clientY };
    });

    window.addEventListener("mouseup", (e: MouseEvent) => {
      const wasLeftDrag = this.isDragging;
      this.isDragging = false;
      this.isRightDragging = false;

      if (wasLeftDrag && e.button === 0) {
        const dist = Math.hypot(
          e.clientX - this.pointerDownPos.x,
          e.clientY - this.pointerDownPos.y
        );
        if (dist < 4 && container.contains(e.target as Node)) {
          this.handleCanvasPickClick(e, container);
        }
      }
    });

    container.addEventListener("mousemove", (e: MouseEvent) => {
      if (!this.isDragging && !this.isRightDragging) return;

      const deltaX = e.clientX - this.previousMousePosition.x;
      const deltaY = e.clientY - this.previousMousePosition.y;
      this.previousMousePosition = { x: e.clientX, y: e.clientY };

      if (this.isDragging) {
        this.spherical = clampSphericalCamera({
          ...this.spherical,
          theta: this.spherical.theta - deltaX * 0.008,
          phi: this.spherical.phi - deltaY * 0.008,
        });
        this.updateCameraFromSpherical();
      } else if (this.isRightDragging) {
        const panSpeed = this.spherical.radius * 0.0015;
        const forward = new THREE.Vector3();
        this.activeCamera.getWorldDirection(forward);
        const right = new THREE.Vector3().crossVectors(forward, this.activeCamera.up).normalize();
        const up = new THREE.Vector3().crossVectors(right, forward).normalize();

        this.target.addScaledVector(right, -deltaX * panSpeed);
        this.target.addScaledVector(up, deltaY * panSpeed);
        this.updateCameraFromSpherical();
      }
    });

    container.addEventListener(
      "wheel",
      (e: WheelEvent) => {
        e.preventDefault();
        const zoomFactor = e.deltaY > 0 ? 1.08 : 0.92;
        this.spherical = clampSphericalCamera({
          ...this.spherical,
          radius: this.spherical.radius * zoomFactor,
        });
        this.updateCameraFromSpherical();
      },
      { passive: false }
    );
  }

  /**
   * Raycast picking against `boundaryGroup` facets (`SC-007`).
   */
  private handleCanvasPickClick(e: MouseEvent, container: HTMLDivElement): void {
    // Ignore clicks on HUD overlay controls
    const targetEl = e.target as HTMLElement;
    if (this.renderer && targetEl !== this.renderer.domElement) {
      return;
    }
    if (!this.showBoundaries() || !this.boundaryGroup || !this.activeCamera) {
      return;
    }

    const rect = container.getBoundingClientRect();
    if (rect.width <= 0 || rect.height <= 0) return;

    const ndcX = ((e.clientX - rect.left) / rect.width) * 2 - 1;
    const ndcY = -((e.clientY - rect.top) / rect.height) * 2 + 1;

    const raycaster = new THREE.Raycaster();
    raycaster.params.Line = { threshold: Math.max(0.15, this.spherical.radius * 0.025) };
    raycaster.setFromCamera(new THREE.Vector2(ndcX, ndcY), this.activeCamera);

    const intersections = raycaster.intersectObjects(this.boundaryGroup.children, true);
    const resolved = resolveBoundaryHitFromIntersections(intersections);

    if ("hit" in resolved) {
      this.pickedBoundaryTag.set(resolved.hit.tag);
      this.lastPickedEvent.set(resolved.hit);
      this.boundaryFacetPicked.emit(resolved.hit);
    } else {
      // RAYCAST_NO_BOUNDARY_HIT clears selection highlight without mutating assignments (`SC-007`)
      this.pickedBoundaryTag.set(null);
    }
  }

  private updateCameraFromSpherical(): void {
    const { radius, theta, phi } = this.spherical;
    const x = this.target.x + radius * Math.sin(phi) * Math.sin(theta);
    const y = this.target.y + radius * Math.cos(phi);
    const z = this.target.z + radius * Math.sin(phi) * Math.cos(theta);

    if (this.perspectiveCamera) {
      this.perspectiveCamera.position.set(x, y, z);
      this.perspectiveCamera.lookAt(this.target);
    }
    if (this.orthographicCamera) {
      this.orthographicCamera.position.set(x, y, z);
      this.orthographicCamera.lookAt(this.target);
      this.syncOrthographicFrustum();
    }
  }

  private fitCameraToMesh(meshData: MeshData): void {
    if (meshData.nodes.length === 0) return;
    const autoFit = computeAutoFitCameraState(meshData);
    this.target.set(autoFit.target[0], autoFit.target[1], autoFit.target[2]);
    this.spherical = { ...autoFit.spherical };
    this.updateCameraFromSpherical();
  }

  private renderMesh(
    meshData: MeshData,
    field: MaterialFieldResponse | null,
    sim: {
      displacements?: [number, number, number][];
      von_mises_stresses?: number[];
      max_displacement?: number;
      max_von_mises?: number;
    } | null,
    scale: number,
    colormapType: ColormapType,
    palette: ColormapPalette,
    colormapActive: boolean,
    clipCfg: ClippingPlaneConfig,
    pickedTag: BoundaryTag | null,
    assignments: ReadonlyMap<BoundaryTag, BoundaryConditionAssignment>
  ): void {
    if (this.meshGroup) {
      this.scene.remove(this.meshGroup);
      disposeThreeObject(this.meshGroup);
      this.meshGroup.clear();
      this.meshGroup = null;
    }

    this.meshGroup = new THREE.Group();

    // 1. Compute displaced node coordinates
    const numNodes = meshData.nodes.length;
    const displacedNodes: [number, number, number][] = new Array(numNodes);
    for (let i = 0; i < numNodes; i++) {
      const orig = meshData.nodes[i];
      if (sim && sim.displacements && sim.displacements[i] && scale > 0) {
        const d = sim.displacements[i];
        displacedNodes[i] = [
          orig[0] + d[0] * scale,
          orig[1] + d[1] * scale,
          orig[2] + d[2] * scale,
        ];
      } else {
        displacedNodes[i] = [orig[0], orig[1], orig[2]];
      }
    }

    // 2. Resolve 3D Clipping Plane (`SC-006`)
    this.threeClippingPlanes = [];
    if (clipCfg.enabled && meshData.dimension === 3) {
      const worldPlane = computeWorldClippingPlane(clipCfg, displacedNodes);
      const plane = new THREE.Plane(
        new THREE.Vector3(worldPlane.normal[0], worldPlane.normal[1], worldPlane.normal[2]),
        worldPlane.constant
      );
      this.threeClippingPlanes = [plane];
    }

    // 3. Determine colormap values and span (`SC-008`)
    let colorValues: number[] | null = null;
    let minVal = 0.0;
    let maxVal = 1.0;

    if (colormapActive) {
      if (colormapType === "VonMises" && sim?.von_mises_stresses?.length === numNodes) {
        colorValues = sim.von_mises_stresses;
        minVal = 0.0;
        maxVal = (sim.max_von_mises ?? 1.0) > 0 ? (sim.max_von_mises ?? 1.0) : 1.0;
      } else if (colormapType === "PrincipalStress" && sim?.von_mises_stresses?.length === numNodes) {
        colorValues = derivePrincipalStressChannel(sim.von_mises_stresses, sim.displacements);
        const norm = normalizeScalarFieldValues(colorValues);
        minVal = norm.min;
        maxVal = norm.max;
      } else if (colormapType === "Displacement" && sim?.displacements?.length === numNodes) {
        colorValues = sim.displacements.map(
          ([ux, uy, uz]) => Math.sqrt(ux * ux + uy * uy + uz * uz) * 1000.0
        );
        minVal = 0.0;
        const maxD = sim.max_displacement ?? 0.001;
        maxVal = maxD * 1000.0 > 0 ? maxD * 1000.0 : 1.0;
      } else if (colormapType === "Material" && field && field.values.length === numNodes) {
        colorValues = field.values;
        minVal = field.min_val;
        maxVal = field.max_val;
      }
    }

    // 4. Build Volume Geometry
    const { volumeGeometry, edgeGeometry } = buildVolumeAndEdgeGeometries(
      meshData,
      displacedNodes,
      colorValues,
      minVal,
      maxVal,
      palette,
      colormapActive
    );

    const hasColors = volumeGeometry.hasAttribute("color");
    const volumeMat = new THREE.MeshStandardMaterial({
      color: hasColors ? 0xffffff : 0x2563eb,
      vertexColors: hasColors,
      side: THREE.DoubleSide,
      roughness: 0.45,
      metalness: 0.1,
      polygonOffset: true,
      polygonOffsetFactor: 1,
      polygonOffsetUnits: 1,
      clippingPlanes: this.threeClippingPlanes,
      clipShadows: true,
    });

    this.volumeMesh = new THREE.Mesh(volumeGeometry, volumeMat);
    this.meshGroup.add(this.volumeMesh);

    // 5. Wireframe lines
    const wireframeMat = new THREE.LineBasicMaterial({
      color: hasColors ? 0x0f172a : 0x475569,
      transparent: true,
      opacity: hasColors ? 0.35 : 0.45,
      clippingPlanes: this.threeClippingPlanes,
    });
    this.wireframeMesh = new THREE.LineSegments(edgeGeometry, wireframeMat);
    this.wireframeMesh.visible = this.showWireframe();
    this.meshGroup.add(this.wireframeMesh);

    // 6. Boundary Elements colored by tag + raycast highlight (`SC-007`)
    this.boundaryGroup = buildBoundaryElementGroup(
      meshData,
      displacedNodes,
      pickedTag,
      this.threeClippingPlanes
    );
    this.boundaryGroup.visible = this.showBoundaries();
    this.meshGroup.add(this.boundaryGroup);

    // 7. Vector Glyph Overlay (`WF-4`)
    this.glyphGroup = buildBoundaryVectorGlyphs(
      meshData,
      displacedNodes,
      assignments,
      this.threeClippingPlanes
    );
    this.glyphGroup.visible = this.showVectorGlyphs();
    this.meshGroup.add(this.glyphGroup);

    // 8. Node markers (Points)
    const nodeGeometry = new THREE.BufferGeometry();
    const nodeCoords = new Float32Array(numNodes * 3);
    for (let i = 0; i < numNodes; i++) {
      nodeCoords[i * 3 + 0] = displacedNodes[i][0];
      nodeCoords[i * 3 + 1] = displacedNodes[i][1];
      nodeCoords[i * 3 + 2] = displacedNodes[i][2];
    }
    nodeGeometry.setAttribute("position", new THREE.BufferAttribute(nodeCoords, 3));
    const nodeMaterial = new THREE.PointsMaterial({
      color: 0x38bdf8,
      size: 4,
      sizeAttenuation: false,
      clippingPlanes: this.threeClippingPlanes,
    });
    this.nodePoints = new THREE.Points(nodeGeometry, nodeMaterial);
    this.nodePoints.visible = this.showNodes();
    this.meshGroup.add(this.nodePoints);

    this.scene.add(this.meshGroup);

    // Only auto-fit camera when the mesh reference itself changes
    if (this.lastFittedMesh !== meshData) {
      this.lastFittedMesh = meshData;
      this.fitCameraToMesh(meshData);
    }
  }
}
