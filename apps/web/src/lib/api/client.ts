// Ubicación: `apps/web/src/lib/api/client.ts`
//
// Descripción: Cliente HTTP base para comunicación con API backend.
//              Incluye headers de autorización PASETO, manejo de errores,
//              y detección de entorno Tauri para usar invoke() cuando esté disponible.
//              Ahora con refresh automático de tokens (A.4).
//
// ADRs relacionados: 0022 (Frontend), 0008 (PASETO), 0007 (Error Handling)

import { get } from "svelte/store";
import { accessTokenStore, authStore } from "$lib/stores/auth.svelte";

const API_BASE_URL = import.meta.env.PUBLIC_API_URL || "http://localhost:3000";
const API_PREFIX = "/api/v1";

const NO_PREFIX_ROUTES = ["/auth/", "/health", "/docs", "/openapi"];

function needsPrefix(endpoint: string): boolean {
	return !NO_PREFIX_ROUTES.some((route) => endpoint.startsWith(route));
}

export class ApiError extends Error {
	constructor(
		public status: number,
		public code: string,
		message: string
	) {
		super(message);
		this.name = "ApiError";
	}
}

function getHeaders(): Record<string, string> {
	const headers: Record<string, string> = {
		"Content-Type": "application/json",
		"Accept": "application/json"
	};

	const token = get(accessTokenStore);
	if (token) {
		headers["Authorization"] = `Bearer ${token}`;
	}

	return headers;
}

/**
 * Intenta hacer refresh del token si hay un refresh_token guardado
 * Retorna true si el refresh fue exitoso, false en caso contrario
 */
async function attemptTokenRefresh(): Promise<boolean> {
	if (typeof window === "undefined") return false;

	const refreshToken = localStorage.getItem("refresh_token");
	if (!refreshToken) return false;

	try {
		const response = await fetch(`${API_BASE_URL}/auth/refresh`, {
			method: "POST",
			headers: {
				"Content-Type": "application/json",
				"Accept": "application/json"
			},
			body: JSON.stringify({ refresh_token: refreshToken })
		});

		if (!response.ok) {
			// Refresh falló - limpiar auth
			console.warn("[Auth] Token refresh failed, clearing auth");
			authStore.clearAuth();
			return false;
		}

		const data = await response.json();

		// Actualizar tokens
		localStorage.setItem("access_token", data.access_token);
		localStorage.setItem("refresh_token", data.refresh_token);
		accessTokenStore.set(data.access_token);

		console.log("[Auth] Token refresh successful");
		return true;
	} catch (error) {
		console.error("[Auth] Error during token refresh:", error);
		authStore.clearAuth();
		return false;
	}
}

/**
 * Realiza una petición a la API con manejo automático de refresh de tokens
 * Si recibe 401, intenta hacer refresh y reintenta la petición
 */
export async function fetchApi<T>(
	endpoint: string,
	options: RequestInit = {}
): Promise<T> {
	const isTauri = typeof window !== "undefined" && "__TAURI__" in window;

	if (isTauri) {
		console.warn("Tauri detected - using HTTP for now");
	}

	const prefix = needsPrefix(endpoint) ? API_PREFIX : "";
	const url = `${API_BASE_URL}${prefix}${endpoint}`;

	// Realizar la petición inicial
	let response = await fetch(url, {
		...options,
		headers: {
			...getHeaders(),
			...options.headers
		}
	});

	// Si recibimos 401 y no estamos en el endpoint de refresh, intentar refresh
	if (response.status === 401 && endpoint !== "/auth/refresh") {
		console.log("[Auth] Received 401, attempting token refresh...");

		const refreshSuccess = await attemptTokenRefresh();

		if (refreshSuccess) {
			// Reintentar la petición original con el nuevo token
			response = await fetch(url, {
				...options,
				headers: {
					...getHeaders(),
					...options.headers
				}
			});
		} else {
			// Refresh falló, redirigir a login
			if (typeof window !== "undefined") {
				window.location.href = "/login";
			}
		}
	}

	if (!response.ok) {
		const errorData = await response.json().catch(() => ({}));
		throw new ApiError(
			response.status,
			errorData.code || "UNKNOWN_ERROR",
			errorData.message || `HTTP ${response.status}: ${response.statusText}`
		);
	}

	if (response.status === 204) {
		return undefined as T;
	}

	return response.json();
}

export const api = {
	get: <T>(endpoint: string) => fetchApi<T>(endpoint, { method: "GET" }),
	post: <T>(endpoint: string, body: unknown) =>
		fetchApi<T>(endpoint, {
			method: "POST",
			body: JSON.stringify(body)
		}),
	put: <T>(endpoint: string, body: unknown) =>
		fetchApi<T>(endpoint, {
			method: "PUT",
			body: JSON.stringify(body)
		}),
	patch: <T>(endpoint: string, body: unknown) =>
		fetchApi<T>(endpoint, {
			method: "PATCH",
			body: JSON.stringify(body)
		}),
	delete: <T>(endpoint: string) =>
		fetchApi<T>(endpoint, { method: "DELETE" })
};