import fs from 'node:fs';
import readline from 'node:readline';

async function check7075() {
  const fileStream = fs.createReadStream('C:/Users/buffo/.gemini/antigravity/brain/9f63e068-2e81-4517-8a54-4587394e739e/.system_generated/logs/transcript_full.jsonl');
  const rl = readline.createInterface({ input: fileStream, crlfDelay: Infinity });

  for await (const line of rl) {
    if (line.includes('"step_index":7071') || line.includes('"step_index":7075') || line.includes('"step_index":7087')) {
      const obj = JSON.parse(line);
      for (const tc of obj.tool_calls || []) {
        console.log('Step', obj.step_index, tc.name, tc.args.Path, 'Target:', tc.args.Target ? tc.args.Target.slice(0, 50) : '', 'Repl:', tc.args.Replacement ? tc.args.Replacement.slice(0, 50) : '');
      }
    }
  }
}

check7075();
