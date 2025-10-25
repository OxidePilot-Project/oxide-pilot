import { test as base, expect } from '@playwright/test';
import fs from 'fs';
import path from 'path';

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

    // Capture Istanbul coverage if available
    try {
      const coverage = await page.evaluate(() => (window as any).__coverage__);
      if (coverage) {
        const outDir = path.resolve(process.cwd(), 'coverage', '.nyc_output');
        fs.mkdirSync(outDir, { recursive: true });
        const outFile = path.join(outDir, `coverage-${process.pid}-${Date.now()}.json`);
        fs.writeFileSync(outFile, JSON.stringify(coverage));
        await testInfo.attach('coverage-status.txt', { body: Buffer.from('coverage: present', 'utf-8'), contentType: 'text/plain' });
      } else {
        // Diagnostics to help confirm instrumentation state
        const diagDir = path.resolve(process.cwd(), 'coverage');
        fs.mkdirSync(diagDir, { recursive: true });
        const diagFile = path.join(diagDir, `diagnostic-${process.pid}-${Date.now()}.txt`);
        fs.writeFileSync(diagFile, 'coverage: missing');
        await testInfo.attach('coverage-status.txt', { body: Buffer.from('coverage: missing', 'utf-8'), contentType: 'text/plain' });
      }
    } catch {
      // ignore
    }

    page.off('console', onConsole);
  },
});

export { expect };
