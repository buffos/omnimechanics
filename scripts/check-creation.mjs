import fs from 'node:fs';
import readline from 'node:readline';

async function checkCreation() {
  const fileStream = fs.createReadStream('C:/Users/buffo/.gemini/antigravity/brain/9f63e068-2e81-4517-8a54-4587394e739e/.system_generated/logs/transcript_full.jsonl');
  const rl = readline.createInterface({ input: fileStream, crlfDelay: Infinity });

  for await (const line of rl) {
    if (line.includes('oraiopoulos_bem_fem.rs')) {
      const obj = JSON.parse(line);
      for (const tc of obj.tool_calls || []) {
        if (tc.name === 'write_file' && tc.args.Path && tc.args.Path.includes('oraiopoulos_bem_fem.rs')) {
          console.log('Step', obj.step_index, 'write_file len:', tc.args.Content.length);
        } else if (tc.name === 'run_command' && tc.args.CommandLine && tc.args.CommandLine.includes('oraiopoulos_bem_fem')) {
          console.log('Step', obj.step_index, 'cmd:', tc.args.CommandLine);
        }
      }
    }
  }
}

checkCreation();
