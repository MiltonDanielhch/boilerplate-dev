// Ubicación: `apps/web/src/lib/stores/auth.svelte.ts`
//
// Descripción: Store global de autenticación usando Svelte 5 Runes.
//              Maneja estado del usuario, token PASETO y estado de login.
//              Persiste en localStorage.
//
// ADRs relacionados: 0022 (Frontend), 0008 (PASETO), ADR 0006 (RBAC)

import type { User } from "$lib/types/user";
import { isTauri } from "$lib/tauri";
import { getCurrentUser } from "$lib/api/auth";

// Estado reactivo usando Svelte 5 runes
let userState = $state<User | null>(null);
let accessTokenState = $state<string | null>(null);
let refreshTokenState = $state<string | null>(null);
let isLoadingState = $state(false);

// Getters derivados
const isLoggedIn = $derived(userState !== null && accessTokenState !== null);
const isAdmin = $derived(userState?.role === "admin");

function hasPermission(permission: string): boolean {
	return userState?.permissions?.includes(permission) ?? false;
}

function setAuth(newUser: User, token: string, refreshToken: string) {
	userState = newUser;
	accessTokenState = token;
	refreshTokenState = refreshToken;
	
	if (typeof localStorage !== "undefined") {
		localStorage.setItem("access_token", token);
		localStorage.setItem("refresh_token", refreshToken);
		localStorage.setItem("user", JSON.stringify(newUser));
		document.cookie = `access_token=${encodeURIComponent(token)}; path=/; max-age=604800; SameSite=Lax`;
		document.cookie = `refresh_token=${encodeURIComponent(refreshToken)}; path=/; max-age=2592000; SameSite=Lax`;
		document.cookie = `user=${encodeURIComponent(JSON.stringify(newUser))}; path=/; max-age=604800; SameSite=Lax`;
	}
}

function clearAuth() {
	userState = null;
	accessTokenState = null;
	refreshTokenState = null;
	
	if (typeof localStorage !== "undefined") {
		localStorage.removeItem("access_token");
		localStorage.removeItem("refresh_token");
		localStorage.removeItem("user");
		document.cookie = "access_token=; path=/; max-age=0";
		document.cookie = "refresh_token=; path=/; max-age=0";
		document.cookie = "user=; path=/; max-age=0";
	}
}

function setLoading(loading: boolean) {
	isLoadingState = loading;
}

async function init() {
	if (isTauri()) {
		try {
			const user = await getCurrentUser();
			if (user) {
				userState = user;
				accessTokenState = "tauri-managed";
				refreshTokenState = "tauri-managed";
			}
		} catch (e) {
			console.error("Error initializing Tauri auth:", e);
		}
		return;
	}

	if (typeof localStorage !== "undefined") {
		const storedToken = localStorage.getItem("access_token");
		const storedRefreshToken = localStorage.getItem("refresh_token");
		const storedUser = localStorage.getItem("user");
		
		if (storedToken) accessTokenState = storedToken;
		if (storedRefreshToken) refreshTokenState = storedRefreshToken;
		if (storedUser) {
			try {
				userState = JSON.parse(storedUser);
			} catch {
				userState = null;
			}
		}
	}
}

// Objeto authStore exports
export const authStore = {
	get user() { return userState; },
	get accessToken() { return accessTokenState; },
	get refreshToken() { return refreshTokenState; },
	get isLoading() { return isLoadingState; },
	get isLoggedIn() { return isLoggedIn; },
	get isAdmin() { return isAdmin; },
	hasPermission,
	setAuth,
	clearAuth,
	setLoading,
	init,
	initFromStorage: init
} as const;