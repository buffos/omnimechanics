import fs from 'node:fs';

const benchFiles = [
  'complexity_scaling',
  'dynamic_operator_splitting',
  'half_space_gibson',
  'mini_paper_suite',
  'static_contrast',
  'transient_elastodynamics',
];

const header = `use super::*;
use crate::material::strategies::*;
use crate::material::traits::*;
use crate::material::types::*;
use crate::material::ReferenceMaterial;
use crate::mesh::generators_2d::*;
use crate::mesh::generators_3d::*;
use crate::mesh::types::*;
use crate::solver::*;
use std::path::Path;
use std::fs::{create_dir_all, File};
use std::io::Write;

`;

for (const name of benchFiles) {
  const p = `src-tauri/src/benchmarks/${name}_test.rs`;
  let code = fs.readFileSync(p, 'utf8');
  code = code.replace(/use super::\*;\n?/g, '');
  fs.writeFileSync(p, header + code);
  console.log('Fixed imports for', name);
}
