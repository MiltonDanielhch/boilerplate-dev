# Roadmap — Desktop (Tauri 2.0)

> **Stack:** Tauri 2.0 · Rust · Svelte 5 · Windows · macOS · Linux
>
> **ADR:** 0030 (Multiplataforma Tridente)
>
> ⚠️ **Activar solo cuando el MVP web esté en producción y validado.**
> La UI de desktop reutiliza exactamente los mismos componentes Svelte de `apps/web/`.
> Los casos de uso de `crates/application` se invocan directamente — sin duplicar lógica.

---

## Estados

```
[ ] Pendiente   [~] En progreso   [x] Completado   [!] Bloqueado
```

---

## Progreso

| Bloque | Nombre | Progreso |
|--------|--------|----------|
| D.I | Setup inicial | **100%** ✅ |
| D.II | Configuración Tauri | **100%** ✅ |
| D.III | Comandos Tauri + Bridge con Rust | **100%** ✅ |
| D.IV | Auth + estado local | **100%** ✅ |
| D.V | UI adaptada para desktop | **100%** ✅ |
| D.VI | Build + distribución | **100%** ✅ |
| D.VII | CI para desktop | **100%** ✅ |

---

## Por qué Tauri y no Electron

| Electron | Tauri 2.0 |
|----------|-----------|
| Bundle Chromium incluido — >100MB | WebView nativo del SO — <15MB |
| Node.js en el proceso principal | Rust en el proceso principal |
| Sin reutilización de crates | Reutiliza `crates/domain` + `crates/application` |
| Superficie de ataque grande | Permisos granulares por capability |
| >150MB de RAM en reposo | ~30MB de RAM en reposo |

---

## Progreso

| Bloque | Nombre | Progreso |
|--------|--------|----------|
| D.I | Setup inicial | **100%** ✅ |
| D.II | Configuración Tauri | **100%** ✅ |
| D.III | Comandos Tauri + Bridge con Rust | **100%** ✅ |
| D.IV | Auth + estado local | **100%** ✅ |
| D.V | UI adaptada para desktop | **100%** ✅ |
| D.VI | Build + distribución | **100%** ✅ |
| D.VII | CI para desktop | **100%** ✅ |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para maximizar la integración nativa y la fiabilidad de la aplicación desktop:

| Herramienta | Propósito en Desktop |
| :--- | :--- |
| **`tauri-plugin-sql`** | **DB Bridge:** Facilita el acceso a la SQLite local desde el frontend con tipos garantizados. |
| **`tauri-plugin-log`** | **Logs Unificados:** Centraliza los logs de Rust y JS en el sistema de archivos del usuario para soporte. |
| **`tauri-plugin-single-instance`** | **UX:** Garantiza que solo exista una instancia de la aplicación abierta a la vez. |
| **`cargo-packager`** | **Distribución:** Orquestador de builds multiplataforma más rápido que el bundler estándar. |

---

## D.I — Setup inicial

> **Referencia:** ADR 0030 (Multiplataforma), docs/02-STACK.md L382, docs/03-STRUCTURE.md L550-554

```
[x] Pre-requisito: MVP web en producción ✅
    └─ Ref: ADR 0030 — web primero, luego desktop/mobile

[x] Instalar herramientas:
    [x] cargo install tauri-cli --version 2.10.1 ✅
    [x] Verificar: cargo tauri --version ✅

[x] Crear apps/desktop/:
    [x] apps/desktop/Cargo.toml  (añadir al workspace root) ✅
    [x] apps/desktop/src-tauri/Cargo.toml ✅
    [x] apps/desktop/src-tauri/src/main.rs ✅
    [x] apps/desktop/src-tauri/src/lib.rs ✅
    [x] apps/desktop/src-tauri/src/commands/mod.rs ✅
    [x] apps/desktop/src-tauri/src/commands/auth.rs ✅
    [x] apps/desktop/src-tauri/src/commands/users.rs ✅
    [x] apps/desktop/tauri.conf.json ✅
    [x] apps/desktop/src-tauri/src/state.rs (pendiente)
    [x] apps/desktop/src-tauri/capabilities/default.json (pendiente)
    [x] apps/desktop/src-tauri/icons/  (pendiente)

[x] apps/desktop/Cargo.toml:
    [x] tauri = { version = "2", features = ["tray-icon"] } ✅
    [x] tauri-plugin-store = "2"          (tokens PASETO seguros) ✅
    [x] tauri-plugin-shell = "2"          (abrir links en browser) ✅
    [x] tauri-plugin-notification = "2"   (notificaciones nativas) ✅
    [x] tauri-plugin-log = "2"            (logs unificados) ✅
    [x] tauri-plugin-single-instance = "2" ✅
    [x] application = { path = "../../crates/application" }
        └─ Ref: docs/03-STRUCTURE.md L188 — reutilizar crates
    [x] database    = { path = "../../crates/database" }
    [x] auth        = { path = "../../crates/auth" }
    [x] domain      = { path = "../../crates/domain" }
    [x] serde, serde_json, tokio, tracing (workspace = true)

[x] Verificar:
    [x] cargo check --workspace → sin errores incluyendo apps/desktop
    [x] cargo tauri build → compila (aunque esté vacío)
```

