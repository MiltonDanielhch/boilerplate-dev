# Verificaciones — Guía de Pruebas por Fase

> Corre estas verificaciones al terminar cada bloque del roadmap.
> Si algo falla, no avances al siguiente bloque.
> 
> **Referencia:** ADR 0010 (Testing), docs/02-STACK.md L429-443

---

## Nota para usuarios Windows/PowerShell

Este documento usa comandos bash por defecto. En PowerShell:

| Bash | PowerShell equivalente |
|------|------------------------|
| `&&` | `;` (ejecutar separado) o `-and` |
| `\|` | `\|` (funciona igual) |
| `grep` | `Select-String` o `findstr` |
| `sqlite3 file.db "query"` | `sqlite3 file.db "query"` (si está en PATH) |
| `2>&1 \| grep` | `2>&1 \| Select-String` |
| `cat file` | `Get-Content file` o `type file` |
| `&&` entre comandos | Ejecutar comandos en líneas separadas |

Ejemplo de conversión:
```bash
# Bash:
grep -r "jsonwebtoken" . --include="*.toml"

# PowerShell:
Select-String -Path "*.toml" -Pattern "jsonwebtoken" -Recurse
# O:
findstr /s /i "jsonwebtoken" *.toml
```

---

## Cómo usar este documento

```
1. Terminas un bloque del ROADMAP
2. Vienes aquí y corres las verificaciones de ese bloque
3. Todo ✅ → avanzas
4. Algo ❌ → resuelves antes de continuar
```

---

## Bloque Génesis — Workspace + Tooling

> **Referencia:** ADR 0012 (Tooling), ADR 0013 (Build), ADR 0028 (Auditoría), docs/02-STACK.md L413-417

### 1. El workspace compila

```bash
cargo check --workspace
```
```powershell
cargo check --workspace
```
**Esperado:** `"Finished"` sin errores  
**Ref:** `docs/03-STRUCTURE.md` L169-172

### 2. No hay JWT en ningún Cargo.toml

```bash
# Bash/Linux/Mac
grep -r "jsonwebtoken" . --include="*.toml"
# o:
find . -name "*.toml" -exec grep -l "jsonwebtoken" {} \;
```
```powershell
# PowerShell (Windows)
Select-String -Path "*.toml" -Pattern "jsonwebtoken" -Recurse
# o alternativa más simple:
findstr /s /i "jsonwebtoken" *.toml
```
**Esperado:** cero resultados  
**Ref:** ADR 0008 — JWT prohibido

### 3. Licencias y CVEs limpios

```bash
cargo deny check
```
```powershell
cargo deny check
```
**Esperado:** sin violations  
**Ref:** ADR 0013, `docs/02-STACK.md` L412

### 4. Herramientas instaladas

```bash
# Bash - comandos con &&
just --version && sqlx --version && cargo nextest --version
```
```powershell
# PowerShell - comandos separados (&& no funciona)
just --version
sqlx --version
cargo nextest --version
# o en una línea con ; (semicolon)
just --version; sqlx --version; cargo nextest --version
```
**Esperado:** versiones impresas sin error  
**Ref:** `docs/02-STACK.md` L411-417

### 5. El hook de lefthook funciona

```bash
git commit --allow-empty -m "test genesis"
```
```powershell
git commit --allow-empty -m "test genesis"
```
**Esperado:** lefthook ejecuta fmt antes del commit  
**Ref:** ADR 0012, `docs/02-STACK.md` L413

### 6. El proto es válido (Fase 2)

```bash
buf lint
```
```powershell
buf lint
```
**Esperado:** sin errores (o `"no .proto files"` en Fase 1)  
**Ref:** `docs/02-STACK.md` L419-424, ADR 0027

---

## Bloque I — Fundación (Dominio + DB + RBAC)

> **Referencia:** ADR 0001 (Hexagonal), ADR 0004 (SQLite), ADR 0006 (RBAC), docs/02-STACK.md L155-170

### Las 6 migraciones

**Ejecutar migraciones:**
```bash
# Bash / PowerShell
just migrate
```
**Esperado — exactamente esto:**
```
Applied 20260305135148/migrate create_users_table
Applied 20260305135149/migrate create_rbac
Applied 20260305135150/migrate create_tokens
Applied 20260305135151/migrate create_audit_logs
Applied 20260305135152/migrate seed_system_data
Applied 20260305135153/migrate create_sessions
```
**Ref:** `docs/01-ARCHITECTURE.md` L139-164, `docs/02-STACK.md` L159-163

