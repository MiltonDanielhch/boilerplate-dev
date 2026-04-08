# Roadmap — Admin Dashboard

> **Objetivo:** Implementar panel de administración para gestionar usuarios, contenido, métricas y configuración del sistema.
>
> **Stack:** Astro · Svelte 5 · TanStack Table · Recharts · Rust Axum · SQLx
>
> **ADRs:** ADR 0001 (Hexagonal), ADR 0006 (RBAC), ADR 0021 (Utoipa), ADR 0022 (Frontend)
>
> **Criterio de activación:** El sistema tiene >100 usuarios o necesita gestión manual de contenido/usuarios por parte de admins.
>
> **Prerrequisitos:** Auth Fullstack (05-ROADMAP-AUTH-FULLSTACK.md) — RBAC con roles de admin
>
> **Relacionado con:** Backend (03-ROADMAP-BACKEND.md), Frontend (04-ROADMAP-FRONTEND.md), RBAC en Auth

---

## Estados

```
[ ] Pendiente   [~] En progreso   [x] Completado   [!] Bloqueado
```

---

## Progreso General

| Bloque | Nombre | Estado | Progreso |
|--------|--------|--------|----------|
| AD.1 | Foundation — Layout & Routing | [ ] | 0% |
| AD.2 | User Management | [ ] | 0% |
| AD.3 | Content Management | [ ] | 0% |
| AD.4 | Analytics Dashboard | [ ] | 0% |
| AD.5 | System Settings | [ ] | 0% |
| AD.6 | Security & Audit | [ ] | 0% |

---

## AD.1 — Foundation — Layout & Routing

> **ADRs:** ADR 0006 (RBAC), ADR 0022 (Frontend)
> **Output:** Layout de admin con sidebar, routing protegido, theme oscuro
> **Tiempo estimado:** 2-3 días

```
[ ] Backend — Middleware de Admin:
    [ ] Crear middleware require_admin_role en crates/infrastructure/src/http/middleware/
        [ ] Verificar PASETO token + role == "admin" o "superadmin"
        [ ] Retornar 403 si no tiene permisos
        └─ Ref: ADR 0006 L45-67, ADR 0008 (PASETO, no JWT)

[ ] Backend — Endpoints base:
    [ ] GET /api/v1/admin/me — Verificar si el user es admin
        └─ Retorna: { is_admin: true, permissions: ["users:read", "users:write"] }

[ ] Frontend — Layout Admin:
    [ ] Crear apps/web/src/layouts/AdminLayout.astro:
        [ ] Sidebar navegación (colapsable en mobile)
        [ ] Header con user info y logout
        [ ] Theme oscuro por defecto (slate-950)
        [ ] Protected: redirige a /login si no es admin
    └─ Ref: ADR 0022 L200-250

[ ] Frontend — Rutas protegidas:
    [ ] /admin/* → AdminLayout
    [ ] /admin/dashboard → Página principal
    [ ] Redirigir /admin → /admin/dashboard
    [ ] 403 page si intenta acceder sin permisos

[ ] Frontend — Componentes base:
    [ ] AdminSidebar.svelte — Navegación con iconos
    [ ] AdminHeader.svelte — Breadcrumbs, user menu
    [ ] AdminCard.svelte — Cards de métricas
    [ ] DataTable.svelte — Tabla genérica con sorting, filtering
        └─ Usar TanStack Table v8
```

---

## AD.2 — User Management

> **ADRs:** ADR 0006 (RBAC), ADR 0008 (PASETO), ADR 0021 (Utoipa)
> **Output:** CRUD completo de usuarios con roles, búsqueda, filtros
> **Tiempo estimado:** 3-4 días

