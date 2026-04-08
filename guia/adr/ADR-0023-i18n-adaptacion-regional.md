# ADR 0023 — i18n: Paraglide JS + Formatters Bolivia

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0022 (Astro + Svelte 5), ADR 0024 (Local-First) |

---

## Contexto

El foco inicial es Bolivia, pero el software debe ser global por diseño. Los problemas comunes
de i18n en proyectos frontend son:

- **Falta de tipado** — strings manuales como `t("welcome")` que rompen silenciosamente si la clave cambia
- **Bundle pesado** — descargar todos los idiomas a la vez desperdicia ancho de banda
- **SEO roto** — páginas sin URLs por idioma ni etiquetas `lang` correctas

---

## Decisión

Usar **Paraglide JS** (ecosistema inlang) para traducciones compiladas y **Astro i18n**
nativo para el enrutamiento por idioma.

### 1 — Traducciones compiladas con Paraglide

En lugar de un JSON cargado en runtime, Paraglide genera funciones TypeScript en tiempo de
compilación. Si se borra una clave, el compilador falla antes de que el error llegue a producción:

```typescript
// apps/web/src/paraglide/messages/es.ts (generado — no editar)
export const welcome_to_lab = (params: { name: string }) =>
    `Bienvenido al laboratorio, ${params.name}`

export const login_title = () => `Iniciar sesión`
export const users_table_empty = () => `No hay usuarios registrados`
```

```svelte
<!-- apps/web/src/components/Welcome.svelte -->
<script lang="ts">
    import * as m from '$paraglide/messages';
    export let name: string;
</script>

<h1>{m.welcome_to_lab({ name })}</h1>
```

Si `welcome_to_lab` se renombra, TypeScript marca el error en todos los componentes — antes de compilar.

### 2 — Enrutamiento por idioma con Astro

```
apps/web/src/pages/
├── login.astro         → /login       (español por defecto, sin prefijo)
├── dashboard/
│   └── index.astro     → /dashboard
└── en/
    ├── login.astro     → /en/login
    └── dashboard/
        └── index.astro → /en/dashboard
```

```typescript
// apps/web/astro.config.mjs
export default defineConfig({
    i18n: {
        defaultLocale: 'es',
        locales:       ['es', 'en'],
        routing: {
            prefixDefaultLocale: false, // /login en lugar de /es/login
        },
    },
    integrations: [
        paraglide({ project: './project.inlang', outdir: './src/paraglide' }),
    ],
});
```

### 3 — Adaptación regional Bolivia (L10n)

```typescript
// apps/web/src/lib/i18n/formatters.ts
const LOCALE_MAP = { es: 'es-BO', en: 'en-US' } as const;

export function formatCurrency(amount: number, locale: keyof typeof LOCALE_MAP) {
    return new Intl.NumberFormat(LOCALE_MAP[locale], {
        style:    'currency',
        currency: locale === 'es' ? 'BOB' : 'USD', // Bolivianos por defecto
    }).format(amount);
}

export function formatDate(isoString: string, locale: keyof typeof LOCALE_MAP) {
    // La DB siempre guarda UTC (ADR 0004) — conversión al timezone del cliente
    return new Intl.DateTimeFormat(LOCALE_MAP[locale], {
        dateStyle: 'short', // DD/MM/YYYY en es-BO
        timeZone:  locale === 'es' ? 'America/La_Paz' : 'UTC',
    }).format(new Date(isoString));
}

// Formatear número con separadores bolivianos
export function formatNumber(n: number, locale: keyof typeof LOCALE_MAP) {
    return new Intl.NumberFormat(LOCALE_MAP[locale]).format(n);
}
```

### 4 — Agregar un nuevo idioma

```bash
# 1. Agregar el locale en astro.config.mjs
locales: ['es', 'en', 'pt']  # Agregar 'pt' para portugués

# 2. Crear el archivo de mensajes
# apps/web/messages/pt.json con TODAS las claves de es.json

# 3. Regenerar con Paraglide
just types  # Incluye buf generate + paraglide generate

# Si falta alguna clave → el compilador falla con error exacto
```

---

## Comparativa: Paraglide vs i18next

| Característica | i18next | Paraglide JS |
|----------------|---------|-------------|
| **Tipado de claves** | Manual con plugins | Automático en compilación |
| **Tamaño del bundle** | Runtime + JSONs completos | Solo el idioma activo, como funciones |
| **Detección de claves faltantes** | En runtime | En compilación — bloquea el build |
| **Curva de aprendizaje** | Baja — ya conocido | Media — paradigma diferente |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para maximizar la eficiencia del flujo de traducción y la calidad regional:

| Herramienta | Propósito en el i18n |
| :--- | :--- |
| **`inlang Fink`** | **Editor Visual:** Permite editar traducciones en una interfaz web sin tocar archivos JSON/JSON5. |
| **`i18n-ally`** | **DX en VS Code:** Extensión para previsualizar y editar traducciones directamente desde el código. |
| **`fluent-syntax`** | **Pluralización Avanzada:** Soporte para el estándar de Mozilla para traducciones con lógica compleja. |
| **`zod-i18n-map`** | **Validación Traducida:** Conecta los errores de esquema con las funciones de Paraglide automáticamente. |

---

## Consecuencias

### ✅ Positivas

- Las traducciones faltantes se detectan en compilación — no en producción
- Bundle mínimo — solo se incluye el idioma que el usuario está viendo
- SEO correcto — URLs por idioma y etiquetas `lang` automáticas con Astro
- Los formatos de fecha, moneda y zona horaria se adaptan por locale sin código adicional

### ⚠️ Negativas / Trade-offs

- Paraglide requiere un paso de generación antes del build
  → `just build` incluye el paso de generación de Paraglide antes de `pnpm build`
  → En CI: `just types` corre antes del build y verifica que no hay diff
- Abandonar el familiar `t("key")` por funciones importadas tiene una curva de adaptación
  → La ventaja es que el IDE da autocompletado y el compilador detecta claves rotas
  → Tiempo de adaptación estimado: 1-2 horas para developers acostumbrados a i18next
- Agregar un nuevo idioma requiere traducir todas las claves antes de activarlo
  → El compilador lo exige — no es posible activar un idioma con traducciones incompletas
  → Esto es una feature, no un bug — previene páginas con strings mezclados

### Decisiones derivadas

- El idioma por defecto es `es` (español) — sin prefijo en la URL
- Las fechas se almacenan siempre en UTC en la DB (ADR 0004) — conversión al timezone local en el cliente
- `just build` incluye el paso de generación de Paraglide antes de `pnpm build`
- Los archivos de mensajes viven en `apps/web/messages/` — estos SÍ se editan manualmente
- Los archivos en `apps/web/src/paraglide/` son generados — nunca editados manualmente
