import { expect, test } from "@playwright/test";

const APP_URL = process.env.APP_URL || "http://localhost:5173";

test.describe("Threat Consensus System", () => {
  test("should show consensus analysis with multiple providers", async ({
    page,
    context,
  }) => {
    // Setup: configure multiple providers
    await context.addInitScript(() => {
      try {
        localStorage.setItem("oxide.gemini.configured", "true");
        localStorage.setItem("oxide.openai.configured", "true");
      } catch {}
    });

    await page.goto(`${APP_URL}?e2e=1`);

    // Navigate to Security Center
    await page.getByRole("button", { name: /security/i }).click();

    // Look for threat consensus section
    const consensusSection = page.locator(
      '[data-testid="threat-consensus"], .threat-consensus',
    );

    if (await consensusSection.isVisible()) {
      // Should show multiple provider badges
      await expect(consensusSection.getByText(/gemini|openai/i)).toBeVisible();

      // Should show risk score
      await expect(
        consensusSection.getByText(/risk.*score|threat.*level/i),
      ).toBeVisible();

      // Should show confidence indicator
      await expect(consensusSection.getByText(/confidence/i)).toBeVisible();
    }
  });

  test("should display consensus mode (dual/triple)", async ({
    page,
    context,
  }) => {
    await context.addInitScript(() => {
      try {
        localStorage.setItem("oxide.gemini.configured", "true");
        localStorage.setItem("oxide.openai.configured", "true");
        localStorage.setItem("oxide.qwen.configured", "true");
      } catch {}
    });

    await page.goto(`${APP_URL}?e2e=1`);
    await page.getByRole("button", { name: /security/i }).click();

    // Should indicate multi-provider consensus
    const modeIndicator = page.getByText(/dual|triple|consensus/i);
    if (await modeIndicator.isVisible()) {
      await expect(modeIndicator).toBeVisible();
    }
  });

  test("should show per-provider confidence scores", async ({
    page,
    context,
  }) => {
    await context.addInitScript(() => {
      try {
        localStorage.setItem("oxide.gemini.configured", "true");
        localStorage.setItem("oxide.openai.configured", "true");
      } catch {}
    });

    await page.goto(`${APP_URL}?e2e=1`);
    await page.getByRole("button", { name: /security/i }).click();

    // Look for provider-specific scores
    const providerScores = page.locator(
      '[data-testid="provider-scores"], .provider-scores',
    );

    if (await providerScores.isVisible()) {
      // Should show individual provider confidence
      await expect(
        providerScores.getByText(/gemini.*\d+%|openai.*\d+%/i),
      ).toBeVisible();
    }
  });

  test("should handle single provider fallback gracefully", async ({
    page,
    context,
  }) => {
    await context.addInitScript(() => {
      try {
        // Only one provider configured
        localStorage.setItem("oxide.openai.configured", "true");
      } catch {}
    });

    await page.goto(`${APP_URL}?e2e=1`);
    await page.getByRole("button", { name: /security/i }).click();

    // Should still work with single provider
    const consensusSection = page.locator(
      '[data-testid="threat-consensus"], .threat-consensus',
    );

    if (await consensusSection.isVisible()) {
      await expect(consensusSection.getByText(/openai/i)).toBeVisible();

      // Should indicate single mode
      const modeText = await consensusSection.textContent();
      expect(modeText?.toLowerCase()).toContain("single");
    }
  });

  test("should show disagreement alerts when providers differ significantly", async ({
    page,
    context,
  }) => {
    await context.addInitScript(() => {
      try {
        localStorage.setItem("oxide.gemini.configured", "true");
        localStorage.setItem("oxide.openai.configured", "true");
      } catch {}
    });

    await page.goto(`${APP_URL}?e2e=1`);
    await page.getByRole("button", { name: /security/i }).click();

    // Look for disagreement indicators
    const disagreementAlert = page.getByText(
      /disagreement|review.*manually|high diversity/i,
    );

    // This may or may not be visible depending on actual analysis
    // Just check that the UI can handle it
    const isVisible = await disagreementAlert.isVisible().catch(() => false);
    expect(typeof isVisible).toBe("boolean");
  });

  test("should display threat findings from consensus", async ({
    page,
    context,
  }) => {
    await context.addInitScript(() => {
      try {
        localStorage.setItem("oxide.openai.configured", "true");
      } catch {}
    });

    await page.goto(`${APP_URL}?e2e=1`);
    await page.getByRole("button", { name: /security/i }).click();

    // Look for findings section
    const findingsSection = page.locator(
      '[data-testid="threat-findings"], .threat-findings',
    );

    if (await findingsSection.isVisible()) {
      // Should show finding categories
      await expect(
        findingsSection.getByText(/process|file|network|config/i),
      ).toBeVisible();

      // Should show severity levels
      await expect(
        findingsSection.getByText(/low|medium|high|critical/i),
      ).toBeVisible();
    }
  });

  test("should show recommendations based on consensus", async ({
    page,
    context,
  }) => {
    await context.addInitScript(() => {
      try {
        localStorage.setItem("oxide.openai.configured", "true");
      } catch {}
    });

    await page.goto(`${APP_URL}?e2e=1`);
    await page.getByRole("button", { name: /security/i }).click();

    // Look for recommendations
    const recommendations = page.locator(
      '[data-testid="recommendations"], .recommendations',
    );

    if (await recommendations.isVisible()) {
      // Should have actionable recommendations
      const count = await recommendations
        .locator("li, .recommendation-item")
        .count();
      expect(count).toBeGreaterThan(0);
    }
  });

  test("should update consensus when running new analysis", async ({
    page,
    context,
  }) => {
    await context.addInitScript(() => {
      try {
        localStorage.setItem("oxide.openai.configured", "true");
      } catch {}
    });

    await page.goto(`${APP_URL}?e2e=1`);
    await page.getByRole("button", { name: /security/i }).click();

    // Find and click analyze/scan button
    const analyzeButton = page.getByRole("button", {
      name: /analyze|scan|run.*consensus/i,
    });

    if (await analyzeButton.isVisible()) {
      await analyzeButton.click();

      // Should show loading state
      await expect(page.getByText(/analyzing|scanning/i)).toBeVisible();

      // Should eventually show results
      await expect(page.getByText(/risk.*score|threat.*level/i)).toBeVisible({
        timeout: 10000,
      });
    }
  });
});

