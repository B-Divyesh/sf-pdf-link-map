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
  const outboundRequests = []; page.on('request', request => { if (!request.url().startsWith('http://127.0.0.1:4178/')) outboundRequests.push(request.url()); });
  await page.goto('http://127.0.0.1:4178/', { waitUntil: 'networkidle' });
  assert.ok(
    await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth),
    'the 390px document must not acquire page-level horizontal scrolling'
  );
  await page.waitForFunction(() => navigator.serviceWorker.getRegistrations().then(registrations => registrations.length > 0));
  assert.equal(await page.evaluate(() => navigator.serviceWorker.getRegistrations().then(registrations => registrations[0]?.scope)), 'http://127.0.0.1:4178/');
  assert.equal(await page.evaluate(() => navigator.serviceWorker.ready.then(registration => registration.scope)), 'http://127.0.0.1:4178/');
  assert.equal(await page.locator('h1').count(), 1);
  await page.getByRole('button', { name: 'No annotations' }).focus();
  await page.keyboard.press('Space');
  await page.getByText('No link annotations to map.').waitFor();
  await page.locator('.code-block pre').first().focus();
  assert.equal(await page.evaluate(() => document.activeElement?.tagName), 'PRE');
  const results = await new AxeBuilder({ page }).analyze();
  assert.deepEqual(results.violations.filter(x => ['critical', 'serious'].includes(x.impact ?? '')), []);
  assert.deepEqual(consoleErrors, []);
  assert.equal(await page.getByText('Team rollout kit is being prepared.').count(), 1);
  assert.equal(await page.getByRole('link', { name: /buy team unlock/i }).count(), 0);
  assert.deepEqual(outboundRequests, []);
  await page.evaluate(() => navigator.serviceWorker.ready);
  await context.setOffline(true);
  await page.reload({ waitUntil: 'domcontentloaded' });
  assert.equal(await page.locator('h1').count(), 1);
  assert.match(await page.locator('#network-state').textContent(), /Offline/);
  await context.setOffline(false);
  await page.goto('http://127.0.0.1:4178/privacy/', { waitUntil: 'networkidle' });
  assert.equal(await page.locator('h1').textContent(), 'Privacy');
  const desktopContext = await browser.newContext({ viewport: { width: 1440, height: 900 } });
  const desktop = await desktopContext.newPage();
  await desktop.goto('http://127.0.0.1:4178/', { waitUntil: 'networkidle' });
  assert.equal(await desktop.locator('h1').count(), 1);
  const desktopResults = await new AxeBuilder({ page: desktop }).analyze();
  assert.deepEqual(desktopResults.violations.filter(x => ['critical', 'serious'].includes(x.impact ?? '')), []);
  await desktopContext.close();
  await browser.close();
} finally { server.kill('SIGTERM'); }
