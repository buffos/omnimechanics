import { Injectable, signal } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { MeshData } from './mesh.service';
import { MaterialStrategyConfig, ReferenceMaterial } from './material.service';

export type SolverMode = 'PureBEM' | 'PureFEM' | 'BEMFEM';
export type BemElementType =
  | 'Constant'
  | 'Linear'
  | 'LinearDoubleNodes'
  | 'Quadratic'
  | 'QuadraticDiscontinuous'
  | 'Tri3'
  | 'Quad4';
export type StressState2D = 'PlaneStress' | 'PlaneStrain';
export type HybridAccelerationType = 'Anderson' | 'Relaxation' | 'None';

export type HybridAcceleration =
  | 'None'
  | { Relaxation: number }
  | { Anderson: { depth: number; beta: number } };
export type SimulationRegime = 'static' | 'harmonic' | 'transient';
export type TransientExcitation = 'HarmonicSine' | 'Pulse' | 'Step' | 'HalfSine';
export type TransientMethod = 'Talbot' | 'Newmark';

export interface ConvergenceLog {
  iteration: number;
  residual_norm: number;
  relative_error: number;
}

export interface SimulationResult {
  solver_mode: SolverMode;
  converged: boolean;
  iterations: number;
  displacements: [number, number, number][];
  deformed_nodes: [number, number, number][];
  von_mises_stresses: number[];
  max_displacement: number;
  max_von_mises: number;
  convergence_history: ConvergenceLog[];
  execution_time_ms: number;
}

export interface SimulationRequest {
  solver_mode: SolverMode;
  mesh: MeshData;
  reference_material: ReferenceMaterial;
  strategy: MaterialStrategyConfig;
  load_magnitude: number;
  max_iterations: number;
  tolerance: number;
  bem_element_type?: BemElementType;
  stress_state?: StressState2D;
  hybrid_acceleration?: HybridAcceleration;
}

export type HarmonicSolverMode = 'PureFEM' | 'PureBEM' | 'Robin';

export interface HarmonicSimulationRequest {
  mesh: MeshData;
  reference_material: ReferenceMaterial;
  strategy: MaterialStrategyConfig;
  frequency_hz: number;
  load_magnitude: number;
  max_gmres_iter: number;
  tolerance: number;
  use_robin_coupling?: boolean;
  solver_mode?: HarmonicSolverMode;
  bem_element_type?: BemElementType;
  stress_state?: StressState2D;
}

export interface HarmonicSimulationResult {
  converged: boolean;
  iterations: number;
  relative_residual: number;
  frequency_hz: number;
  amplitudes: [number, number, number][];
  phase_lags: [number, number, number][];
  deformed_nodes: [number, number, number][];
  von_mises_stresses: number[];
  max_amplitude: number;
  max_von_mises: number;
  execution_time_ms: number;
}

export interface SimulationFrame {
  time: number;
  displacements: [number, number, number][];
  deformed_nodes: [number, number, number][];
  von_mises_stresses: number[];
  max_displacement: number;
}

export interface TransientSimulationRequest {
  mesh: MeshData;
  reference_material: ReferenceMaterial;
  strategy: MaterialStrategyConfig;
  total_time: number;
  time_steps: number;
  num_talbot_points: number;
  load_magnitude: number;
  pulse_duration: number;
  excitation_type?: TransientExcitation;
  excitation_frequency?: number;
  integration_method?: TransientMethod;
  hht_alpha?: number;
  rayleigh_alpha?: number;
  rayleigh_beta?: number;
  solver_mode?: SolverMode;
  bem_element_type?: BemElementType;
}

export interface TransientSimulationResult {
  time_points: number[];
  frames: SimulationFrame[];
  max_overall_displacement: number;
  execution_time_ms: number;
}

@Injectable({
  providedIn: 'root',
})
export class SolverService {
  // Current active regime
  readonly simulationRegime = signal<SimulationRegime>('static');

