# Roadmap — Auth Fullstack (Back + Front juntos)

> **Este documento cubre el flujo completo de autenticación end-to-end.**
> Back y Front coordinados para que el login, registro y dashboard funcionen juntos.
>
> **Pre-requisitos:**
> - Backend Bloques I, II, III completados (`ROADMAP-BACKEND.md`)
> - Frontend FE.I, FE.II, FE.III completados (`ROADMAP-FRONTEND.md`)
>
> **ADRs:** 0001 · 0003 · 0006 · 0007 · 0008 · 0021 · 0022

---

## Estados

```
[ ] Pendiente   [~] En progreso   [x] Completado   [!] Bloqueado
```

---

## Progreso

| Sección | Nombre | Progreso |
|---------|--------|----------|
| A.1 | Registro — back + front | 0% |
| A.2 | Login — back + front | 0% |
| A.3 | Sesión activa y protección de rutas | 0% |
| A.4 | Refresh de tokens | 0% |
| A.5 | Logout | 0% |
| A.6 | RBAC — permisos en acción | 0% |
| A.7 | Test E2E completo | 0% |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para fortalecer la seguridad y la experiencia de usuario en el flujo de identidad:

| Herramienta | Propósito en el Auth |
| :--- | :--- |
| **`svelte-sonner`** | **Notificaciones:** Toasts elegantes para feedback de login, registro y errores de sesión. |
| **`zxcvbn-ts`** | **Fortaleza de Claves:** Estimación en tiempo real de la entropía de la contraseña durante el registro. |
| **`playwright`** | **Testing E2E:** Simulación de flujos de usuario reales para validar el redireccionamiento y la sesión. |
| **`lucide-svelte`** | **Iconografía:** Para botones de "ver/ocultar contraseña" y estados visuales de validación. |

---

## A.1 — Registro (POST /auth/register)

> **Referencia:** ADR 0008 (PASETO), ADR 0007 (Errores), ADR 0009 (Rate Limit), ADR 0029 (Leads), ADR 0018 (Jobs), docs/02-STACK.md L203-226, docs/01-ARCHITECTURE.md L223-229

### Backend

```
[ ] Handler crates/infrastructure/src/http/handlers/auth.rs:
    └─ Ref: docs/03-STRUCTURE.md L273-276
    [ ] extrae CreateUserRequest { email, password }
    [ ] llama RegisterUseCase::execute()
    [ ] retorna 201 + { user_id }
    [ ] 409 si email ya existe: { "error": "email_already_exists", "message": "..." }
        └─ Ref: ADR 0007, docs/02-STACK.md L100-102
    [ ] 400 si email inválido: { "error": "invalid_email", "message": "..." }
        └─ Ref: ADR 0007
    [ ] 400 si password corta: { "error": "password_too_short", "message": "..." }
        └─ Ref: ADR 0007
    [ ] Rate limit: 1 req/s (mismo que /auth/login)
        └─ Ref: ADR 0009, docs/02-STACK.md L143

[ ] RegisterUseCase ejecuta en orden:
    └─ Ref: docs/01-ARCHITECTURE.md L223-229
    [ ] Email::new() valida + normaliza a minúsculas
    [ ] find_active_by_email() → 409 si existe
        └─ Ref: ADR 0006 — Soft Delete
    [ ] hash_password(argon2id, OWASP params) → ~200ms intencional
        └─ Ref: ADR 0008, docs/02-STACK.md L205
    [ ] users.save()
    [ ] audit.log(register_success)
        └─ Ref: ADR 0006
    [ ] encola EmailJob:Welcome (no bloquea)
        └─ Ref: ADR 0018, ADR 0019, docs/02-STACK.md L274-295
    [ ] retorna UserId

[ ] #[utoipa::path] en el handler con request_body y responses (201, 400, 409)
    └─ Ref: ADR 0021, docs/02-STACK.md L246
```

### Frontend

