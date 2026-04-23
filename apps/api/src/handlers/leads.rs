// Ubicación: `apps/api/src/handlers/leads.rs`
//
// Descripción: Handler para captura de leads desde landing page.
//
// ADRs relacionados: ADR 0003 (Axum), ADR 0029 (Landing + Leads)

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use axum::{
    extract::{ConnectInfo, State},
    response::Json,
};
use domain::entities::Lead;
use domain::ports::LeadRepository;
use domain::value_objects::Email;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing::{info, warn};
use uuid::Uuid;

/// POST /api/v1/leads — Captura lead (rate limit 3/min en prod)
pub async fn capture(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(body): Json<CaptureLeadRequest>,
) -> ApiResult<Json<CaptureLeadResponse>> {
    info!(email = %body.email, "Lead capture request received");

    // Validar email
    if !is_valid_email(&body.email) {
        return Err(ApiError::BadRequest(
            "Invalid email format".to_string()
        ));
    }

    // Normalizar email (lowercase)
    let email_str = body.email.to_lowercase();

    // Verificar si ya existe un lead con este email
    match state.lead_repo.find_by_email(&email_str).await {
        Ok(Some(_existing)) => {
            // Lead ya existe, actualizar o ignorar según política de negocio
            // Por ahora devolvemos éxito (idempotencia)
            info!(email = %email_str, "Lead already exists, returning success");
            return Ok(Json(CaptureLeadResponse {
                success: true,
                message: "Thank you! We'll be in touch soon.".to_string(),
                lead_id: None,
            }));
        }
        Ok(None) => {
            // No existe, continuar con la creación
        }
        Err(e) => {
            warn!(email = %email_str, error = %e, "Failed to check existing lead");
            return Err(ApiError::Internal(
                "Failed to process lead".to_string()
            ));
        }
    }

    // Crear Email value object
    let email = Email::new(&email_str)
        .map_err(|_| ApiError::BadRequest("Invalid email format".to_string()))?;

    // Crear nuevo lead
    let lead = Lead {
        id: Uuid::new_v4().to_string(),
        name: body.name,
        email,
        phone: body.phone,
        company: body.company,
        message: body.message,
        source: body.source,
        utm_source: body.utm_source,
        utm_medium: body.utm_medium,
        utm_campaign: body.utm_campaign,
        ip_address: Some(addr.ip().to_string()),
        user_agent: body.user_agent,
        is_contacted: false,
        contact_notes: None,
        contacted_at: None,
        created_at: time::OffsetDateTime::now_utc(),
    };

    // Guardar en DB
    match state.lead_repo.save(&lead).await {
        Ok(()) => {
            info!(lead_id = %lead.id, email = %lead.email, "Lead captured successfully");
            Ok(Json(CaptureLeadResponse {
                success: true,
                message: "Thank you! We'll be in touch soon.".to_string(),
                lead_id: Some(lead.id),
            }))
        }
        Err(e) => {
            warn!(email = %lead.email, error = %e, "Failed to save lead");
            Err(ApiError::Internal(
                "Failed to process lead".to_string()
            ))
        }
    }
}

/// Validación simple de email
fn is_valid_email(email: &str) -> bool {
    // Validación básica: contiene @ y un dominio válido
    let email = email.trim();
    if email.len() < 5 || email.len() > 254 {
        return false;
    }
    
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    
    let local = parts[0];
    let domain = parts[1];
    
    // Validar local part
    if local.is_empty() || local.len() > 64 {
        return false;
    }
    
    // Validar domain
    if domain.is_empty() || !domain.contains('.') {
        return false;
    }
    
    true
}

#[derive(Debug, Deserialize)]
pub struct CaptureLeadRequest {
    pub email: String,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub message: Option<String>,
    pub source: Option<String>,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_campaign: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CaptureLeadResponse {
    pub success: bool,
    pub message: String,
    pub lead_id: Option<String>,
}
