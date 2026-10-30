import fs from 'node:fs';

fs.mkdirSync('src-tauri/src/solver/tests', { recursive: true });

// Move tests_static to tests/static_tests.rs
const sCode = fs.readFileSync('src-tauri/src/solver/tests_static.rs', 'utf8');
fs.writeFileSync('src-tauri/src/solver/tests/static_tests.rs', sCode);
fs.unlinkSync('src-tauri/src/solver/tests_static.rs');

// Move tests_benchmarks to tests/benchmark_tests.rs
const bCode = fs.readFileSync('src-tauri/src/solver/tests_benchmarks.rs', 'utf8');
fs.writeFileSync('src-tauri/src/solver/tests/benchmark_tests.rs', bCode);
fs.unlinkSync('src-tauri/src/solver/tests_benchmarks.rs');

// Move tests_harmonic to tests/harmonic_tests.rs
const hCode = fs.readFileSync('src-tauri/src/solver/tests_harmonic.rs', 'utf8');
fs.writeFileSync('src-tauri/src/solver/tests/harmonic_tests.rs', hCode);
fs.unlinkSync('src-tauri/src/solver/tests_harmonic.rs');

// Move tests_transient to tests/transient_tests.rs
const tCode = fs.readFileSync('src-tauri/src/solver/tests_transient.rs', 'utf8');
fs.writeFileSync('src-tauri/src/solver/tests/transient_tests.rs', tCode);
fs.unlinkSync('src-tauri/src/solver/tests_transient.rs');

// Remove tests.rs file so tests/mod.rs is used
if (fs.existsSync('src-tauri/src/solver/tests.rs')) {
  fs.unlinkSync('src-tauri/src/solver/tests.rs');
}

// Create tests/mod.rs
const modCode = `#[cfg(test)]
mod static_tests;
#[cfg(test)]
mod benchmark_tests;
#[cfg(test)]
mod harmonic_tests;
#[cfg(test)]
mod transient_tests;
`;
fs.writeFileSync('src-tauri/src/solver/tests/mod.rs', modCode);

// Fix Complex64 in bessel_test.rs and krylov_solver_test.rs
const besselTestP = 'src-tauri/src/solver/bessel_test.rs';
if (fs.existsSync(besselTestP)) {
  const besselTest = fs.readFileSync(besselTestP, 'utf8');
  if (!besselTest.includes('Complex64')) {
    fs.writeFileSync(besselTestP, 'use num_complex::Complex64;\n' + besselTest);
  } else if (!besselTest.includes('use num_complex::Complex64;')) {
    fs.writeFileSync(besselTestP, 'use num_complex::Complex64;\n' + besselTest);
  }
}

const krylovTestP = 'src-tauri/src/solver/krylov_solver_test.rs';
if (fs.existsSync(krylovTestP)) {
  const krylovTest = fs.readFileSync(krylovTestP, 'utf8');
  if (!krylovTest.includes('use num_complex::Complex64;')) {
    fs.writeFileSync(krylovTestP, 'use num_complex::Complex64;\n' + krylovTest);
  }
}

console.log('Modularized solver tests into tests/ directory successfully!');
