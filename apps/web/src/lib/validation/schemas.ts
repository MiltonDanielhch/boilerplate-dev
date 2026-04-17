// Ubicación: `apps/web/src/lib/validation/schemas.ts`
//
// Descripción: Schemas de validación usando ArkType para formularios
//              de autenticación y gestión de usuarios.
//              TODO: Sincronizar con contratos OpenAPI del backend.
//
// ADRs relacionados: 0022 (Frontend), 0021 (OpenAPI)

import { type } from "arktype";

// Schema de login - email válido + password mínimo 8 caracteres
export const LoginSchema = type({
	email: "string.email",
	password: "string >= 8"
});

export type LoginInput = typeof LoginSchema.infer;

// Schema de registro - password más fuerte (12+ caracteres)
export const RegisterSchema = type({
	email: "string.email",
	password: "string >= 12",
	name: "string?"
});

export type RegisterInput = typeof RegisterSchema.infer;

// Schema para leads/captación (con honeypot anti-spam)
export const LeadSchema = type({
	email: "string.email",
	name: "string?",
	honeypot: "string?" // Campo oculto para detectar bots
});

export type LeadInput = typeof LeadSchema.infer;

// Schema para creación de usuario (admin)
export const CreateUserSchema = type({
	email: "string.email",
	password: "string >= 12",
	name: "string?",
	role: "'admin' | 'user' | 'moderator'"
});

export type CreateUserInput = typeof CreateUserSchema.infer;

// Schema para actualizar usuario
export const UpdateUserSchema = type({
	email: "string.email?",
	name: "string?",
	isActive: "boolean?",
	role: "'admin' | 'user' | 'moderator'?"
});

export type UpdateUserInput = typeof UpdateUserSchema.infer;
