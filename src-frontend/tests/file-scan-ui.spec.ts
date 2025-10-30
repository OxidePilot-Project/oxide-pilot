import { expect, test } from "./fixtures";

// Browser-mode UI test for the Antivirus File Scan card
// Verifies rendering and disabled actions when not running in Tauri

test.describe("Antivirus File Scan UI (browser mode)", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/app?e2e=1");
    await page.waitForLoadState("domcontentloaded");
    await page.waitForSelector(".pattern-dashboard", { timeout: 30000 });
    await expect(page.getByRole("button", { name: "Security" })).toBeVisible({
      timeout: 15000,
    });
  });

  test("renders File Scan card and disables actions", async ({ page }) => {
    // Open Security tab
    await page.getByRole("button", { name: "Security" }).click();

    // Find the File Scan card by heading
    const cardHeading = page.getByRole("heading", {
      name: "Antivirus: File Scan",
    });
    await expect(cardHeading).toBeVisible();
    const card = page.locator(".card").filter({ has: cardHeading });

    // Browse button should be disabled in browser mode (no Tauri)
    const browseBtn = card.getByRole("button", { name: /Browse/ });
    await expect(browseBtn).toBeVisible();
    await expect(browseBtn).toBeDisabled();

    // Scan File button should be disabled in browser mode (no Tauri)
    const scanBtn = card.getByRole("button", { name: "Scan File" });
    await expect(scanBtn).toBeVisible();
    await expect(scanBtn).toBeDisabled();

    // Checkboxes visible
    const vtCheckbox = card.getByRole("checkbox", { name: /Use VirusTotal/i });
    const quarantineCheckbox = card.getByRole("checkbox", {
      name: /Quarantine/i,
    });
    await expect(vtCheckbox).toBeVisible();
    await expect(quarantineCheckbox).toBeVisible();

    // No VT warning note in browser mode (key status unknown)
    await expect(
      card.getByText("VirusTotal key not configured.", { exact: false }),
    ).toHaveCount(0);

    // No visual snapshots in browser-mode test to keep it stable across engines
  });
});
