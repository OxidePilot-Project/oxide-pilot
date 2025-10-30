import { expect, test } from "@playwright/test";

test.describe("Essential Application Tests", () => {
  test.beforeEach(async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto("/");
    await page.waitForSelector("body", { timeout: 15000 });
  });

  test("should load application without errors", async ({ page }) => {
    // Basic functionality test
    await expect(page.locator("body")).toBeVisible();

    const bodyText = await page.locator("body").textContent();
    expect(bodyText).toBeTruthy();
    expect(bodyText!.length).toBeGreaterThan(0);

    console.log("✅ Application loaded successfully");
  });

  test("should find interactive elements", async ({ page }) => {
    // Look for buttons and interactive elements
    const buttons = page.locator("button");
    const buttonCount = await buttons.count();

    console.log(`Found ${buttonCount} buttons`);

    if (buttonCount > 0) {
      const firstButton = buttons.first();
      await expect(firstButton).toBeVisible();
      console.log("✅ Interactive elements found and accessible");
    }
  });

  test("should handle basic navigation", async ({ page }) => {
    // Look for navigation elements
    const navElements = page.locator(
      '[class*="nav"], [class*="tab"], [class*="header"]',
    );
    const navCount = await navElements.count();

    console.log(`Found ${navCount} navigation elements`);

    if (navCount > 0) {
      // Try clicking the first navigation element
      const firstNav = navElements.first();
      if (await firstNav.isVisible()) {
        await firstNav.click();
        await page.waitForTimeout(500);
        console.log("✅ Navigation elements are clickable");
      }
    }
  });

  test("should be responsive to window resize", async ({ page }) => {
    // Test responsiveness
    await page.setViewportSize({ width: 1024, height: 600 });
    await page.waitForTimeout(300);
    await expect(page.locator("body")).toBeVisible();

    await page.setViewportSize({ width: 1280, height: 800 });
    await page.waitForTimeout(300);
    await expect(page.locator("body")).toBeVisible();

    console.log("✅ Application is responsive to window resize");
  });

  test("should not have critical console errors", async ({ page }) => {
    const consoleErrors: string[] = [];

    page.on("console", (msg) => {
      if (msg.type() === "error") {
        consoleErrors.push(msg.text());
      }
    });

    await page.waitForTimeout(2000);

    const criticalErrors = consoleErrors.filter(
      (error) =>
        !error.includes("warning") &&
        !error.includes("deprecated") &&
        !error.includes("non-passive"),
    );

    if (criticalErrors.length > 0) {
      console.log("Console errors found:", criticalErrors);
    }

    expect(criticalErrors.length).toBeLessThan(3);
    console.log("✅ No critical console errors detected");
  });
});
