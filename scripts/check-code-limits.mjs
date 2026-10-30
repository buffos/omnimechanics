import fs from 'node:fs';
import path from 'node:path';

const LIMITS = {
  '.rs': 1000,
  '.ts': 1000,
  '.js': 1000,
  '.html': 500,
};

const IGNORED_DIRS = new Set([
  'node_modules',
  '.git',
  'dist',
  'target',
  'target_test',
  '.agents',
  '.system_generated',
  '.angular',
]);

const SCAN_ROOTS = ['src', 'src-tauri'];

function scanDirectory(dir) {
  let fileList = [];
  if (!fs.existsSync(dir)) return fileList;
  const entries = fs.readdirSync(dir, { withFileTypes: true });

  for (const entry of entries) {
    if (IGNORED_DIRS.has(entry.name)) continue;
    const fullPath = path.join(dir, entry.name);

    if (entry.isDirectory()) {
      fileList = fileList.concat(scanDirectory(fullPath));
    } else if (entry.isFile()) {
      const ext = path.extname(entry.name).toLowerCase();
      if (ext in LIMITS) {
        fileList.push({
          path: fullPath,
          ext,
          name: entry.name,
        });
      }
    }
  }

  return fileList;
}

function countLines(filePath) {
  const content = fs.readFileSync(filePath, 'utf8');
  // Handle empty file edge case
  if (content.length === 0) return 0;
  return content.split(/\r?\n/).length;
}

function checkRustInlineTests(filePath, fileName) {
  // Dedicated test files or files in a tests/ directory are exempt
  const normalized = filePath.replace(/\\/g, '/');
  if (
    normalized.includes('/tests/') ||
    fileName === 'tests.rs' ||
    fileName.endsWith('_test.rs') ||
    fileName.endsWith('_tests.rs') ||
    fileName.startsWith('test_')
  ) {
    return null;
  }

  const content = fs.readFileSync(filePath, 'utf8');
  // Check for inline #[test] or inline test blocks like mod tests { ... } or #[cfg(test)] with inline block
  const hasInlineTest = /#\[test\]/.test(content);
  const hasModTests = /mod\s+tests\s*\{/.test(content);
  const hasInlineCfgBlock = /#\[cfg\(test\)\]\s*(mod\s+\w+\s*\{|fn\s+)/.test(content);

  if (hasInlineTest || hasModTests || hasInlineCfgBlock) {
    return 'Rust tests must be placed in a separate test file (e.g. tests/*.rs or *_test.rs), not inline in production files.';
  }

  return null;
}

function runCheck() {
  console.log('\x1b[36m%s\x1b[0m', '🔍 Running Code Quality & File Size Check...');
  console.log(`- Rust (.rs), TypeScript (.ts), JavaScript (.js): Max ${LIMITS['.rs']} lines`);
  console.log(`- HTML (.html): Max ${LIMITS['.html']} lines`);
  console.log('- Rust unit/integration tests must reside in separate test files.\n');

  const allFiles = SCAN_ROOTS.flatMap(root => scanDirectory(root));
  const lineCountViolations = [];
  const rustTestViolations = [];

  for (const file of allFiles) {
    const lines = countLines(file.path);
    const maxAllowed = LIMITS[file.ext];

    if (lines > maxAllowed) {
      lineCountViolations.push({
        path: file.path,
        lines,
        maxAllowed,
        excess: lines - maxAllowed,
      });
    }

    if (file.ext === '.rs') {
      const testViolation = checkRustInlineTests(file.path, file.name);
      if (testViolation) {
        rustTestViolations.push({
          path: file.path,
          reason: testViolation,
        });
      }
    }
  }

  let failed = false;

  if (lineCountViolations.length > 0) {
    failed = true;
    console.error('\x1b[31m%s\x1b[0m', `❌ File Size Limit Exceeded (${lineCountViolations.length} file(s)):`);
    for (const v of lineCountViolations) {
      console.error(
        `   • \x1b[33m${v.path}\x1b[0m: \x1b[31m${v.lines} lines\x1b[0m (Allowed max: ${v.maxAllowed}, excess: +${v.excess})`
      );
    }
    console.error('\n   \x1b[35mImmediate refactoring required before commit.\x1b[0m\n');
  }

  if (rustTestViolations.length > 0) {
    failed = true;
    console.error('\x1b[31m%s\x1b[0m', `❌ Rust Inline Test Violations (${rustTestViolations.length} file(s)):`);
    for (const v of rustTestViolations) {
      console.error(`   • \x1b[33m${v.path}\x1b[0m: ${v.reason}`);
    }
    console.error('\n   \x1b[35mMove tests to companion *_test.rs or src-tauri/tests/*.rs before commit.\x1b[0m\n');
  }

  if (failed) {
    console.error('\x1b[41m\x1b[37m COMMIT REJECTED \x1b[0m - Please resolve violations above.\n');
    process.exit(1);
  }

  console.log('\x1b[32m%s\x1b[0m', `✔ All ${allFiles.length} files passed line count and test location checks!`);
  process.exit(0);
}

runCheck();
