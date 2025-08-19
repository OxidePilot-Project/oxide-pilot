import { test, expect } from './fixtures';

// E2E: Validate Local Models panel behavior in web preview (non-Tauri)
// Uses ?e2e=1 to bypass setup so Settings tab is accessible

test.describe('Local Models panel (web preview)', () => {
  test('shows browser-mode warning and hides Tauri-only controls', async ({ page }) => {
    await page.goto('/app?e2e=1');

    await page.getByRole('button', { name: 'Settings' }).click();

    // Assert the warning from LocalModelsPanel.svelte when !isTauri
    await expect(
      page.getByText('Local models are only available in the desktop (Tauri) app.')
    ).toBeVisible();

    // Tauri-only controls should be absent in web preview
    await expect(page.locator('#port')).toHaveCount(0); // server port input
    await expect(page.getByRole('button', { name: /^Start$/ })).toHaveCount(0);
    await expect(page.getByRole('heading', { name: 'Quick Chat (Local)' })).toHaveCount(0);
  });
});
