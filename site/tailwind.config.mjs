import starlightPlugin from '@astrojs/starlight-tailwind';

// Ember/amber accent ramp — drawn from the Forge logo (black anvil on a
// copper/amber forge-glow). Replaces the previous blue accent.
const accent = {
  200: '#fcdcbc',
  300: '#f9c98c',
  400: '#f4ad57',
  500: '#ed8936',
  600: '#dd6f24',
  700: '#b85619',
  800: '#8f4216',
  900: '#743615',
  950: '#3f1c08',
};

// Warm-neutral gray ramp (blue cast removed) leaning blacker, to sit under the
// ember accent and the black logo.
const gray = {
  100: '#f5f4f1',
  200: '#eceae4',
  300: '#c6c1b8',
  400: '#8d877c',
  500: '#57514a',
  700: '#383330',
  800: '#221f1c',
  900: '#100e0c',
};

/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{astro,html,js,jsx,md,mdx,svelte,ts,tsx,vue}'],
  theme: {
    extend: {
      colors: {
        accent,
        gray,
        forge: {
          primary: '#ed8936',
          secondary: '#f59e0b',
          dark: '#050505',
          light: '#fafafa',
        },
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'Menlo', 'monospace'],
      },
    },
  },
  plugins: [starlightPlugin()],
};
