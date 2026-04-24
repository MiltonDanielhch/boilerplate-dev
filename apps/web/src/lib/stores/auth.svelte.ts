// Ubicación: `apps/web/src/lib/stores/auth.svelte.ts`
//
// Descripción: Store global de autenticación usando Svelte stores (writable).
//              Maneja estado del usuario, token PASETO y estado de login.
//              Persiste en localStorage y sincroniza entre tabs.
//
// ADRs relacionados: 0022 (Frontend), 0008 (PASETO), ADR 0006 (RBAC)

import { writable, get, derived, type Readable, type Writable } from "svelte/store";
import type { User } from "$lib/types/user";
import { isTauri } from "$lib/tauri";
import { getCurrentUser } from "$lib/api/auth";

// Stores individuales para reactivity
export const userStore: Writable<User | null> = writable(null);
export const accessTokenStore: Writable<string | null> = writable(null);
export const refreshTokenStore: Writable<string | null> = writable(null);
export const isLoadingStore = writable(false);

// Store derivado para isLoggedIn
export const isLoggedInStore: Readable<boolean> = derived(
	[userStore, accessTokenStore],
	([$user, $token]) => $user !== null && $token !== null
);

// Store derivado para isAdmin
export const isAdminStore: Readable<boolean> = derived(
	userStore,
	($user) => $user?.role === "admin"
);

// Objeto authStore con API completa para backward compatibility
export const authStore = {
	userStore,
	accessTokenStore,
	isLoadingStore,
	isLoggedInStore,
	isAdminStore,

	get user() {
		return get(userStore);
	},
	get accessToken() {
		return get(accessTokenStore);
	},
	get refreshToken() {
		return get(refreshTokenStore);
	},
	get isLoading() {
		return get(isLoadingStore);
	},
	get isLoggedIn() {
		return get(isLoggedInStore);
	},
	get isAdmin() {
		return get(isAdminStore);
	},

	hasPermission(permission: string) {
		const user = get(userStore);
		if (!user) return false;
		return user.permissions?.includes(permission) ?? false;
	},

	setAuth(newUser: User, token: string, refreshToken: string) {
		userStore.set(newUser);
		accessTokenStore.set(token);
		refreshTokenStore.set(refreshToken);
		if (typeof localStorage !== "undefined") {
			localStorage.setItem("access_token", token);
			localStorage.setItem("refresh_token", refreshToken);
			localStorage.setItem("user", JSON.stringify(newUser));
			// Also set cookies for SSR middleware
			document.cookie = `access_token=${encodeURIComponent(token)}; path=/; max-age=604800; SameSite=Lax`;
			document.cookie = `refresh_token=${encodeURIComponent(refreshToken)}; path=/; max-age=2592000; SameSite=Lax`;
			document.cookie = `user=${encodeURIComponent(JSON.stringify(newUser))}; path=/; max-age=604800; SameSite=Lax`;
		}
	},

	clearAuth() {
		userStore.set(null);
		accessTokenStore.set(null);
		refreshTokenStore.set(null);
		if (typeof localStorage !== "undefined") {
			localStorage.removeItem("access_token");
			localStorage.removeItem("refresh_token");
			localStorage.removeItem("user");
			// Clear cookies
			document.cookie = "access_token=; path=/; max-age=0";
			document.cookie = "refresh_token=; path=/; max-age=0";
			document.cookie = "user=; path=/; max-age=0";
		}
	},

	setLoading(loading: boolean) {
		isLoadingStore.set(loading);
	},

	async init() {
		if (isTauri()) {
			// En Desktop, intentamos recuperar la sesión desde el backend Rust
			try {
				const user = await getCurrentUser();
				if (user) {
					// El backend Rust ya tiene el token, pero el store Svelte necesita estar al día
					// (Aunque en Desktop los tokens se manejan internamente, para compatibilidad UI
					// los marcamos como 'tauri-managed')
					userStore.set(user);
					accessTokenStore.set("tauri-managed");
					refreshTokenStore.set("tauri-managed");
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
			if (storedToken) accessTokenStore.set(storedToken);
			if (storedRefreshToken) refreshTokenStore.set(storedRefreshToken);
			if (storedUser) {
				try {
					const user = JSON.parse(storedUser);
					userStore.set(user);
				} catch {
					userStore.set(null);
				}
			}
		}
	},

	initFromStorage() {
		return this.init();
	}
};