```
[ ] pages/register.astro + components/auth/RegisterForm.svelte:
    └─ Ref: docs/03-STRUCTURE.md L478-483
    [ ] Campos: email, password (con toggle ver/ocultar), confirmación
    [ ] ArkType validation en tiempo real:
        └─ Ref: docs/02-STACK.md L389
        [ ] email válido
        [ ] password >= 12 caracteres
        [ ] entropía de contraseña (zxcvbn)
        [ ] confirmación coincide
    [ ] Notificaciones con sonner tras éxito/error
    [ ] Botón deshabilitado mientras isLoading = true
    [ ] TanStack mutation → POST /auth/register
        └─ Ref: docs/02-STACK.md L386
    [ ] onSuccess: mostrar "¡Registrado! Revisa tu email" + redirect /login en 3s
    [ ] onError 409: "Este email ya está registrado"
        └─ Ref: ADR 0007
    [ ] onError 400: mostrar mensaje exacto del campo inválido
        └─ Ref: ADR 0007
    [ ] Honeypot: campo oculto que bots rellenan → no enviar si tiene valor
        └─ Ref: ADR 0029

[ ] Verificar flujo completo:
    [ ] Email inválido → error en tiempo real (sin enviar)
    [ ] Email duplicado → mensaje claro sin revelar datos del sistema
        └─ Ref: ADR 0007, ADR 0008 — no revelar info
    [ ] Registro exitoso → redirect a login con mensaje de bienvenida
        └─ Ref: ADR 0019 — EmailJob:Welcome
```

---

## A.2 — Login (POST /auth/login)

> **Referencia:** ADR 0008 (PASETO), ADR 0007 (Errores), ADR 0009 (Rate Limit), ADR 0006 (Audit), docs/02-STACK.md L203-226, docs/01-ARCHITECTURE.md L230-235

### Backend

```
[ ] Handler crates/infrastructure/src/http/handlers/auth.rs:
    └─ Ref: docs/03-STRUCTURE.md L273-276
    [ ] extrae LoginRequest { email, password }
    [ ] llama LoginUseCase::execute()
    [ ] retorna 200 + { access_token, refresh_token, user }
    [ ] 401 si credenciales inválidas: { "error": "invalid_credentials" }
        └─ Ref: ADR 0007 — mismo mensaje para email no existe Y password incorrecta
    [ ] Rate limit estricto: 1 req/s, burst 5
        └─ Ref: ADR 0009, docs/02-STACK.md L143

[ ] LoginUseCase ejecuta en orden:
    └─ Ref: docs/01-ARCHITECTURE.md L230-235
    [ ] find_active_by_email() → 401 si no existe (mismo mensaje que password incorrecta)
        └─ Ref: ADR 0007 — previene enumeración de usuarios
    [ ] verify_password(argon2id, tiempo constante) → 401 si no coincide
        └─ Ref: ADR 0008, docs/02-STACK.md L205
    [ ] generate_access_token(user_id, 15min) → "v4.local.xxx"
        └─ Ref: ADR 0008, docs/02-STACK.md L213-217
    [ ] generate_opaque_token() → refresh token de 32 bytes
        └─ Ref: ADR 0008
    [ ] hash_token() → guardar hash en DB (nunca el token en claro)
        └─ Ref: ADR 0008
    [ ] sessions.save(user_id, ip, user_agent, expiry)
    [ ] audit.log(login_success, ip, user_agent)
        └─ Ref: ADR 0006
    [ ] retorna AuthTokens { access_token, refresh_token }

[ ] El mismo mensaje para usuario no encontrado Y password incorrecta
    └─ Ref: ADR 0007, ADR 0008 — previene enumeración de usuarios
    → previene enumeración de usuarios
```

### Frontend

> **Referencia:** ADR 0022 (Frontend), ADR 0008 (PASETO), docs/02-STACK.md L386-389, docs/03-STRUCTURE.md L478-480

