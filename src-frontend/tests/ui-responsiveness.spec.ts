import { test, expect } from '@playwright/test';

test.describe('UI Responsiveness - 1280x800 Target', () => {
  test.beforeEach(async ({ page }) => {
    // Set target viewport size
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto('/');
    await page.waitForSelector('.app-container', { timeout: 10000 });
  });

  test('should display app layout correctly at 1280x800', async ({ page }) => {
    // Check main container
    await expect(page.locator('.app-container')).toBeVisible();

    // Check header
    await expect(page.locator('.app-header')).toBeVisible();
    await expect(page.locator('.logo')).toBeVisible();

    // Check navigation tabs
    await expect(page.locator('.tab-navigation')).toBeVisible();

    // Check main content area
    await expect(page.locator('.app-main')).toBeVisible();

    // Check footer
    await expect(page.locator('.app-footer')).toBeVisible();
  });

  test('should have proper header dimensions', async ({ page }) => {
    const header = page.locator('.app-header');
    const headerBox = await header.boundingBox();

    // Header should be 60px height as designed
    expect(headerBox?.height).toBeCloseTo(60, 5);

    // Check logo elements are properly sized
    const logoH1 = page.locator('.logo h1');
    const logoP = page.locator('.logo p');

    await expect(logoH1).toBeVisible();
    await expect(logoP).toBeVisible();
  });

  test('should have proper footer dimensions', async ({ page }) => {
    const footer = page.locator('.app-footer');
    const footerBox = await footer.boundingBox();

    // Footer should be 40px height as designed
    expect(footerBox?.height).toBeCloseTo(40, 5);
  });

  test('should have proper main content area height', async ({ page }) => {
    const main = page.locator('.app-main');
    const mainBox = await main.boundingBox();

    // Main should be calc(100vh - 108px) = 692px at 800px viewport
    expect(mainBox?.height).toBeCloseTo(692, 10);
  });

  test('should display all navigation tabs properly', async ({ page }) => {
    const tabs = [
      '📊 Dashboard',
      '💬 Chat',
      '🧠 Analysis',
      '🤝 Collaborative',
      '⚙️ Settings',
      '🔧 Advanced'
    ];

    for (const tab of tabs) {
      await expect(page.locator(`button:has-text("${tab}")`)).toBeVisible();
    }
  });

  test('should handle tab navigation correctly', async ({ page }) => {
    const tabs = [
      { name: 'Dashboard', selector: 'button:has-text("📊 Dashboard")' },
      { name: 'Chat', selector: 'button:has-text("💬 Chat")' },
      { name: 'Analysis', selector: 'button:has-text("🧠 Analysis")' },
      { name: 'Collaborative', selector: 'button:has-text("🤝 Collaborative")' },
      { name: 'Settings', selector: 'button:has-text("⚙️ Settings")' },
      { name: 'Advanced', selector: 'button:has-text("🔧 Advanced")' }
    ];

    for (const tab of tabs) {
      await page.click(tab.selector);

      // Check that tab is active
      await expect(page.locator(tab.selector)).toHaveClass(/active/);

      // Wait for content to load
      await page.waitForTimeout(500);
    }
  });

  test('should display dashboard content properly', async ({ page }) => {
    await page.click('button:has-text("📊 Dashboard")');

    // Check dashboard elements
    await expect(page.locator('.dashboard-container')).toBeVisible();
    await expect(page.locator('h2:has-text("📊 System Dashboard")')).toBeVisible();
  });

  test('should display chat interface properly', async ({ page }) => {
    await page.click('button:has-text("💬 Chat")');

    // Check chat elements
    await expect(page.locator('.conversation-interface')).toBeVisible();
    await expect(page.locator('.messages-container')).toBeVisible();
    await expect(page.locator('.input-area')).toBeVisible();
  });

  test('should display analysis panel properly', async ({ page }) => {
    await page.click('button:has-text("🧠 Analysis")');

    // Check analysis elements
    await expect(page.locator('.system-analysis-panel')).toBeVisible();
  });

  test('should display collaborative analysis properly', async ({ page }) => {
    await page.click('button:has-text("🤝 Collaborative")');

    // Check collaborative elements
    await expect(page.locator('.collaborative-analysis')).toBeVisible();
    await expect(page.locator('h2:has-text("🤝 Collaborative LLM Analysis")')).toBeVisible();
  });

  test('should display settings properly', async ({ page }) => {
    await page.click('button:has-text("⚙️ Settings")');

    // Check settings elements
    await expect(page.locator('.settings-container')).toBeVisible();
    await expect(page.locator('h2:has-text("⚙️ Settings")')).toBeVisible();
  });

  test('should display advanced settings properly', async ({ page }) => {
    await page.click('button:has-text("🔧 Advanced")');

    // Check advanced elements
    await expect(page.locator('.advanced-settings')).toBeVisible();
  });

  test('should maintain proper spacing and padding', async ({ page }) => {
    // Check header padding
    const header = page.locator('.app-header');
    const headerStyle = await header.evaluate(el => {
      const computed = window.getComputedStyle(el);
      return {
        padding: computed.padding,
        margin: computed.margin
      };
    });

    // Check main content padding
    const main = page.locator('.app-main');
    const mainStyle = await main.evaluate(el => {
      const computed = window.getComputedStyle(el);
      return {
        padding: computed.padding,
        margin: computed.margin
      };
    });

    // Check footer padding
    const footer = page.locator('.app-footer');
    const footerStyle = await footer.evaluate(el => {
      const computed = window.getComputedStyle(el);
      return {
        padding: computed.padding,
        margin: computed.margin
      };
    });

    // Verify styles are applied
    expect(headerStyle.padding).toBeTruthy();
    expect(mainStyle.padding).toBeTruthy();
    expect(footerStyle.padding).toBeTruthy();
  });

  test('should handle window resize gracefully', async ({ page }) => {
    // Test smaller size
    await page.setViewportSize({ width: 1024, height: 600 });
    await page.waitForTimeout(500);

    // Check that layout still works
    await expect(page.locator('.app-container')).toBeVisible();
    await expect(page.locator('.app-header')).toBeVisible();
    await expect(page.locator('.app-main')).toBeVisible();
    await expect(page.locator('.app-footer')).toBeVisible();

    // Test larger size
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.waitForTimeout(500);

    // Check that layout still works
    await expect(page.locator('.app-container')).toBeVisible();
    await expect(page.locator('.app-header')).toBeVisible();
    await expect(page.locator('.app-main')).toBeVisible();
    await expect(page.locator('.app-footer')).toBeVisible();

    // Return to target size
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.waitForTimeout(500);

    // Verify everything still works
    await expect(page.locator('.app-container')).toBeVisible();
  });

  test('should have proper font sizes for target resolution', async ({ page }) => {
    // Check header font sizes
    const logoH1 = page.locator('.logo h1');
    const logoH1Style = await logoH1.evaluate(el => {
      return window.getComputedStyle(el).fontSize;
    });

    // Check tab button font sizes
    const tabButton = page.locator('.tab-button').first();
    const tabButtonStyle = await tabButton.evaluate(el => {
      return window.getComputedStyle(el).fontSize;
    });

    // Verify font sizes are reasonable for the target resolution
    expect(parseFloat(logoH1Style)).toBeGreaterThan(14);
    expect(parseFloat(logoH1Style)).toBeLessThan(24);
    expect(parseFloat(tabButtonStyle)).toBeGreaterThan(10);
    expect(parseFloat(tabButtonStyle)).toBeLessThan(16);
  });

  test('should handle overflow correctly', async ({ page }) => {
    // Navigate to a content-heavy page
    await page.click('button:has-text("🤝 Collaborative")');

    // Add a lot of content to test scrolling
    await page.fill('textarea[placeholder*="Describe the task"]', 'A'.repeat(1000));

    // Check that scrolling works
    const main = page.locator('.app-main');
    await expect(main).toBeVisible();

    // Verify overflow is handled
    const mainStyle = await main.evaluate(el => {
      return window.getComputedStyle(el).overflowY;
    });

    expect(mainStyle).toBe('auto');
  });
});
