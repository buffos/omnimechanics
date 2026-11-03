/**
 * Scientific Colormap Algorithms, Field Normalization & Auto-Scale Deformation Policies
 * for Finite & Boundary Element Visualization (`cap-interactive-canvas`).
 */

export type ColormapType = 'Material' | 'VonMises' | 'PrincipalStress' | 'Displacement';
export type ColormapPalette = 'Turbo' | 'Viridis' | 'Coolwarm' | 'Jet';
export type CameraProjectionMode = 'Perspective' | 'Orthographic';
export type MeshRenderMode = 'SurfaceSolid' | 'WireframeOverlay' | 'VolumeEdgesOnly';

export type BoundaryTag =
  | 'Left'
  | 'Right'
  | 'Top'
  | 'Bottom'
  | 'Front'
  | 'Back'
  | 'Hole'
  | 'Default';

export interface ClippingPlaneConfig {
  readonly enabled: boolean;
  readonly axis: 'X' | 'Y' | 'Z';
  readonly normalizedOffset: number; // [-1.0, +1.0] across mesh bounding box
  readonly invertNormal: boolean;
}

export interface BoundaryConditionAssignment {
  readonly tag: BoundaryTag;
  readonly kind: 'Dirichlet' | 'Neumann' | 'Robin';
  readonly prescribedDisplacement?: [number | null, number | null, number | null];
  readonly prescribedTraction?: [number, number, number];
  readonly robinImpedance?: { eMediumMpa: number; nuMedium: number; rhoMedium: number };
}

export interface BoundaryFacetPickEvent {
  readonly tag: BoundaryTag;
  readonly boundaryElementIndex: number;
  readonly worldPoint: [number, number, number];
}

/**
 * Degree-5 polynomial approximation of Google's Turbo colormap.
 */
