// Ubicación: `apps/web/src/lib/stores/auth.svelte.ts`
//
// Descripción: Store global de autenticación usando Svelte 5 Runes.
//              Maneja estado del usuario, token PASETO y estado de login.
//              Persiste en localStorage (opcional) y sincroniza entre tabs.
//
// ADRs relacionados: 0022 (Frontend), 0008 (PASETO), ADR 0006 (RBAC)

import type { User } from "$lib/types/user";

// Estado reactivo con Svelte 5 Runes
let user = $state<User | null>(null);
let accessToken = $state<string | null>(null);
let isLoading = $state(false);

// Exportar estado readonly
export const authStore = {
	get user() {
		return user;
	},
	get accessToken() {
		return accessToken;
	},
	get isLoading() {
		return isLoading;
	},
	get isLoggedIn() {
		return user !== null && accessToken !== null;
	},
	get isAdmin() {
		return user?.role === "admin";
	},
	get hasPermission() {
		return (permission: string) => {
			if (!user) return false;
			return user.permissions?.includes(permission) ?? false;
		};
	},

	// Actions
	setAuth(newUser: User, token: string) {
		user = newUser;
		accessToken = token;
		// Persistir en localStorage
		if (typeof localStorage !== "undefined") {
			localStorage.setItem("access_token", token);
			localStorage.setItem("user", JSON.stringify(newUser));
		}
	},

	clearAuth() {
		user = null;
		accessToken = null;
		if (typeof localStorage !== "undefined") {
			localStorage.removeItem("access_token");
			localStorage.removeItem("user");
		}
	},

	setLoading(loading: boolean) {
		isLoading = loading;
	},

	// Inicializar desde localStorage (llamar en +layout.svelte o onMount)
	initFromStorage() {
		if (typeof localStorage === "undefined") return;
		const token = localStorage.getItem("access_token");
		const userStr = localStorage.getItem("user");
		if (token && userStr) {
			try {
				accessToken = token;
				user = JSON.parse(userStr);
			} catch {
				this.clearAuth();
			}
		}
	}
};