```
[ ] Backend — Domain + Use Cases:
    [ ] Extender User entity con campos admin:
        [ ] is_active: bool (soft delete)
        [ ] email_verified: bool
        [ ] last_login_at: Option<DateTime<Utc>>
        [ ] created_by: Option<Uuid> (quién creó el user)
    [ ] Use Case: ListUsersQuery con filtros:
        [ ] Por rol, por estado (active/inactive), por fecha
        [ ] Pagination: page, per_page (max 100)
        [ ] Search: email ILIKE '%query%'
    [ ] Use Case: UpdateUserCommand (admin puede editar otros users)
    [ ] Use Case: DeactivateUserCommand (soft delete)
    [ ] Use Case: ImpersonateUser (admin actúa como otro user)
        └─ Ref: ADR 0006 L80-120

[ ] Backend — Repository:
    [ ] UserRepository::list_with_filters(filters: UserFilters) -> PaginatedResult<User>
    [ ] UserRepository::update_by_admin(id, changes) -> Result<User, DomainError>
    [ ] UserRepository::soft_delete(id) -> Result<(), DomainError>
    [ ] UserRepository::count_by_role() -> Vec<(Role, i64)>

[ ] Backend — API Endpoints:
    [ ] GET /api/v1/admin/users — Listar usuarios (paginado, filtrado)
        └─ Query params: ?page=1&per_page=20&role=admin&search=john&active=true
    [ ] GET /api/v1/admin/users/:id — Detalle de usuario
    [ ] PUT /api/v1/admin/users/:id — Actualizar usuario
    [ ] DELETE /api/v1/admin/users/:id — Desactivar usuario (soft delete)
    [ ] POST /api/v1/admin/users/:id/impersonate — Generar token de impersonación
    [ ] GET /api/v1/admin/users/stats — Estadísticas (total, por rol, activos)
        └─ Ref: ADR 0021 L150-200

[ ] Frontend — Página Users:
    [ ] /admin/users — Lista de usuarios:
        [ ] DataTable con columns: email, role, status, last_login, actions
        [ ] Filtros: dropdown de roles, toggle active/inactive, search input
        [ ] Pagination controls
        [ ] Botón "Create User" → modal/form
    └─ Usar TanStack Table + Svelte 5 runes

[ ] Frontend — User Detail Modal:
    [ ] Ver información completa del usuario
    [ ] Editar campos: role, is_active, email_verified
    [ ] Botón "Impersonate" (con confirmación)
    [ ] Botón "Deactivate" (con confirmación de riesgo)

[ ] Frontend — Create User Form:
    [ ] Email, password (generada auto), role
    [ ] Checkbox: email_verified, send_welcome_email
    [ ] Validación con ArkType
```

---

## AD.3 — Content Management

> **ADRs:** ADR 0001 (Hexagonal), ADR 0029 (Landing), ADR 0022 (Frontend)
> **Output:** Gestión de leads, contenido landing, CMS básico
> **Tiempo estimado:** 2-3 días

```
[ ] Backend — Entities:
    [ ] Lead (ya existe de landing) — añadir campos admin:
        [ ] status: new | contacted | qualified | converted | archived
        [ ] notes: Text (admin comments)
        [ ] assigned_to: Option<Uuid> (admin asignado)
    [ ] ContentBlock: CMS editable por admin:
        [ ] key: String (único, ej: "hero_title")
        [ ] content: Text
        [ ] content_type: text | markdown | html
        [ ] last_modified_by: Uuid

[ ] Backend — Use Cases:
    [ ] ListLeadsQuery con filtros (status, date range, assigned_to)
    [ ] UpdateLeadStatusCommand
    [ ] AssignLeadCommand
    [ ] AddLeadNoteCommand
    [ ] ListContentBlocksQuery
    [ ] UpdateContentBlockCommand

[ ] Backend — API:
    [ ] GET /api/v1/admin/leads — Listar leads con filtros
    [ ] PUT /api/v1/admin/leads/:id/status — Cambiar status
    [ ] POST /api/v1/admin/leads/:id/notes — Añadir nota
    [ ] GET /api/v1/admin/content — Listar bloques de contenido
    [ ] PUT /api/v1/admin/content/:key — Actualizar contenido

[ ] Frontend — Leads Management:
    [ ] /admin/leads — Tabla de leads:
        [ ] Columns: email, status, source, date, assigned_to, actions
        [ ] Filtros: por status, date range, search por email
        [ ] Actions: View, Edit status, Assign, Archive
    [ ] Lead Detail:
        [ ] Timeline de cambios (status history)
        [ ] Notes section (add, view)
        [ ] Quick actions: Mark contacted, Convert, Archive

[ ] Frontend — CMS:
    [ ] /admin/content — Editor de contenido:
        [ ] Lista de bloques editables (hero, features, pricing, etc.)
        [ ] Editor WYSIWYG markdown (ej: Milkdown o editor simple)
        [ ] Preview en vivo de cambios
        [ ] Publish / Save draft
```

