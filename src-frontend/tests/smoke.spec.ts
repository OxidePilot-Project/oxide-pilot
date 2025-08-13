import { test, expect } from '@playwright/test';

// Basic smoke test that works whether auth is configured or not
// Assumes dev server runs on baseURL (configure in playwright.config.ts)

test('app loads and shows core UI', async ({ page }) => {
  await page.goto('/app');

  // Header title (unique H1 heading)
  const title = page.getByRole('heading', { level: 1, name: /Oxide\s+Pilot/i });
  await expect(title).toBeVisible();

  // Status heading always shows either "Setup Required" or "System Ready"
  const statusHeading = page.getByRole('heading', { level: 2, name: /Setup Required|System Ready/ });
  await expect(statusHeading).toBeVisible();

  // If in setup, provider buttons should be visible
  const isSetup = (await page.getByRole('heading', { level: 2, name: /Setup Required/ }).count()) > 0;
  if (isSetup) {
    await expect(page.getByRole('button', { name: /Google Gemini/i })).toBeVisible();
    await expect(page.getByRole('button', { name: /Qwen/i })).toBeVisible();
  } else {
    // Otherwise, app should show main navigation
    await expect(page.getByRole('button', { name: /Dashboard/i })).toBeVisible();
    await expect(page.getByRole('button', { name: /Chat/i })).toBeVisible();
  }
});