**Verificar tablas creadas:**
```bash
# Bash / PowerShell (sqlite3 debe estar en PATH)
sqlite3 ./data/boilerplate.db ".tables"
```
**Esperado:** `audit_logs  permissions  role_permissions  roles sessions tokens user_roles users`  
**Ref:** `docs/01-ARCHITECTURE.md` L139-164

**Verificar que el admin existe:**
```bash
# Bash / PowerShell
sqlite3 ./data/boilerplate.db "SELECT email FROM users;"
```
**Esperado:** `admin@admin.com`  
**Ref:** `docs/01-ARCHITECTURE.md` L143

**Verificar permisos del admin:**
```bash
# Bash (query multi-línea)
sqlite3 ./data/boilerplate.db "
  SELECT p.resource || ':' || p.action as permission
  FROM permissions p
  JOIN role_permissions rp ON rp.permission_id = p.id
  JOIN roles r ON r.id = rp.role_id
  WHERE r.name = 'Admin'
  ORDER BY p.resource, p.action;
"
```
**Esperado:** `audit:read`, `roles:read`, `roles:write`, `users:read`, `users:write`  
**Ref:** ADR 0006, `docs/02-STACK.md` L228-233

### El dominio no tiene dependencias externas

```bash
# El Cargo.toml de domain no debe tener sqlx ni axum
cat crates/domain/Cargo.toml
# Esperado: solo thiserror, uuid, time, serde
# └─ Ref: ADR 0001, docs/03-STRUCTURE.md L188-194

# Los tests de dominio pasan sin base de datos
cargo nextest run -p domain
# Esperado: todos los tests pasan en <100ms
# └─ Ref: ADR 0010, docs/02-STACK.md L429-443

# Verificar que Email valida correctamente
cargo test -p domain email
# Esperado: email_valido ✅, email_sin_arroba ✅, email_vacio ✅
# └─ Ref: docs/02-STACK.md L88 — Email como value object
```

### Los repositorios SQLx funcionan

```bash
# Tests de integración con SQLite en memoria
cargo nextest run -p database
# Esperado: guardar_y_recuperar, soft_delete_oculta, has_permission_admin pasan
# └─ Ref: ADR 0004, docs/02-STACK.md L160-163

# Verificar que .sqlx/ está generado para builds offline
just prepare
# Esperado: .sqlx/ actualizado, "query data written"
# └─ Ref: ADR 0013, docs/02-STACK.md L415
```

---

## Bloque II — API (Axum + Middleware)

> **Referencia:** ADR 0003 (Axum), ADR 0009 (Rate Limit), ADR 0021 (OpenAPI), docs/02-STACK.md L132-154

```bash
# Arrancar el servidor
just dev-api
# Esperado: "servidor iniciado en puerto 8080"
# └─ Ref: docs/02-STACK.md L148

# Health check
curl http://localhost:8080/health
# Esperado: {"status":"ok","database":"connected"}
# └─ Ref: docs/03-STRUCTURE.md L278

# La API responde JSON
curl http://localhost:8080/api/v1/users
# Esperado: 401 Unauthorized (ruta protegida)
# └─ Ref: ADR 0008, docs/02-STACK.md L203-226

# El rate limit funciona — superar 30 requests
for i in {1..35}; do curl -s -o /dev/null -w "%{http_code}\n" http://localhost:8080/api/v1/users; done
# Esperado: los últimos retornan 429
# └─ Ref: ADR 0009, docs/02-STACK.md L143

# Sin JWT en los headers de respuesta
curl -I http://localhost:8080/health | grep -i "x-powered-by\|server"
# Esperado: sin headers que revelen tecnología
# └─ Ref: ADR 0014, docs/02-STACK.md L360-368
```

---

## Bloque III — Seguridad (PASETO + Auth + RBAC)

> **Referencia:** ADR 0006 (RBAC), ADR 0007 (Errores), ADR 0008 (PASETO), docs/02-STACK.md L100-102, L203-233

