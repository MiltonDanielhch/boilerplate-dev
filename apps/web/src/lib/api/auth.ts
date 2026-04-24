// Ubicación: `apps/web/src/lib/api/auth.ts`
//
// Descripción: Módulo de API para autenticación — login, registro,
//              refresh token y logout. Integra con auth store.
//
// ADRs relacionados: 0022 (Frontend), 0008 (PASETO), 0021 (OpenAPI)

import { api } from "./client";
import { authStore } from "$lib/stores/auth.svelte";
import type { LoginResponse, User, RegisterInput } from "$lib/types/user";
import type { LoginInput } from "$lib/validation/schemas";
import { isTauri, tauriInvoke } from "$lib/tauri";

interface LoginRequest {
	email: string;
	password: string;
}

interface RegisterRequest {
	email: string;
	password: string;
	name?: string;
}

// Login de usuario
export async function login(credentials: LoginInput): Promise<void> {
	authStore.setLoading(true);
	try {
		if (isTauri()) {
			// En Desktop, invocamos al comando Rust directamente (Offline-first)
			// El comando login en Rust ya guarda los tokens localmente
			const response = await tauriInvoke<LoginResponse>("login", {
				email: credentials.email,
				password: credentials.password
			});
			authStore.setAuth(response.user, response.access_token, response.refresh_token);
		} else {
			// En Web, usamos la API REST
			const response = await api.post<LoginResponse>("/auth/login", credentials);
			authStore.setAuth(response.user, response.access_token, response.refresh_token);
		}
	} finally {
		authStore.setLoading(false);
	}
}

// Registro de usuario
export async function register(data: RegisterInput): Promise<void> {
	authStore.setLoading(true);
	try {
		if (isTauri()) {
			await tauriInvoke("create_user", {
				email: data.email,
				password: data.password,
				name: data.name
			});
		} else {
			await api.post("/auth/register", data);
		}
		// Auto-login después de registro
		await login({ email: data.email, password: data.password });
	} finally {
		authStore.setLoading(false);
	}
}

// Logout
export async function logout(): Promise<void> {
	authStore.setLoading(true);
	try {
		if (isTauri()) {
			await tauriInvoke("logout");
		} else {
			// Opcional: notificar al backend para invalidar token
			await api.post("/auth/logout", {});
		}
	} finally {
		authStore.clearAuth();
		authStore.setLoading(false);
	}
}

// Refresh token
export async function refreshAccessToken(): Promise<boolean> {
	const refreshToken = authStore.refreshToken;
	if (!refreshToken) {
		authStore.clearAuth();
		return false;
	}
	try {
		const response = await api.post<LoginResponse>("/auth/refresh", { refresh_token: refreshToken });
		// Keep existing refresh token if response doesn't include a new one
		const newRefreshToken = response.refresh_token || refreshToken;
		authStore.setAuth(response.user, response.access_token, newRefreshToken);
		return true;
	} catch {
		authStore.clearAuth();
		return false;
	}
}

// Obtener usuario actual
export async function getCurrentUser(): Promise<User | null> {
	try {
		if (isTauri()) {
			return await tauriInvoke<User | null>("get_current_user");
		} else {
			return await api.get<User>("/auth/me");
		}
	} catch {
		return null;
	}
}
