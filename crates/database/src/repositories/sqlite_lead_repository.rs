//! Ubicación: `crates/database/src/repositories/sqlite_lead_repository.rs`
//!
//! Descripción: Implementación SQLite del repositorio de leads.
//!              Gestiona la captura y seguimiento de leads desde landing page.
//!
//! ADRs relacionados: ADR 0029 (Landing Page)

use domain::entities::Lead;
use domain::errors::DomainError;
use domain::ports::LeadRepository;
use domain::value_objects::Email;
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
                               ip_address, user_agent, status, is_contacted, 
                               contact_notes, contacted_at, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
            "#
        )
        .bind(&lead.id)
        .bind(&lead.name)
        .bind(&lead.email.to_string())
        .bind(&lead.phone)
        .bind(&lead.company)
        .bind(&lead.message)
        .bind(&lead.source)
        .bind(&lead.utm_source)
        .bind(&lead.utm_medium)
        .bind(&lead.utm_campaign)
        .bind(&lead.ip_address)
        .bind(&lead.user_agent)
        .bind(&lead.status)
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
                   ip_address, user_agent, status, is_contacted,
                   contact_notes, contacted_at, created_at
            FROM leads WHERE email = ?1
            "#
        )
        .bind(email)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(r.into_lead()?)),
            None => Ok(None),
        }
    }

    /// Buscar lead por ID
    async fn find_by_id(&self, id: &str) -> Result<Option<Lead>, DomainError> {
        let row = sqlx::query_as::<_, LeadRow>(
            r#"
            SELECT id, name, email, phone, company, message,
                   source, utm_source, utm_medium, utm_campaign,
                   ip_address, user_agent, status, is_contacted,
                   contact_notes, contacted_at, created_at
            FROM leads WHERE id = ?1
            "#
        )
        .bind(id)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(r.into_lead()?)),
            None => Ok(None),
        }
    }

    async fn list(
        &self, 
        limit: i64, 
        offset: i64,
        search: Option<String>,
        status: Option<String>,
        from_date: Option<time::OffsetDateTime>,
        to_date: Option<time::OffsetDateTime>
    ) -> Result<Vec<Lead>, DomainError> {
        let mut query = String::from(
            r#"
            SELECT id, name, email, phone, company, message,
                   source, utm_source, utm_medium, utm_campaign,
                   ip_address, user_agent, status, is_contacted,
                   contact_notes, contacted_at, created_at
            FROM leads
            WHERE 1=1
            "#
        );

        if let Some(s) = &search {
            if !s.is_empty() {
                query.push_str("AND (email LIKE ? OR name LIKE ?) ");
            }
        }

        if let Some(st) = &status {
            if !st.is_empty() {
                query.push_str("AND status = ? ");
            }
        }

        if from_date.is_some() {
            query.push_str("AND created_at >= ? ");
        }

        if to_date.is_some() {
            query.push_str("AND created_at <= ? ");
        }

        query.push_str("ORDER BY created_at DESC LIMIT ? OFFSET ?");

        let mut sql_query = sqlx::query_as::<_, LeadRow>(&query);

        if let Some(s) = search {
            if !s.is_empty() {
                let pattern = format!("%{}%", s);
                sql_query = sql_query.bind(pattern.clone()).bind(pattern);
            }
        }

        if let Some(st) = status {
            if !st.is_empty() {
                sql_query = sql_query.bind(st);
            }
        }

        if let Some(fd) = from_date {
            sql_query = sql_query.bind(fd);
        }

        if let Some(td) = to_date {
            sql_query = sql_query.bind(td);
        }

        sql_query = sql_query.bind(limit).bind(offset);

        let rows = sql_query
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into_lead()).collect::<Result<Vec<_>, _>>()?)
    }

    async fn set_status(&self, id: &str, status: &str) -> Result<(), DomainError> {
        sqlx::query("UPDATE leads SET status = ? WHERE id = ?")
            .bind(status)
            .bind(id)
            .execute(&*self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;
        Ok(())
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

    async fn get_counts_by_date(&self, days: i64) -> Result<Vec<(String, i64)>, DomainError> {
        let rows = sqlx::query_as::<_, (String, i64)>(
            r#"
            WITH RECURSIVE dates(date) AS (
                SELECT DATE('now', '-' || (? - 1) || ' days')
                UNION ALL
                SELECT DATE(date, '+1 day') FROM dates WHERE date < DATE('now')
            )
            SELECT 
                d.date,
                COUNT(l.id) as count
            FROM dates d
            LEFT JOIN leads l ON DATE(l.created_at) = d.date
            GROUP BY d.date
            ORDER BY d.date ASC
            "#
        )
        .bind(days)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(rows)
    }

    async fn get_counts_by_status(&self) -> Result<Vec<(String, i64)>, DomainError> {
        let rows = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT status, COUNT(*) as count
            FROM leads
            GROUP BY status
            ORDER BY count DESC
            "#
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(rows)
    }
}

/// Estructura intermedia para SQLx
#[derive(sqlx::FromRow)]
pub struct LeadRow {
    pub id: String,
    pub name: Option<String>,
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
    status: String,
    is_contacted: bool,
    contact_notes: Option<String>,
    contacted_at: Option<time::OffsetDateTime>,
    created_at: time::OffsetDateTime,
}

impl LeadRow {
    fn into_lead(self) -> Result<Lead, DomainError> {
        let email = Email::new(&self.email)
            .map_err(|e| DomainError::Validation(format!("Invalid email in database: {e}")))?;
        
        Ok(Lead {
            id: self.id,
            name: self.name,
            email,
            phone: self.phone,
            company: self.company,
            message: self.message,
            source: self.source,
            utm_source: self.utm_source,
            utm_medium: self.utm_medium,
            utm_campaign: self.utm_campaign,
            ip_address: self.ip_address,
            user_agent: self.user_agent,
            status: self.status,
            is_contacted: self.is_contacted,
            contact_notes: self.contact_notes,
            contacted_at: self.contacted_at,
            created_at: self.created_at,
        })
    }
}
