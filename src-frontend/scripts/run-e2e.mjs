#!/usr/bin/env node
import { spawn } from 'node:child_process';
import net from 'node:net';

function getArg(flag) {
  const i = process.argv.indexOf(flag);
  return i >= 0 ? process.argv[i + 1] : undefined;
}

function hasFlag(flag) {
  return process.argv.includes(flag);
}

async function getFreePort() {
  return await new Promise((resolve, reject) => {
    const srv = net.createServer();
    srv.on('error', reject);
    srv.listen(0, '127.0.0.1', () => {
      const addr = srv.address();
      if (typeof addr === 'object' && addr) {
        const port = addr.port;
        srv.close(() => resolve(port));
      } else {
        srv.close(() => reject(new Error('Failed to get free port')));
      }
    });
  });
}

(async () => {
  const vitePort = await getFreePort();
  const useDocker = hasFlag('--docker');
  const usePreview = hasFlag('--preview');
  const passArgs = process.argv.slice(2).filter(a => a !== '--docker');
  const passArgsNoPreview = passArgs.filter(a => a !== '--preview');

  const env = { ...process.env, PLAYWRIGHT_VITE_PORT: String(vitePort) };
  if (useDocker) env.COGNEE_SIDECAR_DOCKER = '1';
  if (usePreview) env.PREVIEW = '1';

  console.log(`[runner] Using ports -> Vite=${env.PLAYWRIGHT_VITE_PORT}${useDocker ? ' (Docker sidecar)' : ''}${usePreview ? ' [preview mode]' : ''}`);

  const cmd = process.platform === 'win32' ? 'npx.cmd' : 'npx';
  const runTests = () => {
    const child = spawn(cmd, ['playwright', 'test', ...passArgsNoPreview], {
      stdio: 'inherit',
      shell: true,
      env,
    });
    child.on('exit', code => process.exit(code ?? 1));
  };

  if (usePreview) {
    // Build first for production-like preview
    const npmCmd = process.platform === 'win32' ? 'npm.cmd' : 'npm';
    const build = spawn(npmCmd, ['run', 'build'], { stdio: 'inherit', shell: true, env });
    build.on('exit', code => {
      if (code !== 0) process.exit(code ?? 1);
      runTests();
    });
  } else {
    runTests();
  }
})();
