import {
  Component,
  ChangeDetectionStrategy,
  signal,
  OnInit,
  inject,
} from "@angular/core";
import { ViewportComponent } from "./components/viewport/viewport.component";
import { MeshService, MeshGeneratorRequest } from "./services/mesh.service";
import { invoke } from "@tauri-apps/api/core";

export type PresetKey =
  | "Beam2D"
  | "PlateWithHole2D"
  | "Beam3D"
  | "BlockWithCavity3D";

@Component({
  selector: "app-root",
  imports: [ViewportComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: "./app.component.html",
  styleUrl: "./app.component.css",
})
export class AppComponent implements OnInit {
  readonly meshService = inject(MeshService);

  activeDimension = signal<"2D" | "3D">("2D");
  activeDomain = signal<"Static" | "TimeDomain" | "LaplaceTalbot">("Static");
  inhomogeneityRatio = signal<number>(20); // 20%
  backendStatus = signal<string>("Connecting to Rust Core...");

  // Preset Selection
  selectedPreset = signal<PresetKey>("Beam2D");

  // Preset Parameters
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

  ngOnInit(): void {
    // 1. Verify Rust Core IPC
    invoke<string>("greet", { name: "Antigravity Engineer" })
      .then(() => {
        this.backendStatus.set("Rust Core Online (faer + rayon ready)");
      })
      .catch((err) => {
        this.backendStatus.set(`Backend Offline (${err})`);
      });

    // 2. Generate initial mesh
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

  onInhomogeneityChange(event: Event): void {
    const val = Number((event.target as HTMLInputElement).value);
    this.inhomogeneityRatio.set(val);
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
      await this.meshService.generateMesh(request);
    } catch (e) {
      console.error("Failed to generate mesh:", e);
    }
  }
}