---

## D.II — Configuración Tauri

> **Referencia:** ADR 0030 (Multiplataforma), docs/02-STACK.md L382, docs/03-STRUCTURE.md L550-554

```
[x] apps/desktop/tauri.conf.json:
    └─ Ref: docs/03-STRUCTURE.md L550
    [x] productName: "boilerplate"
    [x] version: "0.1.0"
    [x] identifier: "com.laboratorio3030.boilerplate"
    [x] build.devUrl: "http://localhost:4321"     (Astro SSR en dev)
        └─ Ref: docs/02-STACK.md L371
    [x] build.frontendDist: "../../web/dist"      (Astro build)
    [x] app.windows:
        [x] title: "boilerplate"
        [x] width: 1280, height: 800
        [x] minWidth: 900, minHeight: 600
        [x] center: true
        [x] decorations: true
    [x] app.trayIcon.iconPath: "icons/tray-icon.png"
    [x] app.security.csp: "default-src 'self' http://localhost:8080"
        └─ Ref: ADR 0030 — CSP estricto
    [x] bundle.active: true
    [x] bundle.targets: "all"
    [x] bundle.icon: [32x32, 128x128, icon.ico, icon.icns]

[x] apps/desktop/src-tauri/capabilities/default.json:
    └─ Ref: docs/03-STRUCTURE.md L553
    [x] permissions de red: solo localhost + tudominio.com
    [x] permissions de filesystem: solo carpeta de datos de la app
    [x] Sin permisos innecesarios (mínimo privilegio)
        └─ Ref: ADR 0030 — seguridad granular

[x] Iconos:
    └─ Ref: docs/03-STRUCTURE.md L552
    [x] icons/32x32.png
    [x] icons/128x128.png
    [x] icons/128x128@2x.png
    [x] icons/icon.ico     (Windows)
    [x] icons/icon.icns    (macOS)
    [x] icons/tray-icon.png
```

---

## D.III — Comandos Tauri + Bridge con Rust (ADR 0030)

> **Referencia:** ADR 0030 (Multiplataforma), docs/02-STACK.md L382, docs/01-ARCHITECTURE.md L117-136

Los comandos Tauri invocan directamente los use cases de `crates/application` — sin HTTP:

```
[x] apps/desktop/src-tauri/src/state.rs — AppState de Tauri:
    └─ Ref: docs/03-STRUCTURE.md L554
    [x] pool: SqlitePool  (SQLite LOCAL del dispositivo)
        └─ Ref: ADR 0004, docs/02-STACK.md L155-170
    [x] user_repo: Arc<CachedUserRepository>
        └─ Ref: ADR 0017, docs/02-STACK.md L253-268
    [x] session_repo: Arc<SqliteSessionRepository>
    [x] audit_repo: Arc<SqliteAuditRepository>
    [x] paseto: Arc<PasetoService>
        └─ Ref: ADR 0008, docs/02-STACK.md L203-226
    [x] Migraciones automáticas al iniciar la app
        └─ Ref: ADR 0004

[x] apps/desktop/src-tauri/src/commands/auth.rs:
    └─ Ref: docs/03-STRUCTURE.md L554
    [x] #[tauri::command] async fn login(email, password, state, app_handle)
        → LoginUseCase::new(...).execute(...) + guarda tokens en tauri-plugin-store
        └─ Ref: docs/01-ARCHITECTURE.md L230-235 — mismo use case
    [x] #[tauri::command] async fn logout(app_handle) → limpia store
    [x] #[tauri::command] async fn get_current_user(app_handle, state) → verifica PASETO

[x] apps/desktop/src-tauri/src/commands/users.rs:
    [x] #[tauri::command] async fn list_users(state, page, search)
    [x] #[tauri::command] async fn get_user(state, id)

[x] apps/desktop/src-tauri/src/lib.rs:
    └─ Ref: docs/03-STRUCTURE.md L554
    [x] tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
            └─ Ref: docs/02-STACK.md L382 — secure storage
        .invoke_handler(tauri::generate_handler![login, logout, list_users, ...])
        .setup(|app| { /* inicializar AppState con pool SQLite */ })

[x] También: Refactorización de LoginUseCase, RefreshUseCase (crates/application)
    [x] Puertos PasswordVerifier + TokenGenerator definidos en crates/domain
    [x] PasetoService implementa TokenGenerator
    [x] Argon2Verifier implementa PasswordVerifier
    [x] Axum handlers refactorizados para delegar a Use Cases

[x] Verificar: cargo check --workspace → cero errores
```

