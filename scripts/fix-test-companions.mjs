import fs from 'node:fs';

const items = [
  { dir: 'src-tauri/src/solver', name: 'anderson' },
  { dir: 'src-tauri/src/solver', name: 'bessel' },
  { dir: 'src-tauri/src/solver', name: 'boundary_shape' },
  { dir: 'src-tauri/src/solver', name: 'krylov_solver' },
  { dir: 'src-tauri/src/solver', name: 'talbot' },
  { dir: 'src-tauri/src/mesh', name: 'generators_2d' },
  { dir: 'src-tauri/src/mesh', name: 'generators_3d' },
  { dir: 'src-tauri/src/benchmarks', name: 'complexity_scaling' },
  { dir: 'src-tauri/src/benchmarks', name: 'dynamic_operator_splitting' },
  { dir: 'src-tauri/src/benchmarks', name: 'half_space_gibson' },
  { dir: 'src-tauri/src/benchmarks', name: 'mini_paper_suite' },
  { dir: 'src-tauri/src/benchmarks', name: 'static_contrast' },
  { dir: 'src-tauri/src/benchmarks', name: 'transient_elastodynamics' },
];

for (const item of items) {
  const prodP = `${item.dir}/${item.name}.rs`;
  const testP = `${item.dir}/${item.name}_test.rs`;

  // Update prod file with path link to external test file
  let prodCode = fs.readFileSync(prodP, 'utf8').trimEnd();
  // Remove any existing cfg(test)
  const idx = prodCode.indexOf('#[cfg(test)]');
  if (idx !== -1) {
    prodCode = prodCode.slice(0, idx).trimEnd();
  }
  prodCode += `\n\n#[cfg(test)]\n#[path = "${item.name}_test.rs"]\nmod tests;\n`;
  fs.writeFileSync(prodP, prodCode);

  // In test file, wrap in use super::*;
  let testCode = fs.readFileSync(testP, 'utf8');
  // clean up extra use super::...
  testCode = testCode.replace(/use super::\w+::\*;?/g, '');
  testCode = testCode.replace(/#\[cfg\(test\)\]\s*mod\s+tests\s*\{([\s\S]*)\}\s*$/, '$1');

  const wrapped = `use super::*;\n\n` + testCode.trim() + '\n';
  fs.writeFileSync(testP, wrapped);
  console.log(`Configured companion test for ${item.name}`);
}
