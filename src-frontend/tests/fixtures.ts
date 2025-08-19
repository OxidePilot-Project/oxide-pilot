import { test as base, expect } from '@playwright/test';

// Extends Playwright test to automatically capture console logs and attach on failure
export const test = base.extend({
  page: async ({ page }, use, testInfo) => {
    const consoleLogs: string[] = [];

    const onConsole = (msg: any) => {
      try {
        consoleLogs.push(`[${msg.type()}] ${msg.text()}`);
      } catch (e) {
        // ignore
      }
    };

    page.on('console', onConsole);

    await use(page);

    // Attach logs if test failed
    if (testInfo.status !== testInfo.expectedStatus) {
      const body = Buffer.from(consoleLogs.join('\n'), 'utf-8');
      await testInfo.attach('browser-console.log', { body, contentType: 'text/plain' });
    }

    page.off('console', onConsole);
  },
});

export { expect };
