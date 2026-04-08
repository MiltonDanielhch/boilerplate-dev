# ADR 0010 — Testing: 4 Capas + cargo-nextest

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0004 (SQLite en memoria para tests de integración) |

---

## Contexto

Sin una estrategia de testing clara, cada capa termina con el mismo tipo de test: tests de
integración pesados que levantan toda la infraestructura. Esto hace el CI lento, los tests
frágiles, y la cobertura de casos límite pobre.

Necesitamos una estrategia que:

- Sea rápida de ejecutar localmente — feedback inmediato para el desarrollador
- Teste cada capa con el nivel de aislamiento correcto
- No requiera Docker ni servicios externos para correr la suite completa
- Detecte regresiones antes de que lleguen a `main`

---

## Decisión

Estrategia de **cuatro capas de testing**, cada una con herramientas y alcance bien definidos.

### Herramientas base

```toml
# Cargo.toml del workspace — dev-dependencies compartidas
[workspace.dev-dependencies]
tokio    = { version = "1", features = ["test", "macros"] }
mockall  = "0.13"
reqwest  = { version = "0.12", features = ["json"] }
httpmock = "0.7"
```

```bash
# Runner 3-5x más rápido que cargo test — paralelismo real
cargo install cargo-nextest
```

---

### Capa 1 — Tests unitarios de dominio

**Dónde:** `crates/domain/src/**`
**Sin:** async, DB, mocks externos, frameworks
**Objetivo:** verificar reglas de negocio de forma exhaustiva y rápida

```rust
// crates/domain/src/value_objects/email.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_valido_se_crea() {
        assert!(Email::new("user@example.com").is_ok());
    }

    #[test]
    fn email_normalizado_a_minusculas() {
        let email = Email::new("User@EXAMPLE.COM").unwrap();
        assert_eq!(email.as_str(), "user@example.com");
    }

    #[test]
    fn email_sin_arroba_invalido() {
        assert!(matches!(
            Email::new("userexample.com").unwrap_err(),
            DomainError::InvalidEmail(_)
        ));
    }

    #[test]
    fn email_sin_dominio_invalido() { assert!(Email::new("user@").is_err()); }

    #[test]
    fn email_vacio_invalido()       { assert!(Email::new("").is_err()); }

    #[test]
    fn soft_delete_marca_usuario_inactivo() {
        let mut user = User::new(
            Email::new("u@example.com").unwrap(),
            PasswordHash::from_hash("h".into()),
        );
        assert!(user.is_active());
        user.soft_delete();
        assert!(!user.is_active());
        assert!(user.deleted_at.is_some());
    }
}
```

---

### Capa 2 — Tests de aplicación con mocks

**Dónde:** `crates/application/src/**`
**Con:** async, mocks con `mockall`, sin DB real
**Objetivo:** verificar orquestación y flujo de casos de uso

```rust
// crates/application/src/use_cases/auth/register.rs
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::{mock, predicate::*};

    mock! {
        UserRepo {}
        impl UserRepository for UserRepo {
            async fn find_active_by_email(&self, email: &Email) -> Result<Option<User>, DomainError>;
            async fn save(&self, user: &User)                   -> Result<(), DomainError>;
        }
    }

    // Helper para mocks compartidos — evita repetición en múltiples tests
    fn fake_hash_fn() -> Arc<dyn Fn(&str) -> Result<String, DomainError> + Send + Sync> {
        Arc::new(|_| Ok("hashed_password".to_string()))
    }

    #[tokio::test]
    async fn registro_con_email_nuevo_funciona() {
        let mut mock = MockUserRepo::new();
        mock.expect_find_active_by_email().once().returning(|_| Ok(None));
        mock.expect_save().once().returning(|_| Ok(()));

        let result = RegisterUseCase::new(Arc::new(mock), fake_hash_fn())
            .execute(RegisterInput {
                email:    "nuevo@example.com".into(),
                password: "contraseña_segura".into(),
            }).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn registro_con_email_duplicado_no_llama_save() {
        let mut mock = MockUserRepo::new();
        mock.expect_find_active_by_email()
            .once()
            .returning(|_| Ok(Some(User::new(
                Email::new("existente@example.com").unwrap(),
                PasswordHash::from_hash("h".into()),
            ))));
        mock.expect_save().never(); // NUNCA debe llamar a save

        let result = RegisterUseCase::new(Arc::new(mock), fake_hash_fn())
            .execute(RegisterInput {
                email:    "existente@example.com".into(),
                password: "contraseña".into(),
            }).await;

        assert!(matches!(result.unwrap_err(), DomainError::EmailAlreadyExists));
    }

    #[tokio::test]
    async fn email_invalido_no_toca_la_db() {
        let mut mock = MockUserRepo::new();
        mock.expect_find_active_by_email().never();
        mock.expect_save().never();

        let result = RegisterUseCase::new(Arc::new(mock), fake_hash_fn())
            .execute(RegisterInput {
                email:    "no-es-email".into(),
                password: "contraseña".into(),
            }).await;

        assert!(matches!(result.unwrap_err(), DomainError::InvalidEmail(_)));
    }
}
```