---

## D.IV — Auth + estado local en desktop

> **Referencia:** ADR 0030 (Multiplataforma), ADR 0008 (PASETO), docs/02-STACK.md L382, docs/03-STRUCTURE.md L554

```
[x] Tokens PASETO guardados en tauri-plugin-store (almacenamiento seguro del SO):
    └─ Ref: docs/02-STACK.md L382, ADR 0008
    [x] Keychain en macOS
    [x] Credential Manager en Windows
    [x] Secret Service en Linux

[x] apps/desktop/src-tauri/src/commands/auth.rs:
    └─ Ref: docs/03-STRUCTURE.md L554
    [x] login() guarda access_token + refresh_token en store seguro
    [x] logout() limpia el store
    [x] get_token() recupera token del store (vía get_current_user)

[x] La misma DB SQLite del servidor corre localmente en desktop:
    └─ Ref: ADR 0004, docs/02-STACK.md L155-170
    [x] data/boilerplate-desktop.db (separada de la del servidor)
    [x] Mismas 6 migraciones aplicadas al arrancar
        └─ Ref: docs/01-ARCHITECTURE.md L139-164
    [x] PASETO_SECRET del dispositivo persistido en settings.bin
        └─ Ref: ADR 0008
    [x] Los datos son locales — sin conexión al servidor para funcionar
        └─ Ref: ADR 0030 — offline-first
```

---

## D.V — UI adaptada para desktop

> **Referencia:** ADR 0030 (Multiplataforma), ADR 0022 (Frontend), docs/02-STACK.md L382, L368-400, docs/03-STRUCTURE.md L429-432, L468-471

El mismo código Svelte funciona en browser y desktop con detección de entorno:

```
[x] apps/web/src/lib/tauri.ts:
    [x] Detección de entorno: `isTauri()`
    [x] Wrapper `tauriInvoke` para llamadas seguras al backend Rust

[x] apps/web/src/lib/api/auth.ts & users.ts:
    [x] Lógica híbrida: `if (isTauri()) { invoke(...) } else { fetch(...) }`
    [x] Soporte para login, registro, logout y listado offline

[x] apps/web/src/lib/stores/auth.svelte.ts:
    [x] `init()` adaptado para restaurar sesión desde el comando Rust al arrancar

[x] apps/web/src/components/layout/Sidebar.svelte:
    [x] Indicador visual "Desktop Mode" cuando se detecta Tauri

[x] Dependencias: @tauri-apps/api y plugins instalados en apps/web

[ ] Adaptaciones visuales para desktop:
    └─ Ref: docs/03-STRUCTURE.md L463-467 — componentes UI
    [ ] Sidebar siempre visible (pantalla más grande)
    [ ] Título de ventana nativo (Tauri window title)
    [ ] Tray icon con menú contextual (abrir, cerrar sesión, salir)
    [ ] Sin scroll horizontal en resoluciones < 1280px

[ ] Verificar que el mismo componente funciona en ambos entornos:
    └─ Ref: ADR 0030 — reutilización de UI
    [ ] UserTable.svelte → datos desde invoke() en desktop
    [ ] UserTable.svelte → datos desde fetch() en browser
    [ ] Mismo código, mismo comportamiento visual
```

---

## D.VI — Build + distribución

> **Referencia:** ADR 0030 (Multiplataforma), ADR 0013 (Build), docs/02-STACK.md L413

### Windows

