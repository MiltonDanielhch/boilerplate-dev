# Diseño del Dashboard — boilerplate

**Stack:** Astro SSR · Svelte 5 Runes · Tailwind v4 · shadcn-svelte · TanStack Query · Utoipa
**ADRs relacionados:** 0022 (Frontend), 0006 (RBAC), 0008 (PASETO), 0021 (OpenAPI)

---

## Filosofía

El dashboard es la interfaz principal para usuarios autenticados. Sigue los mismos
principios del stack: HTML primero desde Astro SSR, Svelte 5 solo donde hay
interactividad real, TanStack Query para estado del servidor.

No es una SPA. Es un conjunto de páginas SSR con islas de interactividad.

---

## Estructura en el monorepo

```
apps/web/src/
├── pages/dashboard/
│   ├── index.astro              # Home / Overview — KPIs + actividad + feed
│   ├── users/
│   │   ├── index.astro          # Lista de usuarios (requires: users:read)
│   │   └── [id].astro           # Perfil de usuario
│   ├── audit/
│   │   └── index.astro          # Logs de auditoría (requires: audit:read)
│   └── settings/
│       └── index.astro          # Configuración de cuenta
├── layouts/
│   ├── DashboardLayout.astro    # Sidebar + Topbar + Canvas — verifica PASETO en servidor
│   └── LandingLayout.astro      # Para la landing page — sin sidebar
└── components/
    ├── layout/
    │   ├── Sidebar.svelte        # Navegación colapsable con Runes
    │   ├── Topbar.svelte         # Búsqueda + notificaciones + perfil
    │   └── CommandPalette.svelte # Ctrl+K — acciones rápidas filtradas por permisos
    ├── dashboard/
    │   ├── KpiCard.svelte        # Tarjeta de métrica individual — TanStack Query
    │   ├── ActivityChart.svelte  # Gráfico de actividad (Chart.js)
    │   ├── EventFeed.svelte      # Feed de últimas acciones de audit_logs
    │   └── SystemHealth.svelte   # Estado DB + Apalis + Litestream sync
    ├── users/
    │   ├── UserTable.svelte      # Tabla con paginación y acciones por fila
    │   ├── UserForm.svelte       # Modal de crear/editar usuario
    │   └── UserCard.svelte
    └── ui/
        ├── Button.svelte
        ├── Input.svelte
        ├── Modal.svelte
        ├── Table.svelte
        ├── EmptyState.svelte     # Obligatorio en toda tabla sin datos
        ├── PermissionGate.svelte # Oculta elementos sin el permiso necesario
        └── ThemeToggle.svelte    # Modo oscuro/claro persistente
```

---

## 1. Layout del Dashboard

Tres zonas fijas en todas las páginas autenticadas:

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

```astro
---
// layouts/DashboardLayout.astro
// Verifica PASETO en el servidor — si no hay sesión, redirige a /login
// Sin flash de contenido no autenticado — ADR 0008
const token = Astro.cookies.get('access_token')?.value;
if (!token) return Astro.redirect('/login');
---
<html>
  <body>
    <Topbar client:load />
    <Sidebar client:load />
    <main class="ml-64 p-6">
      <slot />
    </main>
    <CommandPalette client:load />
  </body>
</html>
```

### Sidebar — colapsable con RBAC

```svelte
<!-- components/layout/Sidebar.svelte -->
<script lang="ts">
    import { getAuthState } from '$lib/stores/auth.svelte';
    let collapsed = $state(false);
    const auth = getAuthState();
</script>

<nav class:collapsed>
    <NavItem href="/dashboard"       icon="home">Inicio</NavItem>
    <NavItem href="/dashboard/users" icon="users"
        permission="users:read">Usuarios</NavItem>
    <NavItem href="/dashboard/audit" icon="shield"
        permission="audit:read">Auditoría</NavItem>
    <NavItem href="/dashboard/settings" icon="settings">Configuración</NavItem>
</nav>
```

### Command Palette (Ctrl+K)

```svelte
<!-- components/layout/CommandPalette.svelte -->
<script lang="ts">
    let open  = $state(false);
    let query = $state('');

    $effect(() => {
        const handler = (e: KeyboardEvent) => {
            if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
                e.preventDefault();
                open = !open;
            }
        };
        document.addEventListener('keydown', handler);
        return () => document.removeEventListener('keydown', handler);
    });

    // Acciones filtradas por permisos del usuario — ADR 0006
    const actions = [
        { label: 'Nuevo usuario', href: '/dashboard/users/new', permission: 'users:write' },
        { label: 'Ver auditoría', href: '/dashboard/audit',     permission: 'audit:read' },
        { label: 'Configuración', href: '/dashboard/settings' },
        { label: 'Cerrar sesión', action: logout },
    ];
</script>
```

---

## 2. Página de inicio (Home / Overview)

Responde: ¿qué requiere mi atención hoy?

```svelte
<!-- components/dashboard/KpiCard.svelte -->
<script lang="ts">
    import { createQuery } from '@tanstack/svelte-query';

    export let title:    string;
    export let endpoint: string;

    const query = createQuery({
        queryKey:        [endpoint],
        queryFn:         () => fetch(endpoint).then(r => r.json()),
        staleTime:       30_000,
        refetchInterval: 60_000,  // Refresca cada minuto
    });
</script>

<div class="kpi-card">
    <span class="title">{title}</span>
    {#if $query.isLoading}
        <Skeleton />
    {:else}
        <span class="value">{$query.data?.value ?? '—'}</span>
    {/if}
</div>
```

