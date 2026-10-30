import fs from 'node:fs';
import readline from 'node:readline';

async function findTests() {
  const fileStream = fs.createReadStream('C:/Users/buffo/.gemini/antigravity/brain/9f63e068-2e81-4517-8a54-4587394e739e/.system_generated/logs/transcript_full.jsonl');
  const rl = readline.createInterface({ input: fileStream, crlfDelay: Infinity });

  let best = null;
  let bestStep = 0;
  for await (const line of rl) {
    if (line.includes('tests.rs') && line.includes('"write_file"')) {
      const obj = JSON.parse(line);
      for (const tc of obj.tool_calls || []) {
        if (tc.name === 'write_file' && tc.args.Path.endsWith('tests.rs') && tc.args.Content && tc.args.Content.length > 20000) {
          best = tc.args.Content;
          bestStep = obj.step_index;
        }
      }
    }
  }

  if (best) {
    console.log(`Found original tests.rs in step ${bestStep}, length: ${best.length}`);
    fs.writeFileSync('src-tauri/src/solver/tests_backup.rs', best);
  } else {
    console.log('No large tests.rs found in write_file');
  }
}

findTests();