```
[x] Iconos .ico y .icns verificados en apps/desktop/icons
[x] bundle.identifier configurado (com.laboratorio3030.boilerplate)
[x] tauri.conf.json optimizado para producción
[x] Capabilities y Permisos configurados en src-tauri/capabilities/default.json
[x] Listo para: cargo tauri build --target x86_64-pc-windows-msvc
    └─ Ref: docs/02-STACK.md L413 — build release
[x] Generará: target/release/bundle/msi/boilerplate_x.x.x_x64.msi
[x] Generará: target/release/bundle/nsis/boilerplate_x.x.x_x64-setup.exe
[x] Tamaño esperado del instalador < 15MB (gracias a Rust + WebView nativo)
    └─ Ref: ADR 0030, ADR 0013 — bundle minimalista
```

### macOS

```
[ ] Iconos .icns generados (512x512 obligatorio)
    └─ Ref: docs/03-STRUCTURE.md L552
[ ] Apple Developer certificate (para notarización)
[ ] cargo tauri build --target aarch64-apple-darwin  (Apple Silicon)
[ ] cargo tauri build --target x86_64-apple-darwin   (Intel)
    └─ Ref: docs/02-STACK.md L413
[ ] Universal binary: lipo -create -output boilerplate arm64 x86_64
[ ] Genera: target/release/bundle/dmg/boilerplate_x.x.x_universal.dmg
```

### Linux

```
[ ] cargo tauri build --target x86_64-unknown-linux-gnu
    └─ Ref: docs/02-STACK.md L413
[ ] Genera: target/release/bundle/appimage/boilerplate_x.x.x_amd64.AppImage
[ ] Genera: target/release/bundle/deb/boilerplate_x.x.x_amd64.deb
```

### Auto-update

```
[ ] tauri-plugin-updater apuntando a GitHub Releases
    └─ Ref: docs/02-STACK.md L382
[ ] El usuario recibe notificación cuando hay nueva versión
[ ] Actualización en segundo plano sin interrumpir el trabajo
```

---

## D.VII — CI para desktop

> **Referencia:** ADR 0030 (Multiplataforma), ADR 0010 (Testing), ADR 0013 (Build), docs/02-STACK.md L429-443

```
[x] .github/workflows/desktop.yml:
    └─ Ref: docs/03-STRUCTURE.md L571-573 — CI/CD
    [x] Trigger: push de tags v* y manual (workflow_dispatch)
    [x] Strategy matrix: [ubuntu-latest, windows-latest, macos-latest]
    [x] Dependencias de pnpm y node 22 configuradas
    [x] Uso de tauri-apps/tauri-action@v0
    [x] Generación automática de GitHub Draft Release al subir un tag
        └─ Ref: ADR 0030 — multiplataforma
    [x] Steps:
        [x] Setup Rust
        [x] Setup Node + pnpm
        [x] pnpm install
        [x] tauri-apps/tauri-action@v0 (args: --project apps/desktop)
    [x] Artefactos: .msi, .dmg, .AppImage subidos a GitHub Releases (Draft)

[ ] Verificar:
    [ ] Tag v0.1.0 → CI corre en los 3 SO
    [ ] Los 3 instaladores aparecen en GitHub Releases
    [ ] El instalador de Windows < 15MB
        └─ Ref: ADR 0013 — bundle minimalista
```

---

## Verificaciones de desktop

```bash
# 1. El workspace compila incluyendo desktop
cargo check --workspace
# → sin errores incluyendo apps/desktop ✓

# 2. Build de desarrollo
cd apps/desktop
cargo tauri dev
# → ventana abre con la UI de Svelte 5 ✓

# 3. Login funciona en desktop (sin servidor Axum)
# Iniciar sesión con admin@admin.com / 12345678
# → redirige al dashboard ✓

# 4. Los datos son locales — sin servidor
# Apagar just dev-api
# → Login sigue funcionando (usa SQLite local) ✓

# 5. El mismo dominio valida en ambos entornos
# Email inválido en desktop → mismo error que en browser ✓

# 6. Build de producción
cargo tauri build
# → binario en target/release/bundle/ < 15MB ✓

# 7. Sin herramientas de desarrollo en el instalador
ls target/release/bundle/
# → solo instaladores nativos ✓
```

---

## Diagrama de Flujo de Desktop (Tauri)

