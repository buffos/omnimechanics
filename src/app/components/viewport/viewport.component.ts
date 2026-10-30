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

// Color mapping for boundary tags
const TAG_COLORS: Record<string, number> = {
  Left: 0x3b82f6,    // Blue (Fixed support)
  Right: 0xef4444,   // Red (Traction load)
  Top: 0xf59e0b,     // Amber / Yellow
  Bottom: 0x10b981,  // Emerald green
  Hole: 0xa855f7,    // Purple / Cavity
  Front: 0x06b6d4,   // Cyan
  Back: 0x6366f1,    // Indigo
  Default: 0x94a3b8, // Slate gray
};

export type ColormapType = "Material" | "VonMises" | "Displacement";

import { turboColormap } from './colormaps';

@Component({
  selector: 'app-viewport',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './viewport.component.html',
  styleUrl: './viewport.component.css',
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

  playAnimation = output<void>();
  pauseAnimation = output<void>();
  setFrameIndex = output<number>();

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
  private harmonicTimer: any = null;

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
      // Step phase forward gracefully (~7 deg per 40ms)
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

  // Colormap & Deformation States
  showColormap = signal<boolean>(true);
  showWireframe = signal<boolean>(true);
  showBoundaries = signal<boolean>(true);
  showNodes = signal<boolean>(false);

  activeColormapType = signal<ColormapType>("Material");
  deformationScale = signal<number>(50.0);

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

  // Computed Boundary Tag Legend directly from mesh data
  readonly detectedTags = computed(() => {
    const m = this.mesh();
    if (!m) return [];
    const tagMap = new Map<string, number>();
    for (const bnd of m.boundary_elements) {
      tagMap.set(bnd.tag, (tagMap.get(bnd.tag) ?? 0) + 1);
    }
    return Array.from(tagMap.entries()).map(([name, count]) => {
      const colorVal = TAG_COLORS[name] ?? TAG_COLORS["Default"];
      const hex = `#${colorVal.toString(16).padStart(6, "0")}`;
      return { name, hex, count };
    });
  });

  // Computed Colorbar Information
  readonly activeColormapInfo = computed(() => {
    const sim = this.activeSimData();
    const type = this.activeColormapType();

    if (type === "VonMises" && sim) {
      return {
        title: "von Mises Stress \u03c3_vM",
        min: 0.0,
        max: sim.max_von_mises > 0 ? sim.max_von_mises : 1.0,
        unit: "MPa",
      };
    } else if (type === "Displacement" && sim) {
      return {
        title: "Displacement ||u||",
        min: 0.0,
        max: sim.max_displacement * 1000.0 > 0 ? sim.max_displacement * 1000.0 : 1.0,
        unit: "mm",
      };
    } else {
      const field = this.materialField();
      if (!field) return null;
      return {
        title: field.property_name,
        min: field.min_val,
        max: field.max_val,
        unit: field.unit,
      };
    }
  });

  private scene!: THREE.Scene;
  private camera!: THREE.PerspectiveCamera;
  private renderer: THREE.WebGLRenderer | null = null;
  private animFrameId: number | null = null;
  private resizeObserver: ResizeObserver | null = null;

  // Active mesh 3D root group
  private meshGroup: THREE.Group | null = null;
  private volumeMesh: THREE.Mesh | null = null;
  private wireframeMesh: THREE.LineSegments | null = null;
  private boundaryGroup: THREE.Group | null = null;
  private nodePoints: THREE.Points | null = null;

  // Orbit controls state
  private isDragging = false;
  private isRightDragging = false;
  private previousMousePosition = { x: 0, y: 0 };
  private spherical = { radius: 15, theta: 0, phi: Math.PI / 2.05 };
  private target = new THREE.Vector3(0, 0, 0);

  constructor() {
    // Initialize Three.js once container is available
    effect(() => {
      const containerRef = this.canvasContainer();
      if (containerRef && !this.renderer) {
        this.initThree(containerRef.nativeElement);
      }
    });

    // Re-render mesh whenever mesh, materialField, activeSimData, deformationScale, or colormap changes
    effect(() => {
      const currentMesh = this.mesh();
      const field = this.materialField();
      const sim = this.activeSimData();
      const scale = this.deformationScale();
      const colormapType = this.activeColormapType();
      const colormapActive = this.showColormap();

      if (this.scene && currentMesh) {
        this.renderMesh(currentMesh, field, sim, scale, colormapType, colormapActive);
      }
    });

    // React to toggle changes
    effect(() => {
      const wf = this.showWireframe();
      if (this.wireframeMesh) {
        this.wireframeMesh.visible = wf;
      }
    });

    effect(() => {
      const bnd = this.showBoundaries();
      if (this.boundaryGroup) {
        this.boundaryGroup.visible = bnd;
      }
    });

    effect(() => {
      const nds = this.showNodes();
      if (this.nodePoints) {
        this.nodePoints.visible = nds;
      }
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

    // Auto-scale deformation on simulation completion so vibration is immediately visible
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
    let minX = Infinity, maxX = -Infinity;
    for (const [x] of m.nodes) {
      if (x < minX) minX = x;
      if (x > maxX) maxX = x;
    }
    const L = Math.max(maxX - minX, 1.0);
    // Standard FEA practice: normalize peak visual deformation to ~8% of structure dimension
    const targetDeflection = 0.08 * L;
    const computedScale = targetDeflection / sim.max_displacement;
    let scaleVal = Math.round(computedScale);
    if (scaleVal > 100) scaleVal = Math.round(scaleVal / 50) * 50;
    this.deformationScale.set(Math.max(1, scaleVal));
  }

  setColormapType(type: ColormapType): void {
    this.activeColormapType.set(type);
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
    if (this.renderer) {
      this.renderer.dispose();
      this.renderer = null;
    }
  }

  private initThree(container: HTMLDivElement): void {
    if (this.renderer) return;

    const width = container.clientWidth || 800;
    const height = container.clientHeight || 600;

    // 1. Scene
    this.scene = new THREE.Scene();
    this.scene.background = new THREE.Color("#090d16");

    // 2. Camera
    this.camera = new THREE.PerspectiveCamera(45, width / height, 0.05, 2000);
    this.updateCameraFromSpherical();

    // 3. Renderer
    this.renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
    this.renderer.setSize(width, height);
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    this.renderer.shadowMap.enabled = true;
    container.appendChild(this.renderer.domElement);

    // 4. Lighting
    const ambientLight = new THREE.AmbientLight(0xffffff, 0.7);
    this.scene.add(ambientLight);

    const dirLight1 = new THREE.DirectionalLight(0xffffff, 0.85);
    dirLight1.position.set(20, 40, 30);
    dirLight1.castShadow = true;
    this.scene.add(dirLight1);

    const dirLight2 = new THREE.DirectionalLight(0x38bdf8, 0.35);
    dirLight2.position.set(-20, -10, -20);
    this.scene.add(dirLight2);

    // 5. Grid Helper
    const grid = new THREE.GridHelper(40, 40, 0x334155, 0x1e293b);
    grid.position.y = -0.01;
    this.scene.add(grid);

    // 6. Interaction
    this.setupInteraction(container);

    // 7. Resize Observer
    this.resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const { width: w, height: h } = entry.contentRect;
        if (w > 0 && h > 0) {
          this.camera.aspect = w / h;
          this.camera.updateProjectionMatrix();
          this.renderer?.setSize(w, h);
        }
      }
    });
    this.resizeObserver.observe(container);

    // 8. Animation Loop
    const animate = () => {
      this.animFrameId = requestAnimationFrame(animate);
      if (this.renderer && this.scene && this.camera) {
        this.renderer.render(this.scene, this.camera);
      }
    };
    animate();

    // If mesh is already available, render it immediately
    if (this.mesh()) {
      this.renderMesh(
        this.mesh()!,
        this.materialField(),
        this.activeSimData(),
        this.deformationScale(),
        this.activeColormapType(),
        this.showColormap()
      );
    }
  }

  private setupInteraction(container: HTMLDivElement): void {
    container.addEventListener("contextmenu", (e) => e.preventDefault());

    container.addEventListener("mousedown", (e: MouseEvent) => {
      if (e.button === 0) {
        this.isDragging = true;
      } else if (e.button === 2) {
        this.isRightDragging = true;
      }
      this.previousMousePosition = { x: e.clientX, y: e.clientY };
    });

    window.addEventListener("mouseup", () => {
      this.isDragging = false;
      this.isRightDragging = false;
    });

    container.addEventListener("mousemove", (e: MouseEvent) => {
      if (!this.isDragging && !this.isRightDragging) return;

      const deltaX = e.clientX - this.previousMousePosition.x;
      const deltaY = e.clientY - this.previousMousePosition.y;
      this.previousMousePosition = { x: e.clientX, y: e.clientY };

      if (this.isDragging) {
        this.spherical.theta -= deltaX * 0.008;
        this.spherical.phi -= deltaY * 0.008;
        this.spherical.phi = Math.max(0.05, Math.min(Math.PI - 0.05, this.spherical.phi));
        this.updateCameraFromSpherical();
      } else if (this.isRightDragging) {
        const panSpeed = this.spherical.radius * 0.0015;
        const forward = new THREE.Vector3();
        this.camera.getWorldDirection(forward);
        const right = new THREE.Vector3().crossVectors(forward, this.camera.up).normalize();
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
        this.spherical.radius = Math.max(0.5, Math.min(250, this.spherical.radius * zoomFactor));
        this.updateCameraFromSpherical();
      },
      { passive: false }
    );
  }

  private updateCameraFromSpherical(): void {
    const { radius, theta, phi } = this.spherical;
    const x = this.target.x + radius * Math.sin(phi) * Math.sin(theta);
    const y = this.target.y + radius * Math.cos(phi);
    const z = this.target.z + radius * Math.sin(phi) * Math.cos(theta);

    this.camera.position.set(x, y, z);
    this.camera.lookAt(this.target);
  }

  private fitCameraToMesh(meshData: MeshData): void {
    if (meshData.nodes.length === 0) return;

    let minX = Infinity, minY = Infinity, minZ = Infinity;
    let maxX = -Infinity, maxY = -Infinity, maxZ = -Infinity;

    for (const [x, y, z] of meshData.nodes) {
      if (x < minX) minX = x;
      if (x > maxX) maxX = x;
      if (y < minY) minY = y;
      if (y > maxY) maxY = y;
      if (z < minZ) minZ = z;
      if (z > maxZ) maxZ = z;
    }

    const cx = (minX + maxX) / 2;
    const cy = (minY + maxY) / 2;
    const cz = (minZ + maxZ) / 2;
    this.target.set(cx, cy, cz);

    const dx = maxX - minX;
    const dy = maxY - minY;
    const dz = maxZ - minZ;
    const maxDim = Math.max(dx, dy, dz, 1.0);

    if (meshData.dimension === 2) {
      this.spherical.theta = 0;
      this.spherical.phi = Math.PI / 2.05;
      this.spherical.radius = maxDim * 1.8;
    } else {
      this.spherical.theta = Math.PI / 4;
      this.spherical.phi = Math.PI / 3;
      this.spherical.radius = maxDim * 2.0;
    }

    this.updateCameraFromSpherical();
  }

  /**
   * Builds and renders the full mesh representation in Three.js:
   * Supports deformed coordinates and field colormap (Material, von Mises stress, or Displacement).
   */
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
    colormapActive: boolean
  ): void {
    if (this.meshGroup) {
      this.scene.remove(this.meshGroup);
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

    // 2. Determine colormap values and span
    let colorValues: number[] | null = null;
    let minVal = 0.0;
    let maxVal = 1.0;

    if (colormapActive) {
      if (colormapType === "VonMises" && sim && sim.von_mises_stresses && sim.von_mises_stresses.length === numNodes) {
        colorValues = sim.von_mises_stresses;
        minVal = 0.0;
        maxVal = (sim.max_von_mises ?? 1.0) > 0 ? (sim.max_von_mises ?? 1.0) : 1.0;
      } else if (colormapType === "Displacement" && sim && sim.displacements && sim.displacements.length === numNodes) {
        colorValues = sim.displacements.map(([ux, uy, uz]) => Math.sqrt(ux * ux + uy * uy + uz * uz) * 1000.0);
        minVal = 0.0;
        const maxD = sim.max_displacement ?? 0.001;
        maxVal = maxD * 1000.0 > 0 ? maxD * 1000.0 : 1.0;
      } else if (colormapType === "Material" && field && field.values.length === numNodes) {
        colorValues = field.values;
        minVal = field.min_val;
        maxVal = field.max_val;
      }
    }

    // 3. Build Volume Geometry
    const { volumeGeometry, edgeGeometry } = this.buildVolumeGeometry(
      meshData,
      displacedNodes,
      colorValues,
      minVal,
      maxVal,
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
    });

    this.volumeMesh = new THREE.Mesh(volumeGeometry, volumeMat);
    this.meshGroup.add(this.volumeMesh);

    // 4. Wireframe lines
    const wireframeMat = new THREE.LineBasicMaterial({
      color: hasColors ? 0x0f172a : 0x475569,
      transparent: true,
      opacity: hasColors ? 0.35 : 0.45,
    });
    this.wireframeMesh = new THREE.LineSegments(edgeGeometry, wireframeMat);
    this.wireframeMesh.visible = this.showWireframe();
    this.meshGroup.add(this.wireframeMesh);

    // 5. Boundary Elements (BEM domain) colored by tag
    this.boundaryGroup = new THREE.Group();

    if (meshData.dimension === 2) {
      this.buildBoundaryLines2D(meshData, displacedNodes);
    } else {
      this.buildBoundarySurfaces3D(meshData, displacedNodes);
    }

    this.boundaryGroup.visible = this.showBoundaries();
    this.meshGroup.add(this.boundaryGroup);

    // 6. Node markers (Points)
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
    });
    this.nodePoints = new THREE.Points(nodeGeometry, nodeMaterial);
    this.nodePoints.visible = this.showNodes();
    this.meshGroup.add(this.nodePoints);

    this.scene.add(this.meshGroup);

    // Fit camera to mesh so it's centered and fully visible
    this.fitCameraToMesh(meshData);
  }

  private buildVolumeGeometry(
    meshData: MeshData,
    displacedNodes: [number, number, number][],
    colorValues: number[] | null,
    minVal: number,
    maxVal: number,
    colormapActive: boolean
  ): {
    volumeGeometry: THREE.BufferGeometry;
    edgeGeometry: THREE.BufferGeometry;
  } {
    const indices: number[] = [];
    const edgeIndices: number[] = [];

    const addEdge = (a: number, b: number) => {
      edgeIndices.push(a, b);
    };

    for (const elem of meshData.volume_elements) {
      const n = elem.nodes;
      switch (elem.elem_type) {
        case "Tri3":
          indices.push(n[0], n[1], n[2]);
          addEdge(n[0], n[1]);
          addEdge(n[1], n[2]);
          addEdge(n[2], n[0]);
          break;

        case "Quad4":
          indices.push(n[0], n[1], n[2]);
          indices.push(n[0], n[2], n[3]);
          addEdge(n[0], n[1]);
          addEdge(n[1], n[2]);
          addEdge(n[2], n[3]);
          addEdge(n[3], n[0]);
          break;

        case "Tet4":
          indices.push(n[0], n[1], n[2]);
          indices.push(n[0], n[2], n[3]);
          indices.push(n[0], n[3], n[1]);
          indices.push(n[1], n[3], n[2]);
          addEdge(n[0], n[1]);
          addEdge(n[1], n[2]);
          addEdge(n[2], n[0]);
          addEdge(n[0], n[3]);
          addEdge(n[1], n[3]);
          addEdge(n[2], n[3]);
          break;

        case "Hex8":
          const hexFaces = [
            [n[0], n[1], n[2], n[3]],
            [n[4], n[7], n[6], n[5]],
            [n[0], n[4], n[5], n[1]],
            [n[2], n[6], n[7], n[3]],
            [n[0], n[3], n[7], n[4]],
            [n[1], n[5], n[6], n[2]],
          ];
          for (const f of hexFaces) {
            indices.push(f[0], f[1], f[2]);
            indices.push(f[0], f[2], f[3]);
          }
          addEdge(n[0], n[1]); addEdge(n[1], n[2]); addEdge(n[2], n[3]); addEdge(n[3], n[0]);
          addEdge(n[4], n[5]); addEdge(n[5], n[6]); addEdge(n[6], n[7]); addEdge(n[7], n[4]);
          addEdge(n[0], n[4]); addEdge(n[1], n[5]); addEdge(n[2], n[6]); addEdge(n[3], n[7]);
          break;
      }
    }

    const flatPositions = new Float32Array(displacedNodes.length * 3);
    for (let i = 0; i < displacedNodes.length; i++) {
      flatPositions[i * 3 + 0] = displacedNodes[i][0];
      flatPositions[i * 3 + 1] = displacedNodes[i][1];
      flatPositions[i * 3 + 2] = displacedNodes[i][2];
    }

    const volumeGeometry = new THREE.BufferGeometry();
    volumeGeometry.setAttribute("position", new THREE.BufferAttribute(flatPositions, 3));
    volumeGeometry.setIndex(indices);

    // Compute scientific vertex colormap if active
    if (colormapActive && colorValues && colorValues.length === displacedNodes.length) {
      const colors = new Float32Array(displacedNodes.length * 3);
      const span = maxVal - minVal > 1e-9 ? maxVal - minVal : 1.0;

      for (let i = 0; i < displacedNodes.length; i++) {
        const val = colorValues[i];
        const t = (val - minVal) / span;
        const [r, g, b] = turboColormap(t);
        colors[i * 3 + 0] = r;
        colors[i * 3 + 1] = g;
        colors[i * 3 + 2] = b;
      }
      volumeGeometry.setAttribute("color", new THREE.BufferAttribute(colors, 3));
    }

    volumeGeometry.computeVertexNormals();

    const edgeGeometry = new THREE.BufferGeometry();
    edgeGeometry.setAttribute("position", new THREE.BufferAttribute(flatPositions, 3));
    edgeGeometry.setIndex(edgeIndices);

    return { volumeGeometry, edgeGeometry };
  }

  private buildBoundaryLines2D(
    meshData: MeshData,
    displacedNodes: [number, number, number][]
  ): void {
    const groupedLines = new Map<string, number[]>();

    for (const bnd of meshData.boundary_elements) {
      if (!groupedLines.has(bnd.tag)) {
        groupedLines.set(bnd.tag, []);
      }
      const arr = groupedLines.get(bnd.tag)!;
      for (const nodeIdx of bnd.nodes) {
        const pt = displacedNodes[nodeIdx];
        arr.push(pt[0], pt[1], pt[2] + 0.01);
      }
    }

    for (const [tag, coords] of groupedLines.entries()) {
      const lineGeom = new THREE.BufferGeometry();
      lineGeom.setAttribute("position", new THREE.Float32BufferAttribute(coords, 3));
      const colorVal = TAG_COLORS[tag] ?? TAG_COLORS["Default"];
      const lineMat = new THREE.LineBasicMaterial({
        color: colorVal,
        linewidth: 3,
      });
      const lineSegments = new THREE.LineSegments(lineGeom, lineMat);
      this.boundaryGroup!.add(lineSegments);
    }
  }

  private buildBoundarySurfaces3D(
    meshData: MeshData,
    displacedNodes: [number, number, number][]
  ): void {
    const groupedTriangles = new Map<string, number[]>();

    for (const bnd of meshData.boundary_elements) {
      if (!groupedTriangles.has(bnd.tag)) {
        groupedTriangles.set(bnd.tag, []);
      }
      const arr = groupedTriangles.get(bnd.tag)!;
      if (bnd.nodes.length >= 3) {
        const p0 = displacedNodes[bnd.nodes[0]];
        const p1 = displacedNodes[bnd.nodes[1]];
        const p2 = displacedNodes[bnd.nodes[2]];
        arr.push(p0[0], p0[1], p0[2]);
        arr.push(p1[0], p1[1], p1[2]);
        arr.push(p2[0], p2[1], p2[2]);

        if (bnd.nodes.length === 4) {
          const p3 = displacedNodes[bnd.nodes[3]];
          arr.push(p0[0], p0[1], p0[2]);
          arr.push(p2[0], p2[1], p2[2]);
          arr.push(p3[0], p3[1], p3[2]);
        }
      }
    }

    for (const [tag, coords] of groupedTriangles.entries()) {
      const geom = new THREE.BufferGeometry();
      geom.setAttribute("position", new THREE.Float32BufferAttribute(coords, 3));
      geom.computeVertexNormals();

      const colorVal = TAG_COLORS[tag] ?? TAG_COLORS["Default"];
      const mat = new THREE.MeshStandardMaterial({
        color: colorVal,
        side: THREE.DoubleSide,
        transparent: true,
        opacity: 0.65,
        depthWrite: false,
      });

      const mesh = new THREE.Mesh(geom, mat);
      this.boundaryGroup!.add(mesh);
    }
  }
}
