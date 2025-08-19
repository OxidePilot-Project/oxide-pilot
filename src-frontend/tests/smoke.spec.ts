import { test, expect } from './fixtures';

// Basic smoke test for dashboard in browser mode (E2E bypass)

test('app loads and shows core UI', async ({ page }) => {
  await page.goto('/app?e2e=1');
  await page.waitForLoadState('domcontentloaded');
  // Attach initial state
  await test.info().attach('initial-dom.html', {
    body: Buffer.from(await page.content(), 'utf-8'),
    contentType: 'text/html',
  });
  await test.info().attach('initial.png', {
    body: await page.screenshot({ fullPage: true }),
    contentType: 'image/png',
  });

  try {
    await page.waitForSelector('.pattern-dashboard', { timeout: 30000 });
  } catch (e) {
    // On failure, capture more context then rethrow
    const href = await page.evaluate(() => window.location.href);
    await test.info().attach('current-url.txt', {
      body: Buffer.from(String(href), 'utf-8'),
      contentType: 'text/plain',
    });
    await test.info().attach('failure-dom.html', {
      body: Buffer.from(await page.content(), 'utf-8'),
      contentType: 'text/html',
    });
    await test.info().attach('failure.png', {
      body: await page.screenshot({ fullPage: true }),
      contentType: 'image/png',
    });
    throw e;
  }

  // Debug: attach DOM and screenshot after dashboard detected
  await test.info().attach('dom-after-wait.html', {
    body: Buffer.from(await page.content(), 'utf-8'),
    contentType: 'text/html',
  });
  await test.info().attach('page-after-wait.png', {
    body: await page.screenshot({ fullPage: true }),
    contentType: 'image/png',
  });

  // Header present (avoid strict match ambiguity due to multiple H1s)
  await expect(page.locator('h1').first()).toBeVisible({ timeout: 30000 });

  // Dashboard tabs should be visible
  await expect(page.getByRole('button', { name: 'Security' })).toBeVisible({ timeout: 30000 });
  await expect(page.getByRole('button', { name: 'Performance' })).toBeVisible();
});
