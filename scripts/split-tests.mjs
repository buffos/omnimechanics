import fs from 'node:fs';

const content = fs.readFileSync('src-tauri/src/solver/tests.rs', 'utf8');

const t2 = content.indexOf('    #[test]\n    fn test_classic_square_tension_bem_benchmark()');
const t3 = content.indexOf('    #[test]\n    fn test_harmonic_pure_bem_2d_and_3d()');
const t4 = content.indexOf('    #[test]\n    fn test_all_transient_elements_talbot_and_newmark()');

const commonHeader = `use crate::material::strategies::*;
use crate::material::types::*;
use crate::mesh::generators_2d::*;
use crate::mesh::generators_3d::*;
use crate::solver::*;
use faer::Mat;
use num_complex::Complex64;

`;

// 1. Static: from line 8 to t2
const staticBody = content.slice(content.indexOf('    #[test]\n    fn test_fem_cantilever_deflection()'), t2);
fs.writeFileSync('src-tauri/src/solver/tests_static.rs', commonHeader + staticBody);

// 2. Benchmarks: from t2 to t3
const benchBody = content.slice(t2, t3);
fs.writeFileSync('src-tauri/src/solver/tests_benchmarks.rs', commonHeader + benchBody);

// 3. Harmonic: from t3 to t4
const harmBody = content.slice(t3, t4);
fs.writeFileSync('src-tauri/src/solver/tests_harmonic.rs', commonHeader + harmBody);

// 4. Transient: from t4 to last '}'
const lastBrace = content.lastIndexOf('}');
const transBody = content.slice(t4, lastBrace);
fs.writeFileSync('src-tauri/src/solver/tests_transient.rs', commonHeader + transBody);

// 5. Rewrite tests.rs
const newTests = `#[cfg(test)]
mod tests_static;
#[cfg(test)]
mod tests_benchmarks;
#[cfg(test)]
mod tests_harmonic;
#[cfg(test)]
mod tests_transient;
`;
fs.writeFileSync('src-tauri/src/solver/tests.rs', newTests);

console.log('Successfully split tests.rs!');
