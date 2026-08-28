import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile, stat } from 'node:fs/promises';
import { join } from 'node:path';

const dist = new URL('../../dist/site/', import.meta.url).pathname;
test('build has deploy entry and legal routes', async () => {
  for (const path of ['index.html', 'privacy/index.html', 'terms/index.html', 'sw.js', 'staticwebapp.config.json']) assert.ok((await stat(join(dist, path))).isFile(), path);
  const serviceWorker = await readFile(join(dist, 'sw.js'), 'utf8');
  assert.match(serviceWorker, /\/assets\/main-[A-Za-z0-9_-]+\.js/);
});
test('landing markup has required semantics and billing contract', async () => {
  const html = await readFile(join(dist, 'index.html'), 'utf8');
  assert.match(html, /<html lang="en">/); assert.equal((html.match(/<h1/g) ?? []).length, 1); assert.match(html, /<main/); assert.match(html, /pilot-api\.sociobot\.in\/api\/v1\/products\/pdf-link-map\/checkout/); assert.match(html, /alt="Illustrated lab notebook/);
});
test('asset budgets are respected', async () => {
  const assets = await import('node:fs/promises').then(fs => fs.readdir(join(dist, 'assets')));
  const js = assets.filter(x => x.endsWith('.js')); const css = assets.filter(x => x.endsWith('.css'));
  let jsBytes = 0; for (const file of js) jsBytes += (await stat(join(dist, 'assets', file))).size;
  let cssBytes = 0; for (const file of css) cssBytes += (await stat(join(dist, 'assets', file))).size;
  assert.ok(jsBytes <= 200 * 1024, `JS ${jsBytes}`); assert.ok(cssBytes <= 50 * 1024, `CSS ${cssBytes}`);
  assert.ok((await stat(join(dist, 'link-map-notebook.webp'))).size <= 300 * 1024);
});
