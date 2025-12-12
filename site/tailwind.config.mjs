import starlightPlugin from '@astrojs/starlight-tailwind';

const accent = {
  200: '#b3d4ff',
  300: '#80b3ff',
  400: '#4d99ff',
  500: '#1a80ff',
  600: '#0066cc',
  700: '#0052a3',
  800: '#003d7a',
  900: '#003366',
  950: '#001a33',
};

const gray = {
  100: '#f5f6fa',
  200: '#eceef5',
  300: '#c0c4d2',
  400: '#888da4',
  500: '#545968',
  700: '#363a4a',
  800: '#252833',
  900: '#17191e',
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
          primary: '#0066cc',
          secondary: '#00b4d8',
          dark: '#0a0a0a',
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