```bash
# 1. Registro de usuario
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123"}'
# Esperado: 201 Created {"id":"...","email":"test@example.com"}
# └─ Ref: docs/03-STRUCTURE.md L273-276

# 2. Login y obtener token
TOKEN=$(curl -s -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123"}' \
  | jq -r '.access_token')
echo "Token: $TOKEN"
# Esperado: token PASETO (empieza con "v4.local.")
# └─ Ref: ADR 0008, docs/02-STACK.md L213-217

# 3. Request autenticado
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/v1/users
# Esperado: 200 OK con lista de usuarios (vacía o con admin)
# └─ Ref: docs/03-STRUCTURE.md L281

# 4. Token inválido es rechazado
curl -H "Authorization: Bearer invalid.token.here" \
  http://localhost:8080/api/v1/users
# Esperado: 401 {"error":"unauthorized","message":"no autenticado"}
# └─ Ref: ADR 0007, docs/02-STACK.md L100-102

# 5. Verificar que PASETO empieza con v4.local (no JWT con "eyJ")
echo $TOKEN | cut -c1-10
# Esperado: "v4.local.X" — NUNCA "eyJhbGciO"
# └─ Ref: ADR 0008 — PASETO v4.local

# 6. Logout
curl -X POST -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/auth/logout
# Esperado: 200 OK
# └─ Ref: docs/03-STRUCTURE.md L273-276

# 7. Token revocado ya no funciona
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/v1/users
# Esperado: 401 Unauthorized
# └─ Ref: ADR 0008

# 8. Soft Delete — verificar que no hace DELETE real
curl -X DELETE -H "Authorization: Bearer $ADMIN_TOKEN" \
  http://localhost:8080/api/v1/users/USER_ID
sqlite3 ./data/boilerplate.db \
  "SELECT deleted_at FROM users WHERE email='test@example.com';"
# Esperado: fecha ISO — NO es NULL. El registro sigue en la DB.
# └─ Ref: ADR 0006 — Soft Delete

# 9. RBAC — usuario sin permiso es rechazado
curl -H "Authorization: Bearer $USER_TOKEN" \
  http://localhost:8080/api/v1/users
# Esperado: 403 {"error":"forbidden","message":"requiere permiso: users:read"}
# └─ Ref: ADR 0006, docs/02-STACK.md L228-233

# 10. Audit log registrado (formato JSON Lines en logs)
just dev-api 2>&1 | grep -E '"target":"audit"' | head -3
# Esperado: logs JSON con target="audit", method, uri, status, user_id
# └─ Ref: ADR 0006, docs/02-STACK.md L313-316, apps/api/src/middleware/audit.rs
```

---

## Bloque IV — Documentación API

> **Referencia:** ADR 0021 (OpenAPI), docs/02-STACK.md L239-250

```bash
# Scalar UI disponible
curl http://localhost:8080/docs
# Esperado: HTML con la UI de Scalar
# └─ Ref: ADR 0021, docs/02-STACK.md L248

# OpenAPI spec válido
curl http://localhost:8080/openapi.json | jq '.info.title'
# Esperado: "boilerplate API"
# └─ Ref: ADR 0021, docs/02-STACK.md L246

# El spec tiene los endpoints de auth
curl http://localhost:8080/openapi.json | jq '.paths | keys[]'
# Esperado: /auth/register, /auth/login, /api/v1/users, etc.
# └─ Ref: docs/03-STRUCTURE.md L278

# El spec tiene el esquema de seguridad PASETO
curl http://localhost:8080/openapi.json | jq '.components.securitySchemes'
# Esperado: {"PasetoAuth":{"type":"http","scheme":"bearer",...}}
# └─ Ref: ADR 0021, ADR 0008
```

---

## Bloque V — Async (Jobs + Cache + Email)

> **Referencia:** ADR 0017 (Cache), ADR 0018 (Jobs), ADR 0019 (Email), docs/02-STACK.md L253-295

```bash
# 1. Registrar un usuario y verificar que el EmailJob se encola
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"job-test@example.com","password":"password123"}'

# Verificar que el job aparece en la tabla de Apalis
sqlite3 ./data/boilerplate.db \
  "SELECT job_type, status FROM jobs ORDER BY created_at DESC LIMIT 5;"
# Esperado: fila con job_type="email::send" y status="Done" o "Running"
# └─ Ref: ADR 0018, docs/02-STACK.md L274-295

# 2. En desarrollo — verificar que el LogMailer imprime en los logs
just dev-api 2>&1 | grep -i "email"
# Esperado: log con el contenido del email de bienvenida
# └─ Ref: ADR 0019, docs/02-STACK.md L280-282

# 3. Cache — verificar que reduce queries repetidas
# Pedir el mismo usuario dos veces y ver los logs de SQLx
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/v1/users/USER_ID
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/v1/users/USER_ID
# Esperado: en logs, la segunda llamada no muestra query SQL (L1 HIT)
# └─ Ref: ADR 0017, docs/02-STACK.md L253-268
```

---

## Bloque VI — Observabilidad

> **Referencia:** ADR 0015 (Monitoreo), ADR 0016 (Observabilidad), docs/02-STACK.md L299-358

