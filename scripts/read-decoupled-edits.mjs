import fs from 'node:fs';
import readline from 'node:readline';

async function listDecoupledEdits() {
  const fileStream = fs.createReadStream('C:/Users/buffo/.gemini/antigravity/brain/9f63e068-2e81-4517-8a54-4587394e739e/.system_generated/logs/transcript_full.jsonl');
  const rl = readline.createInterface({ input: fileStream, crlfDelay: Infinity });

  for await (const line of rl) {
    if (line.includes('decoupled_hybrid.rs') && line.includes('"name":"edit_file"')) {
      const obj = JSON.parse(line);
      for (const tc of obj.tool_calls || []) {
        if (tc.name === 'edit_file' && tc.args.Path.includes('decoupled_hybrid.rs')) {
          console.log(`Step ${obj.step_index}: Target: ${tc.args.Target.slice(0, 40).replace(/\n/g, ' ')}...`);
        }
      }
    }
  }
}

listDecoupledEdits();
