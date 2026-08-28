import { readdir, writeFile } from 'node:fs/promises';
import { join, relative } from 'node:path';

const root = new URL('../../dist/site/', import.meta.url).pathname;
async function files(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  return (await Promise.all(entries.map(entry => entry.isDirectory() ? files(join(directory, entry.name)) : [join(directory, entry.name)]))).flat();
}
const builtFiles = (await files(root)).filter(path => !path.endsWith('/sw.js')).map(path => `/${relative(root, path).replaceAll('\\', '/')}`);
const shell = [...new Set(['/', '/privacy/', '/terms/', ...builtFiles])];
const source = `const CACHE='pdf-link-map-v1';const SHELL=${JSON.stringify(shell)};self.addEventListener('install',event=>event.waitUntil(caches.open(CACHE).then(cache=>cache.addAll(SHELL)).then(()=>self.skipWaiting())));self.addEventListener('activate',event=>event.waitUntil(caches.keys().then(keys=>Promise.all(keys.filter(key=>key!==CACHE).map(key=>caches.delete(key)))).then(()=>self.clients.claim())));self.addEventListener('fetch',event=>{if(event.request.method!=='GET'||new URL(event.request.url).origin!==location.origin)return;event.respondWith(caches.match(event.request).then(cached=>cached||fetch(event.request).then(response=>{const copy=response.clone();caches.open(CACHE).then(cache=>cache.put(event.request,copy));return response})))})`;
await writeFile(join(root, 'sw.js'), source);
