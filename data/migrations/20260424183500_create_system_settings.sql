-- Migration: create_system_settings
-- Created: 2026-04-24 18:35:00
-- Descripción: Tabla para configuraciones globales del sistema.

CREATE TABLE IF NOT EXISTS system_settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    description TEXT
);

INSERT OR IGNORE INTO system_settings (key, value, description) VALUES 
('site_name', 'My Premium App', 'Nombre del sitio visible en correos y títulos'),
('allow_registration', 'true', 'Habilitar o deshabilitar nuevos registros'),
('maintenance_mode', 'false', 'Activar modo mantenimiento en toda la plataforma');