```bash
# 1. Logs son JSON válido
just dev-api 2>&1 | head -5 | jq .
# Esperado: JSON con timestamp, level, message, request_id
# └─ Ref: docs/02-STACK.md L335-340

# 2. request_id está en todos los logs del mismo request
just dev-api 2>&1 | grep "request_id" | head -3
# Esperado: mismo request_id en los logs de un mismo request
# └─ Ref: ADR 0016, docs/02-STACK.md L340-341

# 3. Queries lentas aparecen como warnings
# (Simular con una query pesada si existe)
just dev-api 2>&1 | grep -i "slow"
# Esperado: warnings para queries >100ms
# └─ Ref: ADR 0016, docs/02-STACK.md L337

# 4. Sentry recibe el error (con SENTRY_DSN configurado)
# Forzar un panic temporal y verificar en el dashboard de Sentry
# └─ Ref: ADR 0016, docs/02-STACK.md L355-358
```

---

## Bloque FE.I — Frontend Foundation

> **Referencia:** ADR 0022 (Frontend), ADR 0023 (i18n), docs/02-STACK.md L368-400

```bash
# 1. pnpm dev arranca sin errores
pnpm --filter web dev
# Esperado: "ready in Xms" sin errores
# └─ Ref: docs/02-STACK.md L371

# 2. Tipos TypeScript generados y válidos
just types
# Esperado: "✅ Tipos generados en apps/web/src/lib/types/api.ts"
# └─ Ref: ADR 0027, docs/02-STACK.md L419-424

# 3. Los tipos no tienen diff (no fueron editados manualmente)
just types-check
# Esperado: sin diff
# └─ Ref: ADR 0027, docs/02-STACK.md L421

# 4. La landing carga
curl http://localhost:4321
# Esperado: HTML con contenido de la landing
# └─ Ref: docs/03-STRUCTURE.md L459-462

# 5. Paraglide i18n funciona
curl http://localhost:4321 | grep -i "lang="
# Esperado: lang="es" o lang="en"
# └─ Ref: ADR 0023, docs/02-STACK.md L392-400
```

---

## Bloque FE.IV-VI — Frontend Components + RBAC + i18n

> **Referencia:** ADR 0022 (Frontend), ADR 0006 (RBAC), docs/03-STRUCTURE.md L454-467

```bash
# 1. El login funciona en el browser (manual)
# Ir a http://localhost:4321/login
# Ingresar admin@admin.com / 12345678
# Esperado: redirige a /dashboard
# └─ Ref: docs/03-STRUCTURE.md L478-480

# 2. El dashboard muestra datos reales (manual)
# Ir a http://localhost:4321/dashboard
# Esperado: KPIs con números del backend
# └─ Ref: docs/03-STRUCTURE.md L454-458

# 3. Los permisos funcionan en la UI (manual)
# Con usuario sin permiso users:write
# Esperado: botón "Crear usuario" no aparece (PermissionGate)
# └─ Ref: ADR 0006, docs/03-STRUCTURE.md L521-523

# 4. Responsive en móvil (manual)
# Chrome DevTools → modo móvil
# Esperado: sidebar colapsa, touch targets > 44px
# └─ Ref: ADR 0022, docs/02-STACK.md L376-377
```

---

## Bloque L.1-L.7 — Landing Page

> **Referencia:** ADR 0029 (Landing), ADR 0009 (Rate Limit), docs/02-STACK.md L368-400

```bash
# 1. El formulario de leads funciona
curl -X POST http://localhost:8080/api/v1/leads \
  -H "Content-Type: application/json" \
  -d '{"email":"lead@example.com","name":"Test Lead"}'
# Esperado: 200 OK
# └─ Ref: docs/03-STRUCTURE.md L273-276

# 2. Rate limit en leads — más de 3 por minuto
for i in {1..5}; do
  curl -s -o /dev/null -w "%{http_code}\n" \
    -X POST http://localhost:8080/api/v1/leads \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"lead$i@example.com\"}"
done
# Esperado: los últimos retornan 429
# └─ Ref: ADR 0009, docs/02-STACK.md L143

# 3. Lighthouse PWA > 80
# Chrome DevTools → Lighthouse → PWA
# Esperado: score > 80
# └─ Ref: ADR 0022

# 4. Honeypot funciona (bots retornan 200 silencioso)
curl -X POST http://localhost:8080/api/v1/leads \
  -H "Content-Type: application/json" \
  -d '{"email":"bot@test.com","honeypot":"filled"}'
# Esperado: 200 OK (silencioso, sin guardar)
# └─ Ref: ADR 0029
```

---

## Bloque A.1-A.7 — Auth Fullstack

> **Referencia:** ADR 0006 (RBAC), ADR 0008 (PASETO), ADR 0022 (Frontend), docs/01-ARCHITECTURE.md L223-235

