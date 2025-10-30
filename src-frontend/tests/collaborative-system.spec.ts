import { expect, test } from "@playwright/test";

test.describe("Collaborative LLM System", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    // Wait for the app to load
    await page.waitForSelector(".app-container", { timeout: 10000 });
  });

  test("should display collaborative tab in navigation", async ({ page }) => {
    // Check if the collaborative tab exists
    const collaborativeTab = page.locator(
      'button:has-text("🤝 Collaborative")',
    );
    await expect(collaborativeTab).toBeVisible();
  });

  test("should navigate to collaborative analysis page", async ({ page }) => {
    // Click on the collaborative tab
    await page.click('button:has-text("🤝 Collaborative")');

    // Verify we're on the collaborative page
    await expect(
      page.locator('h2:has-text("🤝 Collaborative LLM Analysis")'),
    ).toBeVisible();
    await expect(
      page.locator('p:has-text("Gemini and Qwen working together")'),
    ).toBeVisible();
  });

  test("should display input form for collaborative analysis", async ({
    page,
  }) => {
    // Navigate to collaborative page
    await page.click('button:has-text("🤝 Collaborative")');

    // Check input elements
    await expect(
      page.locator('textarea[placeholder*="Describe the task"]'),
    ).toBeVisible();
    await expect(page.locator("select#taskType")).toBeVisible();
    await expect(
      page.locator('button:has-text("🚀 Run Collaborative Analysis")'),
    ).toBeVisible();
  });

  test("should show task type options", async ({ page }) => {
    // Navigate to collaborative page
    await page.click('button:has-text("🤝 Collaborative")');

    // Check task type options
    const taskTypeSelect = page.locator("select#taskType");
    await expect(taskTypeSelect).toBeVisible();

    // Verify all task types are available
    const options = await taskTypeSelect.locator("option").allTextContents();
    expect(options).toContain("System Analysis");
    expect(options).toContain("Security Assessment");
    expect(options).toContain("Performance Optimization");
    expect(options).toContain("Troubleshooting");
    expect(options).toContain("General Query");
  });

  test("should validate input before analysis", async ({ page }) => {
    // Navigate to collaborative page
    await page.click('button:has-text("🤝 Collaborative")');

    // Try to run analysis without input
    const analyzeButton = page.locator(
      'button:has-text("🚀 Run Collaborative Analysis")',
    );
    await expect(analyzeButton).toBeDisabled();

    // Add some input
    await page.fill(
      'textarea[placeholder*="Describe the task"]',
      "Test analysis request",
    );

    // Button should now be enabled
    await expect(analyzeButton).toBeEnabled();
  });

  test("should show web preview for collaborative analysis", async ({
    page,
  }) => {
    // Navigate to collaborative page
    await page.click('button:has-text("🤝 Collaborative")');

    // Fill in the form
    await page.fill(
      'textarea[placeholder*="Describe the task"]',
      "Test system analysis",
    );
    await page.selectOption("select#taskType", "system_analysis");

    // Click analyze button
    await page.click('button:has-text("🚀 Run Collaborative Analysis")');

    // Wait for results (web preview mode)
    await page.waitForSelector(".analysis-results", { timeout: 10000 });

    // Verify results structure
    await expect(
      page.locator('h3:has-text("📊 Analysis Results")'),
    ).toBeVisible();
    await expect(
      page.locator('.metric-label:has-text("Confidence:")'),
    ).toBeVisible();
    await expect(
      page.locator('.metric-label:has-text("Consensus:")'),
    ).toBeVisible();

    // Check for primary response
    await expect(
      page.locator('h4:has-text("🎯 Primary Response")'),
    ).toBeVisible();
    await expect(page.locator(".provider-badge.gemini")).toBeVisible();

    // Check for specialized analysis
    await expect(
      page.locator('h4:has-text("🔍 Specialized Analysis")'),
    ).toBeVisible();

    // Check for execution plan
    await expect(
      page.locator('h4:has-text("📋 Execution Plan")'),
    ).toBeVisible();
  });

  test("should display metrics with proper colors", async ({ page }) => {
    // Navigate to collaborative page
    await page.click('button:has-text("🤝 Collaborative")');

    // Fill in the form and run analysis
    await page.fill(
      'textarea[placeholder*="Describe the task"]',
      "Test analysis",
    );
    await page.click('button:has-text("🚀 Run Collaborative Analysis")');

    // Wait for results
    await page.waitForSelector(".analysis-results", { timeout: 10000 });

    // Check metrics are displayed
    const confidenceMetric = page.locator(".metric-value").first();
    const consensusMetric = page.locator(".metric-value").nth(1);

    await expect(confidenceMetric).toBeVisible();
    await expect(consensusMetric).toBeVisible();

    // Verify metrics show percentage values
    const confidenceText = await confidenceMetric.textContent();
    const consensusText = await consensusMetric.textContent();

    expect(confidenceText).toMatch(/\d+\.\d+%/);
    expect(consensusText).toMatch(/\d+\.\d+%/);
  });

  test("should show provider badges correctly", async ({ page }) => {
    // Navigate to collaborative page
    await page.click('button:has-text("🤝 Collaborative")');

    // Fill in the form and run analysis
    await page.fill(
      'textarea[placeholder*="Describe the task"]',
      "Test analysis",
    );
    await page.click('button:has-text("🚀 Run Collaborative Analysis")');

    // Wait for results
    await page.waitForSelector(".analysis-results", { timeout: 10000 });

    // Check for Gemini and Qwen badges
    await expect(page.locator(".provider-badge.gemini")).toBeVisible();
    await expect(page.locator(".provider-badge.qwen")).toBeVisible();

    // Verify badge text
    const geminiBadges = page.locator(".provider-badge.gemini");
    const qwenBadges = page.locator(".provider-badge.qwen");

    await expect(geminiBadges.first()).toHaveText("GEMINI");
    await expect(qwenBadges.first()).toHaveText("QWEN");
  });

  test("should display execution plan steps", async ({ page }) => {
    // Navigate to collaborative page
    await page.click('button:has-text("🤝 Collaborative")');

    // Fill in the form and run analysis
    await page.fill(
      'textarea[placeholder*="Describe the task"]',
      "Test analysis",
    );
    await page.click('button:has-text("🚀 Run Collaborative Analysis")');

    // Wait for results
    await page.waitForSelector(".analysis-results", { timeout: 10000 });

    // Check execution plan section
    await expect(
      page.locator('h4:has-text("📋 Execution Plan")'),
    ).toBeVisible();

    // Check for execution steps
    const executionSteps = page.locator(".execution-step");
    const stepCount = await executionSteps.count();
    expect(stepCount).toBeGreaterThan(0);

    // Verify step numbers
    const stepNumbers = page.locator(".step-number");
    await expect(stepNumbers.first()).toHaveText("1");
  });

  test("should show timestamp in results footer", async ({ page }) => {
    // Navigate to collaborative page
    await page.click('button:has-text("🤝 Collaborative")');

    // Fill in the form and run analysis
    await page.fill(
      'textarea[placeholder*="Describe the task"]',
      "Test analysis",
    );
    await page.click('button:has-text("🚀 Run Collaborative Analysis")');

    // Wait for results
    await page.waitForSelector(".analysis-results", { timeout: 10000 });

    // Check footer with timestamp
    const footer = page.locator(".results-footer small");
    await expect(footer).toBeVisible();

    const footerText = await footer.textContent();
    expect(footerText).toContain("Analysis completed at");
  });

  test("should handle different task types", async ({ page }) => {
    // Navigate to collaborative page
    await page.click('button:has-text("🤝 Collaborative")');

    const taskTypes = [
      "system_analysis",
      "security_assessment",
      "performance_optimization",
      "troubleshooting",
      "user_query",
    ];

    for (const taskType of taskTypes) {
      // Select task type
      await page.selectOption("select#taskType", taskType);

      // Fill in the form
      await page.fill(
        'textarea[placeholder*="Describe the task"]',
        `Test ${taskType} analysis`,
      );

      // Run analysis
      await page.click('button:has-text("🚀 Run Collaborative Analysis")');

      // Wait for results
      await page.waitForSelector(".analysis-results", { timeout: 10000 });

      // Verify results are displayed
      await expect(
        page.locator('h3:has-text("📊 Analysis Results")'),
      ).toBeVisible();

      // Clear results for next iteration
      await page.fill('textarea[placeholder*="Describe the task"]', "");
    }
  });

  test("should be responsive on different screen sizes", async ({ page }) => {
    // Test on mobile size
    await page.setViewportSize({ width: 375, height: 667 });
    await page.click('button:has-text("🤝 Collaborative")');

    // Check that elements are still visible and properly arranged
    await expect(
      page.locator('h2:has-text("🤝 Collaborative LLM Analysis")'),
    ).toBeVisible();
    await expect(
      page.locator('textarea[placeholder*="Describe the task"]'),
    ).toBeVisible();

    // Test on tablet size
    await page.setViewportSize({ width: 768, height: 1024 });
    await page.reload();
    await page.click('button:has-text("🤝 Collaborative")');

    // Check responsive layout
    await expect(page.locator(".collaborative-analysis")).toBeVisible();

    // Test on desktop size (1280x800)
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.reload();
    await page.click('button:has-text("🤝 Collaborative")');

    // Verify optimal layout for target size
    await expect(page.locator(".collaborative-analysis")).toBeVisible();
  });
});