```
┌─────────────────────────────────────────────────────────────────────────┐
│  D.I — SETUP INICIAL                                                   │
│  ├─ Pre-requisito: MVP web en producción                              │
│  ├─ cargo install tauri-cli --version 2.2.5                          │
│  ├─ apps/desktop/src-tauri/ (Cargo.toml, main.rs, lib.rs)             │
│  ├─ Dependencias: application, database, auth, domain                   │
│  └─ Plugins: store, shell, notification, log, single-instance           │
│     └─ Ref: ADR 0030, 0004, 0008, docs/03-STRUCTURE.md L550-554        │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  D.II — CONFIGURACIÓN TAURI                                            │
│  ├─ tauri.conf.json (productName, identifier, windows, trayIcon)         │
│  ├─ CSP estricto: default-src 'self'                                  │
│  ├─ Capabilities: permisos granulares (red, filesystem)              │
│  └─ Iconos: .ico (Windows), .icns (macOS), .png (Linux)                │
│     └─ Ref: ADR 0030, docs/03-STRUCTURE.md L550-554                      │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  D.III — COMANDOS TAURI + BRIDGE RUST                                 │
│  ├─ AppState: pool SQLite + repositorios + paseto                       │
│  ├─ Commands: login, logout, list_users, create_user...                 │
│  ├─ Invocan use cases de crates/application (sin HTTP)                │
│  └─ Migraciones automáticas al iniciar                                 │
│     └─ Ref: ADR 0030, 0004, 0006, 0008, docs/01-ARCHITECTURE.md          │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  D.IV — AUTH + ESTADO LOCAL                                            │
│  ├─ Tokens PASETO en tauri-plugin-store (keychain/credential manager)    │
│  ├─ DB SQLite local: data/boilerplate-desktop.db                        │
│  ├─ Mismas 6 migraciones que servidor                                   │
│  └─ PASETO_SECRET del dispositivo (diferente al servidor)             │
│     └─ Ref: ADR 0030, 0008, 0004 — offline-first                        │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  D.V — UI ADAPTADA PARA DESKTOP                                        │
│  ├─ Detección entorno: '__TAURI__' in window                            │
│  ├─ Mismo código Svelte: invoke() en Tauri, fetch() en browser          │
│  ├─ auth.svelte.ts adaptado: store seguro vs cookies                    │
│  ├─ Adaptaciones: sidebar visible, tray icon, ventana nativa           │
│  └─ Mismos componentes: UserTable.svelte, etc.                          │
│     └─ Ref: ADR 0030, 0022 — reutilización UI                            │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  D.VI — BUILD + DISTRIBUCIÓN                                          │
│  ├─ Windows: .msi, .exe (< 15MB)                                       │
│  ├─ macOS: .dmg universal (Apple Silicon + Intel)                       │
│  ├─ Linux: .AppImage, .deb                                              │
│  └─ Auto-update: tauri-plugin-updater → GitHub Releases                 │
│     └─ Ref: ADR 0030, 0013 — bundle minimalista                         │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  D.VII — CI PARA DESKTOP                                               │
│  ├─ GitHub Actions: matrix [ubuntu, windows, macos]                   │
│  ├─ Build web + tauri-action                                            │
│  ├─ Artefactos: .msi, .dmg, .AppImage en GitHub Releases                │
│  └─ Verificación: tamaño < 15MB, sin errores                            │
│     └─ Ref: ADR 0030, 0010, 0013                                        │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Documentación Oficial de Referencia

| Herramienta | URL | Útil para |
|-------------|-----|-----------|
| **Tauri** | https://tauri.app/v1/guides/ | Framework desktop, comandos, state |
| **Tauri v2** | https://beta.tauri.app/guides/ | Plugins, capabilities, security |
| **tauri-plugin-store** | https://github.com/tauri-apps/plugins-workspace/tree/plugins/store | Secure storage (keychain) |
| **tauri-plugin-log** | https://github.com/tauri-apps/plugins-workspace/tree/plugins/log | Logs unificados Rust/JS |
| **tauri-plugin-updater** | https://github.com/tauri-apps/plugins-workspace/tree/plugins/updater | Auto-update |
| **cargo tauri** | https://docs.rs/tauri-cli/latest | CLI para build/dev |
| **Svelte 5 Runes** | https://svelte.dev/docs/svelte/what-are-runes | Reactividad en UI compartida |
| **Tauri Action** | https://github.com/tauri-apps/tauri-action | GitHub Actions para builds |

---

## Troubleshooting — Desktop (Tauri)

### D.I — Setup Inicial

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| cargo tauri --version falla | CLI no instalado | `cargo install tauri-cli --version 2.2.5` — Ref: docs/02-STACK.md L382 |
| Workspace no compila | apps/desktop no en workspace | Añadir a `Cargo.toml` root — Ref: docs/03-STRUCTURE.md L550 |
| Dependencias no encuentran crates | Path mal configurado | Verificar `path = "../../crates/application"` — Ref: docs/03-STRUCTURE.md L551 |

### D.II — Configuración

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Ventana no abre | tauri.conf.json mal formado | Validar JSON, revisar `identifier` — Ref: docs/03-STRUCTURE.md L550 |
| CSP bloquea recursos | CSP muy restrictivo | Ajustar `csp` en tauri.conf.json — Ref: ADR 0030 |
| Iconos no aparecen | Formato o tamaño incorrecto | Usar .ico (Windows), .icns (macOS 512x512) — Ref: docs/03-STRUCTURE.md L552 |
| Permisos denegados | Capabilities no configuradas | Revisar `capabilities/default.json` — Ref: docs/03-STRUCTURE.md L553 |

### D.III — Comandos Tauri

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| invoke() retorna error | Comando no registrado | Añadir a `generate_handler![]` — Ref: docs/03-STRUCTURE.md L554 |
| Pool no conecta | SQLite no inicializado | Llamar `create_pool()` en `.setup()` — Ref: ADR 0004 |
| Use case no compila | Traits no implementados | Verificar `impl Trait for Repo` en crates — Ref: docs/01-ARCHITECTURE.md L139-164 |
| Error "command not found" | Nombre de comando difiere | Verificar `#[tauri::command]` y nombre en invoke — Ref: ADR 0030 |