```bash
# 1. Flujo completo end-to-end (backend + frontend)
# POST /auth/register → 201
# POST /auth/login → 200 + v4.local token
# GET /api/v1/users con Bearer → 200
# POST /auth/logout → 200
# GET /api/v1/users con token revocado → 401
# └─ Ref: ADR 0008, ADR 0022

# 2. Refresh token rotación
# Login → obtener refresh_token
# POST /auth/refresh con refresh_token → nuevos tokens
# POST /auth/refresh con token anterior → 401
# └─ Ref: ADR 0008 — rotación obligatoria

# 3. Sesión SSR en Astro
# /dashboard con token válido → carga sin flash
# /dashboard sin token → redirect /login
# /login con sesión activa → redirect /dashboard
# └─ Ref: ADR 0022, docs/03-STRUCTURE.md L454-458
```

---

## Bloque INF.I-IV — Infraestructura

> **Referencia:** ADR 0013 (Build), ADR 0014 (Deploy), ADR 0004 (Litestream), docs/02-STACK.md L360-368

```bash
# 1. La imagen compila
just build
# Esperado: imagen creada sin errores
# └─ Ref: docs/02-STACK.md L413

# 2. El tamaño de la imagen es aceptable
podman image ls boilerplate
# Esperado: SIZE < 15MB
# └─ Ref: ADR 0013 — bundle minimalista

# 3. No tiene shell
podman run --rm boilerplate sh
# Esperado: error — no hay shell en distroless
# └─ Ref: ADR 0013

# 4. SQLX_OFFLINE funciona (simular CI sin DB)
SQLX_OFFLINE=true cargo build --release
# Esperado: compila correctamente usando .sqlx/
# └─ Ref: ADR 0013, docs/02-STACK.md L415

# 5. Health check responde dentro del contenedor
podman run --rm -p 8080:8080 boilerplate &
sleep 3
curl http://localhost:8080/health
# Esperado: {"status":"ok"}
# └─ Ref: ADR 0014, docs/02-STACK.md L366

# 6. HTTPS funciona en el VPS
curl https://tudominio.com/health
# Esperado: {"status":"ok"} con TLS válido
# └─ Ref: ADR 0014, docs/02-STACK.md L362

# 7. Kamal deploy funciona
just deploy
# Esperado: "Finished all in Xs" sin errores
# └─ Ref: ADR 0014, docs/02-STACK.md L360-368

# 8. Zero-downtime verificado
# Mientras el deploy corre, hacer requests continuos
while true; do curl -s -o /dev/null -w "%{http_code}\n" https://tudominio.com/health; sleep 0.5; done
# Esperado: todos los status codes son 200, ninguno es 502/503
# └─ Ref: ADR 0014

# 9. Rollback funciona
kamal rollback
curl https://tudominio.com/health
# Esperado: {"status":"ok"} — volvió a la versión anterior
# └─ Ref: ADR 0014

# 10. Litestream replica activamente
sqlite3 ./data/boilerplate.db "INSERT INTO leads (id, email) VALUES ('test-ls', 'ls@test.com');"
sleep 3
litestream snapshots s3://tu-bucket/boilerplate/db
# Esperado: snapshot reciente con fecha actual
# └─ Ref: ADR 0004, docs/02-STACK.md L168

# 11. Restauración funciona
cp ./data/boilerplate.db ./data/boilerplate.db.bak
litestream restore -o ./data/boilerplate-restored.db s3://tu-bucket/boilerplate/db
sqlite3 ./data/boilerplate-restored.db "SELECT email FROM leads WHERE id='test-ls';"
# Esperado: ls@test.com — el registro sobrevivió
# └─ Ref: ADR 0004, docs/02-STACK.md L170
```

---

## Bloque D.I-VII — Tauri Desktop

> **Referencia:** ADR 0030 (Multiplataforma), ADR 0004 (SQLite), ADR 0008 (PASETO), docs/02-STACK.md L382

```bash
# 1. El workspace compila incluyendo desktop
cargo check --workspace
# Esperado: sin errores incluyendo apps/desktop
# └─ Ref: docs/03-STRUCTURE.md L550-554

# 2. Build de desarrollo
cd apps/desktop
cargo tauri dev
# Esperado: ventana abre con la UI de Svelte 5
# └─ Ref: docs/02-STACK.md L375, L382

# 3. Login funciona en desktop (sin servidor Axum)
# Iniciar sesión con admin@admin.com / 12345678
# Esperado: redirige al dashboard
# └─ Ref: ADR 0030

# 4. Los datos son locales — sin servidor
# Apagar just dev-api
# Login sigue funcionando (SQLite local)
# └─ Ref: ADR 0004, ADR 0030

# 5. El mismo dominio valida en ambos entornos
# Email inválido en desktop → mismo error que en browser
# └─ Ref: ADR 0030 — código compartido

# 6. Build de producción
cargo tauri build
# Esperado: binario en target/release/bundle/ < 15MB
# └─ Ref: ADR 0013

# 7. Sin herramientas de desarrollo en el instalador
ls target/release/bundle/
# Esperado: solo instaladores nativos
# └─ Ref: ADR 0013
```