---

## 3. Plantilla de páginas de datos (tabla + acciones)

Patrón estándar para cualquier módulo con lista de datos:

```svelte
<!-- components/users/UserTable.svelte -->
<script lang="ts">
    import { createQuery } from '@tanstack/svelte-query';

    let page   = $state(1);
    let search = $state('');

    const query = createQuery({
        queryKey: ['users', page, search],
        queryFn:  () => fetchUsers({ page, search }),
        staleTime: 30_000,
    });
</script>

{#if $query.isLoading}
    <TableSkeleton rows={10} />
{:else if $query.data?.items.length === 0}
    <!-- EmptyState es OBLIGATORIO en toda tabla sin datos -->
    <EmptyState
        title="No hay usuarios"
        description="Crea el primer usuario para empezar"
        action={{ label: 'Crear usuario', href: '#create' }}
    />
{:else}
    <Table>
        {#each $query.data.items as user (user.id)}
            <TableRow>
                <Td>{user.email}</Td>
                <Td><RoleBadge roles={user.roles} /></Td>
                <Td>{formatDate(user.created_at)}</Td>    <!-- ADR 0023 -->
                <Td>
                    <PermissionGate permission="users:write"> <!-- ADR 0006 -->
                        <ActionMenu
                            onEdit={() => selected = user.id}
                            onDelete={() => softDelete(user.id)}
                        />
                    </PermissionGate>
                </Td>
            </TableRow>
        {/each}
    </Table>
    <Pagination page={page} total={$query.data.total} onPageChange={(p) => page = p} />
{/if}
```

---

## 4. Sistema de temas y CSS variables

Un solo archivo de tokens — el dashboard y la landing comparten la misma base:

```css
/* src/styles/global.css */
:root {
    /* Marca */
    --color-primary:     #534AB7;
    --color-primary-50:  #EEEDFE;
    --color-primary-800: #3C3489;

    /* Superficie */
    --color-canvas:  #F8F8F6;
    --color-card:    #FFFFFF;
    --color-border:  rgba(0, 0, 0, 0.08);

    /* Tipografía */
    --font-sans: 'Inter Variable', system-ui, sans-serif;
    --font-mono: 'JetBrains Mono', monospace;

    /* Layout */
    --sidebar-width: 256px;
    --topbar-height: 60px;
}

[data-theme="dark"] {
    --color-canvas: #1A1A18;
    --color-card:   #242422;
    --color-border: rgba(255, 255, 255, 0.08);
}
```

```svelte
<!-- components/ui/ThemeToggle.svelte -->
<script lang="ts">
    let dark = $state(
        localStorage.getItem('theme') === 'dark'
        || window.matchMedia('(prefers-color-scheme: dark)').matches
    );

    $effect(() => {
        document.documentElement.setAttribute('data-theme', dark ? 'dark' : 'light');
        localStorage.setItem('theme', dark ? 'dark' : 'light');
    });
</script>

<button onclick={() => dark = !dark} aria-label="Cambiar tema">
    {dark ? '☀️' : '🌙'}
</button>
```

---

## 5. Sistema de permisos en la UI — ADR 0006

```svelte
<!-- components/ui/PermissionGate.svelte -->
<script lang="ts">
    import { getAuthState } from '$lib/stores/auth.svelte';
    export let permission: string;

    const auth    = getAuthState();
    const allowed = $derived(auth.user?.permissions.includes(permission) ?? false);
</script>

{#if allowed}
    <slot />
{/if}
```

```svelte
<!-- Uso en cualquier componente -->
<PermissionGate permission="users:write">
    <Button onclick={openCreateModal}>Crear usuario</Button>
</PermissionGate>

<PermissionGate permission="audit:read">
    <NavItem href="/dashboard/audit">Auditoría</NavItem>
</PermissionGate>
```

Los permisos se verifican en dos capas:
- **Servidor (handler):** `require_permission("users:write")` middleware — ADR 0006
- **UI (cliente):** `PermissionGate` oculta elementos — experiencia de usuario limpia

---

## 6. Módulo de salud del sistema

```svelte
<!-- components/dashboard/SystemHealth.svelte -->
<script lang="ts">
    import { createQuery } from '@tanstack/svelte-query';

    const health = createQuery({
        queryKey:        ['health'],
        queryFn:         () => fetch('/api/v1/health').then(r => r.json()),
        refetchInterval: 10_000,  // Cada 10 segundos
    });
</script>

<div class="health-panel">
    <HealthIndicator label="Base de datos"      status={$health.data?.database} />
    <HealthIndicator label="Jobs (Apalis)"      status={$health.data?.jobs_queue} />
    <HealthIndicator label="Última replicación" value={$health.data?.litestream_last_sync} />
    <HealthIndicator label="Versión"            value={$health.data?.version} />
</div>
```

---

## Notas de implementación

| Regla | ADR |
|-------|-----|
| `DashboardLayout.astro` verifica PASETO en servidor — sin flash de contenido | 0008 |
| Permisos RBAC verificados en servidor (handler) Y en UI (PermissionGate) | 0006 |
| TanStack Query gestiona el cache del servidor — sin duplicar fetch | 0022 |
| Command Palette respeta los permisos del usuario | 0006 |
| `data-theme` en `<html>` para modo oscuro — compatible con Tailwind y CSS vars | 0022 |
| `EmptyState` obligatorio en toda tabla — el dashboard se ve bien en proyectos nuevos | 0022 |
| `formatDate` usa timezone `America/La_Paz` por defecto — ADR 0023 | 0023 |
