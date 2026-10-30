import fs from 'node:fs';
import readline from 'node:readline';

async function checkSteps() {
  const fileStream = fs.createReadStream('C:/Users/buffo/.gemini/antigravity/brain/9f63e068-2e81-4517-8a54-4587394e739e/.system_generated/logs/transcript_full.jsonl');
  const rl = readline.createInterface({ input: fileStream, crlfDelay: Infinity });

  for await (const line of rl) {
    if (line.includes('"name":"edit_file"') || line.includes('"name":"write_file"')) {
      const obj = JSON.parse(line);
      if (obj.step_index >= 6900 && obj.step_index <= 7400) {
        for (const tc of obj.tool_calls || []) {
          if (tc.args.Path && (tc.args.Path.includes('oraiopoulos') || tc.args.Path.includes('decoupled'))) {
            console.log(obj.step_index, tc.name, tc.args.Path, tc.args.Target ? 'edit target len ' + tc.args.Target.length : 'write len ' + tc.args.Content.length);
          }
        }
      }
    }
  }
}

checkSteps();
