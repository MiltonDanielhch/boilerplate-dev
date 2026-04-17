// Ubicación: `apps/web/src/lib/api/users.ts`
//
// Descripción: Módulo de API para gestión de usuarios — CRUD completo
//              con soft delete. Requiere permisos específicos (ADR 0006).
//
// ADRs relacionados: 0022 (Frontend), 0006 (RBAC), 0021 (OpenAPI)

import { api } from "./client";
import type { User, UserCreateInput, UserUpdateInput } from "$lib/types/user";

interface ListUsersResponse {
	users: User[];
	total: number;
	page: number;
	perPage: number;
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
	const searchParams = new URLSearchParams();
	if (params.page) searchParams.set("page", params.page.toString());
	if (params.perPage) searchParams.set("per_page", params.perPage.toString());
	if (params.search) searchParams.set("search", params.search);
	if (params.role) searchParams.set("role", params.role);
	if (params.isActive !== undefined) searchParams.set("is_active", params.isActive.toString());

	const query = searchParams.toString();
	return api.get<ListUsersResponse>(`/users${query ? `?${query}` : ""}`);
}

// Obtener usuario por ID
export async function getUser(id: string): Promise<User> {
	return api.get<User>(`/users/${id}`);
}

// Crear usuario (admin only)
export async function createUser(data: UserCreateInput): Promise<User> {
	return api.post<User>("/users", data);
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
