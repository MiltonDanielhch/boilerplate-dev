# ADR 0030 — Multiplataforma: Web + Tauri + KMP

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0001 (Arquitectura Hexagonal), ADR 0027 (ConnectRPC), ADR 0022 (Frontend) |

---

## Contexto

El ecosistema del proyecto debe ejecutarse en navegadores, desktop y dispositivos móviles
en campo — posiblemente con conectividad intermitente.

La arquitectura hexagonal del ADR 0001 ya prepara el terreno: los crates `domain`,
`application` y `auth` no tienen dependencias de plataforma y pueden compilarse para
Android, iOS, macOS y Windows sin modificación.

---

## Decisión

Adoptar una estrategia de **Núcleo Compartido** con una escalera de migración tecnológica
basada en la carga de trabajo real — no en especulación.

```
Fase 1 (MVP web validado) → Desktop + Mobile con Tauri 2.0
Fase 2 (si escala)        → Optimización con WebGPU/Canvas dentro de Svelte
Fase 3 (si requiere nativo 100%) → KMP + UniFFI como capa visual
```

---

## Fase 1 — Tauri 2.0 + Svelte 5

Para el 90% de los casos de uso, Tauri 2.0 es suficiente y reutiliza directamente los crates de Rust.

| Plataforma | Motor de render | Comunicación con Rust | Peso |
|-----------|----------------|----------------------|------|
| Web | Browser nativo | HTTP + ConnectRPC | N/A |
| Desktop (Tauri) | WebView nativo (Edge/WebKit) | IPC directo | <15MB |
| Mobile (Tauri) | WebView nativo | Direct Bridge | <15MB |

```
boilerplate/
├── apps/
│   ├── api/        # Axum — servidor HTTP
│   ├── web/        # Astro SSR + Svelte 5
│   ├── desktop/    # Tauri 2.0 — reutiliza crates/domain directamente
│   └── mobile/     # Tauri Mobile — Android + iOS
└── crates/
    ├── domain/     # Compilable para todas las plataformas sin cambios
    ├── auth/       # argon2id + PASETO — corre en el dispositivo sin servidor
    └── ...
```

**Por qué Tauri en lugar de Electron:** Tauri usa el WebView nativo del sistema operativo.
Resultado: apps de <15MB ideales para instalación con datos móviles en zonas con conectividad limitada.

### Comandos Tauri — invocan casos de uso directamente

```rust
// apps/desktop/src-tauri/src/commands.rs
// Los comandos invocan directamente los casos de uso de crates/application
#[tauri::command]
async fn login(
    email:    String,
    password: String,
    state:    tauri::State<'_, AppState>,
) -> Result<AuthResponse, String> {
    LoginUseCase::new(&state.user_repo, &state.paseto, &state.session_repo)
        .execute(LoginInput { email, password })
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_users(state: tauri::State<'_, AppState>) -> Result<Vec<UserDto>, String> {
    ListUsersUseCase::new(&state.user_repo)
        .execute()
        .await
        .map_err(|e| e.to_string())
}
```

### Detección de entorno en Svelte 5

```typescript
// apps/web/src/lib/api/client.ts
// El mismo componente Svelte funciona en web y en Tauri
async function apiCall<T>(endpoint: string, options?: RequestInit): Promise<T> {
    if (typeof window !== 'undefined' && '__TAURI__' in window) {
        // En Tauri: usar IPC directo — sin servidor HTTP
        const command = endpointToCommand(endpoint);
        return invoke<T>(command, options?.body ? JSON.parse(options.body as string) : {});
    }
    // En web: fetch normal
    return fetch(`/api/v1${endpoint}`, options).then(r => r.json());
}
```

### SQLite local en Tauri (ADR 0024 — Local-First)

```rust
// apps/desktop/src-tauri/src/state.rs
pub struct AppState {
    pub pool:         SqlitePool,  // SQLite LOCAL del dispositivo
    pub user_repo:    Arc<dyn UserRepository>,
    pub paseto:       Arc<PasetoService>,
    pub session_repo: Arc<dyn SessionRepository>,
}

// Al arrancar: las 6 migraciones se aplican automáticamente en el dispositivo
pub async fn build_desktop_state() -> AppState {
    let pool = create_pool("sqlite:./data/boilerplate.db").await.unwrap();
    sqlx::migrate!("../../data/migrations").run(&pool).await.unwrap();
    // ...
}
```

---

