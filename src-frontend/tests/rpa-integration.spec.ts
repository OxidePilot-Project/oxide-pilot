import { expect, test } from "@playwright/test";

test.describe("RPA Integration Tests", () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the app with E2E bypass
    await page.goto("/?e2e=1");

    // Wait for the app to load
    await expect(page.locator("h1")).toContainText("Oxide Pilot");
  });

  test("should display RPA tab in navigation", async ({ page }) => {
    // Check that RPA tab is visible
    const rpaTab = page.locator('button:has-text("🤖 RPA")');
    await expect(rpaTab).toBeVisible();
  });

  test("should navigate to RPA dashboard", async ({ page }) => {
    // Click on RPA tab
    await page.click('button:has-text("🤖 RPA")');

    // Check that RPA dashboard is displayed
    await expect(
      page.locator('h2:has-text("🤖 RPA Control Center")'),
    ).toBeVisible();

    // Check that welcome card is displayed (RPA not initialized)
    await expect(
      page.locator('h3:has-text("Welcome to RPA Control Center")'),
    ).toBeVisible();

    // Check that initialize button is present
    await expect(page.locator('button:has-text("Get Started")')).toBeVisible();
  });

  test("should show RPA dashboard tabs", async ({ page }) => {
    // Navigate to RPA dashboard
    await page.click('button:has-text("🤖 RPA")');

    // Check that all tabs are present
    await expect(page.locator('button:has-text("📊 Overview")')).toBeVisible();
    await expect(page.locator('button:has-text("📋 Audit Log")')).toBeVisible();
    await expect(page.locator('button:has-text("↩️ Rollback")')).toBeVisible();
    await expect(
      page.locator('button:has-text("🔐 Permissions")'),
    ).toBeVisible();

    // Check that audit, rollback, and permissions tabs are disabled initially
    await expect(
      page.locator('button:has-text("📋 Audit Log")'),
    ).toBeDisabled();
    await expect(page.locator('button:has-text("↩️ Rollback")')).toBeDisabled();
    await expect(
      page.locator('button:has-text("🔐 Permissions")'),
    ).toBeDisabled();
  });

  test("should display status indicator", async ({ page }) => {
    // Navigate to RPA dashboard
    await page.click('button:has-text("🤖 RPA")');

    // Check that status indicator shows RPA as inactive
    await expect(
      page.locator('.status-indicator:has-text("RPA Inactive")'),
    ).toBeVisible();

    // Check that status dot is offline
    await expect(page.locator(".status-dot.offline")).toBeVisible();
  });

  test("should show permissions tab placeholder", async ({ page }) => {
    // Navigate to RPA dashboard
    await page.click('button:has-text("🤖 RPA")');

    // Try to click permissions tab (should be disabled, but let's test the content)
    // We'll simulate what would happen if it was enabled
    await page.evaluate(() => {
      const permissionsTab = document.querySelector(
        'button:has-text("🔐 Permissions")',
      ) as HTMLButtonElement;
      if (permissionsTab) {
        permissionsTab.disabled = false;
      }
    });

    await page.click('button:has-text("🔐 Permissions")');

    // Check that coming soon message is displayed
    await expect(
      page.locator('h3:has-text("Permission Management")'),
    ).toBeVisible();
    await expect(
      page.locator(
        "text=Advanced permission management interface is coming soon",
      ),
    ).toBeVisible();
  });

  test("should have responsive design", async ({ page }) => {
    // Navigate to RPA dashboard
    await page.click('button:has-text("🤖 RPA")');

    // Test mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });

    // Check that dashboard is still visible and functional
    await expect(
      page.locator('h2:has-text("🤖 RPA Control Center")'),
    ).toBeVisible();
    await expect(page.locator('button:has-text("Get Started")')).toBeVisible();

    // Check that tabs are scrollable on mobile
    const tabNavigation = page.locator(".tab-navigation");
    await expect(tabNavigation).toBeVisible();

    // Reset to desktop viewport
    await page.setViewportSize({ width: 1280, height: 720 });
  });

  test("should maintain tab state when switching", async ({ page }) => {
    // Navigate to RPA dashboard
    await page.click('button:has-text("🤖 RPA")');

    // Should start on Overview tab
    await expect(
      page.locator('button:has-text("📊 Overview").active'),
    ).toBeVisible();

    // Switch to another main tab and back
    await page.click('button:has-text("💬 Chat")');
    await expect(
      page.locator('h2:has-text("🤖 RPA Control Center")'),
    ).not.toBeVisible();

    // Switch back to RPA
    await page.click('button:has-text("🤖 RPA")');
    await expect(
      page.locator('h2:has-text("🤖 RPA Control Center")'),
    ).toBeVisible();

    // Should still be on Overview tab
    await expect(
      page.locator('button:has-text("📊 Overview").active'),
    ).toBeVisible();
  });

  test("should show quick actions in overview", async ({ page }) => {
    // Navigate to RPA dashboard
    await page.click('button:has-text("🤖 RPA")');

    // Check that quick actions section is present
    await expect(page.locator('h3:has-text("Quick Actions")')).toBeVisible();

    // Check that action buttons are present
    await expect(
      page.locator('.action-btn:has-text("View Audit Log")'),
    ).toBeVisible();
    await expect(
      page.locator('.action-btn:has-text("Rollback Actions")'),
    ).toBeVisible();
    await expect(
      page.locator('.action-btn:has-text("Manage Permissions")'),
    ).toBeVisible();

    // Check that rollback action is disabled (no reversible actions)
    await expect(
      page.locator('.action-btn:has-text("Rollback Actions")'),
    ).toBeDisabled();
  });
});

test.describe("RPA Components Integration", () => {
  test("should load audit panel component", async ({ page }) => {
    await page.goto("/?e2e=1");
    await page.click('button:has-text("🤖 RPA")');

    // Enable audit tab for testing
    await page.evaluate(() => {
      const auditTab = document.querySelector(
        'button:has-text("📋 Audit Log")',
      ) as HTMLButtonElement;
      if (auditTab) {
        auditTab.disabled = false;
      }
    });

    await page.click('button:has-text("📋 Audit Log")');

    // Check that audit panel loads
    await expect(page.locator('h2:has-text("RPA Audit Log")')).toBeVisible();
    await expect(page.locator('button:has-text("Refresh")')).toBeVisible();
  });

  test("should load rollback panel component", async ({ page }) => {
    await page.goto("/?e2e=1");
    await page.click('button:has-text("🤖 RPA")');

    // Enable rollback tab for testing
    await page.evaluate(() => {
      const rollbackTab = document.querySelector(
        'button:has-text("↩️ Rollback")',
      ) as HTMLButtonElement;
      if (rollbackTab) {
        rollbackTab.disabled = false;
      }
    });

    await page.click('button:has-text("↩️ Rollback")');

    // Check that rollback panel loads
    await expect(page.locator('h2:has-text("Rollback History")')).toBeVisible();
    await expect(
      page.locator('button:has-text("Rollback Last")'),
    ).toBeVisible();
  });
});