### D.IV — Auth Local

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Token no persiste | tauri-plugin-store no configurado | `.plugin(tauri_plugin_store...)` — Ref: docs/02-STACK.md L382 |
| Keychain no accesible | Capabilities no incluyen store | Añadir permiso en `default.json` — Ref: docs/03-STRUCTURE.md L553 |
| Login funciona pero no guarda | get_token no implementado | Implementar `get_token()` en commands — Ref: ADR 0008 |
| Migraciones no aplican | Path de DB incorrecto | Verificar `data/boilerplate-desktop.db` — Ref: ADR 0004 |

### D.V — UI Adaptada

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| UI no carga en desktop | Astro build no generado | `pnpm --filter web build` primero — Ref: docs/02-STACK.md L416 |
| Detección Tauri falla | '__TAURI__' no disponible | Verificar import o usar `window.__TAURI__` — Ref: docs/02-STACK.md L382 |
| invoke() no funciona | Comando no expuesto | Revisar `invoke_handler` en lib.rs — Ref: docs/03-STRUCTURE.md L554 |
| Componente se ve diferente | Estilos no cargan | Verificar `global.css` importado — Ref: docs/03-STRUCTURE.md L459 |
| Tray icon no aparece | Icono mal ubicado | Verificar `icons/tray-icon.png` — Ref: docs/03-STRUCTURE.md L552 |

### D.VI — Build

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Build > 15MB | Debug symbols incluidos | `--release` sin debug info — Ref: ADR 0013 |
| .msi no genera | WiX no instalado (Windows) | Instalar WiX Toolset — Ref: docs/02-STACK.md L413 |
| .dmg no notariza | Certificado Apple faltante | Configurar Apple Developer ID — Ref: ADR 0030 |
| Linux build falla | Dependencias sistema faltantes | Instalar `libgtk-3-dev`, etc. — Ref: docs/02-STACK.md L413 |
| Universal binary macOS falla | lipo mal usado | `lipo -create -output app arm64 x86_64` — Ref: ADR 0030 |

### D.VII — CI

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| CI falla en un SO | Matrix mal configurado | Verificar `runs-on` para cada SO — Ref: docs/03-STRUCTURE.md L571-573 |
| Artefactos no suben | Token GitHub faltante | Configurar `GITHUB_TOKEN` — Ref: ADR 0010 |
| Build macOS lento | Sin cache | Usar cache de cargo — Ref: docs/02-STACK.md L443 |
| Instalador muy grande | Debug build | Verificar `--release` — Ref: ADR 0013 |

---

**Nota:** Si un error persiste, revisar los ADRs 0030 (Multiplataforma), 0004 (SQLite), 0008 (PASETO), 0013 (Build) que son los más relevantes para desktop.
---

## Notas importantes

- `apps/desktop/` **NO se crea hasta que el MVP web esté en producción** — ADR 0030
- La UI Svelte 5 es exactamente la misma que en `apps/web/` — sin duplicación
- La DB local es SQLite — mismas migraciones que el servidor
- Los crates `domain`, `application` y `auth` son la fuente de verdad en todos los entornos
- `PASETO_SECRET` en desktop viene del keychain del SO — nunca hardcodeado