---

## Bloque M.I-V — Mobile

> **Referencia:** ADR 0030 (Multiplataforma), ADR 0024 (Local-First), docs/02-STACK.md L382

```bash
# 1. La web es responsive
# Abrir en Chrome DevTools → modo móvil → sin scroll horizontal
# └─ Ref: ADR 0022, docs/03-STRUCTURE.md L450-453

# 2. PWA instalable
# Chrome Android → "Añadir a pantalla de inicio" → app funciona
# └─ Ref: ADR 0022, ADR 0024

# 3. Tauri Android arranca
npx tauri android dev
# Esperado: App abre en emulador, UI visible
# └─ Ref: docs/02-STACK.md L382

# 4. Login funciona sin internet
# Activar modo avión en el dispositivo
# Intentar login → funciona (SQLite local)
# └─ Ref: ADR 0024, ADR 0004

# 5. El mismo código de validación
# Email inválido → mismo error en servidor, desktop y mobile
# └─ Ref: ADR 0030 — código compartido
```

---

## Troubleshooting — Frontend (Astro + Svelte)

> **Referencia:** ADR 0022 (Frontend), ADR 0023 (i18n), docs/02-STACK.md L368-400

### FE.I — Foundation + Types

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| `pnpm dev` error "Cannot find module" | Dependencias no instaladas | `pnpm install` en `apps/web/` — Ref: ADR 0012 |
| `just types` genera tipos vacíos | buf no configurado | Verificar `buf.yaml` y `proto/` existen — Ref: ADR 0027 |
| `api.ts` tiene diff después de `just types` | Tipos editados manualmente | Restaurar original: `git checkout apps/web/src/lib/types/api.ts` — Ref: ADR 0027 |
| TypeScript error en imports de Svelte | Path alias mal configurado | Verificar `tsconfig.json` `paths: {"$lib/*": ["./src/lib/*"]}` — Ref: docs/03-STRUCTURE.md L425-484 |
| HMR (hot reload) no funciona | Astro dev server en otra red | Verificar `astro.config.mjs` server.host — Ref: docs/02-STACK.md L371 |

### FE.II — Components + State

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Estado no persiste entre navegación | Runes sin contexto global | Usar Svelte 5 Context API o stores — Ref: docs/02-STACK.md L378-381 |
| Form inputs no validan | ArkType no configurado | Verificar `schema.assert(data)` en onsubmit — Ref: docs/02-STACK.md L397 |
| `createMutation` no funciona | TanStack Query sin provider | Añadir `QueryClientProvider` en layout — Ref: docs/02-STACK.md L383-387 |
| Botón "Crear usuario" aparece sin permiso | PermissionGate no usado | Envolver botón en `<PermissionGate permission="users:write">` — Ref: ADR 0006 |

### FE.III — Layouts + Navigation

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Sidebar no colapsa en móvil | Media query mal | Usar Tailwind `lg:hidden` o breakpoint correcto — Ref: ADR 0022, docs/02-STACK.md L376-377 |
| CommandPalette no abre | Event listener en body | Verificar `on:keydown` en `BaseLayout.astro` — Ref: docs/03-STRUCTURE.md L472-476 |
| Flash de contenido no autenticado | SSR sin validación de sesión | Añadir `Astro.redirect('/login')` en layout protegido — Ref: ADR 0022 |
| UserTable scroll horizontal no funciona | `overflow-x-auto` en contenedor | Añadir `class="overflow-x-auto"` al `<table>` wrapper — Ref: docs/03-STRUCTURE.md L516-518 |

### FE.IV-VI — i18n + Formatters

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Paraglide no carga strings | `project.inlang/settings.json` mal | Verificar `$import { de } from './de.js'` generado — Ref: ADR 0023 |
| Cambio de idioma no funciona | `setLocale()` no llamado | Usar `setLocale(lang)` en select de idioma — Ref: ADR 0023 |
| Fechas mal formateadas | `Intl.DateTimeFormat` sin locale | Pasar `getLocale()` al formatter — Ref: docs/02-STACK.md L400 |
| Hora en UTC en vez de local | `timeZone` no especificado | Añadir `timeZone: 'America/La_Paz'` al options — Ref: ADR 0023 |

