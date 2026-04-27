-- Migration: add_status_to_leads
-- Created: 2026-04-24 18:28:00
-- Descripción: Añade columna status a la tabla leads para gestión en admin panel.

ALTER TABLE leads ADD COLUMN status TEXT NOT NULL DEFAULT 'new';
CREATE INDEX IF NOT EXISTS idx_leads_status ON leads(status);
