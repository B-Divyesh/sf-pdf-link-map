import { after, before, test } from 'node:test';
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { chromium } from 'playwright';

let server;
let browser;

before(async () => {
  server = spawn(process.execPath, ['site/tests/static-server.mjs'], { stdio: 'ignore' });
  await new Promise((resolve, reject) => {
    const timer = setTimeout(resolve, 800);
    server.once('exit', code => {
      clearTimeout(timer);
      reject(new Error(`server exited ${code}`));
    });
  });
  browser = await chromium.launch({ headless: true });
});

after(async () => {
  await browser?.close();
  server?.kill('SIGTERM');
});

test('@claim:offline-shell', async () => {
  const context = await browser.newContext({ viewport: { width: 390, height: 844 } });
  try {
    const page = await context.newPage();
    await page.goto('http://127.0.0.1:4178/', { waitUntil: 'networkidle' });
    await page.evaluate(() => navigator.serviceWorker.ready);
    await context.setOffline(true);
    await page.reload({ waitUntil: 'domcontentloaded' });
    assert.equal(await page.locator('h1').count(), 1);
    assert.match(await page.locator('#network-state').textContent(), /Offline/);
    assert.equal(await page.evaluate(() => Boolean(navigator.serviceWorker.controller)), true);
  } finally {
    await context.close();
  }
});

test('@claim:no-web-tracking', async () => {
  const context = await browser.newContext({ viewport: { width: 390, height: 844 } });
  try {
    const page = await context.newPage();
    const outboundRequests = [];
    page.on('request', request => {
      if (!request.url().startsWith('http://127.0.0.1:4178/')) outboundRequests.push(request.url());
    });
    await page.goto('http://127.0.0.1:4178/', { waitUntil: 'networkidle' });
    assert.deepEqual(outboundRequests, []);
  } finally {
    await context.close();
  }
});
