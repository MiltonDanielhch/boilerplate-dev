-- Migration: create_rbac
-- Created: 2026-03-05 13:51:49
--
-- Referencia: ADR 0006 (RBAC), docs/01-ARCHITECTURE.md L149-151
--
-- Tablas: roles, permissions, role_permissions
-- Features: Soft Delete en roles, permisos formato "recurso:acción"

-- ─── Tabla roles ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS roles (
    id          TEXT PRIMARY KEY NOT NULL,      -- UUID v7
    name        TEXT NOT NULL,                  -- ej: "Admin", "User"
    description TEXT,
    is_system   BOOLEAN NOT NULL DEFAULT FALSE, -- TRUE = no se puede eliminar
    created_at  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at  TIMESTAMPTZ                     -- Soft Delete (ADR 0006)
);

-- UNIQUE INDEX parcial para nombre de rol (excluye eliminados)
CREATE UNIQUE INDEX idx_roles_name_active
    ON roles(name)
    WHERE deleted_at IS NULL;

-- Trigger updated_at
CREATE TRIGGER IF NOT EXISTS trg_roles_updated_at
    AFTER UPDATE ON roles
    FOR EACH ROW
    WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE roles SET updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
END;

-- ─── Tabla permissions ───────────────────────────────────────────────────────
-- Formato: "recurso:acción" (ej: "users:read", "users:write")
-- Ref: ADR 0006 — permisos explícitos y auditables
CREATE TABLE IF NOT EXISTS permissions (
    id          TEXT PRIMARY KEY NOT NULL,      -- UUID v7
    resource    TEXT NOT NULL,                  -- ej: "users", "roles", "audit"
    action      TEXT NOT NULL,                  -- ej: "read", "write", "delete"
    created_at  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT uq_permissions_resource_action
        UNIQUE (resource, action)
);

-- Índice para búsqueda por recurso
CREATE INDEX idx_permissions_resource ON permissions(resource);

-- ─── Tabla role_permissions (relación N:M) ───────────────────────────────────
CREATE TABLE IF NOT EXISTS role_permissions (
    role_id       TEXT NOT NULL,
    permission_id TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (role_id, permission_id),

    CONSTRAINT fk_role_permissions_role_id
        FOREIGN KEY (role_id) REFERENCES roles(id)
        ON DELETE CASCADE,

    CONSTRAINT fk_role_permissions_permission_id
        FOREIGN KEY (permission_id) REFERENCES permissions(id)
        ON DELETE CASCADE
);

-- Índice para búsqueda inversa
CREATE INDEX idx_role_permissions_permission_id ON role_permissions(permission_id);