---

### Capa 3 — Tests de integración con SQLite en memoria

**Dónde:** `crates/database/tests/**`
**Con:** async, SQLite `:memory:` real, sin mocks
**Objetivo:** verificar que el SQL funciona y los repositorios implementan el contrato

```rust
// crates/database/tests/user_repository_test.rs
async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await
        .expect("failed to create test pool");

    sqlx::migrate!("../../data/migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    pool
}

fn fake_user(email: &str) -> User {
    User::new(
        Email::new(email).unwrap(),
        PasswordHash::from_hash("hash".into()),
    )
}

#[tokio::test]
async fn guardar_y_recuperar_usuario() {
    let pool = setup_test_db().await;
    let repo = SqliteUserRepository::new(Arc::new(pool));
    let user = fake_user("test@example.com");

    repo.save(&user).await.unwrap();
    let found = repo.find_active_by_email(&Email::new("test@example.com").unwrap()).await.unwrap();
    assert!(found.is_some());
}

#[tokio::test]
async fn soft_delete_oculta_el_usuario() {
    let pool = setup_test_db().await;
    let repo = SqliteUserRepository::new(Arc::new(pool));
    let user = fake_user("delete@example.com");
    let id   = user.id.clone();

    repo.save(&user).await.unwrap();
    repo.soft_delete(&id).await.unwrap();

    // No aparece en búsquedas activas — sigue en la DB pero invisible
    let found = repo.find_active_by_email(
        &Email::new("delete@example.com").unwrap()
    ).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn has_permission_con_rol_admin() {
    let pool  = setup_test_db().await;
    let repo  = SqliteUserRepository::new(Arc::new(pool));
    let admin = repo.find_active_by_email(
        &Email::new("admin@admin.com").unwrap()
    ).await.unwrap().unwrap();

    // El seed (migración 5) crea admin con todos los permisos
    assert!(repo.has_permission(&admin.id, "users:read").await.unwrap());
    assert!(repo.has_permission(&admin.id, "users:write").await.unwrap());
    assert!(!repo.has_permission(&admin.id, "permiso:inexistente").await.unwrap());
}
```

---

### Capa 4 — Tests E2E con servidor real

**Dónde:** `apps/api/tests/**`
**Con:** servidor completo en puerto aleatorio, `reqwest` como cliente
**Objetivo:** verificar flujos completos de usuario de extremo a extremo

