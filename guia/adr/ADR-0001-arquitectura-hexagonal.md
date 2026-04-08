# ADR 0001 — Arquitectura Hexagonal y Monolito Modular

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Revisado en** | ADR 0003 (Stack Backend), ADR 0027 (ConnectRPC), ADR 0030 (Multiplataforma) |

---

## Contexto

El primer problema de todo proyecto es elegir cómo organizar el código antes de escribir
la primera línea. La tentación de los microservicios es real, pero para un equipo pequeño
con un VPS de $5 introduce complejidad operacional que no se justifica en etapas tempranas.

Necesitábamos una arquitectura que cumpliera con:

- **Eficiencia operacional** — pocos contenedores, fácil de mantener y deployar
- **Independencia de framework** — el negocio no debe ser esclavo de ninguna herramienta
- **Escalabilidad estructural** — que el código soporte crecer sin reescribirse
- **Legibilidad para humanos e IA** — tipos fuertes y contratos claros para trabajo con agentes
- **Reutilización multiplataforma** — los crates de dominio funcionan en servidor, desktop y móvil

---

## Decisión

Usar un **monolito modular** como unidad de deploy, con **arquitectura hexagonal** como
disciplina interna de organización del código.

> 2 contenedores bien estructurados vencen a 20 microservicios mal pensados.

### Las tres capas del modelo

```
┌─────────────────────────────────────────────┐
│              Adaptadores (afuera)            │
│   HTTP · CLI · Workers · Email · S3 · NATS   │
│   Tauri IPC · KMP UniFFI                     │
│  ┌───────────────────────────────────────┐   │
│  │         Puertos (contratos)           │   │
│  │  GuardarUsuario · EnviarNotificación  │   │
│  │  ┌─────────────────────────────────┐  │   │
│  │  │      Dominio (el corazón)       │  │   │
│  │  │  Entidades · Reglas de negocio  │  │   │
│  │  │  No conoce HTTP ni SQL          │  │   │
│  │  │  No conoce la plataforma        │  │   │
│  │  └─────────────────────────────────┘  │   │
│  └───────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

Las dependencias **solo apuntan hacia adentro**. El dominio no importa ningún crate externo
y no sabe si corre en un servidor, en un desktop con Tauri o en un móvil con KMP.

### Estructura de crates en el monorepo

```
boilerplate/
├── apps/
│   ├── api/           # Axum — entrada HTTP
│   ├── web/           # Astro SSR + Svelte 5
│   ├── desktop/       # Tauri 2.0 (Fase 2 — reutiliza crates/domain)
│   ├── mobile/        # Tauri Mobile / KMP (Fase 3)
│   └── cli/           # Sintonía CLI
│
└── crates/
    ├── domain/        # Núcleo puro — reutilizable en todas las apps
    ├── application/   # Casos de uso — solo domain
    ├── infrastructure/# HTTP, config, router
    ├── database/      # SQLx + Moka — único crate con sqlx
    ├── auth/          # argon2id + PASETO v4
    ├── mailer/        # Puerto Mailer + Resend
    ├── storage/       # Puerto Storage + Tigris
    └── events/        # NATS JetStream (Fase 2)
```

**La ventaja real:** los crates `domain`, `application`, `auth` y `database` son compilables
para Android, iOS, macOS y Windows. El mismo código de validación, hashing y reglas de negocio
corre en el servidor y en la app nativa — sin duplicación, sin inconsistencias.

Si `crates/domain/Cargo.toml` no declara `sqlx`, es imposible importarlo desde domain.
**El compilador hace cumplir la arquitectura — no las convenciones ni los code reviews.**

### Ejemplo: puerto y adaptador en Rust

```rust
// crates/domain/src/ports/user_repository.rs
// El dominio define el contrato — no sabe nada de SQL ni de plataforma
pub trait UserRepository: Send + Sync {
    async fn find_active_by_email(&self, email: &Email) -> Result<Option<User>, DomainError>;
    async fn save(&self, user: &User)                   -> Result<(), DomainError>;
    async fn soft_delete(&self, id: &UserId)            -> Result<(), DomainError>;
    async fn has_permission(&self, id: &UserId, perm: &str) -> Result<bool, DomainError>;
}

