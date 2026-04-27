-- Migration: create_content_blocks
-- Created: 2026-04-24 18:31:00
-- Descripción: Tabla para CMS básico de bloques de contenido dinámico.

CREATE TABLE IF NOT EXISTS content_blocks (
    key TEXT PRIMARY KEY NOT NULL,
    content TEXT NOT NULL,
    content_type TEXT NOT NULL DEFAULT 'text', -- text, markdown, html
    last_modified_by TEXT, -- UUID del admin
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Semilla inicial para landing page
INSERT OR IGNORE INTO content_blocks (key, content, content_type) VALUES 
('hero_title', 'Revolutionize Your Workflow', 'text'),
('hero_subtitle', 'The ultimate boilerplate for modern applications.', 'text'),
('footer_copy', '© 2026 Antigravity Team. All rights reserved.', 'text');
