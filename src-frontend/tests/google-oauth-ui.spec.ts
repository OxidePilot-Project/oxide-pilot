import { expect, test } from "./fixtures";

// Browser-mode tests validating UI reaction to google_auth_complete event and clear-session behavior

test.describe("Google OAuth UI (browser mode)", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForLoadState("domcontentloaded");
    // Expect setup screen (no ?e2e=1 bypass). Heading includes an emoji.
    await expect(
      page.getByRole("heading", { name: /Setup Required/i }),
    ).toBeVisible();
    // Ensure Google provider panel is present via main heading
    await expect(
      page.getByRole("heading", { name: /Google Gemini API Configuration/i }),
    ).toBeVisible();
  });

  test("dispatching success event transitions to dashboard", async ({
    page,
  }) => {
    // Switch to OAuth method to ensure OAuth UI is present
    await page.getByRole("button", { name: /OAuth 2\.0/i }).click();
    // Ensure Google auth component rendered (OAuth section present)
    await expect(
      page.getByRole("button", { name: /Authenticate with Google/i }),
    ).toBeVisible();

    // Dispatch browser CustomEvent simulating Tauri event
    await page.evaluate(() => {
      const detail = {
        status: "success",
        provider: "google",
        timestamp: Math.floor(Date.now() / 1000),
      };
      window.dispatchEvent(new CustomEvent("google_auth_complete", { detail }));
    });

    // AppLayout should handle authComplete -> dashboard: first, setup heading disappears
    await expect(
      page.getByRole("heading", { name: /Setup Required/i }),
    ).not.toBeVisible({ timeout: 7000 });
    // Then header nav appears
    await expect(page.getByRole("button", { name: /Dashboard/i })).toBeVisible({
      timeout: 7000,
    });

    // Header nav tabs should appear (labels have emojis)
    await expect(page.getByRole("button", { name: /Chat/i })).toBeVisible();
    await expect(page.getByRole("button", { name: /Analysis/i })).toBeVisible();
    await expect(page.getByRole("button", { name: /Settings/i })).toBeVisible();
  });

  test("dispatching error event shows error status and stays on setup", async ({
    page,
  }) => {
    // Switch to OAuth method to ensure status area is within view
    await page.getByRole("button", { name: /OAuth 2\.0/i }).click();
    // Send error event
    await page.evaluate(() => {
      const detail = {
        status: "error",
        provider: "google",
        message: "mock failure",
      };
      window.dispatchEvent(new CustomEvent("google_auth_complete", { detail }));
    });

    // Error status message should appear in the page
    await expect(page.locator("body")).toContainText(
      /Google authentication failed: mock failure/i,
    );

    // Still on setup (no Dashboard tab yet)
    await expect(
      page.getByRole("heading", { name: /Setup Required/i }),
    ).toBeVisible();
  });

  test("Clear Session shows browser-mode info message", async ({ page }) => {
    const clearBtn = page.getByRole("button", {
      name: /Clear saved Google session/i,
    });
    await expect(clearBtn).toBeVisible();
    await clearBtn.click();

    // Info message from browser mode should appear in the page
    await expect(page.locator("body")).toContainText(/desktop-only operation/i);
  });
});
