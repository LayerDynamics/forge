import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
export default {
  // Enable TypeScript preprocessing
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      pages: 'web',
      assets: 'web',
      fallback: 'index.html',
      precompress: false,
      strict: false
    }),
    files: {
      // Use 'client' directory for SvelteKit source to avoid conflict with src/main.ts (Deno backend)
      routes: 'client/routes',
      lib: 'client/lib',
      appTemplate: 'client/app.html',
      assets: 'static'
    },
    paths: {
      base: ''
    }
  }
};
