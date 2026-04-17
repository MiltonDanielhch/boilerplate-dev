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
		const response = await api.post<LoginResponse>("/auth/login", credentials);
		authStore.setAuth(response.user, response.access_token);
	} finally {
		authStore.setLoading(false);
	}
}

// Registro de usuario
export async function register(data: RegisterInput): Promise<void> {
	authStore.setLoading(true);
	try {
		await api.post("/auth/register", data);
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
		// Opcional: notificar al backend para invalidar token
		await api.post("/auth/logout", {});
	} finally {
		authStore.clearAuth();
		authStore.setLoading(false);
	}
}

// Refresh token
export async function refreshToken(): Promise<boolean> {
	try {
		const response = await api.post<LoginResponse>("/auth/refresh", {});
		authStore.setAuth(response.user, response.access_token);
		return true;
	} catch {
		authStore.clearAuth();
		return false;
	}
}

// Obtener usuario actual
export async function getCurrentUser(): Promise<User | null> {
	try {
		const user = await api.get<User>("/auth/me");
		return user;
	} catch {
		return null;
	}
}