---

## AD.4 — Analytics Dashboard

> **ADRs:** ADR 0016 (Observabilidad), ADR 0022 (Frontend)
> **Output:** Dashboard con métricas de negocio (no técnicas), gráficos
> **Tiempo estimado:** 2-3 días

```
[ ] Backend — Analytics Aggregation:
    [ ] Crear crate crates/analytics/ (o usar eventos existentes)
    [ ] Metricas de negocio:
        [ ] Daily Active Users (DAU) — unique sessions últimos 7 días
        [ ] Monthly Active Users (MAU) — unique sessions últimos 30 días
        [ ] New users por día (registrations)
        [ ] Conversion funnel: visit → signup → activation
        [ ] Retention: cohort analysis (opcional, complejo)
    [ ] Aggregations SQL:
        [ ] SELECT DATE(created_at), COUNT(*) FROM users GROUP BY DATE
        [ ] SELECT status, COUNT(*) FROM leads GROUP BY status
    [ ] Cache con Moka para queries pesadas (TTL 5 min)
        └─ Ref: ADR 0017

[ ] Backend — API:
    [ ] GET /api/v1/admin/analytics/overview — Métricas clave (KPIs)
        └─ { dau: 1234, mau: 5678, new_today: 42, conversion_rate: 3.5 }
    [ ] GET /api/v1/admin/analytics/users?period=7d — Serie temporal de users
        └─ [{ date: "2026-01-01", new: 10, active: 100 }, ...]
    [ ] GET /api/v1/admin/analytics/leads?period=30d — Leads funnel
    [ ] GET /api/v1/admin/analytics/realtime — Stats de últimos 5 minutos
        └─ { active_now: 42, page_views_last_5min: 150 }

[ ] Frontend — Dashboard Home:
    [ ] /admin/dashboard — Overview:
        [ ] KPI Cards: DAU, MAU, Total Users, Conversion Rate
        [ ] Sparklines: mini gráficos de tendencia
        [ ] Alerts: usuarios nuevos hoy, leads sin asignar
    └─ Usar Recharts para gráficos

[ ] Frontend — Charts:
    [ ] Line chart: Users over time (7d, 30d, 90d)
    [ ] Bar chart: Signups por día
    [ ] Pie chart: Leads by status
    [ ] Funnel chart: Conversion steps
    [ ] Real-time counter: Active users now (WebSocket polling)

[ ] Frontend — Reports:
    [ ] Date range picker (start, end)
    [ ] Export to CSV (users, leads)
    [ ] Print-friendly view
```

---

## AD.5 — System Settings

> **ADRs:** ADR 0002 (Configuración Tipada), ADR 0016 (Observabilidad)
> **Output:** Panel de configuración del sistema (feature flags, emails, etc.)
> **Tiempo estimado:** 1-2 días

