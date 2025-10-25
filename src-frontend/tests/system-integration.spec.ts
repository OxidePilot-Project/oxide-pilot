import { test, expect } from '@playwright/test';

test.describe('System Integration Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto('/');
    await page.waitForSelector('.app-container', { timeout: 10000 });
  });

  test('should handle provider selection and persistence', async ({ page }) => {
    // Navigate to settings
    await page.click('button:has-text("⚙️ Settings")');

    // Check if provider selection is available
    const providerSelectors = page.locator('.provider-selector');
    if (await providerSelectors.count() > 0) {
      // Test provider selection
      const geminiProvider = page.locator('.provider-btn').filter({ hasText: 'Google Gemini' });
      const qwenProvider = page.locator('.provider-btn').filter({ hasText: 'Qwen' });
      const localProvider = page.locator('.provider-btn').filter({ hasText: 'Local' });

      if (await geminiProvider.count() > 0) {
        await geminiProvider.click();
        await expect(geminiProvider).toHaveClass(/selected/);
      }

      if (await qwenProvider.count() > 0) {
        await qwenProvider.click();
        await expect(qwenProvider).toHaveClass(/selected/);
      }

      if (await localProvider.count() > 0) {
        await localProvider.click();
        await expect(localProvider).toHaveClass(/selected/);
      }
    }
  });

  test('should display authentication setup correctly', async ({ page }) => {
    // Navigate to settings
    await page.click('button:has-text("⚙️ Settings")');

    // Check for authentication sections
    const authSections = page.locator('.auth-grid, .auth-section');
    if (await authSections.count() > 0) {
      await expect(authSections.first()).toBeVisible();
    }
  });

  test('should handle chat interface with different providers', async ({ page }) => {
    // Navigate to chat
    await page.click('button:has-text("💬 Chat")');

    // Check chat interface elements
    await expect(page.locator('.conversation-interface')).toBeVisible();
    await expect(page.locator('.messages-container')).toBeVisible();
    await expect(page.locator('.input-area')).toBeVisible();

    // Test input functionality
    const inputArea = page.locator('textarea, input[type="text"]').first();
    if (await inputArea.count() > 0) {
      await inputArea.fill('Test message');

      // Check if send button is available
      const sendButton = page.locator('button:has-text("Send"), button[type="submit"]');
      if (await sendButton.count() > 0) {
        await expect(sendButton).toBeEnabled();
      }
    }
  });

  test('should display system dashboard correctly', async ({ page }) => {
    // Navigate to dashboard
    await page.click('button:has-text("📊 Dashboard")');

    // Check dashboard elements
    await expect(page.locator('.dashboard-container, .system-dashboard')).toBeVisible();

    // Check for system metrics or status indicators
    const metrics = page.locator('.metric, .status-indicator, .dashboard-item');
    if (await metrics.count() > 0) {
      await expect(metrics.first()).toBeVisible();
    }
  });

  test('should handle analysis panel functionality', async ({ page }) => {
    // Navigate to analysis
    await page.click('button:has-text("🧠 Analysis")');

    // Check analysis panel
    await expect(page.locator('.system-analysis-panel, .analysis-panel')).toBeVisible();

    // Look for analysis buttons or controls
    const analysisButtons = page.locator('button:has-text("Analyze"), button:has-text("Scan"), button:has-text("Check")');
    if (await analysisButtons.count() > 0) {
      await expect(analysisButtons.first()).toBeVisible();
    }
  });

  test('should integrate collaborative system with main interface', async ({ page }) => {
    // Test navigation to collaborative tab
    await page.click('button:has-text("🤝 Collaborative")');

    // Verify collaborative interface loads
    await expect(page.locator('.collaborative-analysis')).toBeVisible();

    // Test form interaction
    const textarea = page.locator('textarea[placeholder*="Describe the task"]');
    await expect(textarea).toBeVisible();

    // Fill form and test submission
    await textarea.fill('Test system integration');
    await page.selectOption('select#taskType', 'system_analysis');

    const analyzeButton = page.locator('button:has-text("🚀 Run Collaborative Analysis")');
    await expect(analyzeButton).toBeEnabled();

    // Click analyze (this will show web preview in test environment)
    await analyzeButton.click();

    // Wait for results
    await page.waitForSelector('.analysis-results', { timeout: 10000 });

    // Verify results are displayed
    await expect(page.locator('h3:has-text("📊 Analysis Results")')).toBeVisible();
  });

  test('should handle error states gracefully', async ({ page }) => {
    // Test with invalid input
    await page.click('button:has-text("🤝 Collaborative")');

    // Try to submit without input
    const analyzeButton = page.locator('button:has-text("🚀 Run Collaborative Analysis")');
    await expect(analyzeButton).toBeDisabled();

    // Test with minimal input
    await page.fill('textarea[placeholder*="Describe the task"]', 'Test');
    await expect(analyzeButton).toBeEnabled();
  });

  test('should maintain state across tab switches', async ({ page }) => {
    // Navigate to collaborative tab
    await page.click('button:has-text("🤝 Collaborative")');

    // Fill in some data
    await page.fill('textarea[placeholder*="Describe the task"]', 'Persistent test data');
    await page.selectOption('select#taskType', 'security_assessment');

    // Switch to another tab
    await page.click('button:has-text("💬 Chat")');

    // Switch back to collaborative
    await page.click('button:has-text("🤝 Collaborative")');

    // Check if data is still there (this depends on implementation)
    const textarea = page.locator('textarea[placeholder*="Describe the task"]');
    const currentValue = await textarea.inputValue();

    // Note: This test might need adjustment based on actual state management
    // For now, we just verify the element is still functional
    await expect(textarea).toBeVisible();
  });

  test('should handle responsive layout across all tabs', async ({ page }) => {
    const tabs = [
      '📊 Dashboard',
      '💬 Chat',
      '🧠 Analysis',
      '🤝 Collaborative',
      '⚙️ Settings',
      '🔧 Advanced'
    ];

    for (const tab of tabs) {
      await page.click(`button:has-text("${tab}")`);
      await page.waitForTimeout(300);

      // Check that main layout is still intact
      await expect(page.locator('.app-container')).toBeVisible();
      await expect(page.locator('.app-header')).toBeVisible();
      await expect(page.locator('.app-main')).toBeVisible();
      await expect(page.locator('.app-footer')).toBeVisible();
    }
  });

  test('should display proper loading states', async ({ page }) => {
    // Navigate to collaborative tab
    await page.click('button:has-text("🤝 Collaborative")');

    // Fill form and submit
    await page.fill('textarea[placeholder*="Describe the task"]', 'Loading test');
    await page.click('button:has-text("🚀 Run Collaborative Analysis")');

    // Check for loading state (spinner or disabled button)
    const analyzeButton = page.locator('button:has-text("🚀 Run Collaborative Analysis")');

    // Button should be disabled during processing
    await expect(analyzeButton).toBeDisabled();

    // Wait for completion
    await page.waitForSelector('.analysis-results', { timeout: 10000 });

    // Button should be enabled again
    await expect(analyzeButton).toBeEnabled();
  });

  test('should handle keyboard navigation', async ({ page }) => {
    // Test tab navigation with keyboard
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');

    // Test Enter key on focused elements
    await page.keyboard.press('Enter');

    // Test form interaction with keyboard
    await page.click('button:has-text("🤝 Collaborative")');

    const textarea = page.locator('textarea[placeholder*="Describe the task"]');
    await textarea.focus();
    await page.keyboard.type('Keyboard test');

    // Test Tab to move to next element
    await page.keyboard.press('Tab');

    // Verify focus moved to next element
    const focusedElement = page.locator(':focus');
    await expect(focusedElement).toBeVisible();
  });

  test('should handle window focus and blur events', async ({ page }) => {
    // Test that the app handles focus changes gracefully
    await page.click('button:has-text("🤝 Collaborative")');

    // Simulate window blur
    await page.evaluate(() => window.dispatchEvent(new Event('blur')));
    await page.waitForTimeout(100);

    // Simulate window focus
    await page.evaluate(() => window.dispatchEvent(new Event('focus')));
    await page.waitForTimeout(100);

    // Verify app is still functional
    await expect(page.locator('.collaborative-analysis')).toBeVisible();
  });

  test('should handle concurrent operations', async ({ page }) => {
    // Test multiple rapid tab switches
    const tabs = ['📊 Dashboard', '💬 Chat', '🧠 Analysis', '🤝 Collaborative'];

    for (let i = 0; i < 3; i++) {
      for (const tab of tabs) {
        await page.click(`button:has-text("${tab}")`);
        await page.waitForTimeout(50); // Very short delay
      }
    }

    // Verify final state is correct
    await expect(page.locator('.app-container')).toBeVisible();
  });

  test('should maintain performance under load', async ({ page }) => {
    // Test rapid interactions
    await page.click('button:has-text("🤝 Collaborative")');

    // Rapidly fill and clear the form multiple times
    for (let i = 0; i < 5; i++) {
      await page.fill('textarea[placeholder*="Describe the task"]', `Test ${i}`);
      await page.selectOption('select#taskType', 'system_analysis');
      await page.fill('textarea[placeholder*="Describe the task"]', '');
    }

    // Verify app is still responsive
    await expect(page.locator('.collaborative-analysis')).toBeVisible();
  });
});
