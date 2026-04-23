-- Migration: add_revoked_at_to_sessions
-- Created: 2026-04-23 08:00:00
--
-- Agrega columna revoked_at para tracking de revocación con timestamp

ALTER TABLE sessions ADD COLUMN revoked_at TIMESTAMP;

-- Index para búsqueda de sesiones revocadas
CREATE INDEX idx_sessions_revoked_at ON sessions(revoked_at) WHERE revoked_at IS NOT NULL;