```
[ ] pages/login.astro + components/auth/LoginForm.svelte:
    └─ Ref: docs/03-STRUCTURE.md L478-480
    [ ] Campos: email, password
    [ ] ArkType validation: email válido, password no vacía
        └─ Ref: docs/02-STACK.md L389
    [ ] TanStack mutation → POST /auth/login
        └─ Ref: docs/02-STACK.md L386
    [ ] onSuccess:
        [ ] setAuth(user, access_token)
            └─ Ref: docs/03-STRUCTURE.md L429-432
        [ ] guardar refresh_token en cookie httpOnly (o localStorage como fallback)
        [ ] redirect /dashboard
    [ ] onError 401: "Credenciales incorrectas"
        └─ Ref: ADR 0007 — no revelar si email existe
    [ ] onError 429: "Demasiados intentos. Espera X segundos."
        └─ Ref: ADR 0009
    [ ] No revelar si el email existe o no
        └─ Ref: ADR 0007, ADR 0008

[ ] Verificar:
    [ ] access_token en localStorage o cookie según configuración
    [ ] El token empieza con "v4.local." — NUNCA "eyJ" (JWT)
        └─ Ref: ADR 0008 — JWT prohibido
    [ ] redirect /dashboard después de login exitoso
    [ ] Si ya hay sesión activa → redirect directo /dashboard sin mostrar login
```

---

## A.3 — Sesión activa y protección de rutas

> **Referencia:** ADR 0008 (PASETO), ADR 0022 (Frontend), docs/03-STRUCTURE.md L454-458, docs/01-ARCHITECTURE.md L194-196

### Backend (ya implementado en III.3)

```
[ ] auth_middleware en todas las rutas /api/v1/*
    └─ Ref: docs/03-STRUCTURE.md L281, docs/01-ARCHITECTURE.md L194-196
[ ] require_permission() en rutas específicas
    └─ Ref: docs/03-STRUCTURE.md L282, ADR 0006
[ ] audit_middleware post-response
    └─ Ref: docs/03-STRUCTURE.md L283, ADR 0006
```

### Frontend

> **Referencia:** ADR 0008 (PASETO SSR), ADR 0022, docs/03-STRUCTURE.md L454-458, docs/01-ARCHITECTURE.md L194-196

```
[ ] DashboardLayout.astro — verificación SSR:
    └─ Ref: docs/03-STRUCTURE.md L454-458
    [ ] Lee access_token de las cookies/headers
    [ ] paseto.verify(token) en el servidor Astro
        └─ Ref: ADR 0008, docs/02-STACK.md L375
    [ ] Si inválido o expirado → redirect /login
        └─ Ref: docs/03-STRUCTURE.md L454-455
    [ ] Sin flash de contenido no autenticado — el redirect es instantáneo

[ ] auth.svelte.ts — estado global:
    └─ Ref: docs/03-STRUCTURE.md L429-432
    [ ] Estado inicializado desde el token en cookies al montar
    [ ] isLoggedIn derivado de user !== null
        └─ Ref: docs/02-STACK.md L386-388 — Svelte Runes
    [ ] user.permissions[] disponible en toda la app
        └─ Ref: ADR 0006

[ ] Verificar:
    [ ] Acceder a /dashboard sin sesión → redirect /login
    [ ] Acceder a /dashboard con sesión → página carga sin flash
    [ ] Token expirado → redirect /login con mensaje "Sesión expirada"
    [ ] /login con sesión activa → redirect /dashboard
```

---

## A.4 — Refresh de tokens

> **Referencia:** ADR 0008 (PASETO), docs/01-ARCHITECTURE.md L232-235, docs/02-STACK.md L213-217

### Backend

```
[ ] POST /auth/refresh — handler en crates/infrastructure/src/http/handlers/auth.rs:
    └─ Ref: docs/03-STRUCTURE.md L273-276
    [ ] extrae refresh_token del body
    [ ] hash_token() y buscar en DB
    [ ] verificar que no está revocado y no expiró
    [ ] REVOCAR el refresh token anterior (rotación obligatoria)
        └─ Ref: ADR 0008 — rotación obligatoria
    [ ] generar nuevo access_token + nuevo refresh_token
        └─ Ref: ADR 0008, docs/02-STACK.md L213-217
    [ ] guardar nuevo refresh hash en DB
        └─ Ref: ADR 0008 — nunca guardar token en claro
    [ ] retorna { access_token, refresh_token }
    [ ] 401 si refresh token inválido, expirado o ya revocado
```

### Frontend

> **Referencia:** ADR 0008, ADR 0022, docs/02-STACK.md L418-420, docs/03-STRUCTURE.md L438

