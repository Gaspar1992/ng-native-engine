import { spawn } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { watchBundle } from './build-bundle.mjs';

const __dirname = dirname(fileURLToPath(import.meta.url));
const rootDir = resolve(__dirname, '../..');

const watcher = watchBundle();

console.log('[NgNative Dev Server] Launching native Rust engine with live Hot Reload...');
const cargoProc = spawn('cargo', ['run'], {
  cwd: rootDir,
  stdio: 'inherit',
  env: process.env,
});

function shutdown(code = 0) {
  watcher.close();
  if (!cargoProc.killed) {
    cargoProc.kill('SIGTERM');
  }
  process.exit(code);
}

cargoProc.on('exit', (code) => {
  watcher.close();
  process.exit(code ?? 0);
});

process.on('SIGINT', () => shutdown(0));
process.on('SIGTERM', () => shutdown(0));
