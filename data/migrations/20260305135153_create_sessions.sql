-- Migration: create_sessions
-- Created: 2026-03-05 13:51:53
--
-- Referencia: docs/01-ARCHITECTURE.md L162-164
--
-- Tabla: sessions
-- Features: tracking de dispositivos, revocación, expiración automática

CREATE TABLE IF NOT EXISTS sessions (
    id              TEXT PRIMARY KEY NOT NULL,  -- UUID v7 (session ID)
    user_id         TEXT NOT NULL,
    refresh_token_hash TEXT NOT NULL,           -- hash del refresh token (PASETO)
    ip_address      TEXT,                       -- última IP conocida
    user_agent      TEXT,                       -- último UA conocido
    is_revoked      BOOLEAN NOT NULL DEFAULT FALSE,
    expires_at      TIMESTAMPTZ NOT NULL,       -- cuando expira el refresh token
    last_activity_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT fk_sessions_user_id
        FOREIGN KEY (user_id) REFERENCES users(id)
        ON DELETE CASCADE
);

-- ─── Índices ─────────────────────────────────────────────────────────────────

-- Búsqueda por refresh token (para validar)
CREATE UNIQUE INDEX idx_sessions_token ON sessions(refresh_token_hash);

-- Búsqueda por usuario (listar sesiones activas)
CREATE INDEX idx_sessions_user_id ON sessions(user_id, created_at DESC);

-- Índice parcial: sesiones activas (no revocadas, no expiradas)
-- Usado por jobs de cleanup y para contar "sesiones activas"
CREATE INDEX idx_sessions_active
    ON sessions(user_id, expires_at)
    WHERE is_revoked = FALSE;

-- ─── Trigger: actualizar last_activity_at ────────────────────────────────────
-- Este trigger se puede llamar explícitamente desde la app al verificar token
CREATE TRIGGER IF NOT EXISTS trg_sessions_activity
    AFTER UPDATE ON sessions
    FOR EACH ROW
    WHEN OLD.last_activity_at = NEW.last_activity_at
BEGIN
    UPDATE sessions SET last_activity_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
END;
