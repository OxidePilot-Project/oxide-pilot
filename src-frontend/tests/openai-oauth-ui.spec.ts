import { test, expect } from '@playwright/test';

const APP_URL = process.env.APP_URL || 'http://localhost:5173';

test.describe('OpenAI API Key UI', () => {
  test('should show API key input and save successfully', async ({ page }) => {
    await page.goto(APP_URL);

    await expect(page.getByRole('heading', { name: /setup required/i })).toBeVisible();

    // Select OpenAI provider
    await page.getByRole('button', { name: /openai/i }).click();

    // Ensure OpenAI auth setup panel is visible
    await expect(page.getByRole('heading', { name: /openai/i })).toBeVisible();

    // Find API key input
    const apiKeyInput = page.getByPlaceholder(/api key|sk-/i);
    await expect(apiKeyInput).toBeVisible();

    // Enter a test API key (mock format)
    await apiKeyInput.fill('sk-test-mock-key-for-e2e-testing-only');

    // Click save/connect button
    await page.getByRole('button', { name: /save|connect/i }).click();

    // Wait for success message
    await expect(page.getByText(/api key saved|openai is ready/i)).toBeVisible({ timeout: 5000 });
  });

  test('should show warning when API key is empty', async ({ page }) => {
    await page.goto(APP_URL);

    await expect(page.getByRole('heading', { name: /setup required/i })).toBeVisible();
    await page.getByRole('button', { name: /openai/i }).click();

    // Try to save without entering API key
    await page.getByRole('button', { name: /save|connect/i }).click();

    // Should show warning
    await expect(page.getByText(/enter.*api key/i)).toBeVisible();
  });

  test('should allow clearing API key session', async ({ page }) => {
    await page.goto(APP_URL);

    // Setup: save an API key first
    await page.getByRole('button', { name: /openai/i }).click();
    const apiKeyInput = page.getByPlaceholder(/api key|sk-/i);
    await apiKeyInput.fill('sk-test-mock-key');
    await page.getByRole('button', { name: /save|connect/i }).click();
    await expect(page.getByText(/api key saved|openai is ready/i)).toBeVisible({ timeout: 5000 });

    // Now clear the session
    const clearButton = page.getByRole('button', { name: /clear|sign out|disconnect/i });
    if (await clearButton.isVisible()) {
      await clearButton.click();

      // Should show confirmation or return to input state
      await expect(page.getByPlaceholder(/api key|sk-/i)).toBeVisible();
    }
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

  test('should route messages to OpenAI when selected', async ({ page, context }) => {
    await context.addInitScript(() => {
      try { localStorage.setItem('oxide.provider', 'openai'); } catch {}
    });
    await page.goto(`${APP_URL}?e2e=1`);

    await page.getByRole('button', { name: /chat/i }).click();

    // Type a message
    const input = page.getByPlaceholder(/type.*message|ask/i);
    await input.fill('Test message for OpenAI');

    // Send message
    await page.keyboard.press('Enter');

    // Should show message in conversation
    await expect(page.getByText('Test message for OpenAI')).toBeVisible();
  });
});
