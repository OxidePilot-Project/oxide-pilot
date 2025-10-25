import { test, expect } from './fixtures';

// These tests run in browser (non-Tauri) mode against the Vite dev server.
// Backend actions are disabled when not in Tauri, but UI should still render.

test.describe('Security & Performance panels (browser mode)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/app?e2e=1');
    await page.waitForLoadState('domcontentloaded');
    // Wait for dashboard container and tabs as readiness signal
    await page.waitForSelector('.pattern-dashboard', { timeout: 30000 });
    await expect(page.getByRole('button', { name: 'Security' })).toBeVisible({ timeout: 30000 });
    // Dashboard is default after auth-flow is bypassed in browser mode
  });

  test('Security tab renders and actions are disabled', async ({ page }) => {
    // Open inner dashboard tab: Security
    await page.getByRole('button', { name: 'Security' }).click();

    // Panel header (use regex to avoid emoji exact-match issues)
    await expect(page.getByRole('heading', { name: /Security Overview/i })).toBeVisible({ timeout: 20000 });
    await expect(page.getByRole('heading', { name: /Security Center/i })).toBeVisible({ timeout: 20000 });

    // Create Session button disabled in browser mode (not Tauri)
    const createBtn = page.getByRole('button', { name: 'Create Session' });
    await expect(createBtn).toBeVisible();
    await expect(createBtn).toBeDisabled();

    // Validate and Check Permission buttons also disabled
    await expect(page.getByRole('button', { name: 'Validate Session' })).toBeDisabled();
    await expect(page.getByRole('button', { name: 'Check Permission' })).toBeDisabled();

    // Events section visible (may be empty)
    await expect(page.getByRole('heading', { name: 'Security Events' })).toBeVisible({ timeout: 20000 });

    // Attach a full-page screenshot for debugging/inspection
    await test.info().attach('security-fullpage', {
      body: await page.screenshot({ fullPage: true }),
      contentType: 'image/png',
    });

    // Visual regression snapshot (opt-in via PW_VRT=1)
    if (process.env.PW_VRT === '1') {
      await expect(page).toHaveScreenshot('security-tab.png', { fullPage: true, maxDiffPixelRatio: 0.05 });
    }
  });

  test('Performance tab renders and monitoring/clear actions are disabled', async ({ page }) => {
    await page.getByRole('button', { name: 'Performance' }).click();

    await expect(page.getByRole('heading', { name: /Performance Analytics/i })).toBeVisible({ timeout: 20000 });
    await expect(page.getByRole('heading', { name: /Performance Alerts & Errors/i })).toBeVisible({ timeout: 20000 });

    // Clear Alerts disabled in browser mode
    const clearBtn = page.getByRole('button', { name: 'Clear Alerts' });
    await expect(clearBtn).toBeVisible();
    await expect(clearBtn).toBeDisabled();

    // Monitoring checkbox disabled in browser mode
    const checkbox = page.getByRole('checkbox');
    await expect(checkbox).toBeDisabled();

    // Recent Errors and Operation Profiles cards render
    await expect(page.getByRole('heading', { name: 'Recent Errors' })).toBeVisible({ timeout: 20000 });
    await expect(page.getByRole('heading', { name: 'Operation Profiles' })).toBeVisible({ timeout: 20000 });

    // Attach a full-page screenshot for debugging/inspection
    await test.info().attach('performance-fullpage', {
      body: await page.screenshot({ fullPage: true }),
      contentType: 'image/png',
    });

    // Visual regression snapshot (opt-in via PW_VRT=1)
    if (process.env.PW_VRT === '1') {
      await expect(page).toHaveScreenshot('performance-tab.png', { fullPage: true, maxDiffPixelRatio: 0.05 });
    }
  });
});