## Fase 2 — Optimización visual: WebGPU / Canvas API

Si una vista específica (mapas de calor, visualizaciones en tiempo real) se vuelve lenta:

```svelte
<!-- apps/web/src/components/HeatMap.svelte -->
<script lang="ts">
    let canvas: HTMLCanvasElement;
    onMount(() => {
        const context = canvas.getContext('webgpu');
        // Renderizado GPU directo desde Svelte — sin cambiar de framework
    });
</script>
<canvas bind:this={canvas} />
```

---

## Fase 3 — KMP + UniFFI (solo si Tauri no alcanza)

**Criterio de activación:** listas de >10.000 elementos lentas, animaciones <60Hz constantes,
o procesamiento offline masivo en el dispositivo.

```rust
// crates/domain — anotaciones UniFFI (solo se agregan, la lógica no cambia)
#[uniffi::export]
pub fn validate_email(email: &str) -> Result<String, DomainError> {
    Email::new(email).map(|e| e.as_str().to_string())
}
```

```kotlin
// Android — generado por UniFFI, corre lógica Rust nativa
val result = DomainKt.validateEmail("user@example.com")
```

```
crates/domain (Rust) — sin cambios
    ↓ uniffi-bindgen genera automáticamente
domain.kt / domain.swift
    ↓
Compose Multiplatform (Android + iOS + Desktop)
```

---

## El mismo crate compila para todas las plataformas

```
crates/domain →
  x86_64-unknown-linux-musl    → Servidor VPS
  x86_64-pc-windows-msvc       → Desktop Windows
  aarch64-apple-darwin         → Desktop macOS Apple Silicon
  aarch64-linux-android        → Android
  aarch64-apple-ios            → iOS
```

---

## Alternativas consideradas

| Opción | Motivo de descarte |
|--------|--------------------|
| React Native | No reutiliza los crates de Rust — duplica toda la lógica de negocio |
| Flutter | Dart como lenguaje secundario — no se integra con crates/domain |
| Electron | Bundle de Chromium >100MB — inaceptable con datos móviles limitados |
| Capacitor / Ionic | Sin acceso directo a los crates de Rust |
| Solo web (PWA) | Sin SQLite local nativa — Local-First limitado (ADR 0024) |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para maximizar la reutilización de código y el rendimiento nativo:

| Herramienta | Propósito en el Tridente |
| :--- | :--- |
| **`uniffi`** | **Bindings Automáticos:** Genera código Kotlin/Swift desde Rust para la Fase 3 de forma transparente. |
| **`diplomat`** | **Interoperabilidad:** Una alternativa a UniFFI para generar interfaces de lenguaje más idiomáticas y ligeras. |
| **`tauri-plugin-sql`** | **Persistencia Nativa:** Facilita el uso de SQLite en Desktop/Mobile sin configurar drivers manualmente. |
| **`swift-rs`** | **iOS Integration:** Permite llamar a funciones de Swift desde Rust para acceso a APIs exclusivas de Apple. |

---

## Consecuencias

### ✅ Positivas

- Un solo developer mantiene Web, Desktop y Mobile con Rust y Svelte
- Las apps de Tauri pesan <15MB — ideales para instalación con datos móviles
- Los crates de dominio son la única fuente de verdad — sin lógica duplicada
- KMP + UniFFI da una salida si el rendimiento nativo se vuelve necesario

### ⚠️ Negativas / Trade-offs

- Tauri requiere WebView del sistema — comportamiento ligeramente diferente en Edge vs WebKit
  → Testear en ambos antes de cada release — las diferencias son menores con CSS moderno
  → Los componentes shadcn-svelte están probados en ambos WebViews
- La Fase 3 (KMP) requiere Android Studio y Xcode — overhead de setup no trivial
  → Solo se activa con criterio concreto de rendimiento — no especulativamente
- La serialización entre Rust y JavaScript (IPC de Tauri) requiere disciplina en los DTOs
  → Los DTOs de Tauri son los mismos que los de la API REST — sin duplicación

### Decisiones derivadas

- `apps/desktop/` y `apps/mobile/` no se crean hasta que el MVP web esté en producción y validado
- Los crates `domain`, `application` y `auth` nunca pueden tener dependencias de plataforma
- La UI de desktop reutiliza los mismos componentes Svelte de `apps/web/` — sin duplicación
- UniFFI se agrega como anotación opcional en `crates/domain` solo en Fase 3 — no antes
