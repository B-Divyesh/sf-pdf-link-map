import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile, stat } from 'node:fs/promises';
import { join } from 'node:path';

const dist = new URL('../../dist/site/', import.meta.url).pathname;
test('build has deploy, discovery, legal, and designed not-found routes', async () => {
  for (const path of ['index.html', 'privacy/index.html', 'terms/index.html', '404.html', 'robots.txt', 'sitemap.xml', 'apple-touch-icon.png', 'link-map-social-card.jpg', 'sw.js', 'staticwebapp.config.json']) assert.ok((await stat(join(dist, path))).isFile(), path);
  const serviceWorker = await readFile(join(dist, 'sw.js'), 'utf8');
  assert.match(serviceWorker, /\/assets\/main-[A-Za-z0-9_-]+\.js/);
  assert.doesNotMatch(serviceWorker, /staticwebapp\.config\.json/);
});
test('landing markup names the intended team, has required semantics, metadata, and no unavailable checkout', async () => {
  const html = await readFile(join(dist, 'index.html'), 'utf8');
  assert.match(html, /<html lang="en">/); assert.equal((html.match(/<h1/g) ?? []).length, 1); assert.match(html, /<main/); assert.match(html, /For operations and technical-document teams converting HTML or DOCX to PDF/); assert.match(html, /Team rollout kit is being prepared/); assert.doesNotMatch(html, /\/checkout/); assert.doesNotMatch(html, /api\.sociobot\.in/); assert.match(html, /alt="Illustrated lab notebook/);
  for (const marker of ['rel="canonical"', 'property="og:title"', 'name="twitter:card"', 'apple-touch-icon']) assert.match(html, new RegExp(marker));
});
test('legal and not-found routes retain the standard shell and route metadata', async () => {
  for (const path of ['privacy/index.html', 'terms/index.html', '404.html']) {
    const html = await readFile(join(dist, path), 'utf8');
    assert.equal((html.match(/<h1/g) ?? []).length, 1, path);
    assert.match(html, /<header class="site-header">/, path);
    assert.match(html, /<footer class="site-footer">/, path);
    assert.match(html, /rel="canonical"/, path);
    assert.match(html, /property="og:title"/, path);
  }
  const config = JSON.parse(await readFile(join(dist, 'staticwebapp.config.json'), 'utf8'));
  assert.equal(config.responseOverrides?.['404']?.rewrite, '/404.html');
  assert.equal(config.responseOverrides?.['404']?.statusCode, undefined);
});
test('each public claim has one matching tagged demo test', async () => {
  const claims = JSON.parse(await readFile(new URL('../../.factory/claims.json', import.meta.url), 'utf8'));
  for (const claim of claims) assert.match(claim.test, new RegExp(`@claim:${claim.id}`), claim.id);
});
test('deployment configuration enforces browser isolation and transport policy', async () => {
  const config = JSON.parse(await readFile(join(dist, 'staticwebapp.config.json'), 'utf8'));
  const headers = config.globalHeaders;
  assert.match(headers['Content-Security-Policy'], /default-src 'self'/);
  assert.match(headers['Content-Security-Policy'], /frame-ancestors 'none'/);
  assert.match(headers['Content-Security-Policy'], /connect-src 'self'/);
  assert.doesNotMatch(headers['Content-Security-Policy'], /api\.sociobot\.in/);
  assert.equal(headers['X-Frame-Options'], 'DENY');
  assert.match(headers['Strict-Transport-Security'], /max-age=63072000/);
});
test('production bundle registers the worker immediately instead of waiting for load', async () => {
  const assets = await import('node:fs/promises').then(fs => fs.readdir(join(dist, 'assets')));
  const main = assets.find(file => /^main-.*\.js$/.test(file));
  assert.ok(main, 'main JavaScript asset');
  const source = await readFile(join(dist, 'assets', main), 'utf8');
  assert.match(source, /serviceWorker\.register\("\/sw\.js",\{scope:"\/"\}\)/);
  assert.doesNotMatch(source, /addEventListener\("load".*serviceWorker\.register/);
  assert.doesNotMatch(source, /api\.sociobot\.in|\/checkout|localStorage|fetch\(/);
});
test('asset budgets are respected', async () => {
  const assets = await import('node:fs/promises').then(fs => fs.readdir(join(dist, 'assets')));
  const js = assets.filter(x => x.endsWith('.js')); const css = assets.filter(x => x.endsWith('.css'));
  let jsBytes = 0; for (const file of js) jsBytes += (await stat(join(dist, 'assets', file))).size;
  let cssBytes = 0; for (const file of css) cssBytes += (await stat(join(dist, 'assets', file))).size;
  assert.ok(jsBytes <= 200 * 1024, `JS ${jsBytes}`); assert.ok(cssBytes <= 50 * 1024, `CSS ${cssBytes}`);
  assert.ok((await stat(join(dist, 'link-map-notebook.webp'))).size <= 300 * 1024);
});
