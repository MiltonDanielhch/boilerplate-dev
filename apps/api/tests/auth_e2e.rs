// Ubicación: `apps/api/tests/auth_e2e.rs`
//
// Descripción: Tests E2E del flujo completo de autenticación.
//              Register → Login → Access → Logout.
//
// ADRs relacionados: ADR 0006 (RBAC), ADR 0008 (PASETO), ADR 0010 (Testing)

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

// ─── Request/Response Types ───────────────────────────────────────────────

#[derive(Serialize)]
struct RegisterRequest {
    email: String,
    password: String,
    name: Option<String>,
}

#[derive(Deserialize)]
struct RegisterResponse {
    id: String,
    email: String,
}

#[derive(Serialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
}

#[derive(Deserialize)]
struct UserListResponse {
    users: Vec<UserSummary>,
    total: u64,
}

#[derive(Deserialize)]
struct UserSummary {
    id: String,
    email: String,
    name: Option<String>,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

// ─── Test Setup ───────────────────────────────────────────────────────────

/// Inicia el servidor en un puerto aleatorio para tests.
async fn spawn_test_server() -> SocketAddr {
    use api::setup::{build_state, load_config};
    use database::pool::create_pool;

    let config = load_config().expect("Failed to load config");
    let pool = create_pool(&config.database_url).await;
    let state = build_state(pool, config);
    let app = api::router::create_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind test server");
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("Server failed");
    });

    // Esperar a que el servidor esté listo
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    addr
}

// ─── Tests ────────────────────────────────────────────────────────────────

/// Flujo completo: Register → Login → Access → Logout
#[tokio::test]
async fn test_auth_flow_complete() {
    let addr = spawn_test_server().await;
    let base_url = format!("http://{}", addr);
    let client = reqwest::Client::new();

    let test_email = format!("test-{}@example.com", uuid::Uuid::new_v4());
    let test_password = "password123";

    // 1. Register
    let register_res = client
        .post(&format!("{}/auth/register", base_url))
        .json(&RegisterRequest {
            email: test_email.clone(),
            password: test_password.to_string(),
            name: Some("Test User".to_string()),
        })
        .send()
        .await
        .expect("Register request failed");

    assert_eq!(
        register_res.status(),
        201,
        "Register should return 201 Created"
    );

    let register_body: RegisterResponse = register_res
        .json()
        .await
        .expect("Failed to parse register response");
    assert_eq!(register_body.email, test_email);
    assert!(!register_body.id.is_empty());

    // 2. Login
    let login_res = client
        .post(&format!("{}/auth/login", base_url))
        .json(&LoginRequest {
            email: test_email.clone(),
            password: test_password.to_string(),
        })
        .send()
        .await
        .expect("Login request failed");

    assert_eq!(login_res.status(), 200, "Login should return 200 OK");

    let login_body: LoginResponse = login_res
        .json()
        .await
        .expect("Failed to parse login response");
    assert!(!login_body.access_token.is_empty());
    assert!(!login_body.refresh_token.is_empty());
    assert!(login_body.expires_in > 0);

    // 3. Verificar que el token es PASETO (empieza con "v4.local.")
    assert!(
        login_body.access_token.starts_with("v4.local."),
        "Token should be PASETO v4.local, got: {}",
        &login_body.access_token[..20.min(login_body.access_token.len())]
    );

    // 4. Access protected route with token
    let users_res = client
        .get(&format!("{}/api/v1/users", base_url))
        .header("Authorization", format!("Bearer {}", login_body.access_token))
        .send()
        .await
        .expect("Users request failed");

    // Debería fallar con 403 porque el usuario nuevo no tiene permisos
    // (solo el admin tiene permisos por defecto)
    assert_eq!(
        users_res.status(),
        403,
        "New user should get 403 Forbidden (no permissions)"
    );

    // 5. Access with invalid token should return 401
    let invalid_res = client
        .get(&format!("{}/api/v1/users", base_url))
        .header("Authorization", "Bearer invalid.token.here")
        .send()
        .await
        .expect("Invalid token request failed");

    assert_eq!(
        invalid_res.status(),
        401,
        "Invalid token should return 401 Unauthorized"
    );

    // 6. Logout
    let logout_res = client
        .post(&format!("{}/auth/logout", base_url))
        .header("Authorization", format!("Bearer {}", login_body.access_token))
        .send()
        .await
        .expect("Logout request failed");

    assert_eq!(logout_res.status(), 200, "Logout should return 200 OK");
}

/// Test: Sin token, las rutas protegidas retornan 401
#[tokio::test]
async fn test_protected_routes_require_auth() {
    let addr = spawn_test_server().await;
    let base_url = format!("http://{}", addr);
    let client = reqwest::Client::new();

    // Sin header Authorization
    let res = client
        .get(&format!("{}/api/v1/users", base_url))
        .send()
        .await
        .expect("Request failed");

    assert_eq!(res.status(), 401, "Missing auth should return 401");

    let body: ErrorResponse = res.json().await.expect("Failed to parse error");
    assert_eq!(body.error, "unauthorized");
}

/// Test: Rutas públicas funcionan sin autenticación
#[tokio::test]
async fn test_public_routes_no_auth_required() {
    let addr = spawn_test_server().await;
    let base_url = format!("http://{}", addr);
    let client = reqwest::Client::new();

    // Health check
    let res = client
        .get(&format!("{}/health", base_url))
        .send()
        .await
        .expect("Health request failed");

    assert_eq!(res.status(), 200, "Health should be public");
}

/// Test: Admin puede acceder a rutas protegidas
#[tokio::test]
async fn test_admin_can_access_protected_routes() {
    let addr = spawn_test_server().await;
    let base_url = format!("http://{}", addr);
    let client = reqwest::Client::new();

    // Login como admin (seeded en migraciones)
    let login_res = client
        .post(&format!("{}/auth/login", base_url))
        .json(&LoginRequest {
            email: "admin@admin.com".to_string(),
            password: "12345678".to_string(),
        })
        .send()
        .await
        .expect("Admin login failed");

    assert_eq!(login_res.status(), 200, "Admin login should succeed");

    let login_body: LoginResponse = login_res
        .json()
        .await
        .expect("Failed to parse login response");

    // Admin debería poder listar usuarios (tiene users:read)
    let users_res = client
        .get(&format!("{}/api/v1/users", base_url))
        .header("Authorization", format!("Bearer {}", login_body.access_token))
        .send()
        .await
        .expect("Users request failed");

    assert_eq!(
        users_res.status(),
        200,
        "Admin should access protected routes"
    );
}
