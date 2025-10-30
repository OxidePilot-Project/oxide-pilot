import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";
import istanbul from "vite-plugin-istanbul";

const istanbulPlugin = istanbul({
  include: ["**/src/**"],
  exclude: ["tests/**", "node_modules/**"],
  extension: [".js", ".mjs", ".ts", ".svelte"],
  requireEnv: false,
  forceBuildInstrument: true,
}) as unknown as { enforce?: string };
(istanbulPlugin as any).enforce = "post";

export default defineConfig({
  plugins: [
    sveltekit(),
    // Run Istanbul after all transforms so output JS is instrumented
    istanbulPlugin as any,
  ],
  build: {
    target: "esnext",
    minify: "esbuild",
    cssMinify: true,
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ["svelte"],
        },
      },
    },
  },
  optimizeDeps: {
    include: ["@tauri-apps/api"],
  },
});
