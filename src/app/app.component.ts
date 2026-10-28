import {
  Component,
  ChangeDetectionStrategy,
  signal,
  OnInit,
  inject,
} from "@angular/core";
import { ViewportComponent } from "./components/viewport/viewport.component";
import { MeshService, MeshGeneratorRequest } from "./services/mesh.service";
import {
  MaterialService,
  MaterialStrategyConfig,
  PropertyMetric,
  Axis,
  GradientForm,
  InclusionShape,
} from "./services/material.service";
import { invoke } from "@tauri-apps/api/core";

export type PresetKey =
  | "Beam2D"
  | "PlateWithHole2D"
  | "Beam3D"
  | "BlockWithCavity3D";

export type StrategyKey =
  | "DirectionalGradient"
  | "AnalyticalExpression"
  | "Inclusion"
  | "VecchioCollinsConcrete"
  | "J2Plasticity";

@Component({
  selector: "app-root",
  imports: [ViewportComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: "./app.component.html",
  styleUrl: "./app.component.css",
})
export class AppComponent implements OnInit {
  readonly meshService = inject(MeshService);
  readonly materialService = inject(MaterialService);

  activeDimension = signal<"2D" | "3D">("2D");
  activeDomain = signal<"Static" | "TimeDomain" | "LaplaceTalbot">("Static");
  backendStatus = signal<string>("Connecting to Rust Core...");

  // Geometry Preset Selection
  selectedPreset = signal<PresetKey>("Beam2D");

  // Geometry Parameters
  beamLength = signal<number>(10.0);
  beamHeight = signal<number>(2.0);
  beamNx = signal<number>(20);
  beamNy = signal<number>(6);
  beamUseTriangles = signal<boolean>(false);

  plateWidth = signal<number>(20.0);
  plateHeight = signal<number>(20.0);
  plateRadius = signal<number>(3.0);
  plateRadialDivs = signal<number>(6);
  plateAngularDivs = signal<number>(24);

  beam3dLength = signal<number>(8.0);
  beam3dHeight = signal<number>(2.0);
  beam3dDepth = signal<number>(2.0);
  beam3dNx = signal<number>(16);
  beam3dNy = signal<number>(4);
  beam3dNz = signal<number>(4);
  beam3dUseTets = signal<boolean>(false);

  // Material & Inhomogeneity State
  selectedStrategy = signal<StrategyKey>("DirectionalGradient");
  activeMetric = signal<PropertyMetric>("YoungsModulus");

  // Gradient Parameters
  gradientAxis = signal<Axis>("Y");
  gradientAlpha = signal<number>(20); // 20%
  gradientPower = signal<number>(2.0);
  gradientForm = signal<GradientForm>("Quadratic");

  // Math Expression
  mathExpression = signal<string>("E0 * (1 + 0.3 * sin(pi * y / 2))");

  // Inclusion Parameters
  inclusionShape = signal<InclusionShape>("CylinderZ");
  inclusionRadius = signal<number>(1.2);
  inclusionE = signal<number>(50.0); // 50 GPa
  inclusionSmooth = signal<boolean>(true);

  // Vecchio-Collins MCFT Parameters
  vcFc = signal<number>(30.0); // 30 MPa
  vcEps0 = signal<number>(0.002);
  vcSimStrain = signal<number>(0.004);

  // J2 Plasticity Parameters
  j2SigmaY = signal<number>(250.0); // 250 MPa
  j2Hardening = signal<number>(1000.0); // 1000 MPa
  j2PlasticStrain = signal<number>(0.015);

  ngOnInit(): void {
    // 1. Verify Rust Core IPC
    invoke<string>("greet", { name: "Antigravity Engineer" })
      .then(() => {
        this.backendStatus.set("Rust Core Online (faer + rayon ready)");
      })
      .catch((err) => {
        this.backendStatus.set(`Backend Offline (${err})`);
      });

    // 2. Initial Mesh & Material generation
    this.generateCurrentMesh();
  }

  setDimension(dim: "2D" | "3D"): void {
    this.activeDimension.set(dim);
    if (dim === "2D" && (this.selectedPreset() === "Beam3D" || this.selectedPreset() === "BlockWithCavity3D")) {
      this.selectedPreset.set("Beam2D");
    } else if (dim === "3D" && (this.selectedPreset() === "Beam2D" || this.selectedPreset() === "PlateWithHole2D")) {
      this.selectedPreset.set("Beam3D");
    }
    this.generateCurrentMesh();
  }

  setDomain(domain: "Static" | "TimeDomain" | "LaplaceTalbot"): void {
    this.activeDomain.set(domain);
  }

  onPresetChange(event: Event): void {
    const val = (event.target as HTMLSelectElement).value as PresetKey;
    this.selectedPreset.set(val);
    if (val === "Beam3D" || val === "BlockWithCavity3D") {
      this.activeDimension.set("3D");
    } else {
      this.activeDimension.set("2D");
    }
    this.generateCurrentMesh();
  }

  onStrategyChange(event: Event): void {
    const val = (event.target as HTMLSelectElement).value as StrategyKey;
    this.selectedStrategy.set(val);
    if (val === "VecchioCollinsConcrete") {
      this.activeMetric.set("SecantModulus");
      this.materialService.reference.set({
        e0: 3.0e10,
        nu0: 0.20,
        rho0: 2400.0,
      });
    } else if (val === "J2Plasticity") {
      this.activeMetric.set("YieldStrength");
      this.materialService.reference.set({
        e0: 2.1e11,
        nu0: 0.30,
        rho0: 7850.0,
      });
    } else {
      this.activeMetric.set("YoungsModulus");
    }
    this.evaluateCurrentMaterial();
  }

  onMetricChange(event: Event): void {
    const val = (event.target as HTMLSelectElement).value as PropertyMetric;
    this.activeMetric.set(val);
    this.evaluateCurrentMaterial();
  }

  onAlphaChange(event: Event): void {
    const val = Number((event.target as HTMLInputElement).value);
    this.gradientAlpha.set(val);
    this.evaluateCurrentMaterial();
  }

  async generateCurrentMesh(): Promise<void> {
    const preset = this.selectedPreset();
    let request: MeshGeneratorRequest;

    if (preset === "Beam2D") {
      request = {
        preset: "Beam2D",
        params: {
          length: this.beamLength(),
          height: this.beamHeight(),
          nx: this.beamNx(),
          ny: this.beamNy(),
          use_triangles: this.beamUseTriangles(),
        },
      };
    } else if (preset === "PlateWithHole2D") {
      request = {
        preset: "PlateWithHole2D",
        params: {
          width: this.plateWidth(),
          height: this.plateHeight(),
          radius: this.plateRadius(),
          radial_divisions: this.plateRadialDivs(),
          tangential_divisions: this.plateAngularDivs(),
        },
      };
    } else if (preset === "Beam3D") {
      request = {
        preset: "Beam3D",
        params: {
          length: this.beam3dLength(),
          height: this.beam3dHeight(),
          depth: this.beam3dDepth(),
          nx: this.beam3dNx(),
          ny: this.beam3dNy(),
          nz: this.beam3dNz(),
          use_tetrahedra: this.beam3dUseTets(),
        },
      };
    } else {
      request = {
        preset: "BlockWithCavity3D",
        params: {
          size: 6.0,
          radius: 1.5,
          divisions: 6,
        },
      };
    }

    try {
      const mesh = await this.meshService.generateMesh(request);
      if (mesh) {
        await this.evaluateCurrentMaterial(mesh.nodes);
      }
    } catch (e) {
      console.error("Failed to generate mesh:", e);
    }
  }

  async evaluateCurrentMaterial(nodesInput?: [number, number, number][]): Promise<void> {
    const nodes = nodesInput ?? this.meshService.currentMesh()?.nodes ?? [];
    if (nodes.length === 0) return;

    const strat = this.selectedStrategy();
    let strategyConfig: MaterialStrategyConfig;

    if (strat === "DirectionalGradient") {
      const lengthScale = this.activeDimension() === "2D" ? this.beamLength() : this.beam3dLength();
      strategyConfig = {
        type: "DirectionalGradient",
        params: {
          axis: this.gradientAxis(),
          alpha: this.gradientAlpha() / 100.0,
          length_scale: lengthScale,
          power: this.gradientPower(),
          form: this.gradientForm(),
        },
      };
    } else if (strat === "AnalyticalExpression") {
      strategyConfig = {
        type: "AnalyticalExpression",
        params: {
          expression: this.mathExpression(),
        },
      };
    } else if (strat === "Inclusion") {
      const cx = this.activeDimension() === "2D" ? this.beamLength() / 2.0 : this.beam3dLength() / 2.0;
      const cy = this.activeDimension() === "2D" ? this.beamHeight() / 2.0 : this.beam3dHeight() / 2.0;
      const cz = this.activeDimension() === "2D" ? 0.0 : this.beam3dDepth() / 2.0;

      strategyConfig = {
        type: "Inclusion",
        params: {
          shape: this.inclusionShape(),
          center: [cx, cy, cz],
          radius: this.inclusionRadius(),
          box_half_extents: null,
          e_inclusion: this.inclusionE() * 1e9, // GPa to Pa
          nu_inclusion: 0.25,
          rho_inclusion: 2500.0,
          smooth_transition: this.inclusionSmooth(),
        },
      };
    } else if (strat === "VecchioCollinsConcrete") {
      strategyConfig = {
        type: "VecchioCollinsConcrete",
        params: {
          fc_prime: this.vcFc() * 1e6, // MPa to Pa
          epsilon_0: this.vcEps0(),
          ft: 0.33 * Math.sqrt(this.vcFc()) * 1e6,
          rebar_ratio: 0.015,
          simulated_strain: this.vcSimStrain(),
        },
      };
    } else {
      strategyConfig = {
        type: "J2Plasticity",
        params: {
          sigma_y0: this.j2SigmaY() * 1e6, // MPa to Pa
          hardening_modulus: this.j2Hardening() * 1e6,
          plastic_strain: this.j2PlasticStrain(),
        },
      };
    }

    try {
      await this.materialService.evaluateField(
        strategyConfig,
        nodes,
        this.activeMetric(),
        this.materialService.reference()
      );
    } catch (e) {
      console.error("Failed to evaluate material field:", e);
    }
  }
}
