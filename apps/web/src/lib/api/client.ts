// Ubicación: `apps/web/src/lib/api/client.ts`
//
// Descripción: Cliente HTTP base para comunicación con API backend.
//              Incluye headers de autorización PASETO, manejo de errores,
//              y detección de entorno Tauri para usar invoke() cuando esté disponible.
//
// ADRs relacionados: 0022 (Frontend), 0008 (PASETO), 0007 (Error Handling)

import { authStore } from "$lib/stores/auth.svelte";

const API_BASE_URL = import.meta.env.PUBLIC_API_URL || "http://localhost:3000";

// Error de API tipado
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

// Headers base para requests
function getHeaders(): Record<string, string> {
	const headers: Record<string, string> = {
		"Content-Type": "application/json",
		"Accept": "application/json"
	};

	// Agregar token PASETO si existe
	const token = authStore.accessToken;
	if (token) {
		headers["Authorization"] = `Bearer ${token}`;
	}

	return headers;
}

// Fetch wrapper con manejo de errores
export async function fetchApi<T>(
	endpoint: string,
	options: RequestInit = {}
): Promise<T> {
	// Detectar si estamos en Tauri
	const isTauri = typeof window !== "undefined" && "__TAURI__" in window;

	if (isTauri) {
		// TODO: Implementar invoke() para Tauri
		console.warn("Tauri detected - using HTTP for now");
	}

	const url = `${API_BASE_URL}${endpoint}`;

	const response = await fetch(url, {
		...options,
		headers: {
			...getHeaders(),
			...options.headers
		}
	});

	// Manejar errores HTTP
	if (!response.ok) {
		const errorData = await response.json().catch(() => ({}));
		throw new ApiError(
			response.status,
			errorData.code || "UNKNOWN_ERROR",
			errorData.message || `HTTP ${response.status}: ${response.statusText}`
		);
	}

	// Parsear respuesta JSON
	if (response.status === 204) {
		return undefined as T;
	}

	return response.json();
}

// Métodos HTTP convenientes
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