// crates/database/src/repositories/sqlite_user_repository.rs
// El adaptador implementa el contrato — solo este crate conoce SQLite
impl UserRepository for SqliteUserRepository {
    async fn find_active_by_email(&self, email: &Email) -> Result<Option<User>, DomainError> {
        sqlx::query_as!(UserRow,
            "SELECT * FROM users WHERE email = ? AND deleted_at IS NULL",
            email.as_str()
        )
        .fetch_optional(&*self.pool).await?
        .map(User::try_from).transpose()
    }
    // ...
}
```

Si mañana se cambia SQLite por Postgres, solo se reemplaza `SqliteUserRepository`.
El dominio, los casos de uso y los tests permanecen intactos.

---

## Las tres fases del desarrollo

### Antes — diseño de la fortaleza

| Patrón | Rol |
|--------|-----|
| Arquitectura Hexagonal | Puertos y Adaptadores — la lógica no sabe quién la llama |
| DDD (Dominio) | Entidades y Value Objects — reglas de negocio puras |
| Puertos (traits) | Contratos que el dominio exige al mundo exterior |
| Onion Architecture | Dependencias que solo apuntan hacia el centro |

### Durante — construcción

| Patrón | Rol |
|--------|-----|
| Adaptadores | Implementaciones reales de los puertos (DB, HTTP, email, IPC) |
| Tipos fuertes en Rust | El contrato es imposible de romper — ni por el dev ni por el agente IA |
| Estilo declarativo | Preferir el *qué* sobre el *cómo* — el código se lee como una oración |
| Principios CUPID | Código **C**omponible, **U**nico, **P**redictible, **I**diomático, **D**ominio-orientado |

### Después — evolución

| Situación | Respuesta |
|-----------|-----------|
| Cambiar la base de datos | Solo se reescribe el adaptador en `crates/database` |
| Cambiar el servidor HTTP | Solo se reescribe el adaptador HTTP en `crates/infrastructure` |
| Cambiar el proveedor de email | Solo se reescribe el adaptador en `crates/mailer` |
| Añadir app desktop Tauri | Nuevo `apps/desktop/` que reutiliza `crates/domain` directamente |
| Añadir app móvil nativa | KMP + UniFFI conecta a `crates/domain` sin reescribirlo |
| Cambiar las reglas de negocio | Solo se toca el dominio — nada más |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para reducir la fricción de la arquitectura hexagonal y asegurar su cumplimiento sin depender exclusivamente de la disciplina manual:

| Herramienta | Propósito en la Arquitectura |
| :--- | :--- |
| **`cargo-boundary`** | **Control de fronteras:** El "arquitecto jefe". Prohíbe imports ilegales entre crates (ej. `domain` usando `database`). |
| **`bon`** | **Ergonomía de Entidades:** Implementa el patrón Builder moderno para crear objetos de dominio sin boilerplate. |
| **`insta`** | **Snapshot Testing:** Permite testear lógica de negocio compleja capturando estados, ideal para trabajar con IA. |
| **`cargo-nextest`** | **Ejecutor de Tests:** Mucho más rápido que `cargo test`, ideal para arquitecturas con muchos crates. |
| **`cargo-expand`** | **Debug de Macros:** Permite ver el código real que generan los derives, vital para entender la magia de la IA. |
| **`taplo`** | **Orden en el Monorepo:** Mantiene todos los `Cargo.toml` del workspace formateados y validados. |

---

## Mandamientos del proyecto

| Mandamiento | Descripción |
|-------------|-------------|
| **Protección del dominio** | El código que describe el negocio es sagrado. Nunca se contamina con SQL, HTTP ni JSON. |
| **Independencia de framework** | Si el framework desaparece mañana, el dominio sobrevive intacto. |
| **Libertad por diseño** | La arquitectura hexagonal no es más trabajo — es el seguro de vida del proyecto. |
| **Sintonía humano-IA** | El código se escribe para que tanto el developer como el agente IA entiendan el propósito sin ambigüedades. Los tipos son la documentación. |
| **Código que perdura** | El maestro no es quien domina un framework, sino quien sabe aislar su lógica de cualquier framework. |
| **Plataforma como adaptador** | Web, desktop o móvil son adaptadores del dominio — no sus dueños. |

---

## Alternativas consideradas

| Opción | Motivo de descarte |
|--------|-------------------|
| Microservicios | Complejidad operacional injustificada para equipo pequeño y VPS de $5 |
| Monolito sin capas | Rápido al inicio, inmantenible a los 6 meses |
| Clean Architecture pura | Más burocracia de capas de la que necesita este proyecto en etapa temprana |
| React Native / Flutter | No reutiliza los crates de dominio en Rust — duplicaría la lógica de negocio |

---

## Consecuencias

### ✅ Positivas

- Deploy simple — 2 contenedores en un VPS de $5 sin orquestación compleja
- El dominio es testeable en aislamiento total, sin base de datos ni servidor HTTP
- Intercambio de tecnología sin tocar la lógica de negocio
- Los crates de dominio son reutilizables en servidor, desktop (Tauri) y móvil (KMP)
- Código legible para agentes IA — los traits son contratos inequívocos
- Crece hacia microservicios si algún día se justifica, módulo por módulo

### ⚠️ Negativas / Trade-offs

- Requiere disciplina — la separación de capas se puede romper si no hay revisión
  → Mitigación: `sintonia check arch` (ADR 0028) detecta imports prohibidos entre crates automáticamente
  → Mitigación: `cargo deny check` en CI bloquea dependencias no autorizadas en cada crate
- Más archivos y carpetas que un CRUD directo al inicio
  → El CLI (ADR 0028) genera todos los archivos por módulo en segundos desde el módulo 4 en adelante
- La curva de entrada es más alta para colaboradores sin experiencia en hexagonal
  → El documento `INICIO.md` guía de cero a un módulo funcionando con código real

### Decisiones derivadas

- El stack del servidor HTTP → **ADR 0003**
- La estrategia de persistencia → **ADR 0004**
- La jerarquía de errores → **ADR 0007**
- El esquema RBAC + Sessions → **ADR 0006**
- La estrategia multiplataforma → **ADR 0030**
- El generador de módulos → **ADR 0028**