```
[ ] Backend — Settings Store:
    [ ] Extender Config con tabla settings (key-value con tipos)
    [ ] Settings editables:
        [ ] maintenance_mode: bool
        [ ] registration_enabled: bool
        [ ] default_user_role: String
        [ ] email_from_address: String
        [ ] session_timeout_minutes: i32
        [ ] max_failed_logins: i32
    [ ] Cache en Moka (settings raramente cambian)

[ ] Backend — API:
    [ ] GET /api/v1/admin/settings — Todas las settings
    [ ] PUT /api/v1/admin/settings/:key — Actualizar setting
    [ ] POST /api/v1/admin/settings/reset — Reset a defaults

[ ] Frontend — Settings Page:
    [ ] /admin/settings — Formulario dinámico:
        [ ] Toggle switches para bools
        [ ] Inputs validados para strings/numbers
        [ ] Dropdowns para opciones predefinidas
        [ ] Secciones: General, Security, Email, Features
    [ ] Danger zone: Maintenance mode (con confirmación extra)
```

---

## AD.6 — Security & Audit

> **ADRs:** ADR 0007 (Audit), ADR 0009 (Rate Limit), ADR 0006 (RBAC)
> **Output:** Logs de auditoría, seguridad, control de accesos avanzado
> **Tiempo estimado:** 2 días

```
[ ] Backend — Audit Log:
    [ ] Extender ADR 0007 con tabla audit_logs_admin:
        [ ] action: enum (user_created, user_updated, user_deactivated, content_updated, settings_changed, impersonation_started)
        [ ] performed_by: Uuid (admin)
        [ ] target_type: String ("User", "Lead", "Content")
        [ ] target_id: String
        [ ] old_values: Option<Json>
        [ ] new_values: Option<Json>
        [ ] ip_address: String
        [ ] user_agent: String
    [ ] Middleware automático para loggear cambios admin

[ ] Backend — API:
    [ ] GET /api/v1/admin/audit-logs — Listar logs con filtros
        [ ] Por admin, por acción, por fecha, por target
    [ ] GET /api/v1/admin/sessions — Sessions activas (para revoke)

[ ] Frontend — Audit Page:
    [ ] /admin/audit — Timeline de cambios:
        [ ] Tabla: timestamp, admin, action, target, details
        [ ] Filtros: por fecha, por acción, por admin
        [ ] Expandir para ver diff de cambios

[ ] Frontend — Security:
    [ ] /admin/security — Panel de seguridad:
        [ ] Lista de sesiones activas (con opción de revoke)
        [ ] Failed login attempts (últimas 24h)
        [ ] IPs bloqueadas por rate limiting
        [ ] 2FA status de admins
```

---

## Design System — Guía de UI/UX para Admin

> **Referencia:** `docs/DASHBOARD-DISENO.md` (integrado aquí)
>
> **Stack:** Astro SSR · Svelte 5 Runes · Tailwind v4 · TanStack Query

### Layout del Dashboard

```
┌─────────────────────────────────────────────┐
│  Topbar — búsqueda · notificaciones · perfil │
├──────────┬──────────────────────────────────┤
│          │                                  │
│ Sidebar  │     Canvas Central               │
│          │     (contenido de la página)     │
│ colaps.  │                                  │
│          │                                  │
└──────────┴──────────────────────────────────┘
```

**Implementación:**
```astro
// layouts/AdminLayout.astro
const token = Astro.cookies.get('access_token')?.value;
if (!token) return Astro.redirect('/login');
---
<Topbar client:load />
<Sidebar client:load />
<main class="ml-64 p-6"><slot /></main>
<CommandPalette client:load />
```

### Componentes Base (AD.1)

| Componente | Props | Función |
|------------|-------|---------|
| `Sidebar.svelte` | `collapsed: boolean` | Navegación con RBAC por permisos |
| `Topbar.svelte` | — | Búsqueda + notificaciones + perfil |
| `CommandPalette.svelte` | — | Ctrl+K, acciones filtradas por permisos |
| `PermissionGate.svelte` | `permission: string` | Oculta elementos sin permiso |
| `EmptyState.svelte` | `title, description, action` | Tablas sin datos — OBLIGATORIO |

### Sistema de Temas (Global CSS Variables)

