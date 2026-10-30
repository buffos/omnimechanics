/**
 * Scientific Colormap Algorithms for Finite & Boundary Element Visualization
 */

export function turboColormap(t: number): [number, number, number] {
  const clampT = Math.max(0, Math.min(1, t));

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