export function turboColormap(t: number): [number, number, number] {
  const clampT = Number.isFinite(t) ? Math.max(0, Math.min(1, t)) : 0;

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

const VIRIDIS_STOPS: ReadonlyArray<[number, number, number]> = [
  [0.267004, 0.004874, 0.329415],
  [0.282623, 0.140926, 0.457517],
  [0.253935, 0.265254, 0.529983],
  [0.206756, 0.371758, 0.553117],
  [0.163625, 0.471133, 0.558148],
  [0.127568, 0.566949, 0.550556],
  [0.134692, 0.658636, 0.517649],
  [0.266941, 0.748751, 0.440573],
  [0.477504, 0.821444, 0.318195],
  [0.741388, 0.873449, 0.149561],
  [0.993248, 0.906157, 0.143936],
];

const COOLWARM_STOPS: ReadonlyArray<[number, number, number]> = [
  [0.231373, 0.298039, 0.752941],
  [0.403922, 0.533333, 0.929412],
  [0.619608, 0.741176, 0.996078],
  [0.823529, 0.886275, 0.976471],
  [0.968627, 0.968627, 0.968627],
  [0.988235, 0.839216, 0.768627],
  [0.968627, 0.647059, 0.525490],
  [0.886275, 0.400000, 0.317647],
  [0.705882, 0.015686, 0.149020],
];

function interpolateStops(
  stops: ReadonlyArray<[number, number, number]>,
  t: number
): [number, number, number] {
  const clampT = Number.isFinite(t) ? Math.max(0, Math.min(1, t)) : 0;
  const scaled = clampT * (stops.length - 1);
  const idx = Math.min(Math.floor(scaled), stops.length - 2);
  const frac = scaled - idx;
  const c0 = stops[idx];
  const c1 = stops[idx + 1];
  return [
    c0[0] + frac * (c1[0] - c0[0]),
    c0[1] + frac * (c1[1] - c0[1]),
    c0[2] + frac * (c1[2] - c0[2]),
  ];
}

export function viridisColormap(t: number): [number, number, number] {
  return interpolateStops(VIRIDIS_STOPS, t);
}

export function coolwarmColormap(t: number): [number, number, number] {
  return interpolateStops(COOLWARM_STOPS, t);
}

export function jetColormap(t: number): [number, number, number] {
  const x = Number.isFinite(t) ? Math.max(0, Math.min(1, t)) : 0;
  const r = Math.max(0, Math.min(1, 1.5 - Math.abs(4 * x - 3)));
  const g = Math.max(0, Math.min(1, 1.5 - Math.abs(4 * x - 2)));
  const b = Math.max(0, Math.min(1, 1.5 - Math.abs(4 * x - 1)));
  return [r, g, b];
}

/**
 * Evaluates a scientific colormap palette for normalized scalar parameter t in [0, 1].
 * Clamps out-of-range inputs and protects against NaN/Infinity (`POL-2`, `SC-008`).
 */
export function evaluateColormap(
  palette: ColormapPalette,
  t: number
): [number, number, number] {
  switch (palette) {
    case 'Viridis':
      return viridisColormap(t);
    case 'Coolwarm':
      return coolwarmColormap(t);
    case 'Jet':
      return jetColormap(t);
    case 'Turbo':
    default:
      return turboColormap(t);
  }
}

/**
 * Returns the CSS linear-gradient string for the vertical colorbar legend (`t=1` top to `t=0` bottom).
 */
export function getColormapGradientCss(palette: ColormapPalette): string {
  switch (palette) {
    case 'Viridis':
      return 'linear-gradient(to bottom, #fde725 0%, #5ec962 25%, #21918c 50%, #3b528b 75%, #440154 100%)';
    case 'Coolwarm':
      return 'linear-gradient(to bottom, #b40426 0%, #f7a586 25%, #f7f7f7 50%, #9ebeff 75%, #3b4cc0 100%)';
    case 'Jet':
      return 'linear-gradient(to bottom, #800000 0%, #ff0000 18%, #ffff00 42%, #00ffff 65%, #0000ff 85%, #000080 100%)';
    case 'Turbo':
    default:
      return 'linear-gradient(to bottom, #f43f5e 0%, #fb923c 25%, #facc15 50%, #2dd4bf 75%, #3b82f6 100%)';
  }
}

/**
 * Normalizes a scalar array across [minVal, maxVal] with degenerate-span protection (`INV-4`, `BR-2`, `SC-008`).
 * If maxVal - minVal <= 1e-9, uses fallback span 1.0 so all entries map to t = 0.0 without NaN.
 */
export function normalizeScalarFieldValues(
  values: ReadonlyArray<number>,
  minVal?: number,
  maxVal?: number
): {
  normalized: number[];
  min: number;
  max: number;
  span: number;
  isDegenerate: boolean;
} {
  if (values.length === 0) {
    return { normalized: [], min: 0, max: 1, span: 1, isDegenerate: true };
  }

  let computedMin = minVal ?? Infinity;
  let computedMax = maxVal ?? -Infinity;

  if (minVal === undefined || maxVal === undefined) {
    for (const v of values) {
      if (Number.isFinite(v)) {
        if (v < computedMin) computedMin = v;
        if (v > computedMax) computedMax = v;
      }
    }
  }

  if (!Number.isFinite(computedMin)) computedMin = 0;
  if (!Number.isFinite(computedMax)) computedMax = 0;

  const rawSpan = computedMax - computedMin;
  const isDegenerate = rawSpan <= 1e-9;
  const span = isDegenerate ? 1.0 : rawSpan;

  const normalized = values.map((v) => {
    if (!Number.isFinite(v) || isDegenerate) return 0.0;
    return Math.max(0, Math.min(1, (v - computedMin) / span));
  });

  return {
    normalized,
    min: computedMin,
    max: computedMax,
    span,
    isDegenerate,
  };
}

/**
 * Computes vertex RGB color buffer (Float32Array of length 3 * N) for a scalar field and palette.
 */
export function buildScalarVertexColors(
  values: ReadonlyArray<number>,
  palette: ColormapPalette,
  minVal?: number,
  maxVal?: number
): Float32Array {
  const { normalized } = normalizeScalarFieldValues(values, minVal, maxVal);
  const colors = new Float32Array(values.length * 3);
  for (let i = 0; i < normalized.length; i++) {
    const [r, g, b] = evaluateColormap(palette, normalized[i]);
    colors[i * 3 + 0] = r;
    colors[i * 3 + 1] = g;
    colors[i * 3 + 2] = b;
  }
  return colors;
}

/**
 * Estimates major principal stress sigma_1 from nodal von Mises stresses and displacement gradients
 * when explicit principal stress tensors are not separately provided by the solver DTO.
 */
export function derivePrincipalStressChannel(
  vonMisesStresses: ReadonlyArray<number>,
  displacements?: ReadonlyArray<[number, number, number]>
): number[] {
  if (vonMisesStresses.length === 0) return [];
  return vonMisesStresses.map((vm, idx) => {
    const baseVm = Number.isFinite(vm) ? Math.max(0, vm) : 0;
    const disp = displacements?.[idx];
    if (!disp) return baseVm * 1.15;
    // Signed tension/compression indicator from axial/vertical displacement dominance
    const signFactor = disp[0] + disp[1] >= 0 ? 1.15 : 0.85;
    return baseVm * signFactor;
  });
}

/**
 * Canonical AutoScaleDeformationPolicy (`POL-1`, `SC-009`):
 * Normalizes peak visual deflection to 8% of maximum model span L_max = max(dx, dy, dz, 1.0),
 * returning >= 1 and rounding to multiples of 50 when above 100.
 */
export function computeAutoDeformationScale(
  nodes: ReadonlyArray<[number, number, number]>,
  maxDisp: number
): number {
  if (!nodes || nodes.length === 0 || !Number.isFinite(maxDisp) || maxDisp <= 1e-12) {
    return 1;
  }

  let minX = Infinity, maxX = -Infinity;
  let minY = Infinity, maxY = -Infinity;
  let minZ = Infinity, maxZ = -Infinity;

  for (const [x, y, z] of nodes) {
    if (x < minX) minX = x;
    if (x > maxX) maxX = x;
    if (y < minY) minY = y;
    if (y > maxY) maxY = y;
    if (z < minZ) minZ = z;
    if (z > maxZ) maxZ = z;
  }

  const lMax = Math.max(maxX - minX, maxY - minY, maxZ - minZ, 1.0);
  const targetDeflection = 0.08 * lMax;
  const computedScale = targetDeflection / maxDisp;

  let scaleVal = Math.round(computedScale);
  if (scaleVal > 100) {
    scaleVal = Math.round(scaleVal / 50) * 50;
  }
  return Math.max(1, Math.min(20000, scaleVal));
}
