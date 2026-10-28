import {
  Component,
  ChangeDetectionStrategy,
  ElementRef,
  viewChild,
  effect,
  OnDestroy,
  signal,
  input,
} from "@angular/core";
import * as THREE from "three";
import { MeshData, BoundaryElement, VolumeElement } from "../../services/mesh.service";
import { MaterialFieldResponse } from "../../services/material.service";

// Color mapping for boundary tags
const TAG_COLORS: Record<string, number> = {
  Left: 0x3b82f6,    // Blue (e.g. Fixed support)
  Right: 0xef4444,   // Red (e.g. Traction load)
  Top: 0xf59e0b,     // Amber / Yellow
  Bottom: 0x10b981,  // Emerald green
  Hole: 0xa855f7,    // Purple / Cavity
  Front: 0x06b6d4,   // Cyan
  Back: 0x6366f1,    // Indigo
  Default: 0x94a3b8, // Slate gray
};

/**
 * High-quality Scientific Turbo Colormap: maps normalized t in [0, 1] to RGB.
 * Blue (min) -> Cyan -> Green -> Yellow -> Orange -> Red (max).
 */
function turboColormap(t: number): [number, number, number] {
  const clampT = Math.max(0, Math.min(1, t));

  // Polynomial fit for Google Turbo colormap (visually linear & accessible)
  const kRed = [0.13572138, 4.61539260, -42.66032258, 132.13108234, -152.94239396, 59.28637943];
  const kGreen = [0.09140261, 2.19418839, 4.84296658, -14.18503327, 4.27729857, 2.82956604];
  const kBlue = [0.10667330, 12.64194608, -60.58204836, 110.36276771, -89.90310912, 27.34824973];

  const evalPoly = (c: number[], x: number) =>
    c[0] + x * (c[1] + x * (c[2] + x * (c[3] + x * (c[4] + x * c[5]))));

  const r = Math.max(0, Math.min(1, evalPoly(kRed, clampT)));
  const g = Math.max(0, Math.min(1, evalPoly(kGreen, clampT)));
  const b = Math.max(0, Math.min(1, evalPoly(kBlue, clampT)));

  return [r, g, b];
}

