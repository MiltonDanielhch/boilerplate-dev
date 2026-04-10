-- Migration: seed_system_data
-- Created: 2026-03-05 13:51:52
--
-- Referencia: docs/01-ARCHITECTURE.md L158-161, ADR 0006
--
-- Datos iniciales del sistema: admin user, roles base, permisos base
-- ⚠️ CAMBIAR PASSWORD ANTES DEL PRIMER DEPLOY (ADR 0002)

-- ─── Permisos base del sistema ─────────────────────────────────────────────
INSERT OR IGNORE INTO permissions (id, resource, action) VALUES
    ('perm-001-users-read',  'users',  'read'),
    ('perm-002-users-write', 'users',  'write'),
    ('perm-003-users-delete','users',  'delete'),
    ('perm-004-roles-read',  'roles',  'read'),
    ('perm-005-roles-write', 'roles',  'write'),
    ('perm-006-audit-read',  'audit',  'read');

-- ─── Roles base ──────────────────────────────────────────────────────────────
INSERT INTO roles (id, name, description, is_system)
SELECT 'role-001-admin', 'Admin', 'Administrador del sistema con acceso total', TRUE
WHERE NOT EXISTS (SELECT 1 FROM roles WHERE name = 'Admin' AND deleted_at IS NULL);

INSERT INTO roles (id, name, description, is_system)
SELECT 'role-002-user', 'User', 'Usuario estándar del sistema', FALSE
WHERE NOT EXISTS (SELECT 1 FROM roles WHERE name = 'User' AND deleted_at IS NULL);

-- ─── Role-Permissions: Admin recibe TODOS los permisos ─────────────────────
-- Usamos CROSS JOIN para asignar todos los permisos existentes al rol Admin
-- SQLite: INSERT OR IGNORE en lugar de ON CONFLICT
INSERT OR IGNORE INTO role_permissions (role_id, permission_id)
    SELECT 'role-001-admin', id FROM permissions;

-- ─── Usuario admin (password: 12345678) ────────────────────────────────────
-- ⚠️ CAMBIAR ANTES DEL PRIMER DEPLOY (ADR 0002)
-- Hash generado con: echo -n "12345678" | argon2 ...
-- SQLite: WHERE NOT EXISTS para partial unique index
INSERT INTO users (id, email, password_hash, name, is_active, email_verified_at)
SELECT
    'usr-001-admin',
    'admin@admin.com',
    '$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$hash_placeholder',  -- ⚠️ CAMBIAR
    'Administrator',
    TRUE,
    CURRENT_TIMESTAMP
WHERE NOT EXISTS (
    SELECT 1 FROM users WHERE email = 'admin@admin.com' AND deleted_at IS NULL
);

-- ─── Asignar rol Admin al usuario admin ──────────────────────────────────────
INSERT OR IGNORE INTO user_roles (user_id, role_id) VALUES
    ('usr-001-admin', 'role-001-admin');
