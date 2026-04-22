//! Ubicación: `crates/database/src/repositories/sqlite_lead_repository.rs`
//!
//! Descripción: Implementación SQLite del repositorio de leads.
//!              Gestiona la captura y seguimiento de leads desde landing page.
//!
//! ADRs relacionados: ADR 0029 (Landing Page)

use domain::entities::Lead;
use domain::errors::DomainError;
use domain::ports::LeadRepository;
use sqlx::SqlitePool;
use std::sync::Arc;

/// Repositorio SQLite para leads
#[derive(Debug, Clone)]
pub struct SqliteLeadRepository {
    pool: Arc<SqlitePool>,
}

impl SqliteLeadRepository {
    /// Crear nueva instancia del repositorio
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

impl LeadRepository for SqliteLeadRepository {
    /// Guardar un nuevo lead
    async fn save(&self, lead: &Lead) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO leads (id, name, email, phone, company, message, 
                               source, utm_source, utm_medium, utm_campaign,
                               ip_address, user_agent, is_contacted, 
                               contact_notes, contacted_at, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
            "#
        )
        .bind(&lead.id)
        .bind(&lead.name)
        .bind(&lead.email)
        .bind(&lead.phone)
        .bind(&lead.company)
        .bind(&lead.message)
        .bind(&lead.source)
        .bind(&lead.utm_source)
        .bind(&lead.utm_medium)
        .bind(&lead.utm_campaign)
        .bind(&lead.ip_address)
        .bind(&lead.user_agent)
        .bind(lead.is_contacted)
        .bind(&lead.contact_notes)
        .bind(lead.contacted_at)
        .bind(lead.created_at)
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    /// Buscar lead por email
    async fn find_by_email(&self, email: &str) -> Result<Option<Lead>, DomainError> {
        let row = sqlx::query_as::<_, LeadRow>(
            r#"
            SELECT id, name, email, phone, company, message,
                   source, utm_source, utm_medium, utm_campaign,
                   ip_address, user_agent, is_contacted,
                   contact_notes, contacted_at, created_at
            FROM leads WHERE email = ?1
            "#
        )
        .bind(email)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(row.map(|r| r.into()))
    }

    /// Buscar lead por ID
    async fn find_by_id(&self, id: &str) -> Result<Option<Lead>, DomainError> {
        let row = sqlx::query_as::<_, LeadRow>(
            r#"
            SELECT id, name, email, phone, company, message,
                   source, utm_source, utm_medium, utm_campaign,
                   ip_address, user_agent, is_contacted,
                   contact_notes, contacted_at, created_at
            FROM leads WHERE id = ?1
            "#
        )
        .bind(id)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(row.map(|r| r.into()))
    }

    /// Listar leads con paginación
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Lead>, DomainError> {
        let rows = sqlx::query_as::<_, LeadRow>(
            r#"
            SELECT id, name, email, phone, company, message,
                   source, utm_source, utm_medium, utm_campaign,
                   ip_address, user_agent, is_contacted,
                   contact_notes, contacted_at, created_at
            FROM leads
            ORDER BY created_at DESC
            LIMIT ?1 OFFSET ?2
            "#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// Marcar lead como contactado
    async fn mark_contacted(&self, id: &str, notes: Option<&str>) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            UPDATE leads
            SET is_contacted = TRUE, contact_notes = ?1, contacted_at = datetime('now')
            WHERE id = ?2
            "#
        )
        .bind(notes)
        .bind(id)
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }
}

/// Estructura intermedia para SQLx
#[derive(sqlx::FromRow)]
struct LeadRow {
    id: String,
    name: String,
    email: String,
    phone: Option<String>,
    company: Option<String>,
    message: Option<String>,
    source: Option<String>,
    utm_source: Option<String>,
    utm_medium: Option<String>,
    utm_campaign: Option<String>,
    ip_address: Option<String>,
    user_agent: Option<String>,
    is_contacted: bool,
    contact_notes: Option<String>,
    contacted_at: Option<time::OffsetDateTime>,
    created_at: time::OffsetDateTime,
}

impl From<LeadRow> for Lead {
    fn from(row: LeadRow) -> Self {
        Lead {
            id: row.id,
            name: row.name,
            email: row.email,
            phone: row.phone,
            company: row.company,
            message: row.message,
            source: row.source,
            utm_source: row.utm_source,
            utm_medium: row.utm_medium,
            utm_campaign: row.utm_campaign,
            ip_address: row.ip_address,
            user_agent: row.user_agent,
            is_contacted: row.is_contacted,
            contact_notes: row.contact_notes,
            contacted_at: row.contacted_at,
            created_at: row.created_at,
        }
    }
}
