// @ts-check
import { defineConfig } from 'astro/config';

import svelte from '@astrojs/svelte';
import node from '@astrojs/node';
import sitemap from '@astrojs/sitemap';
import AstroPWA from '@vite-pwa/astro';

const isDev = process.env.NODE_ENV === 'development';

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
    optimizeDeps: {
      exclude: ['bits-ui']
    },
    ssr: {
      noExternal: ['bits-ui', '@internationalized/date']
    },
    plugins: [
      {
        name: 'handle-bits-ui-css',
        enforce: 'pre',
        async load(id) {
          // Intercept bits-ui virtual CSS files
          if (id.includes('bits-ui') && id.includes('?svelte&type=style')) {
            // Read the file and clean it
            const fs = await import('fs');
            const path = await import('path');
            
            // Extract the actual file path (before the ?query)
            const actualPath = id.split('?')[0];
            
            try {
              let content = fs.readFileSync(actualPath, 'utf-8');
              
              // Extract just the CSS from the .svelte file
              // Find <style> tag content
              const styleMatch = content.match(/<style[^>]*>([\s\S]*?)<\/style>/);
              if (styleMatch) {
                return styleMatch[1].trim() || '/* no styles */';
              }
              
              return '/* no styles */';
            } catch (e) {
              return '/* error loading */';
            }
          }
        }
      }
    ]
  },
  i18n: {
    defaultLocale: 'es',
    locales: ['es', 'en'],
    routing: {
      prefixDefaultLocale: false
    }
  }
});