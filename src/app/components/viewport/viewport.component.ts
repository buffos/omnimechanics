import {
  Component,
  ChangeDetectionStrategy,
  ElementRef,
  viewChild,
  effect,
  OnDestroy,
  signal,
} from "@angular/core";
import * as THREE from "three";

@Component({
  selector: "app-viewport",
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="viewport-container" #canvasContainer>
      <!-- HUD / Overlay controls -->
      <div class="viewport-hud">
        <div class="hud-chip">
          <span class="status-dot"></span>
          <span>Three.js WebGL Engine</span>
        </div>
        <div class="hud-stats">
          <span>Camera: {{ cameraPos() }}</span>
        </div>
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
      background: rgba(18, 22, 34, 0.7);
      backdrop-filter: blur(8px);
      border: 1px solid rgba(255, 255, 255, 0.08);
      color: #94a3b8;
      padding: 6px 12px;
      border-radius: 8px;
      font-size: 0.8rem;
      font-family: monospace;
    }
  `,
})
export class ViewportComponent implements OnDestroy {
  canvasContainer = viewChild.required<ElementRef<HTMLDivElement>>("canvasContainer");

  cameraPos = signal<string>("(5.0, 4.0, 6.0)");

  private scene!: THREE.Scene;
  private camera!: THREE.PerspectiveCamera;
  private renderer!: THREE.WebGLRenderer;
  private animFrameId: number | null = null;
  private resizeObserver: ResizeObserver | null = null;

  // Interaction controls (Orbit)
  private isDragging = false;
  private previousMousePosition = { x: 0, y: 0 };
  private spherical = { radius: 10, theta: Math.PI / 4, phi: Math.PI / 3 };
  private target = new THREE.Vector3(0, 0, 0);

  // Sample test model
  private testMesh: THREE.Mesh | null = null;

  constructor() {
    effect(() => {
      const containerRef = this.canvasContainer();
      if (containerRef) {
        this.initThree(containerRef.nativeElement);
      }
    });
  }

  private initThree(container: HTMLDivElement): void {
    const width = container.clientWidth || 800;
    const height = container.clientHeight || 600;

    // 1. Scene
    this.scene = new THREE.Scene();
    this.scene.background = new THREE.Color("#0d1117");

    // 2. Camera
    this.camera = new THREE.PerspectiveCamera(45, width / height, 0.1, 1000);
    this.updateCameraFromSpherical();

    // 3. Renderer
    this.renderer = new THREE.WebGLRenderer({ antialias: true });
    this.renderer.setSize(width, height);
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    this.renderer.shadowMap.enabled = true;
    container.appendChild(this.renderer.domElement);

    // 4. Lights
    const ambientLight = new THREE.AmbientLight(0xffffff, 0.6);
    this.scene.add(ambientLight);

    const dirLight1 = new THREE.DirectionalLight(0xffffff, 0.8);
    dirLight1.position.set(10, 20, 15);
    dirLight1.castShadow = true;
    this.scene.add(dirLight1);

    const dirLight2 = new THREE.DirectionalLight(0x38bdf8, 0.4);
    dirLight2.position.set(-10, -5, -10);
    this.scene.add(dirLight2);

    // 5. Helpers (Grid & Axes)
    const gridHelper = new THREE.GridHelper(12, 24, 0x38bdf8, 0x1e293b);
    gridHelper.position.y = -1;
    this.scene.add(gridHelper);

    const axesHelper = new THREE.AxesHelper(2.5);
    axesHelper.position.set(-5, -0.99, -5);
    this.scene.add(axesHelper);

    // 6. Test Model: A 3D Beam with Inhomogeneity Colormap (Prototype Preview)
    this.createSampleBeam();

    // 7. Event listeners for mouse interaction
    this.setupMouseEvents(container);

    // 8. Resize observer
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

    // 9. Animation loop
    this.animate();
  }

  private createSampleBeam(): void {
    // 3D Beam: Length = 6, Height = 1.5, Depth = 1.5
    const geometry = new THREE.BoxGeometry(6, 1.5, 1.5, 24, 6, 6);

    // Vertex colors simulating a continuous material gradient mu(x,y)
    const count = geometry.attributes['position'].count;
    const colors = new Float32Array(count * 3);
    const pos = geometry.attributes['position'];

    for (let i = 0; i < count; i++) {
      const x = pos.getX(i);
      const y = pos.getY(i);
      // Normalized factor [0, 1] representing 20% spatial gradient
      const factor = 0.5 + (x / 6.0) * 0.4 + (y / 1.5) * 0.2;
      // Colormap from cyan to orange/red
      colors[i * 3 + 0] = factor; // R
      colors[i * 3 + 1] = 0.8 * (1 - factor * 0.5); // G
      colors[i * 3 + 2] = 1.0 - factor * 0.8; // B
    }

    geometry.setAttribute("color", new THREE.BufferAttribute(colors, 3));

    const material = new THREE.MeshStandardMaterial({
      vertexColors: true,
      roughness: 0.35,
      metalness: 0.15,
      wireframe: false,
    });

    this.testMesh = new THREE.Mesh(geometry, material);
    this.testMesh.position.set(0, 0.5, 0);
    this.scene.add(this.testMesh);

    // Wireframe overlay for boundary/volume element visualization
    const wireframeMat = new THREE.MeshBasicMaterial({
      color: 0x0f172a,
      wireframe: true,
      transparent: true,
      opacity: 0.25,
    });
    const wireframeMesh = new THREE.Mesh(geometry, wireframeMat);
    this.testMesh.add(wireframeMesh);
  }

  private setupMouseEvents(domElement: HTMLElement): void {
    domElement.addEventListener("mousedown", (e: MouseEvent) => {
      this.isDragging = true;
      this.previousMousePosition = { x: e.clientX, y: e.clientY };
    });

    window.addEventListener("mouseup", () => {
      this.isDragging = false;
    });

    domElement.addEventListener("mousemove", (e: MouseEvent) => {
      if (!this.isDragging) return;

      const deltaX = e.clientX - this.previousMousePosition.x;
      const deltaY = e.clientY - this.previousMousePosition.y;

      this.spherical.theta -= deltaX * 0.008;
      this.spherical.phi -= deltaY * 0.008;

      // Constrain phi to avoid gimbal flip
      this.spherical.phi = Math.max(0.1, Math.min(Math.PI - 0.1, this.spherical.phi));

      this.updateCameraFromSpherical();
      this.previousMousePosition = { x: e.clientX, y: e.clientY };
    });

    domElement.addEventListener(
      "wheel",
      (e: WheelEvent) => {
        e.preventDefault();
        this.spherical.radius += e.deltaY * 0.01;
        this.spherical.radius = Math.max(2, Math.min(30, this.spherical.radius));
        this.updateCameraFromSpherical();
      },
      { passive: false }
    );
  }

  private updateCameraFromSpherical(): void {
    const { radius, theta, phi } = this.spherical;
    const x = radius * Math.sin(phi) * Math.sin(theta);
    const y = radius * Math.cos(phi);
    const z = radius * Math.sin(phi) * Math.cos(theta);

    this.camera.position.set(x, y, z);
    this.camera.lookAt(this.target);

    this.cameraPos.set(`(${x.toFixed(1)}, ${y.toFixed(1)}, ${z.toFixed(1)})`);
  }

  private animate = (): void => {
    this.animFrameId = requestAnimationFrame(this.animate);

    // Subtle idle animation
    if (this.testMesh) {
      this.testMesh.rotation.y += 0.002;
    }

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
