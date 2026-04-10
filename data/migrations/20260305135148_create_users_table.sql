-- Migration: create_users_table
-- Created: 2026-03-05 13:51:48
--
-- Referencia: ADR 0006 (RBAC Soft Delete), docs/01-ARCHITECTURE.md L145-148
--
-- Tablas: users, user_roles
-- Features: Soft Delete, UNIQUE INDEX parcial, trigger updated_at

-- ─── Tabla users ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS users (
    id              TEXT PRIMARY KEY NOT NULL,  -- UUID v7
    email           TEXT NOT NULL,
    password_hash   TEXT NOT NULL,              -- argon2id
    name            TEXT,
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    email_verified_at TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at      TIMESTAMPTZ,                -- Soft Delete (ADR 0006)

    -- Constraints
    CONSTRAINT chk_email_format CHECK (email LIKE '%_@__%.__%')
);

-- ─── UNIQUE INDEX parcial en email (excluye soft-deleted) ────────────────────
-- Ref: ADR 0006 — permite reutilizar email de usuarios eliminados
CREATE UNIQUE INDEX idx_users_email_active
    ON users(email)
    WHERE deleted_at IS NULL;

-- ─── Trigger para auto-actualizar updated_at ─────────────────────────────────
CREATE TRIGGER IF NOT EXISTS trg_users_updated_at
    AFTER UPDATE ON users
    FOR EACH ROW
    WHEN OLD.updated_at = NEW.updated_at  -- evita loop infinito
BEGIN
    UPDATE users SET updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
END;

-- ─── Tabla user_roles (relación N:M) ─────────────────────────────────────────
CREATE TABLE IF NOT EXISTS user_roles (
    user_id     TEXT NOT NULL,
    role_id     TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (user_id, role_id),

    CONSTRAINT fk_user_roles_user_id
        FOREIGN KEY (user_id) REFERENCES users(id)
        ON DELETE CASCADE,

    CONSTRAINT fk_user_roles_role_id
        FOREIGN KEY (role_id) REFERENCES roles(id)
        ON DELETE CASCADE
);

-- Índice para búsqueda inversa (roles → usuarios)
CREATE INDEX idx_user_roles_role_id ON user_roles(role_id);
