import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import tailwind from '@astrojs/tailwind';

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
          ],
        },
        {
          label: 'API Reference',
          autogenerate: { directory: 'api' },
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