```css
:root {
    --color-primary: #534AB7;
    --color-canvas:  #F8F8F6;
    --color-card:    #FFFFFF;
    --sidebar-width: 256px;
    --topbar-height: 60px;
}
[data-theme="dark"] {
    --color-canvas: #1A1A18;
    --color-card:   #242422;
}
```

**ThemeToggle.svelte:** Persiste en localStorage, respeta `prefers-color-scheme`.

### Patrones de Data Tables (AD.2-AD.4)

```svelte
<script>
    let page = $state(1), search = $state('');
    const query = createQuery({
        queryKey: ['users', page, search],
        queryFn: () => fetchUsers({ page, search }),
        staleTime: 30_000,
    });
</script>

{#if $query.isLoading}
    <TableSkeleton rows={10} />
{:else if $query.data?.items.length === 0}
    <EmptyState title="No hay usuarios" action={{ label: 'Crear', href: '#' }} />
{:else}
    <Table>
        {#each $query.data.items as user (user.id)}
            <TableRow>
                <PermissionGate permission="users:write">
                    <ActionMenu onEdit={...} onDelete={...} />
                </PermissionGate>
            </TableRow>
        {/each}
    </Table>
    <Pagination page={page} total={$query.data.total} />
{/if}
```

### KPI Cards (AD.4 Analytics)

```svelte
<script>
    const query = createQuery({
        queryKey: ['analytics', 'dau'],
        queryFn: () => fetch('/api/v1/admin/analytics/overview').then(r => r.json()),
        refetchInterval: 60_000,
    });
</script>

<KpiCard title="Usuarios Activos" value={$query.data?.dau} trend="+12%" />
```

### RBAC en UI — PermissionGate

```svelte
<!-- Oculta botón si no tiene permiso -->
<PermissionGate permission="users:write">
    <Button onclick={openCreateModal}>Crear usuario</Button>
</PermissionGate>

<!-- Oculta nav item -->
<PermissionGate permission="audit:read">
    <NavItem href="/admin/audit">Auditoría</NavItem>
</PermissionGate>
```

> **Doble verificación:** Permisos se chequean en servidor (handler middleware) Y en UI (PermissionGate). — ADR 0006

### EmptyState Obligatorio

Toda tabla debe tener estado vacío diseñado:
```svelte
<EmptyState
    title="No hay usuarios aún"
    description="Crea el primer usuario para empezar a gestionar el sistema."
    action={{ label: 'Crear usuario', href: '/admin/users/new' }}
/>
```

---

## Verificación Final

### Backend

```bash
# Verificar RBAC de admin
curl -H "Authorization: Bearer $USER_TOKEN" http://localhost:8080/api/v1/admin/users
# Esperado: 403 Forbidden (user normal no puede)

curl -H "Authorization: Bearer $ADMIN_TOKEN" http://localhost:8080/api/v1/admin/users
# Esperado: 200 OK con lista de usuarios

# Verificar paginación
curl "http://localhost:8080/api/v1/admin/users?page=1&per_page=5" \
  -H "Authorization: Bearer $ADMIN_TOKEN"
# Esperado: { data: [...], meta: { total: 100, page: 1, per_page: 5 } }

# Verificar analytics
curl -H "Authorization: Bearer $ADMIN_TOKEN" http://localhost:8080/api/v1/admin/analytics/overview
# Esperado: { dau: N, mau: N, conversion_rate: N.N }

# Verificar audit logging
sqlite3 data/database.sqlite "SELECT * FROM audit_logs ORDER BY created_at DESC LIMIT 5;"
# Esperado: Logs de acciones admin recientes
```

### Frontend

```bash
# Build
cd apps/web && pnpm build
# Esperado: sin errores, /admin/* incluido

# Types
cd apps/web && pnpm check
# Esperado: 0 errors

# Admin route accessible (con auth)
curl http://localhost:4321/admin/dashboard
# Esperado: 200 con HTML (o redirect a login si no auth)
```

### Arquitectura

