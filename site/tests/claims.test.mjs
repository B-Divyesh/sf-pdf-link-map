import { after, before, test } from 'node:test';
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createServer } from 'node:http';
import { createHash } from 'node:crypto';
import { readFile, rm, writeFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { chromium } from 'playwright';

let server;
let browser;
const cli = new URL('../../target/release/pdf-link-map', import.meta.url).pathname;

function runCommand(command, args, env = process.env) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, { env });
    let stdout = '';
    let stderr = '';
    child.stdout.on('data', chunk => { stdout += chunk; });
    child.stderr.on('data', chunk => { stderr += chunk; });
    child.once('error', reject);
    child.once('close', code => resolve({ code, stdout, stderr }));
  });
}

function runCli(args, env = process.env) {
  return runCommand(cli, args, env);
}

async function createDemo() {
  const result = await runCli(['--demo', '--json']);
  assert.equal(result.code, 0, result.stderr);
  const input = result.stderr.match(/^Demo input: (.+)$/m)?.[1];
  const report = result.stderr.match(/^Demo report: (.+)$/m)?.[1];
  assert.ok(input, 'demo input path');
  assert.ok(report, 'demo report path');
  return { input, report, json: JSON.parse(result.stdout) };
}

function startProbe() {
  return new Promise((resolve, reject) => {
    let requests = 0;
    const probe = createServer((_request, response) => {
      requests += 1;
      response.end('unexpected request');
    });
    probe.once('error', reject);
    probe.listen(0, '127.0.0.1', () => {
      const address = probe.address();
      if (!address || typeof address === 'string') {
        reject(new Error('probe address was unavailable'));
        return;
      }
      resolve({
        probe,
        port: address.port,
        requests: () => requests,
      });
    });
  });
}

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
    await page.goto('http://127.0.0.1:4178/?demo=1#demo', { waitUntil: 'networkidle' });
    await page.evaluate(() => navigator.serviceWorker.ready);
    await page.waitForFunction(() => Boolean(navigator.serviceWorker.controller));
    await page.goto('http://127.0.0.1:4178/?demo=1#demo', { waitUntil: 'networkidle' });
    await context.setOffline(true);
    await page.reload({ waitUntil: 'domcontentloaded' });
    assert.equal(await page.locator('h1').count(), 1);
    assert.match(await page.locator('#network-state').textContent(), /Offline/);
    assert.equal(await page.locator('#demo-banner').isVisible(), true);
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
    await page.goto('http://127.0.0.1:4178/?demo=1#demo', { waitUntil: 'networkidle' });
    assert.deepEqual(outboundRequests, []);
    assert.deepEqual(await context.cookies(), []);
  } finally {
    await context.close();
  }
});

test('@claim:local-only-cli', async () => {
  const { probe, port, requests } = await startProbe();
  let sample;
  try {
    sample = await createDemo();
    const original = await readFile(sample.input);
    const remote = Buffer.from('https://example.test/docs');
    const local = Buffer.from(`http://127.0.0.1:${port}/xx`);
    assert.equal(local.length, remote.length, 'replacement keeps PDF offsets stable');
    const offset = original.indexOf(remote);
    assert.ok(offset >= 0, 'demo fixture includes an external URI');
    const edited = Buffer.from(original);
    local.copy(edited, offset);
    await writeFile(sample.input, edited);
    const beforeHash = createHash('sha256').update(edited).digest('hex');
    const output = join(dirname(sample.input), 'local-only-claim.html');
    const result = await runCli([sample.input, '--output', output, '--json']);
    assert.equal(result.code, 0, result.stderr);
    const report = JSON.parse(result.stdout);
    const external = report.links.find(link => link.kind === 'external');
    assert.equal(external?.target, local.toString());
    assert.match(external?.note ?? '', /Recorded only; never requested/);
    assert.equal(createHash('sha256').update(await readFile(sample.input)).digest('hex'), beforeHash);
    await new Promise(resolve => setTimeout(resolve, 100));
    assert.equal(requests(), 0, 'the local HTTP probe must not receive an external-link request');
  } finally {
    await new Promise(resolve => probe.close(resolve));
    if (sample) await rm(dirname(sample.input), { recursive: true, force: true });
  }
});

test('@claim:cli-audit-and-ci', async () => {
  let sample;
  try {
    sample = await createDemo();
    assert.equal(sample.json.summary.total_links, 3);
    assert.equal(sample.json.summary.external_links, 1);
    assert.ok(sample.json.links.some(link => link.kind === 'internal' && link.status === 'valid'));
    assert.ok(sample.json.links.some(link => link.status === 'broken'));
    assert.ok(sample.json.findings.some(finding => finding.code === 'missing_manifest_anchor'));
    assert.match(await readFile(sample.report, 'utf8'), /Annotation map/);
    const explicitCoverage = await runCommand('cargo', ['test', '--test', 'fixture_audit', 'explicit_destinations_are_resolved', '--', '--exact']);
    assert.equal(explicitCoverage.code, 0, explicitCoverage.stderr);
    const failedPolicy = await runCli(['--demo', '--fail-on', 'broken'], {});
    assert.equal(failedPolicy.code, 1, failedPolicy.stderr);
    const malformed = join(dirname(sample.input), 'not-a-pdf.pdf');
    await writeFile(malformed, 'not a PDF');
    const parseFailure = await runCli([malformed], {});
    assert.equal(parseFailure.code, 2, parseFailure.stderr);
  } finally {
    if (sample) await rm(dirname(sample.input), { recursive: true, force: true });
  }
});
