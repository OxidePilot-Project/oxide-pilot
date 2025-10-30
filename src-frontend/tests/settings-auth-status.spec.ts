import { expect, test } from "@playwright/test";

const APP_URL = process.env.APP_URL || "http://localhost:5173";

test.describe("Settings: provider statuses and OpenAI clear session (simulated)", () => {
  test("lists all providers with status badges and updates OpenAI status", async ({
    page,
  }) => {
    await page.goto(APP_URL);

    // Go to Settings
    await page.getByRole("button", { name: /settings/i }).click();

    // Headings exist
    await expect(
      page.getByRole("heading", { name: /google gemini/i }),
    ).toBeVisible();
    await expect(page.getByRole("heading", { name: /^qwen$/i })).toBeVisible();
    await expect(
      page.getByRole("heading", { name: /openai \(gpt.*5\)/i }),
    ).toBeVisible();

    // Status badges exist (will show Not connected in web preview)
    await expect(page.locator('[data-testid="status-gemini"]')).toBeVisible();
    await expect(page.locator('[data-testid="status-qwen"]')).toBeVisible();
    await expect(page.locator('[data-testid="status-openai"]')).toBeVisible();

    // Ensure OpenAI card is visible
    const openaiCard = page.locator(".openai-auth-setup");
    await expect(openaiCard).toBeVisible();

    // Simulate success event -> UI should show Connected in the OpenAI card header
    await page.evaluate(() => {
      window.dispatchEvent(
        new CustomEvent("openai_oauth", { detail: { action: "success" } }),
      );
    });
    await expect(openaiCard.getByText(/connected/i)).toBeVisible();

    // Simulate clear session -> UI should show Not connected
    await page.evaluate(() => {
      window.dispatchEvent(
        new CustomEvent("openai_oauth", { detail: { action: "clear" } }),
      );
    });
    await expect(openaiCard.getByText(/not connected/i)).toBeVisible();
  });
});
