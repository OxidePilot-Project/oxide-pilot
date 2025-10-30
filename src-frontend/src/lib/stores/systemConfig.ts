import { writable } from "svelte/store";

// Minimal system config store used by InitialSetup.svelte
export const systemConfig = writable<any>(null);