- [ ] Admin endpoints protegidos por middleware RBAC
- [ ] No hay lógica de admin en dominio (solo en application/web)
- [ ] Audit logs capturan todos los cambios importantes
- [ ] Soft delete (no hard delete) para users
- [ ] Analytics queries optimizadas (con índices)
- [ ] Settings cacheadas (no query DB cada request)

---

## Troubleshooting — Admin Dashboard

### Error: "Forbidden" al acceder a /admin

**Síntoma:** 403 en todos los endpoints de admin
**Causa:** El usuario no tiene rol "admin" o "superadmin"
**Solución:**
```bash
# Asignar rol admin manualmente
sqlite3 data/database.sqlite "UPDATE users SET role = 'admin' WHERE email = 'tu@email.com';"
```

### Error: Analytics muy lentos

**Síntoma:** Dashboard tarda >5s en cargar
**Causa:** Queries de agregación sin índices
**Solución:**
```sql
-- Añadir índices para analytics
CREATE INDEX idx_users_created_at ON users(created_at);
CREATE INDEX idx_leads_status ON leads(status);
CREATE INDEX idx_audit_created_at ON audit_logs(created_at);
```

### Error: DataTable no muestra datos

**Síntoma:** Tabla vacía aunque hay users en DB
**Causa:** Frontend espera formato diferente de paginación
**Solución:** Verificar que API retorna `{ data: [...], meta: {...} }` no `[...]` directo

### Error: Impersonate no funciona

**Síntoma:** Al impersonar, sigue viendo datos del admin
**Causa:** Token de impersonación no se está usando correctamente
**Solución:** Verificar que el token de impersonación reemplaza el header Authorization

---

## Notas de Diseño

### UX Principles para Admin

1. **Density over whitespace:** Admins quieren ver más datos, no menos
2. **Actions bulk:** Permitir acciones en múltiples items (checkboxes)
3. **Confirmaciones:** Operaciones destructivas (delete) requieren confirmación explícita
4. **Feedback inmediato:** Toast notifications para todas las acciones
5. **Keyboard shortcuts:** Ctrl+K para search, Escape para cerrar modales

### Security Considerations

1. **Nunca exponer passwords** — ni hashes — en APIs de admin
2. **Impersonate logging:** Siempre loggear quién impersonó a quién y cuándo
3. **Rate limiting:** Admin endpoints tienen rate limits más estrictos (target de ataque)
4. **Audit todo:** Cada write debe dejar trace en audit_logs

### Performance

1. **Cursor-based pagination** para tablas grandes (no offset)
2. **Debounced search** (300ms) para no saturar backend
3. **Virtual scrolling** si tablas >1000 filas visibles
4. **Cache settings** — no query DB para settings en cada request

---

## Checklist de "Listo para Producción"

- [ ] Todos los endpoints de admin requieren autenticación + rol admin
- [ ] Audit logs funcionan para todas las operaciones de escritura
- [ ] Analytics muestran datos reales (no mocks)
- [ ] Soft delete implementado (users "eliminados" aún existen en DB)
- [ ] Mobile responsive (admin usable en tablet)
- [ ] Tests E2E: login como admin → crear user → verificar en lista → logout
- [ ] Documentación: capturas de pantalla del admin para wiki
- [ ] Backup strategy: audit_logs deben retenerse por ley (GDPR/SOC2)

---

**Creado:** 2026-04-04
**Última actualización:** 2026-04-04
**Próxima revisión:** Cuando se implementen 3+ bloques
**Responsable:** Staff Engineer + Product Owner

---

## Relación con Otros Roadmaps

| Roadmap | Relación |
|---------|----------|
| `05-ROADMAP-AUTH-FULLSTACK.md` | Admin requiere RBAC completo de Auth |
| `06-ROADMAP-LANDING.md` | Admin gestiona leads capturados en landing |
| `50-ROADMAP-FASE2.md` | NATS puede enviar eventos de admin (analytics) |
| `ADR 0006` | RBAC es el fundamento de permisos de admin |
| `ADR 0007` | Audit logging debe extenderse para acciones admin |
