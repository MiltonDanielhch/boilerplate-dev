// Ubicación: `apps/web/src/lib/api/users.ts`
//
// Descripción: Módulo de API para gestión de usuarios — CRUD completo
//              con soft delete. Requiere permisos específicos (ADR 0006).
//
// ADRs relacionados: 0022 (Frontend), 0006 (RBAC), 0021 (OpenAPI)

import { api } from "./client";
import type { User, UserCreateInput, UserUpdateInput } from "$lib/types/user";
import { isTauri, tauriInvoke } from "$lib/tauri";

interface ListUsersResponse {
	users: User[];
	total: number;
	limit: number;
	offset: number;
}

interface ListUsersParams {
	page?: number;
	perPage?: number;
	search?: string;
	role?: string;
	isActive?: boolean;
}

// Listar usuarios con paginación
export async function listUsers(params: ListUsersParams = {}): Promise<ListUsersResponse> {
	if (isTauri()) {
		// En desktop, list_users devuelve Vec<User> directamente por ahora
		// Adaptamos la respuesta al formato ListUsersResponse
		const users = await tauriInvoke<User[]>("list_users", {
			page: params.page || 1,
			perPage: params.perPage || 20,
			search: params.search
		});
		return {
			users,
			total: users.length, // TODO: Implementar total en Tauri
			limit: params.perPage || 20,
			offset: ((params.page || 1) - 1) * (params.perPage || 20)
		};
	}

	const searchParams = new URLSearchParams();
	if (params.page) searchParams.set("page", params.page.toString());
	if (params.perPage) searchParams.set("per_page", params.perPage.toString());
	if (params.search) searchParams.set("search", params.search);
	if (params.role) searchParams.set("role", params.role);
	if (params.isActive !== undefined) searchParams.set("is_active", params.isActive.toString());

	const query = searchParams.toString();
	return api.get<ListUsersResponse>(`/users${query ? "?" + query : ""}`);
}

// Obtener usuario por ID
export async function getUser(id: string): Promise<User> {
	if (isTauri()) {
		return await tauriInvoke<User>("get_user", { id });
	}
	return api.get<User>(`/users/${id}`);
}

// Crear usuario (admin only) - usa /auth/register internally
export async function createUser(data: UserCreateInput): Promise<User> {
  // POST /auth/register returns { user_id, email, message }
  const res = await api.post<{ user_id: string; email: string; message: string }>("/auth/register", {
    email: data.email,
    password: data.password,
    name: data.name
  });
  // Convertir respuesta al formato User extendido
  return {
    id: res.user_id,
    email: res.email,
    name: data.name || null,
    role: "user" as const,
    isActive: true,
    emailVerifiedAt: null,
    lastLoginAt: null,
    createdBy: null,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString()
  };
}

// Actualizar usuario
export async function updateUser(id: string, data: UserUpdateInput): Promise<User> {
	return api.patch<User>(`/users/${id}`, data);
}

// Soft delete usuario (ADR 0006)
export async function softDeleteUser(id: string): Promise<void> {
	return api.delete(`/users/${id}`);
}

// Reactivar usuario soft-deleted
export async function restoreUser(id: string): Promise<User> {
	return api.post<User>(`/users/${id}/restore`, {});
}

// Forzar eliminación permanente (solo super admin)
export async function hardDeleteUser(id: string): Promise<void> {
	return api.delete(`/users/${id}/hard`);
}

// Impersonate (admin only)
export async function impersonateUser(id: string): Promise<string> {
	const res = await api.post<{ access_token: string }>(`/users/${id}/impersonate`, {});
	return res.access_token;
}