### FE.VII — PWA + Responsive

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| PWA no se instala | Manifest mal vinculado | Verificar `<link rel="manifest" href="/manifest.json">` en `<head>` — Ref: ADR 0022 |
| Service Worker no registra | `astro.config.mjs` sin SW | Añadir `@vite-pwa/astro` o `@astrojs/service-worker` — Ref: ADR 0024 |
| Lighthouse PWA < 80 | Faltan iconos o theme_color | Completar `manifest.json` con 192x192 y 512x512 — Ref: ADR 0022 |
| Touch targets < 44px | Tailwind sizing pequeño | Usar `min-h-11 min-w-11` (44px = 2.75rem = 11 * 0.25rem) — Ref: ADR 0022 |
| Layout roto en móvil | Viewport meta tag faltante | Añadir `<meta name="viewport" content="width=device-width, initial-scale=1">` — Ref: docs/03-STRUCTURE.md L450-453 |

### L.1-L.7 — Landing Page

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Form leads no envía | TanStack mutation sin endpoint | Verificar `createMutation` apunta a `/api/v1/leads` — Ref: ADR 0029 |
| Rate limit 429 inmediato | IP bloqueada por pruebas previas | Esperar 60 segundos o reiniciar servidor — Ref: ADR 0009 |
| Meta tags OG no aparecen | `AstroSeo` en client-side render | Mover a SSR layout con `<AstroSeo>` en `<head>` — Ref: ADR 0022 |
| Core Web Vitals > 2.5s | Imágenes sin optimizar | Usar `<Image />` de Astro con `widths={[400, 800, 1200]}` — Ref: ADR 0022 |
| Honeypot visible | CSS no oculta el campo | Añadir `class="hidden"` o `position: absolute; left: -9999px` — Ref: ADR 0029 |

---

## Verificaciones de código — Estándares ADR 0011

> **Referencia:** ADR 0011 (Estándares de Desarrollo), ADR 0028 (Auditoría)
>
> Reglas de atomicidad y calidad de código. Ejecutar en cada PR o con `just check-code`.

```bash
# 1. Funciones ≤30 líneas de lógica real
find crates/ apps/ -name "*.rs" -exec sh -c '
  for file; do
    awk "/^fn /{fn=NR;line=0} /^fn /{next} fn{line++} line>30{print \">30 líneas: $file\"; exit 1}" "$file"
  done
' sh {} +
# Esperado: sin output (todas las funciones ≤30 líneas)
# └─ Ref: ADR 0011 — Atomicidad de funciones

# 2. Archivos ≤200 líneas (excluyendo tests y docs)
find crates/ apps/ -name "*.rs" ! -name "*test*" ! -name "*doc*" | while read f; do
  lines=$(wc -l < "$f")
  if [ "$lines" -gt 200 ]; then
    echo "Muy largo ($lines líneas): $f"
  fi
done
# Esperado: sin output (todos los archivos ≤200 líneas)
# └─ Ref: ADR 0011 — Responsabilidad única

# 3. Sin // TODO sin ticket asignado
grep -rn "// TODO[^#]" crates/ apps/ --include="*.rs" --include="*.ts" --include="*.svelte"
# Esperado: cero resultados o formato: // TODO(#123): descripción
# └─ Ref: ADR 0011 — Deuda técnica documentada

# 4. Sin FIXME o HACK sin contexto
grep -rn "FIXME\|HACK" crates/ apps/ --include="*.rs" --include="*.ts" --include="*.svelte" | grep -v "contexto\|explicación"
# Esperado: cero resultados, o explicación detallada junto al FIXME
# └─ Ref: ADR 0011 — Código explícito

# 5. Regla del Boy Scout: sin código muerto comentado
grep -rn "^\s*//.*println!\|^\s*//.*dbg!" crates/ --include="*.rs"
# Esperado: cero resultados (borrar código muerto, no comentarlo)
# └─ Ref: ADR 0011 — Boy Scout Rule

# 6. Documentación pública para APIs
cargo doc --no-deps 2>&1 | grep "missing" || true
# Esperado: cero warnings de documentación faltante en items públicos
# └─ Ref: ADR 0011 — Código mantenible

# 7. sintonia check lines (cuando esté disponible)
# just check-lines  # Verificar límites automatizados
# just check-fn     # Verificar longitud de funciones
```

