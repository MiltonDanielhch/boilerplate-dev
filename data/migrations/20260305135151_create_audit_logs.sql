-- Migration: create_audit_logs
-- Created: 2026-03-05 13:51:51
--
-- Referencia: ADR 0006, docs/01-ARCHITECTURE.md L155-157
--
-- Tabla: audit_logs
-- Features: Insert-only, FK opcional (SET NULL), índices de búsqueda

CREATE TABLE IF NOT EXISTS audit_logs (
    id          TEXT PRIMARY KEY NOT NULL,      -- UUID v7
    user_id     TEXT,                           -- Nullable (anon allowed)
    action      TEXT NOT NULL,                  -- ej: "users:create", "auth:login"
    resource    TEXT NOT NULL,                  -- ej: "users", "roles"
    resource_id TEXT,                           -- ID del recurso afectado
    details     TEXT,                           -- JSON con cambios (old/new)
    ip_address  TEXT,                           -- IPv4 o IPv6
    user_agent  TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- FK opcional: si el usuario se elimina, el log queda con NULL
    CONSTRAINT fk_audit_logs_user_id
        FOREIGN KEY (user_id) REFERENCES users(id)
        ON DELETE SET NULL
);

-- ─── Índices ─────────────────────────────────────────────────────────────────

-- Búsqueda por recurso + ID (qué pasó con X entidad)
CREATE INDEX idx_audit_resource_search
    ON audit_logs(resource, resource_id, created_at);

-- Historial de un usuario (qué hizo X usuario)
CREATE INDEX idx_audit_user_history
    ON audit_logs(user_id, created_at);

-- Búsqueda por tipo de acción (todos los logins, todos los create de users)
CREATE INDEX idx_audit_action ON audit_logs(action, created_at);

-- Búsqueda temporal (logs de hoy, esta semana)
CREATE INDEX idx_audit_created_at ON audit_logs(created_at);
