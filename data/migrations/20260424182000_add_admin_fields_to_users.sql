-- Migration: add_admin_fields_to_users
-- Created: 2026-04-24 18:20:00
--
-- Descripción: Añade campos para rastreo de admin y login en la tabla users.

ALTER TABLE users ADD COLUMN last_login_at TIMESTAMPTZ;
ALTER TABLE users ADD COLUMN created_by TEXT; -- UUID del admin que lo creó
