import { defineMiddleware } from 'astro:middleware';
import { setLocale } from '$lib/paraglide/runtime.js';

// i18n middleware - maneja el routing de idiomas en SSR
export const onRequest = defineMiddleware(async (context, next) => {
  // Leer cookie de idioma si existe (guardada por el cliente)
  const localeCookie = context.cookies.get('locale')?.value;
  
  if (localeCookie && (localeCookie === 'es' || localeCookie === 'en')) {
    // Configurar el idioma en el runtime de Paraglide para esta petición
    setLocale(localeCookie as 'es' | 'en', { reload: false });
    context.locals.locale = localeCookie;
  } else {
    // Si no hay cookie, establecer español como default en SSR
    // El cliente manejará su propia preferencia
    setLocale('es', { reload: false });
    context.locals.locale = 'es';
  }
  
  // Continuar con la petición
  return next();
});

declare module 'astro' {
  interface Locals {
    locale?: string;
  }
}
