import { chromium } from 'playwright';
import AxeBuilder from '@axe-core/playwright';
import { spawn } from 'node:child_process';
import assert from 'node:assert/strict';

const server = spawn(process.execPath, ['site/tests/static-server.mjs'], { stdio: 'ignore' });
const ready = new Promise((resolve, reject) => { const timer = setTimeout(resolve, 800); server.once('exit', code => { clearTimeout(timer); reject(new Error(`server exited ${code}`)); }); });
try {
  await ready;
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({ viewport: { width: 390, height: 844 } });
  const page = await context.newPage();
  const consoleErrors = []; page.on('console', msg => { if (msg.type() === 'error') consoleErrors.push(msg.text()); });
  await page.goto('http://127.0.0.1:4178/', { waitUntil: 'networkidle' });
  assert.equal(await page.locator('h1').count(), 1);
  await page.getByRole('button', { name: 'No annotations' }).click();
  await page.getByText('No link annotations to map.').waitFor();
  const results = await new AxeBuilder({ page }).analyze();
  assert.deepEqual(results.violations.filter(x => ['critical', 'serious'].includes(x.impact ?? '')), []);
  assert.deepEqual(consoleErrors, []);
  await page.evaluate(() => {
    localStorage.setItem('sb_license:pdf-link-map', 'cached-test-token');
    localStorage.setItem('sb_license_verdict:pdf-link-map', JSON.stringify({ valid: true, checkedAt: Date.now(), reason: 'ok' }));
  });
  await page.reload({ waitUntil: 'networkidle' });
  assert.equal(await page.locator('#team-kit').isVisible(), true);
  await page.evaluate(() => navigator.serviceWorker.ready);
  await context.setOffline(true);
  await page.reload({ waitUntil: 'domcontentloaded' });
  assert.equal(await page.locator('h1').count(), 1);
  assert.match(await page.locator('#network-state').textContent(), /Offline/);
  await context.setOffline(false);
  await page.goto('http://127.0.0.1:4178/privacy/', { waitUntil: 'networkidle' });
  assert.equal(await page.locator('h1').textContent(), 'Privacy');
  await browser.close();
} finally { server.kill('SIGTERM'); }
