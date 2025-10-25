import { test, expect } from '@playwright/test';

const APP_URL = process.env.APP_URL || 'http://localhost:5173';

test.describe('OpenAI OAuth UI (browser-mode simulation)', () => {
  test('should complete OpenAI OAuth via simulated event and land on dashboard', async ({ page }) => {
    await page.goto(APP_URL);

    await expect(page.getByRole('heading', { name: /setup required/i })).toBeVisible();

    // Select OpenAI provider
    await page.getByRole('button', { name: /openai \(gpt.*5\)/i }).click();

    // Ensure OpenAI auth setup panel is visible
    await expect(page.getByRole('heading', { name: /openai \(gpt.*5\)/i })).toBeVisible();

    // Dispatch simulated success event
    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('openai_oauth', { detail: { action: 'success' } }));
    });

    // Wait for setup to disappear and dashboard to load
    await expect(page.getByRole('heading', { name: /setup required/i })).toHaveCount(0);
    await expect(page.locator('.pattern-dashboard')).toBeVisible();
  });

  test('should show error message on simulated error', async ({ page }) => {
    await page.goto(APP_URL);

    await expect(page.getByRole('heading', { name: /setup required/i })).toBeVisible();
    await page.getByRole('button', { name: /openai \(gpt.*5\)/i }).click();

    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('openai_oauth', { detail: { action: 'error', message: 'Denied' } }));
    });

    await expect(page.getByText(/oauth2 authentication failed/i)).toBeVisible();
  });
});

test.describe('OpenAI provider routing (web preview mode)', () => {
  test('should show OpenAI provider badge in Chat when provider is openai', async ({ page, context }) => {
    // Force provider and bypass setup
    await context.addInitScript(() => {
      try { localStorage.setItem('oxide.provider', 'openai'); } catch {}
    });
    await page.goto(`${APP_URL}?e2e=1`);

    // Go to Chat tab
    await page.getByRole('button', { name: /chat/i }).click();

    // Expect provider badge
    await expect(page.locator('.provider-badge', { hasText: 'OpenAI' })).toBeVisible();
  });
});
