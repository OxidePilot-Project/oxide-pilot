// Centralized Tauri invoke utility with SSR-safe lazy import
// Usage: import { tauriInvoke } from "$lib/utils/tauri";

import { isTauri } from "$lib/utils/env";

export type InvokeFn = <T = any>(
  cmd: string,
  args?: Record<string, unknown>,
) => Promise<T>;

let cachedInvoke: InvokeFn | null = null;

async function getInvoke(): Promise<InvokeFn> {
  if (!isTauri) {
    throw new Error(
      "Not running in Tauri context. This action requires the desktop app runtime.",
    );
  }
  if (!cachedInvoke) {
    // Lazy import to avoid SSR issues
    const mod = await import("@tauri-apps/api/tauri");
    cachedInvoke = mod.invoke as InvokeFn;
  }
  return cachedInvoke;
}

export async function tauriInvoke<T = any>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    const invoke = await getInvoke();
    return await invoke<T>(cmd, args);
  } catch (err) {
    // Attach command context for better debugging
    const message = err instanceof Error ? err.message : String(err);
    throw new Error(`tauriInvoke(${cmd}) failed: ${message}`);
  }
}
