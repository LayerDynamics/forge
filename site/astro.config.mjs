import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import tailwind from '@astrojs/tailwind';
import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

// Load custom WAT (WebAssembly Text) grammar
const __dirname = dirname(fileURLToPath(import.meta.url));
const watGrammar = JSON.parse(
  readFileSync(join(__dirname, 'grammars/wat.tmLanguage.json'), 'utf-8')
);

export default defineConfig({
  site: 'https://forge-deno.com',
  integrations: [
    starlight({
      title: 'Forge',
      description: 'Build cross-platform desktop apps with TypeScript and Deno',
      logo: {
        src: './src/assets/logo.svg',
        replacesTitle: false,
      },
      expressiveCode: {
        shiki: {
          // Custom language grammars
          langs: [watGrammar],
          // Language aliases for unsupported code block languages
          langAlias: {
            // 'ascii' for directory tree diagrams -> plain text
            ascii: 'text',
            // 'wast' is an alias for 'wat' (WebAssembly Script format)
            wast: 'wat',
          },
        },
      },
      social: {
        github: 'https://github.com/LayerDynamics/forge',
      },
      editLink: {
        baseUrl: 'https://github.com/LayerDynamics/forge/edit/main/site/',
      },
      customCss: ['./src/styles/custom.css'],
      sidebar: [
        {
          label: 'Getting Started',
          items: [
            'getting-started',
            'architecture',
            'internals',
            'roadmap',
          ],
        },
        {
          label: 'API Reference',
          autogenerate: { directory: 'api' },
        },
        {
          label: 'Crates',
          autogenerate: { directory: 'crates' },
        },
        {
          label: 'Examples',
          autogenerate: { directory: 'examples' },
        },
        {
          label: 'Guides',
          autogenerate: { directory: 'guides' },
        },
      ],
      head: [
        {
          tag: 'meta',
          attrs: {
            property: 'og:image',
            content: 'https://forge-deno.com/og-image.png',
          },
        },
      ],
    }),
    tailwind({ applyBaseStyles: false }),
  ],
});