@Component({
  selector: "app-viewport",
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="viewport-container" #canvasContainer>
      <!-- HUD / Overlay controls -->
      <div class="viewport-hud">
        <div class="hud-chip">
          <span class="status-dot"></span>
          <span>Three.js WebGL Conformal Mesh</span>
        </div>
        @if (mesh()) {
          <div class="hud-stats">
            <span>Dim: {{ mesh()!.dimension }}D</span>
            <span class="divider">|</span>
            <span>Nodes: {{ mesh()!.stats.num_nodes }}</span>
            <span class="divider">|</span>
            <span>Vol (FEM): {{ mesh()!.stats.num_volume_elements }}</span>
            <span class="divider">|</span>
            <span>Bnd (BEM): {{ mesh()!.stats.num_boundary_elements }}</span>
          </div>
        }
      </div>

      <!-- Boundary Tag Legend -->
      @if (mesh() && detectedTags().length > 0) {
        <div class="legend-panel">
          <div class="legend-title">Boundary Tags (BEM &amp; Boundary Conditions)</div>
          <div class="legend-items">
            @for (tag of detectedTags(); track tag.name) {
              <div class="legend-item">
                <span class="legend-swatch" [style.background-color]="tag.hex"></span>
                <span class="legend-label">{{ tag.name }} ({{ tag.count }})</span>
              </div>
            }
          </div>
        </div>
      }

      <!-- Material Inhomogeneity Colorbar -->
      @if (materialField() && showColormap()) {
        <div class="colorbar-panel">
          <div class="colorbar-title">{{ materialField()!.property_name }}</div>
          <div class="colorbar-content">
            <div class="colorbar-gradient"></div>
            <div class="colorbar-labels">
              <span class="label-max">{{ materialField()!.max_val.toFixed(1) }} {{ materialField()!.unit }}</span>
              <span class="label-mid">{{ ((materialField()!.max_val + materialField()!.min_val) / 2).toFixed(1) }}</span>
              <span class="label-min">{{ materialField()!.min_val.toFixed(1) }} {{ materialField()!.unit }}</span>
            </div>
          </div>
        </div>
      }

      <!-- Viewport Display Toggles -->
      <div class="hud-toggles">
        <button
          type="button"
          class="toggle-btn"
          [class.active]="showColormap()"
          (click)="toggleColormap()"
        >
          Material Colormap
        </button>
        <button
          type="button"
          class="toggle-btn"
          [class.active]="showWireframe()"
          (click)="toggleWireframe()"
        >
          Wireframe
        </button>
        <button
          type="button"
          class="toggle-btn"
          [class.active]="showBoundaries()"
          (click)="toggleBoundaries()"
        >
          Boundary Tags
        </button>
        <button
          type="button"
          class="toggle-btn"
          [class.active]="showNodes()"
          (click)="toggleNodes()"
        >
          Nodes
        </button>
        <button type="button" class="toggle-btn reset-btn" (click)="resetView()">
          Reset View
        </button>
      </div>
    </div>
  `,
  styles: `
    :host {
      display: block;
      width: 100%;
      height: 100%;
      position: relative;
      overflow: hidden;
    }

    .viewport-container {
      width: 100%;
      height: 100%;
      position: relative;
    }

    .viewport-hud {
      position: absolute;
      top: 16px;
      left: 16px;
      display: flex;
      flex-wrap: wrap;
      gap: 12px;
      align-items: center;
      z-index: 10;
      pointer-events: none;
    }

    .hud-chip {
      background: rgba(18, 22, 34, 0.85);
      backdrop-filter: blur(8px);
      border: 1px solid rgba(255, 255, 255, 0.12);
      color: #e2e8f0;
      padding: 6px 14px;
      border-radius: 9999px;
      font-size: 0.825rem;
      font-weight: 500;
      display: flex;
      align-items: center;
      gap: 8px;
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.25);
    }

    .status-dot {
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background: #10b981;
      box-shadow: 0 0 8px #10b981;
    }

    .hud-stats {
      background: rgba(18, 22, 34, 0.85);
      backdrop-filter: blur(8px);
      border: 1px solid rgba(255, 255, 255, 0.1);
      color: #94a3b8;
      padding: 6px 14px;
      border-radius: 8px;
      font-size: 0.8rem;
      font-family: monospace;
      display: flex;
      gap: 8px;
    }

    .divider {
      color: rgba(255, 255, 255, 0.2);
    }

    .legend-panel {
      position: absolute;
      bottom: 16px;
      left: 16px;
      background: rgba(18, 22, 34, 0.88);
      backdrop-filter: blur(10px);
      border: 1px solid rgba(255, 255, 255, 0.12);
      border-radius: 8px;
      padding: 10px 14px;
      z-index: 10;
      pointer-events: none;
      box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
    }

    .legend-title {
      font-size: 0.75rem;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.05em;
      color: #94a3b8;
      margin-bottom: 6px;
    }

    .legend-items {
      display: flex;
      flex-wrap: wrap;
      gap: 12px;
    }

    .legend-item {
      display: flex;
      align-items: center;
      gap: 6px;
      font-size: 0.775rem;
      color: #f1f5f9;
    }

    .legend-swatch {
      width: 10px;
      height: 10px;
      border-radius: 2px;
    }

    /* Colorbar Panel */
    .colorbar-panel {
      position: absolute;
      right: 16px;
      top: 70px;
      background: rgba(18, 22, 34, 0.88);
      backdrop-filter: blur(10px);
      border: 1px solid rgba(255, 255, 255, 0.12);
      border-radius: 8px;
      padding: 12px 14px;
      z-index: 10;
      pointer-events: none;
      box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
      display: flex;
      flex-direction: column;
      gap: 8px;
      min-width: 130px;
    }

    .colorbar-title {
      font-size: 0.75rem;
      font-weight: 600;
      color: #38bdf8;
      text-align: center;
    }

    .colorbar-content {
      display: flex;
      gap: 10px;
      height: 180px;
    }

    .colorbar-gradient {
      width: 14px;
      height: 100%;
      border-radius: 4px;
      background: linear-gradient(
        to bottom,
        #f43f5e 0%,
        #fb923c 25%,
        #facc15 50%,
        #2dd4bf 75%,
        #3b82f6 100%
      );
      border: 1px solid rgba(255, 255, 255, 0.2);
    }

    .colorbar-labels {
      display: flex;
      flex-direction: column;
      justify-content: space-between;
      font-size: 0.725rem;
      font-family: monospace;
      color: #cbd5e1;
    }

    .hud-toggles {
      position: absolute;
      top: 16px;
      right: 16px;
      display: flex;
      gap: 6px;
      z-index: 10;
    }

    .toggle-btn {
      background: rgba(18, 22, 34, 0.8);
      backdrop-filter: blur(8px);
      border: 1px solid rgba(255, 255, 255, 0.15);
      color: #cbd5e1;
      font-size: 0.775rem;
      font-weight: 500;
      padding: 6px 12px;
      border-radius: 6px;
      cursor: pointer;
      transition: all 0.15s ease;
    }

    .toggle-btn:hover {
      background: rgba(30, 41, 59, 0.9);
      color: #fff;
      border-color: rgba(255, 255, 255, 0.3);
    }

    .toggle-btn.active {
      background: #2563eb;
      color: #ffffff;
      border-color: #3b82f6;
      box-shadow: 0 0 10px rgba(37, 99, 235, 0.4);
    }

    .reset-btn {
      border-color: rgba(255, 255, 255, 0.2);
    }
  `,
})
export class ViewportComponent implements OnDestroy {
  canvasContainer = viewChild.required<ElementRef<HTMLDivElement>>("canvasContainer");

  // Inputs
  mesh = input<MeshData | null>(null);
  materialField = input<MaterialFieldResponse | null>(null);

  // Display toggles
  showColormap = signal<boolean>(true);
  showWireframe = signal<boolean>(true);
  showBoundaries = signal<boolean>(true);
  showNodes = signal<boolean>(false);

  // Boundary tag legend items
  detectedTags = signal<{ name: string; hex: string; count: number }[]>([]);

  private scene!: THREE.Scene;
  private camera!: THREE.PerspectiveCamera;
  private renderer!: THREE.WebGLRenderer;
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
  private spherical = { radius: 15, theta: Math.PI / 4, phi: Math.PI / 3 };
  private target = new THREE.Vector3(0, 0, 0);

  constructor() {
    // Initialize Three.js once container is available
    effect(() => {
      const containerRef = this.canvasContainer();
      if (containerRef) {
        this.initThree(containerRef.nativeElement);
      }
    });

    // Re-render mesh whenever mesh or materialField changes
    effect(() => {
      const currentMesh = this.mesh();
      const field = this.materialField();
      const colormapActive = this.showColormap();
      if (this.scene && currentMesh) {
        this.renderMesh(currentMesh, field, colormapActive);
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
      this.spherical = { radius: 12, theta: Math.PI / 4, phi: Math.PI / 3 };
      this.target.set(0, 0, 0);
      this.updateCameraFromSpherical();
      return;
    }
    this.fitCameraToMesh(this.mesh()!);
  }

  private initThree(container: HTMLDivElement): void {
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

    // 4. Lights
    const ambientLight = new THREE.AmbientLight(0xffffff, 0.7);
    this.scene.add(ambientLight);

    const dirLight1 = new THREE.DirectionalLight(0xffffff, 0.85);
    dirLight1.position.set(20, 40, 30);
    dirLight1.castShadow = true;
    this.scene.add(dirLight1);

    const dirLight2 = new THREE.DirectionalLight(0x38bdf8, 0.45);
    dirLight2.position.set(-20, -20, -20);
    this.scene.add(dirLight2);

    // 5. Grid Helper
    const gridHelper = new THREE.GridHelper(30, 30, 0x38bdf8, 0x1e293b);
    gridHelper.position.y = -0.01;
    this.scene.add(gridHelper);

    // 6. Mouse Interaction
    this.setupMouseEvents(container);

    // 7. Resize Observer
    this.resizeObserver = new ResizeObserver(() => {
      if (!container || !this.renderer) return;
      const w = container.clientWidth;
      const h = container.clientHeight;
      if (w > 0 && h > 0) {
        this.camera.aspect = w / h;
        this.camera.updateProjectionMatrix();
        this.renderer.setSize(w, h);
      }
    });
    this.resizeObserver.observe(container);

    // 8. Animation loop
    this.animate();
  }

  private renderMesh(
    meshData: MeshData,
    field: MaterialFieldResponse | null,
    colormapActive: boolean
  ): void {
    // Clean up previous mesh objects
    if (this.meshGroup) {
      this.scene.remove(this.meshGroup);
      this.meshGroup.clear();
      this.meshGroup = null;
      this.volumeMesh = null;
      this.wireframeMesh = null;
      this.boundaryGroup = null;
      this.nodePoints = null;
    }

    if (!meshData || meshData.nodes.length === 0) return;

    this.meshGroup = new THREE.Group();

    // 1. Build volume geometry with optional vertex colors
    const { volumeGeometry, edgeGeometry } = this.buildVolumeGeometry(
      meshData,
      field,
      colormapActive
    );

    const hasColors = colormapActive && field && field.values.length === meshData.nodes.length;

    const volumeMaterial = new THREE.MeshStandardMaterial({
      color: hasColors ? 0xffffff : (meshData.dimension === 2 ? 0x1e293b : 0x243047),
      vertexColors: !!hasColors,
      roughness: 0.4,
      metalness: 0.1,
      side: THREE.DoubleSide,
      polygonOffset: true,
      polygonOffsetFactor: 1,
      polygonOffsetUnits: 1,
    });
    this.volumeMesh = new THREE.Mesh(volumeGeometry, volumeMaterial);
    this.meshGroup.add(this.volumeMesh);

    // 2. Wireframe lines
    const wireframeMat = new THREE.LineBasicMaterial({
      color: hasColors ? 0x0f172a : 0x475569,
      transparent: true,
      opacity: hasColors ? 0.35 : 0.45,
    });
    this.wireframeMesh = new THREE.LineSegments(edgeGeometry, wireframeMat);
    this.wireframeMesh.visible = this.showWireframe();
    this.meshGroup.add(this.wireframeMesh);

    // 3. Boundary Elements (BEM domain) colored by tag
    this.boundaryGroup = new THREE.Group();
    const tagMap = new Map<string, number>();

    if (meshData.dimension === 2) {
      this.buildBoundaryLines2D(meshData, tagMap);
    } else {
      this.buildBoundarySurfaces3D(meshData, tagMap);
    }

    this.boundaryGroup.visible = this.showBoundaries();
    this.meshGroup.add(this.boundaryGroup);

    // 4. Node markers (Points)
    const nodeGeometry = new THREE.BufferGeometry();
    const nodeCoords = new Float32Array(meshData.nodes.length * 3);
    for (let i = 0; i < meshData.nodes.length; i++) {
      nodeCoords[i * 3 + 0] = meshData.nodes[i][0];
      nodeCoords[i * 3 + 1] = meshData.nodes[i][1];
      nodeCoords[i * 3 + 2] = meshData.nodes[i][2];
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

    // Update legend tags
    const tagsList = Array.from(tagMap.entries()).map(([name, count]) => {
      const colorVal = TAG_COLORS[name] ?? TAG_COLORS['Default'];
      const hex = `#${colorVal.toString(16).padStart(6, "0")}`;
      return { name, hex, count };
    });
    this.detectedTags.set(tagsList);

    // Fit camera
    this.fitCameraToMesh(meshData);
  }

  private buildVolumeGeometry(
    meshData: MeshData,
    field: MaterialFieldResponse | null,
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

    const flatPositions = new Float32Array(meshData.nodes.length * 3);
    for (let i = 0; i < meshData.nodes.length; i++) {
      flatPositions[i * 3 + 0] = meshData.nodes[i][0];
      flatPositions[i * 3 + 1] = meshData.nodes[i][1];
      flatPositions[i * 3 + 2] = meshData.nodes[i][2];
    }

    const volumeGeometry = new THREE.BufferGeometry();
    volumeGeometry.setAttribute("position", new THREE.BufferAttribute(flatPositions, 3));
    volumeGeometry.setIndex(indices);

    // Compute scientific vertex colormap if material field is active
    if (colormapActive && field && field.values.length === meshData.nodes.length) {
      const colors = new Float32Array(meshData.nodes.length * 3);
      const span = field.max_val - field.min_val;

      for (let i = 0; i < meshData.nodes.length; i++) {
        const val = field.values[i];
        const t = span > 1e-9 ? (val - field.min_val) / span : 0.5;
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

  private buildBoundaryLines2D(meshData: MeshData, tagMap: Map<string, number>): void {
    const groupedLines = new Map<string, number[]>();

    for (const bnd of meshData.boundary_elements) {
      tagMap.set(bnd.tag, (tagMap.get(bnd.tag) ?? 0) + 1);
      if (!groupedLines.has(bnd.tag)) {
        groupedLines.set(bnd.tag, []);
      }
      const arr = groupedLines.get(bnd.tag)!;
      for (const nodeIdx of bnd.nodes) {
        const pt = meshData.nodes[nodeIdx];
        arr.push(pt[0], pt[1], pt[2] + 0.005);
      }
    }

    for (const [tag, coords] of groupedLines.entries()) {
      const lineGeom = new THREE.BufferGeometry();
      lineGeom.setAttribute("position", new THREE.Float32BufferAttribute(coords, 3));
      const colorVal = TAG_COLORS[tag] ?? TAG_COLORS['Default'];
      const lineMat = new THREE.LineBasicMaterial({
        color: colorVal,
        linewidth: 3,
      });
      const lineSegments = new THREE.LineSegments(lineGeom, lineMat);
      this.boundaryGroup!.add(lineSegments);
    }
  }

  private buildBoundarySurfaces3D(meshData: MeshData, tagMap: Map<string, number>): void {
    const groupedTriangles = new Map<string, number[]>();

    for (const bnd of meshData.boundary_elements) {
      tagMap.set(bnd.tag, (tagMap.get(bnd.tag) ?? 0) + 1);
      if (!groupedTriangles.has(bnd.tag)) {
        groupedTriangles.set(bnd.tag, []);
      }
      const arr = groupedTriangles.get(bnd.tag)!;
      if (bnd.nodes.length >= 3) {
        const p0 = meshData.nodes[bnd.nodes[0]];
        const p1 = meshData.nodes[bnd.nodes[1]];
        const p2 = meshData.nodes[bnd.nodes[2]];
        arr.push(p0[0], p0[1], p0[2]);
        arr.push(p1[0], p1[1], p1[2]);
        arr.push(p2[0], p2[1], p2[2]);
      }
    }

    for (const [tag, coords] of groupedTriangles.entries()) {
      const geom = new THREE.BufferGeometry();
      geom.setAttribute("position", new THREE.Float32BufferAttribute(coords, 3));
      geom.computeVertexNormals();

      const colorVal = TAG_COLORS[tag] ?? TAG_COLORS['Default'];
      const mat = new THREE.MeshStandardMaterial({
        color: colorVal,
        roughness: 0.3,
        metalness: 0.2,
        side: THREE.DoubleSide,
        polygonOffset: true,
        polygonOffsetFactor: -1,
        polygonOffsetUnits: -1,
      });
      const bndMesh = new THREE.Mesh(geom, mat);
      this.boundaryGroup!.add(bndMesh);
    }
  }

  private fitCameraToMesh(meshData: MeshData): void {
    let minX = Infinity, maxX = -Infinity;
    let minY = Infinity, maxY = -Infinity;
    let minZ = Infinity, maxZ = -Infinity;

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

  private setupMouseEvents(domElement: HTMLElement): void {
    domElement.addEventListener("contextmenu", (e) => e.preventDefault());

    domElement.addEventListener("mousedown", (e: MouseEvent) => {
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

    domElement.addEventListener("mousemove", (e: MouseEvent) => {
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

    domElement.addEventListener(
      "wheel",
      (e: WheelEvent) => {
        e.preventDefault();
        this.spherical.radius += e.deltaY * 0.015;
        this.spherical.radius = Math.max(0.5, Math.min(200, this.spherical.radius));
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

  private animate = (): void => {
    this.animFrameId = requestAnimationFrame(this.animate);
    if (this.renderer && this.scene && this.camera) {
      this.renderer.render(this.scene, this.camera);
    }
  };

  ngOnDestroy(): void {
    if (this.animFrameId !== null) {
      cancelAnimationFrame(this.animFrameId);
    }
    if (this.resizeObserver) {
      this.resizeObserver.disconnect();
    }
    if (this.renderer) {
      this.renderer.dispose();
    }
  }
}
