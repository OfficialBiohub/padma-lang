import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, rmSync, statSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const artifact = resolve(root, 'dist', 'padma-vscode-0.1.0.vsix');
const vsce = resolve(root, 'node_modules', '.bin', process.platform === 'win32' ? 'vsce.cmd' : 'vsce');

rmSync(artifact, { force: true });
mkdirSync(dirname(artifact), { recursive: true });
execFileSync(vsce, ['package', '--no-dependencies', '--out', artifact], {
  cwd: root,
  stdio: 'inherit',
});

if (!existsSync(artifact) || statSync(artifact).size === 0) {
  throw new Error('VS Code extension packaging did not produce a non-empty .vsix artifact.');
}

console.log(`Validated VSIX artifact: ${artifact} (${statSync(artifact).size} bytes)`);
