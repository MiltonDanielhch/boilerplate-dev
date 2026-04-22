-- Migration: Crear tabla leads para landing page
-- Ubicación: data/migrations/20260422085600_create_leads_table.sql
-- Descripción: Almacena leads capturados desde la landing page con tracking UTM
-- ADRs: ADR 0029 (Landing Page)

-- Tabla principal de leads
CREATE TABLE IF NOT EXISTS leads (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    phone TEXT,
    company TEXT,
    message TEXT,

    -- Tracking UTM para análisis de campañas
    source TEXT,
    utm_source TEXT,
    utm_medium TEXT,
    utm_campaign TEXT,

    -- Metadata de captura
    ip_address TEXT,
    user_agent TEXT,

    -- Estado de contacto (CRM interno)
    is_contacted BOOLEAN NOT NULL DEFAULT FALSE,
    contact_notes TEXT,
    contacted_at TIMESTAMP,

    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Índices para búsquedas comunes
CREATE INDEX IF NOT EXISTS idx_leads_email ON leads(email);
CREATE INDEX IF NOT EXISTS idx_leads_created_at ON leads(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_leads_is_contacted ON leads(is_contacted);
CREATE INDEX IF NOT EXISTS idx_leads_utm_source ON leads(utm_source);
CREATE INDEX IF NOT EXISTS idx_leads_source ON leads(source);

-- Evitar duplicados de email (mismo lead puede enviar múltiples veces pero lo detectamos)
CREATE UNIQUE INDEX IF NOT EXISTS idx_leads_email_unique ON leads(email);
