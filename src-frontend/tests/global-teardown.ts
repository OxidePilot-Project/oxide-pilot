import { execFileSync } from "node:child_process";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

export default async function globalTeardown() {
  try {
    // Support both CJS (__dirname) and ESM (import.meta.url)
    const baseDir = ((): string => {
      try {
        // @ts-ignore
        if (typeof __dirname !== "undefined") {
          // @ts-ignore
          return __dirname as string;
        }
      } catch {}
      const __filename = fileURLToPath(import.meta.url);
      return path.dirname(__filename);
    })();

    const repoRoot = path.resolve(baseDir, "..", "..");
    const pidFile = path.join(repoRoot, "cognee-sidecar.pid");
    const dockerMarker = path.join(repoRoot, "cognee-sidecar.docker");
    const composeFile = path.join(repoRoot, "docker-compose.cognee.yml");

    let pid: number | null = null;
    try {
      if (fs.existsSync(pidFile)) {
        const content = fs.readFileSync(pidFile, "utf-8").trim();
        pid = content ? Number(content) : null;
      }
    } catch {}

    if (pid && Number.isFinite(pid)) {
      try {
        // Check if PID exists before attempting to kill (locale-agnostic match by PID string)
        const out = execFileSync("tasklist", ["/FI", `PID eq ${pid}`], {
          encoding: "utf-8",
        });
        const exists = out && out.includes(String(pid));
        if (exists) {
          console.log(`[global-teardown] Stopping sidecar PID=${pid} ...`);
          execFileSync("taskkill", ["/PID", String(pid), "/T", "/F"], {
            stdio: "ignore",
          });
        } else {
          // Sidecar already stopped; keep logs clean
          console.log(
            `[global-teardown] Sidecar PID=${pid} not found; nothing to stop.`,
          );
        }
      } catch {
        // Keep teardown quiet on any errors
        console.log(
          `[global-teardown] Teardown completed (no-op or already stopped).`,
        );
      }
    }

    try {
      fs.unlinkSync(pidFile);
    } catch {}

    // If Docker marker exists, bring down compose stack
    try {
      if (fs.existsSync(dockerMarker) && fs.existsSync(composeFile)) {
        console.log(
          "[global-teardown] Stopping Docker Compose stack for Cognee sidecar...",
        );
        try {
          execFileSync(`docker compose -f "${composeFile}" down -v`, {
            stdio: "inherit",
          });
        } catch (err) {
          console.warn(
            `[global-teardown] Docker compose down failed: ${(err as Error).message}`,
          );
        }
      }
    } catch {}
    try {
      fs.unlinkSync(dockerMarker);
    } catch {}
  } catch (e) {
    console.warn(
      `[global-teardown] Ignoring teardown error: ${(e as Error)?.message || e}`,
    );
  }
}