```
[ ] Interceptor en lib/api/client.ts:
    └─ Ref: docs/03-STRUCTURE.md L438, docs/02-STACK.md L418-420
    [ ] Si response es 401 y hay refresh_token guardado:
        [ ] POST /auth/refresh automáticamente
        [ ] Si exitoso: actualizar access_token + reintentar request original
        [ ] Si falla: clearAuth() + redirect /login
    [ ] Sin intervención del usuario — transparente

[ ] Verificar:
    [ ] Token expirado → se refresca automáticamente
    [ ] Refresh token inválido → logout automático
    [ ] El usuario no nota el refresh si no hay error
```

---

## A.5 — Logout

> **Referencia:** ADR 0008 (PASETO), ADR 0006 (Audit), docs/03-STRUCTURE.md L273-276, docs/01-ARCHITECTURE.md L198-200

### Backend

```
[ ] POST /auth/logout (requiere auth) — handler en crates/infrastructure/src/http/handlers/auth.rs:
    └─ Ref: docs/03-STRUCTURE.md L273-276
    [ ] extrae session_id de las Extensions (inyectado por auth_middleware)
        └─ Ref: docs/01-ARCHITECTURE.md L194-196
    [ ] sessions.revoke(session_id)
    [ ] tokens.revoke_by_user(user_id)  (invalida TODOS los refresh tokens)
        └─ Ref: ADR 0008 — rotación de tokens
    [ ] audit.log(logout)
        └─ Ref: ADR 0006
    [ ] retorna 200
```

### Frontend

> **Referencia:** ADR 0022 (Frontend), docs/03-STRUCTURE.md L468-471, docs/02-STACK.md L386

```
[ ] Botón de logout en Topbar.svelte:
    └─ Ref: docs/03-STRUCTURE.md L468-471
    [ ] TanStack mutation → POST /auth/logout
        └─ Ref: docs/02-STACK.md L386
    [ ] onSettled (siempre, aunque falle):
        [ ] clearAuth()
            └─ Ref: docs/03-STRUCTURE.md L429-432
        [ ] limpiar access_token de localStorage/cookies
        [ ] limpiar refresh_token
        [ ] redirect /login
    [ ] El logout funciona aunque el servidor esté caído
        └─ Ref: ADR 0002 — degrade gracefully

[ ] Verificar:
    [ ] Logout limpia todo el estado local
    [ ] El token revocado ya no funciona para requests posteriores
        └─ Ref: ADR 0008
    [ ] Redirect a /login después del logout
```

---

## A.6 — RBAC en acción

> **Referencia:** ADR 0006 (RBAC), ADR 0022 (Frontend), docs/02-STACK.md L228-233, docs/01-ARCHITECTURE.md L203-206

### Backend (ya en III.3)

```
[ ] require_permission("users:read") en GET /api/v1/users
    └─ Ref: docs/03-STRUCTURE.md L282, docs/02-STACK.md L228-233
[ ] require_permission("users:write") en POST/PUT/DELETE /api/v1/users
    └─ Ref: ADR 0006, docs/02-STACK.md L228-233
[ ] require_permission("audit:read") en GET /api/v1/audit
    └─ Ref: ADR 0006
```

### Frontend

> **Referencia:** ADR 0006, ADR 0022, docs/03-STRUCTURE.md L463-467, L521-523

```
[ ] user.permissions[] cargado en el login y guardado en auth store
    └─ Ref: docs/03-STRUCTURE.md L429-432
[ ] PermissionGate oculta botones sin permiso
    └─ Ref: docs/03-STRUCTURE.md L521-523, docs/02-STACK.md L228-233
[ ] Sidebar oculta items de navegación sin permiso
    └─ Ref: docs/03-STRUCTURE.md L463-467, ADR 0006
[ ] Páginas verifican permiso en SSR → redirect si no tiene
    └─ Ref: docs/03-STRUCTURE.md L454-458

[ ] Escenarios para verificar:
    [ ] Admin (todos los permisos) → ve todo, puede hacer todo
    [ ] User (permisos básicos) → no ve "Crear usuario", no accede a /audit
    [ ] Token de User en /api/v1/users POST → 403 del backend
        └─ Ref: docs/01-ARCHITECTURE.md L203-206 — RBAC en backend primero
    [ ] UI de User → botón "Crear usuario" no aparece (PermissionGate)
```

---

## A.7 — Test E2E completo (ADR 0010)

