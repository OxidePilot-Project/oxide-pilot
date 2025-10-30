import { expect, test } from "./fixtures";

// Tests run in browser mode (no Tauri). We verify rendering, disabled controls,
// and simulate folder scan events via CustomEvent('folder_scan').

test.describe("SecurityCenter - Folder Scan UI", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/app?e2e=1");
    await page.waitForLoadState("domcontentloaded");
    await page.waitForSelector(".pattern-dashboard", { timeout: 30000 });
    // Switch to Security tab in PatternDashboard
    const securityTabBtn = page.getByRole("button", { name: /^Security$/ });
    await expect(securityTabBtn).toBeVisible({ timeout: 15000 });
    await securityTabBtn.click();
  });

  test("renders Folder Scan card and controls", async ({ page }) => {
    const cardHeader = page.getByRole("heading", {
      name: /Antivirus: Folder Scan/i,
    });
    await expect(cardHeader).toBeVisible();
    const card = page.locator(".card").filter({ has: cardHeader });

    const folderInput = card.locator('input[placeholder="Select a folder..."]');
    const browseBtn = card.getByRole("button", { name: /Browse/i });
    const startBtn = card.getByRole("button", { name: /Start Scan/i });
    const cancelBtn = card.getByRole("button", { name: /Cancel/i });

    await expect(folderInput).toBeVisible();
    await expect(browseBtn).toBeVisible();
    await expect(startBtn).toBeVisible();
    await expect(cancelBtn).toBeVisible();

    // Not running under Tauri => browse is disabled, start disabled (no folder), cancel disabled
    await expect(browseBtn).toBeDisabled();
    await expect(startBtn).toBeDisabled();
    await expect(cancelBtn).toBeDisabled();
  });

  test("simulates folder scan progress and completion in browser mode", async ({
    page,
  }) => {
    // Ensure card is visible
    const cardHeader = page.getByRole("heading", {
      name: /Antivirus: Folder Scan/i,
    });
    await expect(cardHeader).toBeVisible();
    const card = page.locator(".card").filter({ has: cardHeader });
    // Inject a simulated scan id and folder
    const scanId = "scan-test-1";
    await page.evaluate(
      ({ scanId }) => {
        window.dispatchEvent(
          new CustomEvent("folder_scan", {
            detail: { action: "set", scan_id: scanId, folder: "C:/mock" },
          }),
        );
        window.dispatchEvent(
          new CustomEvent("folder_scan", {
            detail: {
              action: "event",
              type: "started",
              payload: { scan_id: scanId },
            },
          }),
        );
      },
      { scanId },
    );

    // Assert initial progress appears
    await expect(card.getByText(/Scan ID:/i)).toContainText(scanId);

    // Send progress updates
    await page.evaluate(
      ({ scanId }) => {
        window.dispatchEvent(
          new CustomEvent("folder_scan", {
            detail: {
              action: "event",
              type: "progress",
              payload: {
                scan_id: scanId,
                discovered: 10,
                scanned: 3,
                total: 20,
                current_file: "C:/mock/file3.txt",
              },
            },
          }),
        );
      },
      { scanId },
    );
    await expect(card.getByText(/Discovered:\s*10/i)).toBeVisible();
    await expect(card.getByText(/Scanned:\s*3\s*\/\s*20/i)).toBeVisible();
    await expect(card.locator(".progress code").last()).toContainText(
      "file3.txt",
    );

    // Complete
    await page.evaluate(
      ({ scanId }) => {
        window.dispatchEvent(
          new CustomEvent("folder_scan", {
            detail: {
              action: "event",
              type: "completed",
              payload: {
                scan_id: scanId,
                scanned: 20,
                total: 20,
                malicious: 2,
                errors: 1,
                duration_ms: 1234,
              },
            },
          }),
        );
      },
      { scanId },
    );
    await expect(card.getByText(/Completed in\s*1234\s*ms/i)).toBeVisible();
    await expect(card.getByText(/Malicious:\s*2/i)).toBeVisible();
    await expect(card.getByText(/Errors:\s*1/i)).toBeVisible();
  });

  test("simulates cancellation flow in browser mode", async ({ page }) => {
    const cardHeader = page.getByRole("heading", {
      name: /Antivirus: Folder Scan/i,
    });
    await expect(cardHeader).toBeVisible();
    const card = page.locator(".card").filter({ has: cardHeader });
    const scanId = "scan-cancel-1";
    await page.evaluate(
      ({ scanId }) => {
        window.dispatchEvent(
          new CustomEvent("folder_scan", {
            detail: { action: "set", scan_id: scanId, folder: "C:/mock" },
          }),
        );
        window.dispatchEvent(
          new CustomEvent("folder_scan", {
            detail: {
              action: "event",
              type: "started",
              payload: { scan_id: scanId },
            },
          }),
        );
        window.dispatchEvent(
          new CustomEvent("folder_scan", {
            detail: {
              action: "event",
              type: "progress",
              payload: {
                scan_id: scanId,
                discovered: 5,
                scanned: 2,
                total: 100,
              },
            },
          }),
        );
        window.dispatchEvent(
          new CustomEvent("folder_scan", {
            detail: {
              action: "event",
              type: "cancelled",
              payload: {
                scan_id: scanId,
                scanned: 2,
                total: 100,
                malicious: 0,
                errors: 0,
                duration_ms: 222,
              },
            },
          }),
        );
      },
      { scanId },
    );

    await expect(card.getByText(/Scan cancelled\./i)).toBeVisible();
    await expect(card.getByText(/Scanned:\s*2\s*\/\s*100/i)).toBeVisible();
  });
});
