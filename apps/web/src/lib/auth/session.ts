/**
 * Session utilities - Auth Fullstack (A.3)
 * Manejo de sesión compatible con SSR y Cliente
 *
 * Referencias: ADR 0008 (PASETO SSR), ADR 0022 (Frontend)
 */

import type { AstroCookies } from 'astro';

const ACCESS_TOKEN_KEY = 'access_token';
const USER_KEY = 'user';

/**
 * Verifica si hay una sesión válida en SSR
 * Usa cookies como fuente de verdad en SSR
 */
export function hasSessionSSR(cookies: AstroCookies): boolean {
  const accessToken = cookies.get(ACCESS_TOKEN_KEY)?.value;
  const user = cookies.get(USER_KEY)?.value;
  
  if (!accessToken || !user) {
    return false;
  }
  
  try {
    // Verificar que el usuario sea parseable
    JSON.parse(user);
    return accessToken.length > 0;
  } catch {
    return false;
  }
}

/**
 * Obtiene el usuario actual en SSR
 */
export function getUserSSR(cookies: AstroCookies): unknown | null {
  const user = cookies.get(USER_KEY)?.value;
  if (!user) return null;
  
  try {
    return JSON.parse(user);
  } catch {
    return null;
  }
}

/**
 * Obtiene el token en SSR
 */
export function getTokenSSR(cookies: AstroCookies): string | null {
  return cookies.get(ACCESS_TOKEN_KEY)?.value ?? null;
}

/**
 * Configura cookies de sesión en SSR
 * Llamado después de login exitoso
 */
export function setSessionCookies(
  cookies: AstroCookies,
  accessToken: string,
  user: unknown,
  options?: {
    maxAge?: number;
    httpOnly?: boolean;
    secure?: boolean;
    sameSite?: 'strict' | 'lax' | 'none';
  }
): void {
  const defaultOptions = {
    maxAge: 60 * 60 * 24, // 24 horas
    httpOnly: false, // Necesitamos acceder desde JS en el cliente
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax' as const,
    path: '/',
  };
  
  const finalOptions = { ...defaultOptions, ...options };
  
  cookies.set(ACCESS_TOKEN_KEY, accessToken, finalOptions);
  cookies.set(USER_KEY, JSON.stringify(user), finalOptions);
}

/**
 * Limpia cookies de sesión en SSR
 */
export function clearSessionCookies(cookies: AstroCookies): void {
  cookies.delete(ACCESS_TOKEN_KEY, { path: '/' });
  cookies.delete(USER_KEY, { path: '/' });
}

/**
 * Lista de rutas protegidas
 */
export const PROTECTED_ROUTES = ['/dashboard', '/dashboard/'];

/**
 * Lista de rutas de auth (no deben accederse con sesión)
 */
export const AUTH_ROUTES = ['/login', '/register'];

/**
 * Verifica si una ruta está protegida
 */
export function isProtectedRoute(pathname: string): boolean {
  return PROTECTED_ROUTES.some(route => 
    pathname === route || pathname.startsWith(route + '/')
  );
}

/**
 * Verifica si es una ruta de auth
 */
export function isAuthRoute(pathname: string): boolean {
  return AUTH_ROUTES.some(route => pathname === route);
}