  // Results
  readonly currentResult = signal<SimulationResult | null>(null);
  readonly harmonicResult = signal<HarmonicSimulationResult | null>(null);
  readonly transientResult = signal<TransientSimulationResult | null>(null);
  readonly talbotTransientResult = signal<TransientSimulationResult | null>(null);
  readonly newmarkTransientResult = signal<TransientSimulationResult | null>(null);

  readonly isSolving = signal<boolean>(false);
  readonly solverError = signal<string | null>(null);

  // Static solver configuration
  readonly selectedMode = signal<SolverMode>('PureFEM');
  readonly bemElementType = signal<BemElementType>('QuadraticDiscontinuous');
  readonly stressState = signal<StressState2D>('PlaneStress');
  readonly hybridAccelerationType = signal<HybridAccelerationType>('Anderson');
  readonly andersonDepth = signal<number>(3);
  readonly andersonBeta = signal<number>(0.65);
  readonly relaxationOmega = signal<number>(0.85);
  readonly loadMagnitude = signal<number>(50000.0); // 50 kN
  readonly maxIterations = signal<number>(50);
  readonly tolerance = signal<number>(1e-4);

  // Dynamic Harmonic parameters
  readonly harmonicSolverMode = signal<HarmonicSolverMode>('PureFEM');
  readonly frequencyHz = signal<number>(20.0); // 20 Hz
  readonly maxGmresIter = signal<number>(40);
  readonly useRobinCoupling = signal<boolean>(false);

  // Dynamic Transient parameters
  readonly transientSolverMode = signal<'PureFEM' | 'PureBEM'>('PureFEM');
  readonly transientMethod = signal<TransientMethod>('Talbot');
  readonly transientExcitation = signal<TransientExcitation>('HarmonicSine');
  readonly transientFrequency = signal<number>(3.0); // 3 Hz default for comparison
  readonly hhtAlpha = signal<number>(0.0); // 0 = standard Newmark, <0 = dissipative HHT
  readonly rayleighAlpha = signal<number>(0.05);
  readonly rayleighBeta = signal<number>(0.0002);
  readonly totalTime = signal<number>(0.50); // 500 ms (enough for multiple wave cycles)
  readonly timeSteps = signal<number>(25);
  readonly numTalbotPoints = signal<number>(32);
  readonly pulseDuration = signal<number>(0.05); // 50 ms
  readonly currentFrameIndex = signal<number>(0);
  readonly isPlayingAnimation = signal<boolean>(false);

  private animationTimer: any = null;

  async runSimulation(
    mesh: MeshData,
    referenceMaterial: ReferenceMaterial,
    strategy: MaterialStrategyConfig
  ): Promise<SimulationResult | null> {
    this.isSolving.set(true);
    this.solverError.set(null);

    let accel: HybridAcceleration = 'None';
    if (this.hybridAccelerationType() === 'Anderson') {
      accel = { Anderson: { depth: this.andersonDepth(), beta: this.andersonBeta() } };
    } else if (this.hybridAccelerationType() === 'Relaxation') {
      accel = { Relaxation: this.relaxationOmega() };
    }

    const request: SimulationRequest = {
      solver_mode: this.selectedMode(),
      mesh,
      reference_material: referenceMaterial,
      strategy,
      load_magnitude: this.loadMagnitude(),
      max_iterations: this.maxIterations(),
      tolerance: this.tolerance(),
      bem_element_type: this.bemElementType(),
      stress_state: this.stressState(),
      hybrid_acceleration: accel,
    };

    try {
      const result = await invoke<SimulationResult>('run_simulation', { request });
      this.currentResult.set(result);
      return result;
    } catch (err) {
      const errorMsg = typeof err === 'string' ? err : JSON.stringify(err);
      this.solverError.set(errorMsg);
      console.error('Simulation error:', errorMsg);
      return null;
    } finally {
      this.isSolving.set(false);
    }
  }

