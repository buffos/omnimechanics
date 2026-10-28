import { Component, ChangeDetectionStrategy, signal, OnInit } from "@angular/core";
import { ViewportComponent } from "./components/viewport/viewport.component";
import { invoke } from "@tauri-apps/api/core";

@Component({
  selector: "app-root",
  imports: [ViewportComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: "./app.component.html",
  styleUrl: "./app.component.css",
})
export class AppComponent implements OnInit {
  activeDimension = signal<"2D" | "3D">("3D");
  activeDomain = signal<"Static" | "TimeDomain" | "LaplaceTalbot">("Static");
  inhomogeneityRatio = signal<number>(20); // 20%
  backendStatus = signal<string>("Connecting to Rust Core...");
  iterationCount = signal<number>(0);
  residualNorm = signal<number>(0.0);

  ngOnInit(): void {
    // Verify IPC connectivity with Rust backend
    invoke<string>("greet", { name: "Antigravity Engineer" })
      .then((res) => {
        this.backendStatus.set("Rust Core Online (faer + rayon ready)");
      })
      .catch((err) => {
        this.backendStatus.set(`Backend Offline (${err})`);
      });
  }

  setDimension(dim: "2D" | "3D"): void {
    this.activeDimension.set(dim);
  }

  setDomain(domain: "Static" | "TimeDomain" | "LaplaceTalbot"): void {
    this.activeDomain.set(domain);
  }

  onInhomogeneityChange(event: Event): void {
    const val = Number((event.target as HTMLInputElement).value);
    this.inhomogeneityRatio.set(val);
  }
}