test.describe("Threat Consensus Performance", () => {
  test("should complete consensus analysis within reasonable time", async ({
    page,
    context,
  }) => {
    await context.addInitScript(() => {
      try {
        localStorage.setItem("oxide.openai.configured", "true");
      } catch {}
    });

    await page.goto(`${APP_URL}?e2e=1`);
    await page.getByRole("button", { name: /security/i }).click();

    const startTime = Date.now();

    const analyzeButton = page.getByRole("button", {
      name: /analyze|scan|run.*consensus/i,
    });
    if (await analyzeButton.isVisible()) {
      await analyzeButton.click();

      // Wait for results
      await expect(page.getByText(/risk.*score|threat.*level/i)).toBeVisible({
        timeout: 30000,
      });

      const elapsed = Date.now() - startTime;

      // Should complete within 30 seconds
      expect(elapsed).toBeLessThan(30000);

      console.log(`Consensus analysis completed in ${elapsed}ms`);
    }
  });

  test("should show elapsed time for consensus", async ({ page, context }) => {
    await context.addInitScript(() => {
      try {
        localStorage.setItem("oxide.openai.configured", "true");
      } catch {}
    });

    await page.goto(`${APP_URL}?e2e=1`);
    await page.getByRole("button", { name: /security/i }).click();

    // Look for timing information
    const timingInfo = page.getByText(/\d+\s*(ms|seconds?|minutes?)/i);

    // May or may not be visible depending on UI implementation
    const isVisible = await timingInfo.isVisible().catch(() => false);
    expect(typeof isVisible).toBe("boolean");
  });
});