```rust
// apps/api/tests/auth_flow_test.rs
async fn spawn_test_server() -> String {
    let pool = setup_test_db().await;
    let addr = SocketAddr::from(([127, 0, 0, 1], 0)); // Puerto aleatorio

    let app    = build_app(pool.clone(), test_config()).await;
    let server = axum::serve(TcpListener::bind(addr).await.unwrap(), app);
    let port   = server.local_addr().port();

    tokio::spawn(async move { server.await.unwrap() });
    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn flujo_completo_registro_login_acceso_logout() {
    let base_url = spawn_test_server().await;
    let client   = reqwest::Client::new();

    // 1. Registro
    let res = client
        .post(format!("{}/auth/register", base_url))
        .json(&json!({ "email": "e2e@example.com", "password": "Password123!" }))
        .send().await.unwrap();
    assert_eq!(res.status(), 201);

    // 2. Login — el token debe empezar con v4.local. (PASETO, no JWT)
    let body        = res.json::<serde_json::Value>().await.unwrap();
    let access_token = body["access_token"].as_str().unwrap();
    assert!(access_token.starts_with("v4.local."), "El token debe ser PASETO, no JWT");

    // 3. Request autenticado
    let res = client
        .get(format!("{}/api/v1/users/me", base_url))
        .bearer_auth(access_token)
        .send().await.unwrap();
    assert_eq!(res.status(), 200);

    // 4. Logout
    let res = client
        .post(format!("{}/auth/logout", base_url))
        .bearer_auth(access_token)
        .send().await.unwrap();
    assert_eq!(res.status(), 200);

    // 5. Token revocado ya no funciona
    let res = client
        .get(format!("{}/api/v1/users/me", base_url))
        .bearer_auth(access_token)
        .send().await.unwrap();
    assert_eq!(res.status(), 401);
}
```

---

### Configuración de CI

```yaml
# .github/workflows/ci.yml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Install cargo-nextest
        run: cargo install cargo-nextest

      - name: Tests (capas 1-3)
        run: cargo nextest run

      - name: Lint
        run: cargo clippy --all-targets -- -D warnings

      - name: Verificar tipos TypeScript
        run: just types-check

      - name: Verificar SQLX offline
        run: just prepare && git diff --exit-code .sqlx/

      - name: Auditoría de seguridad
        run: |
          cargo install cargo-deny cargo-audit
          cargo deny check
          cargo audit
```

---

## Resumen de capas

| Capa | Ubicación | Velocidad | Aislamiento | Cuándo correr |
|------|-----------|-----------|-------------|---------------|
| 1 — Dominio | `crates/domain/src` | ~ms | Total (sin deps) | `just test` — siempre |
| 2 — Aplicación | `crates/application/src` | ~ms | Mocks | `just test` — siempre |
| 3 — Integración | `crates/database/tests` | ~s | SQLite :memory: | `just test` — siempre |
| 4 — E2E | `apps/api/tests` | ~10s | Servidor real | `just test-all` — solo CI |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para elevar la confianza en el código y automatizar la detección de bugs complejos:

| Herramienta | Propósito en la Calidad |
| :--- | :--- |
| **`cargo-mutants`** | **Mutation Testing:** Modifica tu código para ver si tus tests fallan. Es la prueba definitiva de cobertura real. |
| **`proptest`** | **Property Testing:** Genera cientos de inputs aleatorios para encontrar casos borde en el dominio. |
| **`insta`** | **Snapshot Testing:** Facilita el testeo de respuestas JSON extensas comparándolas contra archivos de referencia. |
| **`cargo-llvm-cov`** | **Cobertura de Código:** Genera reportes visuales de qué líneas de código no están siendo tocadas por tests. |

---

## Consecuencias

### ✅ Positivas

- Tests de dominio corren en milisegundos — feedback inmediato sin levantar nada
- Tests de integración sin Docker — SQLite en memoria es más rápido y simple
- `cargo-nextest` paraleliza la suite — 3-5x más rápido que `cargo test`
- El test E2E verifica que el token es PASETO (`v4.local.`) — detecta si alguien agrega JWT

### ⚠️ Negativas / Trade-offs

- Tests E2E son más lentos (~10s por suite)
  → Solo corren en CI y en `just test-all` — no en el ciclo local de desarrollo
  → `just test` (capas 1-3) corre en <5 segundos — suficiente para feedback local
- `mockall` genera código verboso
  → Crear helpers compartidos en `crates/application/src/test_helpers/mod.rs`
    con los mocks más usados — evita repetición entre archivos de test
  → Un mock de >50 líneas es señal de que el trait tiene demasiadas responsabilidades

### Decisiones derivadas

- `just test` corre capas 1-3 — para desarrollo local
- `just test-all` corre las 4 capas — para CI y antes de releases
- `cargo audit` bloquea el CI si hay vulnerabilidades conocidas en dependencias
- El test E2E del flujo de auth verifica explícitamente que el token comienza con `v4.local.`