**Scripts recomendados en justfile:**
```just
# Verificar estándares de código
check-lines:
    @echo "Verificando archivos ≤200 líneas..."
    @find crates/ apps/ -name "*.rs" ! -name "*test*" ! -name "*doc*" | while read f; do \
        lines=$(wc -l < "$$f"); \
        if [ "$$lines" -gt 200 ]; then \
            echo "⚠️  $$lines líneas: $$f"; \
        fi; \
    done

check-fn:
    @echo "Verificando funciones ≤30 líneas..."
    @# Requiere cargo-expand o análisis AST
    @cargo clippy -- -W clippy::too_many_lines
```

---

> **Referencia:** ADR 0001 (Hexagonal), ADR 0008 (PASETO), ADR 0006 (RBAC), ADR 0028 (Auditoría)

```bash
# 1. domain NO importa sqlx
grep -r "sqlx" crates/domain/Cargo.toml
# Esperado: cero resultados
# └─ Ref: ADR 0001, docs/03-STRUCTURE.md L188-194

# 2. domain NO importa axum
grep -r "axum" crates/domain/Cargo.toml
# Esperado: cero resultados
# └─ Ref: ADR 0001, docs/03-STRUCTURE.md L188-194

# 3. JWT prohibido en todo el workspace
grep -r "jsonwebtoken" . --include="*.toml"
# Esperado: cero resultados
# └─ Ref: ADR 0008 — JWT prohibido

# 4. No hay DELETE real en users
grep -rn "DELETE FROM users" . --include="*.rs"
# Esperado: cero resultados — solo UPDATE deleted_at
# └─ Ref: ADR 0006 — Soft Delete

# 5. No hay tipos TypeScript escritos a mano en api.ts
head -3 apps/web/src/lib/types/api.ts
# Esperado: comentario "// GENERATED by buf generate — do not edit"
# └─ Ref: ADR 0027, docs/02-STACK.md L419-424

# 6. sintonia check arch (cuando el CLI esté implementado)
sintonia check arch
# Esperado: "✅ Arquitectura limpia"
# └─ Ref: ADR 0028

# 7. Todos los tests pasan
cargo nextest run --all-targets
# Esperado: todos en verde
# └─ Ref: ADR 0010, docs/02-STACK.md L442-443

# 8. Sin warnings en el código
cargo clippy --all-targets -- -D warnings
# Esperado: "warning: 0 warnings emitted"
# └─ Ref: ADR 0013, docs/02-STACK.md L412

# 9. Sin vulnerabilidades conocidas
just audit
# Esperado: cargo deny y cargo audit sin issues
# └─ Ref: ADR 0013, docs/02-STACK.md L412
```

---

## Checklist de "listo para producción"

> **Referencia:** ADR 0010 (Testing), ADR 0013 (Build), ADR 0014 (Deploy), ADR 0008 (PASETO)

Antes de hacer el primer deploy real, verificar todo esto:

```
[ ] cargo nextest run --all-targets → todos pasan
    └─ Ref: ADR 0010, docs/02-STACK.md L442-443
[ ] cargo clippy --all-targets -D warnings → cero warnings
    └─ Ref: ADR 0013, docs/02-STACK.md L412
[ ] just audit → cargo deny + cargo audit sin issues
    └─ Ref: ADR 0013
[ ] just types-check → api.ts sin diff
    └─ Ref: ADR 0027
[ ] just prepare → .sqlx/ actualizado
    └─ Ref: ADR 0013, docs/02-STACK.md L415
[ ] grep "jsonwebtoken" → cero resultados
    └─ Ref: ADR 0008
[ ] grep "DELETE FROM users" → cero resultados
    └─ Ref: ADR 0006
[ ] La imagen pesa menos de 15MB
    └─ Ref: ADR 0013
[ ] podman run ... sh → falla (distroless sin shell)
    └─ Ref: ADR 0013
[ ] SQLX_OFFLINE=true cargo build --release → funciona
    └─ Ref: ADR 0013
[ ] curl /health → 200 OK dentro del contenedor
    └─ Ref: ADR 0014, docs/02-STACK.md L366
[ ] .env.local NO está en git (git status)
    └─ Ref: ADR 0002, docs/02-STACK.md L91
[ ] PASETO_SECRET tiene 32 bytes (wc -c .env.local | grep PASETO)
    └─ Ref: ADR 0008, docs/02-STACK.md L88
[ ] admin@admin.com password cambiada antes del deploy
    └─ Ref: docs/01-ARCHITECTURE.md L143
[ ] Litestream está replicando (litestream snapshots)
    └─ Ref: ADR 0004, docs/02-STACK.md L168
[ ] Healthchecks.io pings configurados
    └─ Ref: ADR 0015, docs/02-STACK.md L348
[ ] kamal rollback probado y funciona
    └─ Ref: ADR 0014, docs/02-STACK.md L360-368
```
