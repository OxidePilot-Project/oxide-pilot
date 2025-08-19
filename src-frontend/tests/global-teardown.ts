import path from 'path';
import fs from 'fs';
import { fileURLToPath } from 'url';
import { execFileSync } from 'node:child_process';

export default async function globalTeardown() {
  try {
    // Support both CJS (__dirname) and ESM (import.meta.url)
    const baseDir = ((): string => {
      try {
        // @ts-ignore
        if (typeof __dirname !== 'undefined') {
          // @ts-ignore
          return __dirname as string;
        }
      } catch {}
      const __filename = fileURLToPath(import.meta.url);
      return path.dirname(__filename);
    })();

    const repoRoot = path.resolve(baseDir, '..', '..');
    const pidFile = path.join(repoRoot, 'cognee-sidecar.pid');
    const dockerMarker = path.join(repoRoot, 'cognee-sidecar.docker');
    const composeFile = path.join(repoRoot, 'docker-compose.cognee.yml');

    let pid: number | null = null;
    try {
      if (fs.existsSync(pidFile)) {
        const content = fs.readFileSync(pidFile, 'utf-8').trim();
        pid = content ? Number(content) : null;
      }
    } catch {}

    if (pid && Number.isFinite(pid)) {
      try {
        // Politely kill the process tree on Windows
        console.log(`[global-teardown] Stopping sidecar PID=${pid} ...`);
        execFileSync('taskkill', ['/PID', String(pid), '/T', '/F'], { stdio: 'ignore' });
      } catch (err) {
        console.warn(`[global-teardown] Failed to stop sidecar: ${(err as Error).message}`);
      }
    }

    try { fs.unlinkSync(pidFile); } catch {}

    // If Docker marker exists, bring down compose stack
    try {
      if (fs.existsSync(dockerMarker) && fs.existsSync(composeFile)) {
        console.log('[global-teardown] Stopping Docker Compose stack for Cognee sidecar...');
        try {
          execFileSync(`docker compose -f "${composeFile}" down -v`, { stdio: 'inherit' });
        } catch (err) {
          console.warn(`[global-teardown] Docker compose down failed: ${(err as Error).message}`);
        }
      }
    } catch {}
    try { fs.unlinkSync(dockerMarker); } catch {}
  } catch (e) {
    console.warn(`[global-teardown] Ignoring teardown error: ${(e as Error)?.message || e}`);
  }
}
