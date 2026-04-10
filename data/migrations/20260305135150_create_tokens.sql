-- Migration: create_tokens
-- Created: 2026-03-05 13:51:50
--
-- Referencia: docs/01-ARCHITECTURE.md L152-153
--
-- Tabla: tokens
-- Features: un solo uso, expiración, verificación email + reset password

CREATE TABLE IF NOT EXISTS tokens (
    id          TEXT PRIMARY KEY NOT NULL,      -- UUID v7
    user_id     TEXT NOT NULL,
    token_hash  TEXT NOT NULL,                  -- hash del token (no plaintext)
    purpose     TEXT NOT NULL,                  -- "email_verification" | "password_reset"
    expires_at  TIMESTAMPTZ NOT NULL,
    used_at     TIMESTAMPTZ,                    -- NULL = no usado
    created_at  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT fk_tokens_user_id
        FOREIGN KEY (user_id) REFERENCES users(id)
        ON DELETE CASCADE,

    -- Solo propósitos válidos
    CONSTRAINT chk_tokens_purpose
        CHECK (purpose IN ('email_verification', 'password_reset'))
);

-- ─── Índices ─────────────────────────────────────────────────────────────────

-- Búsqueda por usuario + propósito (para verificar si tiene token activo)
CREATE INDEX idx_tokens_user_purpose ON tokens(user_id, purpose);

-- Búsqueda por token_hash (para validar token)
CREATE UNIQUE INDEX idx_tokens_hash ON tokens(token_hash);

-- Tokens no usados que expiran pronto (para cleanup job)
CREATE INDEX idx_tokens_expiry_unused
    ON tokens(expires_at)
    WHERE used_at IS NULL;
