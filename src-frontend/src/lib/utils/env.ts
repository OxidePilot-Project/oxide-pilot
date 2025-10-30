// Shared environment utilities for the Svelte frontend
// Detect Tauri runtime using several heuristics
export const isTauri: boolean =
  typeof window !== "undefined" &&
  ("__TAURI__" in (window as any) ||
    "__TAURI_IPC__" in (window as any) ||
    (window as any).__TAURI_METADATA__ !== undefined ||
    window.location.protocol === "tauri:");
