import { expect, test } from "./fixtures";

// E2E: Validate provider-aware chat routing in web preview mode
// Uses ?e2e=1 to bypass auth and rely on localStorage provider selection

test.describe("Provider-aware chat routing", () => {
  test("Local provider shows badge, warning, and local-only message", async ({
    page,
  }) => {
    await page.addInitScript(() => {
      localStorage.setItem("oxide.provider", "local");
    });
    await page.goto("/app?e2e=1");

    await page.getByRole("button", { name: "Chat" }).click();

    await expect(page.locator(".provider-badge")).toHaveText("Local", {
      timeout: 15000,
    });
    await expect(page.locator(".provider-warning")).toBeVisible();

    const input = page.getByPlaceholder(
      "Message Oxide Pilot... (Press Enter to send, Shift+Enter for new line)",
    );
    await input.fill("hello from local");
    await page.keyboard.press("Enter");

    await expect(
      page.locator(".message.assistant .message-text").last(),
    ).toContainText(
      "Local models are only available in the desktop (Tauri) app.",
      { timeout: 15000 },
    );
  });

  test("Gemini provider falls back to web preview response", async ({
    page,
  }) => {
    await page.addInitScript(() => {
      localStorage.setItem("oxide.provider", "gemini");
    });
    await page.goto("/app?e2e=1");

    await page.getByRole("button", { name: "Chat" }).click();

    await expect(page.locator(".provider-badge")).toHaveText("Gemini", {
      timeout: 15000,
    });

    const input = page.getByPlaceholder(
      "Message Oxide Pilot... (Press Enter to send, Shift+Enter for new line)",
    );
    await input.fill("ping");
    await page.keyboard.press("Enter");

    await expect(
      page.locator(".message.assistant .message-text").last(),
    ).toContainText("Web preview: I received your message:", {
      timeout: 15000,
    });
  });

  test("Qwen provider falls back to web preview response", async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem("oxide.provider", "qwen");
    });
    await page.goto("/app?e2e=1");

    await page.getByRole("button", { name: "Chat" }).click();

    await expect(page.locator(".provider-badge")).toHaveText("Qwen", {
      timeout: 15000,
    });

    const input = page.getByPlaceholder(
      "Message Oxide Pilot... (Press Enter to send, Shift+Enter for new line)",
    );
    await input.fill("hola qwen");
    await page.keyboard.press("Enter");

    await expect(
      page.locator(".message.assistant .message-text").last(),
    ).toContainText("Web preview: I received your message:", {
      timeout: 15000,
    });
  });
});
