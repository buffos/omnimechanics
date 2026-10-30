import fs from 'node:fs';
import path from 'node:path';

function fixFile(p) {
  let code = fs.readFileSync(p, 'utf8');
  // Check if it has #[cfg(test)]\n(pub )?mod tests {
  const match = code.match(/#\[cfg\(test\)\]\s*(?:pub\s+)?mod\s+tests\s*\{([\s\S]*)\}\s*$/);
  if (match) {
    const inner = match[1];
    code = code.replace(/#\[cfg\(test\)\]\s*(?:pub\s+)?mod\s+tests\s*\{[\s\S]*\}\s*$/, inner.trim() + '\n');
    fs.writeFileSync(p, code);
    console.log('Unwrapped mod tests in', p);
  }
}

const dirs = ['src-tauri/src/solver', 'src-tauri/src/mesh', 'src-tauri/src/benchmarks'];
for (const d of dirs) {
  const files = fs.readdirSync(d);
  for (const f of files) {
    if (f.endsWith('_test.rs')) {
      fixFile(path.join(d, f));
    }
  }
}
