# ADR 0022 — Frontend: Astro SSR + Svelte 5 Runes

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0027 (ConnectRPC — tipos generados para el frontend), ADR 0008 (PASETO — gestión de tokens) |

---

## Contexto

El frontend necesita cumplir con tres objetivos simultáneos:

- **Rendimiento de carga** — SEO y Core Web Vitals excelentes desde el primer byte
- **Interactividad rica** — componentes reactivos donde el usuario lo necesita, no en toda la página
- **Developer experience** — tipado completo, estado predecible, componentes reutilizables

Las opciones evaluadas fueron Next.js (React SSR), SvelteKit y Astro + Svelte. El factor
determinante fue la filosofía "HTML primero, JavaScript solo donde sea necesario" — alineada
con el principio del proyecto de usar solo lo que el problema requiere.

---

## Decisión

**Astro** para el esqueleto SSR y enrutamiento. **Svelte 5** con Runes para las islas de interactividad.

### Por qué esta combinación

| Capa | Responsabilidad |
|------|----------------|
| Astro SSR | Genera HTML en el servidor — velocidad de carga inicial máxima |
| Astro Islands | Hidrata solo los componentes que necesitan interactividad |
| Svelte 5 Runes | Estado reactivo mínimo y directo — compila a JS puro sin runtime |
| TanStack Query | Estado del servidor — cache, refetch, loading states |
| ArkType | Validación con inferencia de tipos — sin duplicar schemas |
| Paraglide JS | i18n compilado — errores de clave faltante en build, no en producción |

Svelte 5 tiene ~5KB de overhead por componente. React tiene ~130KB de runtime base.

### Setup inicial

```bash
pnpm create astro apps/web -- --template minimal
cd apps/web
pnpm astro add svelte tailwind
pnpm add @tanstack/svelte-query arktype @inlang/paraglide-astro bits-ui lucide-svelte clsx tailwind-merge
# Inicialización oficial para configurar componentes, temas y alias
npx shadcn-svelte@latest init
```

### Configuración de Astro

```typescript
// apps/web/astro.config.mjs
import { defineConfig } from 'astro/config';
import svelte   from '@astrojs/svelte';
import tailwind from '@astrojs/tailwind';
import node     from '@astrojs/node';
import paraglide from '@inlang/paraglide-astro';

export default defineConfig({
    output:  'server',
    adapter: node({ mode: 'standalone' }),
    integrations: [
        svelte(),
        tailwind({ applyBaseStyles: false }),
        paraglide({ project: './project.inlang', outdir: './src/paraglide' }),
    ],
});
```

### Estructura de directorios

```
apps/web/src/
├── pages/
│   ├── index.astro               # Landing page (ADR 0022 — ROADMAP-LANDING)
│   ├── login.astro
│   ├── register.astro
│   └── dashboard/
│       ├── index.astro           # KPIs + feed de eventos
│       ├── users/
│       │   ├── index.astro       # Tabla paginada
│       │   └── [id].astro        # Perfil de usuario
│       └── audit/
│           └── index.astro       # Requiere audit:read
│
├── components/
│   ├── ui/                       # shadcn-svelte + Bits UI
│   │   ├── Button.svelte
│   │   ├── Input.svelte
│   │   ├── EmptyState.svelte     # Obligatorio en toda tabla sin datos
│   │   └── PermissionGate.svelte # Oculta elementos sin el permiso necesario
│   ├── layout/
│   │   ├── Sidebar.svelte        # Colapsable, respeta RBAC
│   │   ├── Topbar.svelte         # Búsqueda + notificaciones + perfil
│   │   └── CommandPalette.svelte # Ctrl+K — acciones filtradas por permisos
│   └── dashboard/
│       ├── KpiCard.svelte
│       ├── ActivityChart.svelte
│       ├── EventFeed.svelte
│       └── SystemHealth.svelte
│
├── lib/
│   ├── api/
│   │   ├── client.ts             # fetch base con auth headers
│   │   ├── users.ts
│   │   └── auth.ts
│   ├── types/
│   │   └── api.ts                # GENERADO por buf generate — no editar manualmente
│   ├── stores/
│   │   └── auth.svelte.ts        # Estado de auth con Runes
│   └── validation/
│       └── schemas.ts            # ArkType schemas
│
├── styles/
│   └── global.css                # CSS variables de marca — tokens compartidos con landing
│
└── layouts/
    ├── BaseLayout.astro
    ├── LandingLayout.astro       # Sin sidebar — para la landing page
    └── DashboardLayout.astro     # Protege rutas — verifica PASETO en servidor
```

### Estado global con Svelte 5 Runes

```typescript
// apps/web/src/lib/stores/auth.svelte.ts
import type { User } from '$lib/types/api';

let user         = $state<User | null>(null);
let accessToken  = $state<string | null>(null);

export function getAuthState() {
    return {
        get user()        { return user; },
        get accessToken() { return accessToken; },
        get isLoggedIn()  { return user !== null; },

        setAuth(newUser: User, token: string) {
            user        = newUser;
            accessToken = token;
        },

        clearAuth() {
            user        = null;
            accessToken = null;
        },
    };
}
```

