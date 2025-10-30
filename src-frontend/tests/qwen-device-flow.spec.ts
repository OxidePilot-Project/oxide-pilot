import { expect, test } from "./fixtures";

// Browser-mode tests validating Qwen Device Flow UI via CustomEvent("qwen_device_flow")

test.describe("Qwen Device Flow UI (browser mode)", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForLoadState("domcontentloaded");
    // Expect setup screen (no ?e2e=1 bypass). Heading includes an emoji.
    await expect(
      page.getByRole("heading", { name: /Setup Required/i }),
    ).toBeVisible();

    // Switch to Qwen provider
    await page.getByRole("button", { name: /Qwen \(Device Code\)/i }).click();

    // Ensure Qwen component is present: header title and Start Device Flow button
    await expect(
      page.getByRole("heading", { name: /Qwen Device Authorization/i }),
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: /Start device authorization/i }),
    ).toBeVisible();
  });

  test("success path transitions to dashboard", async ({ page }) => {
    // Start: simulate device flow issuance
    await page.evaluate(() => {
      const detail = {
        action: "start",
        device_code: "MOCK-DEVICE-CODE",
        user_code: "ABCD-1234",
        verification_uri: "https://verify.example/qwen",
        expires_in: 15,
        interval: 2,
      };
      window.dispatchEvent(new CustomEvent("qwen_device_flow", { detail }));
    });

    // UI should reflect codes and link
    await expect(page.getByText("User Code")).toBeVisible();
    await expect(
      page.getByRole("link", { name: /https:\/\/verify\.example\/qwen/i }),
    ).toBeVisible();

    // QR canvas should be present
    await expect(
      page.locator('canvas[aria-label="Qwen verification QR"]'),
    ).toBeVisible();

    // Expires In label visible (value counts down; avoid asserting exact seconds)
    await expect(page.getByText("Expires In")).toBeVisible();

    // Now simulate success
    await page.evaluate(() => {
      const detail = { action: "success" };
      window.dispatchEvent(new CustomEvent("qwen_device_flow", { detail }));
    });

    // Setup heading should disappear and dashboard nav should appear
    await expect(
      page.getByRole("heading", { name: /Setup Required/i }),
    ).not.toBeVisible({ timeout: 7000 });
    await expect(page.getByRole("button", { name: /Dashboard/i })).toBeVisible({
      timeout: 7000,
    });
  });

  test("error path shows error and stays on setup", async ({ page }) => {
    // Start then error
    await page.evaluate(() => {
      window.dispatchEvent(
        new CustomEvent("qwen_device_flow", {
          detail: {
            action: "start",
            user_code: "ZZZZ-9999",
            verification_uri: "https://verify.example/qwen",
          },
        }),
      );
      window.dispatchEvent(
        new CustomEvent("qwen_device_flow", {
          detail: { action: "error", message: "mock failure" },
        }),
      );
    });

    // Error message visible
    await expect(page.locator("body")).toContainText(/mock failure/i);
    // Still on setup (no Dashboard tab yet)
    await expect(
      page.getByRole("heading", { name: /Setup Required/i }),
    ).toBeVisible();
  });

  test("Clear Session shows browser-mode info message", async ({ page }) => {
    // Click Clear Session in Qwen header
    const clearBtn = page.getByRole("button", { name: /Clear Session/i });
    await expect(clearBtn).toBeVisible();
    await clearBtn.click();

    // Info message from browser mode should appear in the page
    await expect(page.locator("body")).toContainText(/desktop-only operation/i);
  });
});
