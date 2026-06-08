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
      // Brand favicon (Forge emblem on the ember plate). Starlight injects the
      // primary <link> for this; the head[] entries below add the .ico fallback
      // for legacy browsers and the apple-touch-icon for iOS home screens.
      favicon: '/favicon.svg',
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
        {
          tag: 'link',
          attrs: {
            rel: 'icon',
            href: '/favicon.ico',
            sizes: '16x16 32x32 48x48',
          },
        },
        {
          tag: 'link',
          attrs: {
            rel: 'apple-touch-icon',
            href: '/apple-touch-icon.png',
            sizes: '180x180',
          },
        },
      ],
    }),
    tailwind({ applyBaseStyles: false }),
  ],
});