> **Referencia:** ADR 0010 (Testing), docs/02-STACK.md L429-443, docs/03-STRUCTURE.md L275

```
[ ] apps/api/tests/auth_fullstack_test.rs:
    └─ Ref: docs/03-STRUCTURE.md L275 — tests E2E en apps/api/tests/
    └─ Ref: docs/02-STACK.md L438 — capa 4 E2E

[ ] flujo_completo_register_login_dashboard_logout():
    1. POST /auth/register → 201
    2. POST /auth/login → 200, access_token empieza con "v4.local."
       └─ Ref: ADR 0008 — verificar formato PASETO
    3. GET /api/v1/users con Bearer token → 200
    4. GET /api/v1/users sin Bearer → 401
    5. POST /auth/logout → 200
    6. GET /api/v1/users con token revocado → 401

[ ] rbac_usuario_sin_permiso():
    1. Login con usuario tipo "User" (sin users:write)
    2. POST /api/v1/users → 403 { "error": "forbidden" }
       └─ Ref: ADR 0006, ADR 0007
    3. GET /api/v1/users → 200 (tiene users:read)

[ ] refresh_token_rotacion():
    └─ Ref: ADR 0008 — rotación obligatoria
    1. Login → { access_token, refresh_token }
    2. POST /auth/refresh con refresh_token → nuevos tokens
    3. POST /auth/refresh con el refresh_token ANTERIOR → 401

[ ] rate_limit_login():
    └─ Ref: ADR 0009
    1. 6 intentos de login fallidos en 1 segundo
    2. El 7º → 429 Too Many Requests con Retry-After header
       └─ Ref: docs/02-STACK.md L143, L132

[ ] cargo nextest run --all-targets → todos pasan
    └─ Ref: ADR 0010, docs/02-STACK.md L442-443
```

---

## Verificación final del flujo Auth

```bash
# Registro
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"demo@test.com","password":"password_segura_123"}'
# → 201 {"user_id":"..."}

# Login
TOKEN=$(curl -s -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"demo@test.com","password":"password_segura_123"}' \
  | jq -r '.access_token')

echo $TOKEN | cut -c1-10  # → "v4.local.."  ← NUNCA "eyJ" (JWT)

# Request autenticada
curl http://localhost:8080/api/v1/users \
  -H "Authorization: Bearer $TOKEN"
# → 200 {"items":[...],"total":...}

# Sin permiso
curl -X POST http://localhost:8080/api/v1/users \
  -H "Authorization: Bearer $TOKEN_USER_SIN_PERMISO" \
  -d '{"email":"nuevo@test.com"}'
# → 403 {"error":"forbidden","message":"requiere permiso: users:write"}

# Logout
curl -X POST http://localhost:8080/auth/logout \
  -H "Authorization: Bearer $TOKEN"
# → 200

# Token revocado
curl http://localhost:8080/api/v1/users \
  -H "Authorization: Bearer $TOKEN"
# → 401
```

---

## Diagrama de Flujo de Autenticación End-to-End

