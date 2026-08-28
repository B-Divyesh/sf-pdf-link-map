import { defineConfig } from 'vite';
import { resolve } from 'node:path';

export default defineConfig({
  root: resolve(__dirname),
  base: '/',
  plugins: [{
    name: 'billing-base',
    transformIndexHtml(html) {
      return html.replaceAll('__BILLING_API_BASE__', process.env.VITE_BILLING_API_BASE ?? 'https://pilot-api.sociobot.in');
    }
  }],
  build: {
    outDir: resolve(__dirname, '../dist/site'),
    emptyOutDir: true,
    target: 'es2022',
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
        privacy: resolve(__dirname, 'privacy/index.html'),
        terms: resolve(__dirname, 'terms/index.html')
      }
    }
  }
});