  async runHarmonicSimulation(
    mesh: MeshData,
    referenceMaterial: ReferenceMaterial,
    strategy: MaterialStrategyConfig
  ): Promise<HarmonicSimulationResult | null> {
    this.isSolving.set(true);
    this.solverError.set(null);

    const request: HarmonicSimulationRequest = {
      mesh,
      reference_material: referenceMaterial,
      strategy,
      frequency_hz: this.frequencyHz(),
      load_magnitude: this.loadMagnitude(),
      max_gmres_iter: this.maxGmresIter(),
      tolerance: this.tolerance(),
      solver_mode: this.harmonicSolverMode(),
      use_robin_coupling: this.harmonicSolverMode() === 'Robin',
      bem_element_type: this.bemElementType(),
      stress_state: this.stressState(),
    };

    try {
      const result = await invoke<HarmonicSimulationResult>('run_harmonic_simulation', { request });
      this.harmonicResult.set(result);
      return result;
    } catch (err) {
      const errorMsg = typeof err === 'string' ? err : JSON.stringify(err);
      this.solverError.set(errorMsg);
      console.error('Harmonic simulation error:', errorMsg);
      return null;
    } finally {
      this.isSolving.set(false);
    }
  }

  async runTransientSimulation(
    mesh: MeshData,
    referenceMaterial: ReferenceMaterial,
    strategy: MaterialStrategyConfig
  ): Promise<TransientSimulationResult | null> {
    this.isSolving.set(true);
    this.solverError.set(null);
    this.pauseAnimation();

    const request: TransientSimulationRequest = {
      mesh,
      reference_material: referenceMaterial,
      strategy,
      total_time: this.totalTime(),
      time_steps: this.timeSteps(),
      num_talbot_points: this.numTalbotPoints(),
      load_magnitude: this.loadMagnitude(),
      pulse_duration: this.pulseDuration(),
      excitation_type: this.transientExcitation(),
      excitation_frequency: this.transientFrequency(),
      integration_method: this.transientMethod(),
      hht_alpha: this.hhtAlpha(),
      rayleigh_alpha: this.rayleighAlpha(),
      rayleigh_beta: this.rayleighBeta(),
      solver_mode: this.transientSolverMode(),
      bem_element_type: this.bemElementType(),
    };

    try {
      const result = await invoke<TransientSimulationResult>('run_transient_simulation', { request });
      this.transientResult.set(result);
      if (this.transientMethod() === 'Talbot') {
        this.talbotTransientResult.set(result);
      } else {
        this.newmarkTransientResult.set(result);
      }
      this.currentFrameIndex.set(0);
      return result;
    } catch (err) {
      const errorMsg = typeof err === 'string' ? err : JSON.stringify(err);
      this.solverError.set(errorMsg);
      console.error('Transient simulation error:', errorMsg);
      return null;
    } finally {
      this.isSolving.set(false);
    }
  }

  playAnimation(): void {
    const res = this.transientResult();
    if (!res || res.frames.length === 0) return;

    this.isPlayingAnimation.set(true);
    if (this.animationTimer) clearInterval(this.animationTimer);

    this.animationTimer = setInterval(() => {
      const total = res.frames.length;
      const next = (this.currentFrameIndex() + 1) % total;
      this.currentFrameIndex.set(next);
    }, 100);
  }

  pauseAnimation(): void {
    this.isPlayingAnimation.set(false);
    if (this.animationTimer) {
      clearInterval(this.animationTimer);
      this.animationTimer = null;
    }
  }

  setFrame(index: number): void {
    const res = this.transientResult();
    if (!res) return;
    const clamped = Math.max(0, Math.min(index, res.frames.length - 1));
    this.currentFrameIndex.set(clamped);
  }

  clearResult(): void {
    this.pauseAnimation();
    this.currentResult.set(null);
    this.harmonicResult.set(null);
    this.transientResult.set(null);
    this.solverError.set(null);
  }
}
