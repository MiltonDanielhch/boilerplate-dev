# Guía de Uso — Cómo Navegar el Sistema

> Manual completo para usar tu arquitectura hexagonal con Rust.

---

## 🎯 Visión General

Este es un **sistema de navegación para construir productos reales** con arquitectura hexagonal en Rust. Tiene 4 capas:

| Capa | Propósito | Ubicación | Cantidad |
|------|-----------|-----------|----------|
| **PROMPT** | Contexto del Staff Engineer | `guia/PROMPT_MAESTRO.md` | 1 archivo |
| **Roadmaps** | Pasos ejecutables (checklists) | `guia/roadmap/*.md` | 14 archivos |
| **ADRs** | Decisiones arquitectónicas (el "por qué") | `guia/adr/*.md` | 31+ activos |
| **Docs** | Documentación técnica (el "cómo") | `guia/docs/*.md` | 10+ archivos |

---

## 🗺️ Mapa Mental de Uso

```
┌─────────────────────────────────────────────────────────────────┐
│  1. INICIAR SESIÓN con el Staff Engineer                        │
│     └─> "Empezar proyecto" o "Continuar con Backend Bloque II"  │
│                                                                 │
│  2. El Engineer lee automáticamente:                            │
│     ├─> PROMPT_MAESTRO.md (contexto global)                     │
│     ├─> Roadmap activo (ej: 02-ROADMAP-GENESIS.md)             │
│     └─> ADRs relevantes (referenciados en el roadmap)         │
│                                                                 │
│  3. Durante el trabajo:                                         │
│     ├─> Tú revisas docs/ si necesitas profundidad técnica       │
│     ├─> El Engineer propone cambios y tú apruebas               │
│     └─> Marcamos checkboxes ✅ en los roadmaps                  │
│                                                                 │
│  4. Al finalizar cada fase:                                     │
│     └─> Verificación con comandos de docs/04-VERIFICATION.md    │
└─────────────────────────────────────────────────────────────────┘
```

---

## 📋 Los 14 Roadmaps — Plan de Ejecución

| # | Fase | Roadmap | Cuándo | Estado |
|---|------|---------|--------|--------|
| 0 | **Génesis** | `02-ROADMAP-GENESIS.md` | Día 1 | ✅ |
| 1 | **Backend I** | `03-ROADMAP-BACKEND.md` Bloque I | Día 1-2 | ✅ |
| 2 | Backend II | `03-ROADMAP-BACKEND.md` Bloque II | Día 2-3 | ✅ |
| 3 | Frontend I | `04-ROADMAP-FRONTEND.md` Bloque I | Paralelo con Backend III | ⏳ |
| 4 | Backend III | `03-ROADMAP-BACKEND.md` Bloque III | Día 3-4 | ⏳ |
| 5 | Frontend II | `04-ROADMAP-FRONTEND.md` Bloque II | Paralelo con Backend III | ⏳ |
| 6 | Auth Fullstack | `05-ROADMAP-AUTH-FULLSTACK.md` | Después de Backend III + Frontend III | ⏳ |
| 7 | Landing | `06-ROADMAP-LANDING.md` | Paralelo con Auth | ⏳ |
| 8 | Infra | `07-ROADMAP-INFRA.md` | MVP backend+frontend listo | ⏳ |
| — | **MVP PRODUCCIÓN** | — | — | — |
| 9 | Desktop | `08-ROADMAP-TAURI-DESKTOP.md` | MVP web validado | ⏳ |
| 10 | Mobile | `09-ROADMAP-MOBILE.md` | Desktop validado | ⏳ |
| 11 | Admin | `80-ROADMAP-ADMIN.md` | Post-MVP | ⏳ |
| 12 | Fase 2 | `50-ROADMAP-FASE2.md` | Post-MVP (NATS + Events) | ⏳ |
| 13 | Fase 3 | `60-ROADMAP-FASE3.md` | Post-MVP (KMP + Mobile Native) | ⏳ |

### Leyenda de estados:
- ✅ Completado
- 🔄 Activo / En progreso
- ⏳ Pendiente
- 🟡 Opcional / Diferido

---

## 📚 Los Documentos Clave (`docs/`)

| Documento | Para qué sirve | Cuándo consultarlo |
|-----------|----------------|-------------------|
| `00-GUIA-USO.md` | Este archivo — cómo usar el sistema | **Ahora** |
| `INICIO.md` | Tutorial completo desde cero | Antes de empezar Génesis |
| `01-ARCHITECTURE.md` | Flujos hexagonales, capas, flujo de peticiones | Cuando tengas dudas de arquitectura |
| `02-STACK.md` | Versiones de crates y herramientas | Antes de instalar dependencias |
| `03-STRUCTURE.md` | Árbol de directorios del monorepo | Cuando crees nuevos crates/módulos |
| `04-VERIFICATION.md` | Comandos para validar cada fase | Al finalizar cada fase |
| `05-MODULES.md` | Catálogo de 90 módulos implementables | Cuando quieras añadir features |
| `SINTONIA-CLI.md` | Referencia del CLI (Fase 2) | Cuando actives scaffolding |

---

## 🏛️ Los ADRs — Las Decisiones ("El Por Qué")

### Cómo leer un ADR:
```
ADR-XXXX-titulo-descriptivo.md

Ejemplo: ADR-0008-seguridad-auth-paseto.md
         └── ^^^^ Tema: Auth con PASETO (nunca JWT)
```

### ADRs más importantes:

