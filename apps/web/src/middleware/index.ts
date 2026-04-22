/**
 * Astro Middleware - Auth Fullstack (A.3)
 * Protección de rutas y verificación de sesión
 *
 * Referencias: ADR 0008 (PASETO SSR), ADR 0022 (Frontend)
 * docs/03-STRUCTURE.md L454-458
 */

import type { MiddlewareHandler } from 'astro';
import { 
  hasSessionSSR, 
  isProtectedRoute, 
  isAuthRoute 
} from '../lib/auth/session.js';

export const onRequest: MiddlewareHandler = async (context, next) => {
  const { url, cookies, redirect } = context;
  const pathname = url.pathname;

  // Skip middleware para assets estáticos y API
  if (pathname.startsWith('/_astro/') || 
      pathname.startsWith('/api/') ||
      pathname.includes('.') ||
      pathname.startsWith('/favicon')) {
    return next();
  }

  const isAuthenticated = hasSessionSSR(cookies);

  // Si está autenticado y trata de acceder a login/register → redirigir a dashboard
  if (isAuthenticated && isAuthRoute(pathname)) {
    console.log(`[Auth Middleware] Usuario autenticado intentando acceder a ${pathname} → redirect /dashboard`);
    return redirect('/dashboard');
  }

  // Si no está autenticado y trata de acceder a ruta protegida → redirigir a login
  if (!isAuthenticated && isProtectedRoute(pathname)) {
    console.log(`[Auth Middleware] Usuario no autenticado intentando acceder a ${pathname} → redirect /login`);
    return redirect('/login');
  }

  // Continuar con la request
  return next();
};
