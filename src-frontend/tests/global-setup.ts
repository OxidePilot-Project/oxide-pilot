import { spawn, execSync } from 'child_process';
import path from 'path';
import fs from 'fs';
import net from 'net';

const SIDECAR_HOST = process.env.COGNEE_SIDECAR_HOST || '127.0.0.1';
const SIDECAR_PORT = Number(process.env.COGNEE_SIDECAR_PORT || 8765);

function waitForPort(host: string, port: number, timeoutMs = 30000, intervalMs = 500): Promise<void> {
  const start = Date.now();
  return new Promise((resolve, reject) => {
    const attempt = () => {
      const socket = new net.Socket();
      socket.setTimeout(2000);
      socket.once('connect', () => {
        socket.destroy();
        resolve();
      });
      socket.once('timeout', () => {
        socket.destroy();
        retry();
      });
      socket.once('error', () => {
        socket.destroy();
        retry();
      });
      socket.connect(port, host);
    };
    const retry = () => {
      if (Date.now() - start > timeoutMs) {
        reject(new Error(`Timed out waiting for ${host}:${port}`));
      } else {
        setTimeout(attempt, intervalMs);
      }
    };
    attempt();
  });
}

export default async function globalSetup() {
  // Playwright runs from src-frontend as CWD; repo root is one level up
  const repoRoot = path.resolve(process.cwd(), '..');

  // Optional: Start Cognee sidecar via Docker Compose with random host port.
  const dockerMode = (() => {
    const v = (process.env.COGNEE_SIDECAR_DOCKER || '').trim();
    return v === '1' || v.toLowerCase() === 'true' || v.toLowerCase() === 'yes';
  })();

  if (dockerMode) {
    try {
      const composeFile = path.join(repoRoot, 'docker-compose.cognee.yml');
      if (!fs.existsSync(composeFile)) {
        console.warn(`[global-setup] Compose file not found at ${composeFile}. Falling back to venv sidecar.`);
      } else {
        console.log('[global-setup] Starting Cognee sidecar via Docker Compose...');
        // Bring up container (build as needed)
        execSync(`docker compose -f "${composeFile}" up -d --build`, { stdio: 'inherit' });
        // Discover mapped host:port
        const out = execSync(`docker compose -f "${composeFile}" port cognee-sidecar 8765`).toString().trim();
        let host = '127.0.0.1';
        let port = 8765;
        if (out) {
          const parts = out.split(':');
          host = parts.slice(0, -1).join(':').replace(/[\[\]]/g, '') || '127.0.0.1';
          port = Number(parts[parts.length - 1]);
        }
        process.env.COGNEE_SIDECAR_HOST = host;
        process.env.COGNEE_SIDECAR_PORT = String(port);
        await waitForPort(host, port, 30000, 500);
        console.log(`[global-setup] Cognee sidecar (Docker) is ready at ${host}:${port}.`);
        // Marker for teardown
        try { fs.writeFileSync(path.join(repoRoot, 'cognee-sidecar.docker'), '1'); } catch {}
        return;
      }
    } catch (e) {
      console.warn(`[global-setup] Docker mode failed: ${(e as Error).message}. Falling back to venv sidecar.`);
    }
  }

  const scriptPath = path.join(repoRoot, 'scripts', 'setup-cognee-sidecar.ps1');
  const pidFile = path.join(repoRoot, 'cognee-sidecar.pid');

  if (!fs.existsSync(scriptPath)) {
    console.warn(`[global-setup] Sidecar script not found at ${scriptPath}. Skipping sidecar start.`);
    return;
  }

  console.log(`[global-setup] Starting Cognee sidecar on ${SIDECAR_HOST}:${SIDECAR_PORT} ...`);
  const child = spawn('powershell.exe', [
    '-NoProfile',
    '-ExecutionPolicy', 'Bypass',
    '-File', scriptPath,
    '-Run',
    '-Foreground',
    '-ListenHost', SIDECAR_HOST,
    '-Port', String(SIDECAR_PORT),
  ], {
    cwd: repoRoot,
    stdio: 'pipe',
    windowsHide: true,
  });

  // Pipe output for debugging
  child.stdout?.on('data', (d: Buffer) => process.stdout.write(`[sidecar] ${d.toString()}`));
  child.stderr?.on('data', (d: Buffer) => process.stderr.write(`[sidecar] ${d.toString()}`));

  // Persist PID for teardown
  try {
    fs.writeFileSync(pidFile, String(child.pid ?? ''));
  } catch {}

  // Wait for port readiness
  try {
    await waitForPort(SIDECAR_HOST, SIDECAR_PORT, 30000, 500);
    console.log('[global-setup] Cognee sidecar is ready.');
  } catch (err) {
    console.warn(`[global-setup] Sidecar readiness check failed: ${(err as Error).message}`);
    // Do not throw to avoid blocking UI-only tests
  }
}
