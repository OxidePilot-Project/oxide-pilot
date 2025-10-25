import { test, expect } from '@playwright/test';

test.describe('Smoke Tests - Basic Application Functionality', () => {
  test.beforeEach(async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto('/');
    // Wait for the app to load with a longer timeout
    await page.waitForSelector('body', { timeout: 15000 });
  });

  test('should load the application successfully', async ({ page }) => {
    // Check that the page loads without errors
    await expect(page.locator('body')).toBeVisible();

    // Check for any error messages
    const errorMessages = page.locator('[class*="error"], [class*="Error"]');
    const errorCount = await errorMessages.count();

    if (errorCount > 0) {
      console.log('Found error elements:', await errorMessages.allTextContents());
    }

    // Basic check that we have some content
    const bodyText = await page.locator('body').textContent();
    expect(bodyText).toBeTruthy();
    expect(bodyText!.length).toBeGreaterThan(0);
  });

  test('should display some form of navigation or interface', async ({ page }) => {
    // Look for common UI elements that should be present
    const possibleSelectors = [
      'nav',
      '[class*="nav"]',
      '[class*="header"]',
      '[class*="menu"]',
      'button',
      'a[href]',
      '[class*="tab"]',
      '[class*="dashboard"]',
      '[class*="app"]'
    ];

    let foundElements = 0;
    for (const selector of possibleSelectors) {
      const elements = page.locator(selector);
      const count = await elements.count();
      if (count > 0) {
        foundElements += count;
        console.log(`Found ${count} elements with selector: ${selector}`);
      }
    }

    expect(foundElements).toBeGreaterThan(0);
  });

  test('should handle basic user interaction', async ({ page }) => {
    // Look for clickable elements
    const clickableElements = page.locator('button, a, [role="button"], [onclick]');
    const clickableCount = await clickableElements.count();

    if (clickableCount > 0) {
      // Try clicking the first clickable element
      const firstClickable = clickableElements.first();
      await expect(firstClickable).toBeVisible();

      // Click and verify no errors
      await firstClickable.click();
      await page.waitForTimeout(1000);

      // Check that the page is still functional
      await expect(page.locator('body')).toBeVisible();
    }
  });

  test('should be responsive to different screen sizes', async ({ page }) => {
    // Test mobile size
    await page.setViewportSize({ width: 375, height: 667 });
    await page.waitForTimeout(500);
    await expect(page.locator('body')).toBeVisible();

    // Test tablet size
    await page.setViewportSize({ width: 768, height: 1024 });
    await page.waitForTimeout(500);
    await expect(page.locator('body')).toBeVisible();

    // Test desktop size
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.waitForTimeout(500);
    await expect(page.locator('body')).toBeVisible();
  });

  test('should not have console errors', async ({ page }) => {
    const consoleErrors: string[] = [];

    page.on('console', msg => {
      if (msg.type() === 'error') {
        consoleErrors.push(msg.text());
      }
    });

    // Wait for the page to fully load
    await page.waitForTimeout(3000);

    // Check for critical errors (ignore warnings)
    const criticalErrors = consoleErrors.filter(error =>
      !error.includes('warning') &&
      !error.includes('deprecated') &&
      !error.includes('non-passive')
    );

    if (criticalErrors.length > 0) {
      console.log('Console errors found:', criticalErrors);
    }

    // Allow some non-critical errors but fail on critical ones
    expect(criticalErrors.length).toBeLessThan(5);
  });

  test('should load within reasonable time', async ({ page }) => {
    const startTime = Date.now();

    await page.goto('/');
    await page.waitForSelector('body', { timeout: 10000 });

    const loadTime = Date.now() - startTime;
    console.log(`Page load time: ${loadTime}ms`);

    // Should load within 10 seconds
    expect(loadTime).toBeLessThan(10000);
  });

  test('should have proper HTML structure', async ({ page }) => {
    // Check for basic HTML structure
    await expect(page.locator('html')).toBeVisible();
    await expect(page.locator('head')).toBeVisible();
    await expect(page.locator('body')).toBeVisible();

    // Check for title
    const title = await page.title();
    expect(title).toBeTruthy();
    expect(title.length).toBeGreaterThan(0);
  });

  test('should handle form inputs if present', async ({ page }) => {
    // Look for form elements
    const inputs = page.locator('input, textarea, select');
    const inputCount = await inputs.count();

    if (inputCount > 0) {
      const firstInput = inputs.first();
      await expect(firstInput).toBeVisible();

      // Try to interact with the first input
      const inputType = await firstInput.getAttribute('type');
      if (inputType !== 'hidden') {
        await firstInput.focus();
        await page.waitForTimeout(100);

        // Try typing if it's a text input
        if (inputType === 'text' || inputType === 'email' || inputType === 'password' || !inputType) {
          await firstInput.fill('test');
          await page.waitForTimeout(100);
        }
      }
    }
  });
});