```
┌─────────────────────────────────────────────────────────────────────────┐
│  A.1 — REGISTRO                                                        │
│  Frontend: RegisterForm.svelte (ArkType validation)                    │
│     ↓ POST /auth/register                                              │
│  Backend: RegisterUseCase                                               │
│     ├─ Email::new() valida                                             │
│     ├─ find_active_by_email() → 409 si existe                          │
│     ├─ hash_password(argon2id)                                        │
│     ├─ users.save()                                                    │
│     ├─ audit.log(register_success)                                    │
│     └─ encola EmailJob:Welcome (no bloquea)                           │
│     ↓ 201 { user_id }                                                  │
│  Frontend: redirect /login en 3s + mensaje de éxito                    │
│     └─ Ref: ADR 0008, 0007, 0018, 0019                                │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  A.2 — LOGIN                                                           │
│  Frontend: LoginForm.svelte (ArkType validation)                        │
│     ↓ POST /auth/login                                                 │
│  Backend: LoginUseCase                                                │
│     ├─ find_active_by_email() → 401 si no existe                     │
│     ├─ verify_password(argon2id) → 401 si no coincide                │
│     ├─ generate_access_token(15min) → "v4.local.xxx"                 │
│     ├─ generate_opaque_token() → refresh token 32 bytes              │
│     ├─ sessions.save()                                                 │
│     ├─ audit.log(login_success)                                       │
│     ↓ 200 { access_token, refresh_token, user }                       │
│  Frontend: setAuth(user, token) → redirect /dashboard                │
│     └─ Ref: ADR 0008, 0007, 0009, 0006                                │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  A.3 — SESIÓN ACTIVA                                                   │
│  Frontend: DashboardLayout.astro (SSR)                                 │
│     ├─ Lee access_token de cookies/headers                             │
│     ├─ paseto.verify(token) en servidor Astro                          │
│     └─ Si inválido → redirect /login (sin flash)                       │
│  Backend: auth_middleware en /api/v1/*                                  │
│     ├─ Extrae Bearer token                                             │
│     ├─ paseto.verify() → UserId en Extensions                          │
│     ├─ require_permission() si aplica                                  │
│     └─ audit_middleware post-response                                  │
│     └─ Ref: ADR 0008, 0022, 0006                                       │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  A.4 — REFRESH DE TOKENS (cuando access_token expira)                  │
│  Frontend: Interceptor en lib/api/client.ts                             │
│     ├─ Detecta 401                                                     │
│     ↓ POST /auth/refresh con refresh_token                             │
│  Backend:                                                              │
│     ├─ hash_token() y busca en DB                                      │
│     ├─ Verifica no revocado/no expirado                                │
│     ├─ REVOCA refresh token anterior (rotación obligatoria)            │
│     ├─ Genera nuevo access_token + nuevo refresh_token                 │
│     ↓ 200 { access_token, refresh_token }                             │
│  Frontend: Actualiza tokens → reintenta request original               │
│     └─ Ref: ADR 0008 — rotación obligatoria                            │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  A.5 — LOGOUT                                                          │
│  Frontend: Botón logout en Topbar.svelte                               │
│     ↓ POST /auth/logout                                                │
│  Backend:                                                              │
│     ├─ Extrae session_id de Extensions                                 │
│     ├─ sessions.revoke(session_id)                                     │
│     ├─ tokens.revoke_by_user(user_id) — TODOS los refresh tokens      │
│     ├─ audit.log(logout)                                               │
│     ↓ 200                                                              │
│  Frontend: clearAuth() → limpia tokens → redirect /login               │
│     (funciona aunque servidor esté caído)                                │
│     └─ Ref: ADR 0008, 0006, 0002                                       │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Documentación Oficial de Referencia

| Herramienta/Crate | URL | Útil para |
|-------------------|-----|-----------|
| **PASETO** | https://paseto.io | Tokens v4.local (no JWT), formato de claims |
| **pasetors (Rust)** | https://docs.rs/pasetors/latest | Implementación PASETO v4 en Rust |
| **Argon2** | https://docs.rs/argon2/latest | Password hashing (OWASP params) |
| **Axum** | https://docs.rs/axum/latest | Middleware auth, extractors |
| **Astro SSR** | https://docs.astro.build/en/guides/server-side-rendering/ | Verificación PASETO en servidor |
| **TanStack Query** | https://tanstack.com/query/latest | Mutations, caching, error handling |
| **ArkType** | https://arktype.io | Validación runtime type-safe |
| **tower-governor** | https://docs.rs/tower-governor/latest | Rate limiting en /auth/* |

---

## Troubleshooting — Auth Fullstack

### A.1 — Registro

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Registro retorna 500 | EmailJob no encolado o falla | Revisar `enqueue` en RegisterUseCase — Ref: ADR 0018 |
| Email duplicado no retorna 409 | find_active_by_email busca en soft deleted | Usar `find_active_by_email` no `find_by_email` — Ref: ADR 0006 |
| Password corta no valida | ArkType schema mal definido | Revisar `type({ password: 'string >= 12' })` — Ref: docs/02-STACK.md L389 |
| Honeypot no funciona | Campo visible o mal nombrado | Usar nombre común como "website" o "company" — Ref: ADR 0029 |
| Email de bienvenida no llega | Resend no configurado o LogMailer en dev | En dev revisar logs, en prod verificar `RESEND_API_KEY` — Ref: ADR 0019 |

### A.2 — Login

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Login siempre 401 | Password hash mal generado o verificado | Verificar argon2id params coinciden — Ref: ADR 0008 |
| Mensaje diferente para email no existe vs password | Fuga de información | Usar mismo mensaje "Credenciales incorrectas" — Ref: ADR 0007 |
| access_token empieza con "eyJ" | Usando JWT en lugar de PASETO | Revisar `paseto.rs` — NUNCA usar jsonwebtoken — Ref: ADR 0008 |
| Rate limit no funciona | GovernorLayer mal ordenado en middleware | `TimeoutLayer` antes de `GovernorLayer` — Ref: ADR 0009 |
| Frontend no redirige tras login | setAuth() no actualiza estado | Verificar `auth.svelte.ts` con $state — Ref: docs/03-STRUCTURE.md L429-432 |

### A.3 — Sesión Activa

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Flash de contenido no autenticado | PASETO verificado solo en cliente | Verificar SSR en DashboardLayout.astro — Ref: docs/03-STRUCTURE.md L454-458 |
| /dashboard accesible sin sesión | auth_middleware no aplicado | Revisar router de Axum — Ref: docs/03-STRUCTURE.md L278 |
| Permisos no cargados | user.permissions no incluido en respuesta de login | Añadir a LoginUseCase return — Ref: docs/01-ARCHITECTURE.md L230-235 |
| Token expirado no redirige | Interceptor no detecta 401 | Verificar lib/api/client.ts — Ref: docs/03-STRUCTURE.md L438 |

### A.4 — Refresh de Tokens

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Refresh retorna 401 siempre | Refresh token hash no coincide | Verificar `hash_token()` usa SHA-256 — Ref: ADR 0008 |
| Token anterior sigue funcionando | No se revoca en refresh | Implementar rotación obligatoria — Ref: ADR 0008 |
| Loop infinito de refresh | Interceptor no actualiza token después de refresh | Verificar actualización de access_token — Ref: docs/03-STRUCTURE.md L438 |
| Frontend no detecta token expirado | Tiempo de expiración mal calculado | 15 minutos desde `iat` claim — Ref: ADR 0008 |

### A.5 — Logout

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Logout retorna 401 | Token ya expirado | Frontend debe limpiar estado igual (onSettled) — Ref: docs/03-STRUCTURE.md L468-471 |
| Token sigue funcionando tras logout | No se revoca refresh token | Llamar `tokens.revoke_by_user(user_id)` — Ref: ADR 0008 |
| Sesión no se limpia en frontend | clearAuth() no limpia localStorage | Verificar `auth.svelte.ts` — Ref: docs/03-STRUCTURE.md L429-432 |
| Logout lento | Síncrono y espera respuesta | Hacer fire-and-forget o usar onSettled — Ref: ADR 0002 |

### A.6 — RBAC

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Usuario sin permiso ve botón "Crear" | PermissionGate no verifica permiso | Wrap botones en `<PermissionGate permission="users:write">` — Ref: ADR 0006 |
| Backend no rechaza request sin permiso | require_permission no aplicado | Añadir middleware a handler — Ref: docs/03-STRUCTURE.md L282 |
| Sidebar muestra items prohibidos | Filter no aplicado a nav items | Revisar `user.permissions.includes()` — Ref: docs/03-STRUCTURE.md L463-467 |
| 403 del backend pero 200 esperado | Permiso mal escrito | Verificar formato "recurso:accion" — Ref: ADR 0006 |

### A.7 — Tests E2E

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Test de rotación falla | Refresh token anterior no revocado | Verificar `refresh_token_rotacion()` implementación — Ref: ADR 0008 |
| Rate limit test inconsistente | Timing muy ajustado | Aumentar ventana o usar `tokio::time::pause()` — Ref: ADR 0009 |
| Test E2E lento | Creación de pool por cada test | Usar `#[sqlx::test]` o pool compartido — Ref: ADR 0010 |
| Flaky tests de auth | Estado compartido entre tests | Aislar cada test con transaction rollback — Ref: ADR 0010 |

---

**Nota:** Si un error persiste, revisar los ADRs 0006, 0007, 0008, 0009 que cubren RBAC, Errores, PASETO y Rate Limiting.
