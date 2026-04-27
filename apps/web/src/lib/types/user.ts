// Ubicación: `apps/web/src/lib/types/user.ts`
//
// Descripción: Tipos TypeScript para entidades de usuario.
//              TODO: Generar automáticamente desde OpenAPI spec.
//
// ADRs relacionados: 0021 (OpenAPI), 0006 (RBAC)

export interface User {
	id: string;
	email: string;
	name: string | null;
	role: "admin" | "user" | "moderator";
	isActive: boolean;
	emailVerifiedAt: string | null;
	lastLoginAt: string | null;
	createdBy: string | null;
	permissions?: string[];
	createdAt: string;
	updatedAt: string;
	deletedAt?: string | null; // Soft delete (ADR 0006)
}

export interface UserCreateInput {
	email: string;
	password: string;
	name?: string;
	role?: "admin" | "user" | "moderator";
}

export interface UserUpdateInput {
	email?: string;
	name?: string;
	isActive?: boolean;
	role?: "admin" | "user" | "moderator";
}

export interface LoginResponse {
	access_token: string;
	refresh_token: string;
	user: User;
}

export interface RegisterInput {
	email: string;
	password: string;
	name?: string;
}
