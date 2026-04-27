// @ts-check
import { defineConfig } from 'astro/config';

import svelte from '@astrojs/svelte';
import node from '@astrojs/node';
import tailwindcss from '@tailwindcss/vite';
import sitemap from '@astrojs/sitemap';
import AstroPWA from '@vite-pwa/astro';

// https://astro.build/config
export default defineConfig({
  output: 'server',
  adapter: node({
    mode: 'standalone'
  }),
  site: 'https://boilerplate.example.com',
  integrations: [
    svelte(), 
    sitemap({
      filter: (page) => !page.includes('/dashboard') && !page.includes('/login')
    }),
    AstroPWA({
      registerType: 'autoUpdate',
      manifest: {
        name: 'Boilerplate Fullstack',
        short_name: 'Boilerplate',
        description: 'Fullstack boilerplate with Rust and Svelte 5',
        theme_color: '#534AB7',
        background_color: '#F8F8F6',
        display: 'standalone',
        start_url: '/',
        icons: [
          {
            src: '/icons/icon-192x192.png',
            sizes: '192x192',
            type: 'image/png',
          },
          {
            src: '/icons/icon-512x512.png',
            sizes: '512x512',
            type: 'image/png',
          },
        ],
      },
      workbox: {
        globPatterns: ['**/*.{js,css,html,ico,png,svg,webp}'],
        navigateFallback: null,
      },
      devOptions: {
        enabled: true,
      },
    })
  ],
  vite: {
    plugins: [tailwindcss()]
  },
  i18n: {
    defaultLocale: 'es',
    locales: ['es', 'en'],
    routing: {
      prefixDefaultLocale: false
    }
  }
});