### Estado del servidor con TanStack Query

```svelte
<!-- apps/web/src/components/dashboard/UserTable.svelte -->
<script lang="ts">
    import { createQuery } from '@tanstack/svelte-query';
    import { fetchUsers }  from '$lib/api/users';

    let page   = $state(1);
    let search = $state('');

    const usersQuery = createQuery({
        queryKey:  () => ['users', page, search],
        queryFn:   () => fetchUsers({ page, search }),
        staleTime: 30_000,
        retry:     2,
    });
</script>

{#if $usersQuery.isLoading}
    <TableSkeleton rows={10} />
{:else if $usersQuery.data?.items.length === 0}
    <EmptyState
        title="No hay usuarios"
        description="Crea el primer usuario para empezar"
    />
{:else if $usersQuery.data}
    <!-- tabla con paginación -->
{/if}
```

### Validación con ArkType

```typescript
// apps/web/src/lib/validation/schemas.ts
import { type } from 'arktype';

export const LoginSchema = type({
    email:    'string.email',
    password: 'string >= 8',
});

export const CreateUserSchema = type({
    email:    'string.email',
    password: 'string >= 12',
});

// ArkType infiere el tipo TypeScript automáticamente — sin duplicar
export type LoginInput      = typeof LoginSchema.infer;
export type CreateUserInput = typeof CreateUserSchema.infer;
```

### Sistema de permisos RBAC en la UI (ADR 0006)

```svelte
<!-- apps/web/src/components/ui/PermissionGate.svelte -->
<script lang="ts">
    import { getAuthState } from '$lib/stores/auth.svelte';
    export let permission: string;

    const auth    = getAuthState();
    const allowed = $derived(auth.user?.permissions?.includes(permission) ?? false);
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
```

---

## Alternativas consideradas

| Opción | Motivo de descarte |
|--------|--------------------|
| Next.js 15 | Runtime de React más pesado; vendor lock-in con Vercel |
| SvelteKit puro | Sin el concepto de islas — más JS que Astro para páginas mayormente estáticas |
| Nuxt 3 | Vue — menos sinergia con el stack Rust del proyecto |
| Remix | React, bundle más pesado |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para maximizar la calidad, el rendimiento y la experiencia de desarrollo del frontend:

| Herramienta | Propósito en el Frontend |
| :--- | :--- |
| **`playwright`** | **Testing E2E:** Automatiza pruebas de flujos de usuario completos en navegadores reales. |
| **`vitest`** | **Testing de Componentes:** Un test runner rápido para pruebas unitarias y de componentes Svelte. |
| **`storybook`** | **Desarrollo de UI:** Aísla y documenta componentes de UI para una construcción y mantenimiento eficientes. |
| **`eslint-plugin-svelte`** | **Calidad de Código:** Aplica reglas de linting específicas para el código Svelte. |
| **`openapi-typescript-codegen`** | **Cliente Type-safe:** Genera automáticamente un cliente de API y tipos TypeScript desde `openapi.json`. |

---

## Consecuencias

### ✅ Positivas

- Carga inicial ultra-rápida — HTML desde servidor, sin hydration obligatoria
- SEO excelente de forma nativa — contenido en el HTML
- Bundle mínimo — Svelte 5 compila sin runtime de framework
- Tipos del backend disponibles via `buf generate` — sin DTOs duplicados
- `PermissionGate` y `Sidebar` con RBAC — coherencia con el sistema de permisos del backend

### ⚠️ Negativas / Trade-offs

- ArkType tiene menor adopción que Zod
  → Verificar integración con shadcn-svelte antes de adoptar en forms muy complejos
  → Si hay problemas: la validación de ArkType solo vive en `$lib/validation/schemas.ts` —
    reemplazar Zod es cambiar ese archivo sin tocar los componentes
- Svelte 5 Runes son relativamente nuevos
  → La documentación de edge cases es escasa, pero los casos fundamentales (state, derived, effect)
    están estables y documentados
  → `$state`, `$derived`, `$effect` son las únicas primitivas necesarias para el 90% de los casos
- Las Astro Islands añaden complejidad conceptual
  → Regla simple: todo lo que no necesita estado del browser → `.astro`; todo lo que
    necesita reactividad → `.svelte` con `client:load` o `client:idle`

### Decisiones derivadas

- `apps/web/src/lib/types/api.ts` es generado por `buf generate` (ADR 0027) — nunca editado manualmente
- `just types` regenera los tipos y debe correr antes de `pnpm build` en CI
- `DashboardLayout.astro` verifica el access token en el servidor — no hay flash de contenido no autenticado
- El auth state usa Svelte Runes — no Context API de Astro ni Nanostores
