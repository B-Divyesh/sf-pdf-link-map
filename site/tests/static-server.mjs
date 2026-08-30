import { createServer } from 'node:http';
import { readFile, stat } from 'node:fs/promises';
import { extname, join, normalize } from 'node:path';
const root = new URL('../../dist/site/', import.meta.url).pathname;
const types = { '.html':'text/html', '.js':'text/javascript', '.css':'text/css', '.svg':'image/svg+xml', '.webp':'image/webp' };
createServer(async (req, res) => {
  try {
    let path = normalize(decodeURIComponent(new URL(req.url ?? '/', 'http://localhost').pathname)).replace(/^\/+/, '');
    // Azure Static Web Apps reads this deployment-control file but does not
    // expose it. Match that behavior so PWA precache regressions are local.
    if (path === 'staticwebapp.config.json') { res.statusCode = 404; res.end('Not found'); return; }
    if (!path || path.endsWith('/')) path += 'index.html';
    let file = join(root, path);
    try {
      if (!(await stat(file)).isFile()) throw new Error('not a file');
    } catch {
      file = join(root, '404.html');
      res.statusCode = 404;
    }
    res.setHeader('content-type', types[extname(file)] ?? 'application/octet-stream'); res.end(await readFile(file));
  } catch { res.statusCode = 404; res.end('Not found'); }
}).listen(4178, '127.0.0.1');
