/// <reference types="node" />
import { defineConfig, devices } from '@playwright/test';
const PORT = Number(process.env.PLAYWRIGHT_VITE_PORT || 5174);
const PREVIEW = (() => {
  const v = (process.env.PREVIEW || '').trim().toLowerCase();
  return v === '1' || v === 'true' || v === 'yes';
})();

export default defineConfig({
  testDir: './tests',
  timeout: 30_000,
  expect: { timeout: 5_000 },
  fullyParallel: false, // Run tests sequentially to avoid resource conflicts
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1, // Use only 1 worker to avoid multiple browser instances
  reporter: [
    ['list'],
    ['junit', { outputFile: 'test-results/junit/results.xml' }],
    ['json', { outputFile: 'test-results/report.json' }],
    ['html', { outputFolder: 'playwright-report', open: 'never' }],
  ],
  use: {
    baseURL: `http://localhost:${PORT}`,
    headless: true, // Always run headless to avoid opening windows
    trace: 'retain-on-failure', // Only trace on failure to reduce overhead
    screenshot: 'only-on-failure', // Only take screenshots on failure
    video: 'retain-on-failure', // Only record video on failure
    viewport: { width: 1280, height: 800 },
    contextOptions: {
      // Use a unique HAR file per worker process to avoid write contention on Windows
      recordHar: { path: `test-results/network-${process.pid}.har`, content: 'embed' },
    },
  },
  // Launch/stop Cognee sidecar for the test session
  globalSetup: './tests/global-setup.ts',
  globalTeardown: './tests/global-teardown.ts',
  webServer: {
    command: PREVIEW
      ? `vite preview --port ${PORT} --strictPort`
      : `vite dev --port ${PORT} --strictPort`,
    port: PORT,
    // Always start our own dev server to avoid accidentally attaching to some
    // other app already running on the same port.
    reuseExistingServer: false,
    timeout: 120_000,
    // Propagate COVERAGE flag to Vite process so vite-plugin-istanbul instruments code
    env: process.env.COVERAGE ? { COVERAGE: '1', ...process.env } : { ...process.env },
  },
  outputDir: 'test-results',
  projects: [
    // Only test on Chromium to reduce resource usage and avoid multiple windows
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
    // Uncomment these if you need cross-browser testing
    // { name: 'firefox', use: { ...devices['Desktop Firefox'] } },
    // { name: 'webkit', use: { ...devices['Desktop Safari'] } },
  ],
});
