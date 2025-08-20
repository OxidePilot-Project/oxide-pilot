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
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 2 : undefined,
  reporter: [
    ['list'],
    ['junit', { outputFile: 'test-results/junit/results.xml' }],
    ['json', { outputFile: 'test-results/report.json' }],
    ['html', { outputFolder: 'playwright-report', open: 'never' }],
  ],
  use: {
    baseURL: `http://localhost:${PORT}`,
    headless: true,
    trace: process.env.CI ? 'on' : 'retain-on-failure',
    screenshot: 'on',
    video: 'retain-on-failure',
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
  },
  outputDir: 'test-results',
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
    { name: 'firefox', use: { ...devices['Desktop Firefox'] } },
    { name: 'webkit', use: { ...devices['Desktop Safari'] } },
  ],
});
