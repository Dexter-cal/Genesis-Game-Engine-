import { test, expect } from '@playwright/test';

test('capture screenshot of genesis ui', async ({ page }) => {
  await page.goto(`file://${process.cwd()}/genesis/genesis-ui.html`);
  await page.setViewportSize({ width: 1920, height: 1080 });
  // Wait for any animations
  await page.waitForTimeout(2000);
  await page.screenshot({ path: 'genesis_ui_current.png', fullPage: true });
});