| ADR | Tema | Qué resuelve |
|-----|------|--------------|
| `ADR-0001` | Arquitectura Hexagonal | Crate `domain/` sin dependencias externas |
| `ADR-0002` | Configuración Tipeada | Fail-fast en variables de entorno |
| `ADR-0004` | Persistencia SQLite | SQLite WAL + Litestream para backups |
| `ADR-0006` | RBAC y Sesiones | Roles, permisos, auditoría completa |
| `ADR-0008` | Auth con PASETO | Por qué NO usamos JWT |
| `ADR-0010` | Testing | 4 capas de testing (unitario → E2E) |
| `ADR-0011` | Estándares | Funciones ≤30 líneas, archivos ≤200 líneas |
| `ADR-0012` | Herramientas | mise, just, lefthook, cargo-nextest |
| `ADR-0014` | Infraestructura | Deploy con Kamal + Caddy |
| `ADR-0030` | Multiplataforma | Tauri Desktop + Mobile + KMP |
| `ADR-0031` | Escalamiento | Cuándo pasar a Fase 2/3 (con datos reales) |

### Cómo se usan en los roadmaps:
```markdown
[ ] Crear entidad User en crates/domain/src/entities/
    └─ Ref: ADR 0001 (domain puro), ADR 0008 (soft delete)
```

Cuando veas `Ref: ADR XXXX`, esa decisión arquitectónica aplica ahí.

---

## 🔄 Flujo de Trabajo Día a Día

### Opción A: Iniciar proyecto nuevo
```
1. Dices: "Empezar proyecto boilerplate desde cero"
2. El Engineer lee ROADMAP-GENESIS.md completo
3. Ejecutamos paso a paso, marcando [x] checkboxes
4. Al terminar: "cargo check --workspace ✅"
```

### Opción B: Continuar trabajo existente
```
1. Dices: "Continuar con Backend Bloque III"
2. El Engineer detecta automáticamente el estado actual
3. Leemos juntos ese bloque del roadmap
4. Implementamos los checkboxes pendientes
```

### Opción C: Debugging
```
1. Dices: "No funciona: cargo check da error en crates/domain"
2. El Engineer consulta docs/04-VERIFICATION.md (sección Troubleshooting)
3. Proponemos solución basada en los síntomas
```

---

## 🎮 Comandos de Navegación (para decirle al Engineer)

| Comando | Qué hace el Engineer |
|---------|----------------------|
| `Empezar proyecto` | Inicia desde Génesis |
| `Continuar con [Fase]` | Va a esa fase específica |
| `Estado del proyecto` | Resume qué checkboxes faltan |
| `Revisar arquitectura` | Verifica que no hayas roto fronteras entre crates |
| `Actualiza TODO` | Marca checkboxes completados |
| `No funciona: [error]` | Debugging con troubleshooting guides |

---

## ⚡ Reglas de Oro (del PROMPT_MAESTRO.md)

1. **Nunca simplificar** — Si el roadmap dice 6 migraciones, hacemos 6
2. **Nunca omitir pasos** — Cada checkbox existe por una razón
3. **Fronteras por Cargo.toml** — `domain/` NUNCA depende de sqlx o axum
4. **Verificar versiones antes de instalar** — Usas `cargo search` / `npm view`
5. **Tests por capa** — Unitarios (domain) → Integración (app) → E2E (api)
6. **Decisión con datos** — No escalamos a Fase 2 sin métricas reales

---

## 🚀 Tu Próximo Paso

### Para empezar ahora:
1. Revisa este documento (`00-GUIA-USO.md`)
2. Lee `guia/docs/INICIO.md` si quieres el tutorial completo
3. Di: **"Empezar proyecto"** para iniciar Génesis

### El Engineer hará:
1. Leer `ROADMAP-GENESIS.md` completo
2. Preparar las primeras 5-10 tareas
3. Esperar tu confirmación antes de ejecutar

---

## 📁 Estructura de Archivos

```
guia/
├── PROMPT_MAESTRO.md           ← Contexto del Staff Engineer
├── docs/
│   ├── 00-GUIA-USO.md          ← Este archivo
│   ├── INICIO.md               ← Tutorial desde cero
│   ├── 01-ARCHITECTURE.md      ← Flujos hexagonales
│   ├── 02-STACK.md             ← Versiones de crates
│   ├── 03-STRUCTURE.md         ← Árbol de directorios
│   ├── 04-VERIFICATION.md      ← Comandos de verificación
│   ├── 05-MODULES.md           ← Catálogo de 90 módulos
│   └── SINTONIA-CLI.md         ← CLI scaffolding (Fase 2)
├── roadmap/
│   ├── 00-ROADMAP-TEMPLATE.md  ← Plantilla para nuevos roadmaps
│   ├── 01-ROADMAP-MASTER.md    ← Índice maestro
│   ├── 02-ROADMAP-GENESIS.md   ← Fase 0: Setup
│   ├── 03-ROADMAP-BACKEND.md   ← Fases 1-2: Backend
│   ├── 04-ROADMAP-FRONTEND.md  ← Fases 2-3: Frontend
│   ├── 05-ROADMAP-AUTH-FULLSTACK.md
│   ├── 06-ROADMAP-LANDING.md
│   ├── 07-ROADMAP-INFRA.md
│   ├── 08-ROADMAP-TAURI-DESKTOP.md
│   ├── 09-ROADMAP-MOBILE.md
│   ├── 50-ROADMAP-FASE2.md     ← Post-MVP: NATS
│   ├── 60-ROADMAP-FASE3.md     ← Post-MVP: KMP
│   ├── 70-ROADMAP-FUTURA.md    ← Post-MVP: Enterprise
│   └── 80-ROADMAP-ADMIN.md     ← Post-MVP: Admin Dashboard
└── adr/
    ├── ADR-0001-*.md           ← 31 ADRs activos
    ├── ...
    └── futura/                  ← 8 ADRs para escalamiento
```

---

**Ref:** PROMPT_MAESTRO.md, 01-ROADMAP-MASTER.md, docs/INICIO.md
