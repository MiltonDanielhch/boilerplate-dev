# Roadmap — Mobile (Tauri Mobile + KMP)

> **Stack:** Tauri 2.0 Mobile · Svelte 5 · Rust · KMP + UniFFI (Fase 3 solo si necesario)
>
> **ADR:** 0030 (Multiplataforma Tridente)
>
> ⚠️ **Activar solo cuando Desktop Tauri esté validado.** Ver criterios abajo.

---

## Estados

```
[ ] Pendiente   [~] En progreso   [x] Completado   [!] Bloqueado
```

---

## Estrategia

```
Fase 1 → Tauri Mobile (apps <15MB, reutiliza crates/domain)
Fase 2 → WebGPU/Canvas si alguna vista necesita rendimiento extra
Fase 3 → KMP + UniFFI SOLO si Tauri no alcanza (criterio: 120Hz nativos)
```

**Por qué Tauri en lugar de Capacitor/Cordova:**
Tauri reutiliza directamente los crates de Rust del proyecto. La lógica de validación,
hashing y reglas de negocio corre en el dispositivo sin duplicación. Apps <15MB —
ideal para zonas con datos móviles limitados como Bolivia.

---

## Progreso

| Bloque | Nombre | Progreso |
|--------|--------|----------|
| M.I | PWA + responsividad (base) | **0%** |
| M.II | Tauri Mobile setup | **0%** |
| M.III | Bridge con crates Rust | **0%** |
| M.IV | Build + distribución | **0%** |
| M.V | Fase 3 — KMP + UniFFI (Bridge nativo) | **0%** |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para maximizar la experiencia nativa y la utilidad en dispositivos móviles:

| Herramienta | Propósito en Mobile |
| :--- | :--- |
| **`tauri-plugin-biometric`** | **Seguridad UX:** Acceso rápido mediante FaceID o huella digital (biometría nativa). |
| **`tauri-plugin-barcode`** | **Captura de Datos:** Escaneo de códigos QR/Barras de alto rendimiento para trabajo en campo. |
| **`tauri-plugin-haptics`** | **Feedback Táctil:** Vibraciones precisas para mejorar la respuesta de la interfaz al tacto. |
| **`tauri-plugin-deep-link`** | **Integración:** Permite que links web abran la aplicación nativa automáticamente. |

---

## M.I — PWA + responsividad (base)

> **Referencia:** ADR 0030 (Multiplataforma), ADR 0022 (Frontend), docs/02-STACK.md L368-400, docs/03-STRUCTURE.md L450-453, L459-462

> Esta fase puede hacerse antes que el desktop — mejora la experiencia en móvil
> sin necesitar Tauri.

```
[ ] Meta viewport en BaseLayout.astro:
    └─ Ref: docs/03-STRUCTURE.md L450-453
    [ ] <meta name="viewport" content="width=device-width, initial-scale=1, user-scalable=no">
    [ ] Touch targets mínimo 44×44px en todos los botones
        └─ Ref: ADR 0022 — accesibilidad móvil
    [ ] Tailwind breakpoints: sm: md: lg: en todos los componentes
        └─ Ref: docs/02-STACK.md L376-377

[ ] Responsive en componentes clave:
    └─ Ref: docs/03-STRUCTURE.md L463-467
    [ ] Sidebar colapsable en móvil (hamburger menu)
    [ ] UserTable con scroll horizontal en pantallas pequeñas
        └─ Ref: docs/03-STRUCTURE.md L516-518
    [ ] CommandPalette ocupa pantalla completa en móvil (Astro/CSS)
    [ ] Formularios con inputs grandes para touch

[ ] PWA manifest:
    └─ Ref: docs/02-STACK.md L392-393, ADR 0023
    [ ] apps/web/public/manifest.json (via @vite-pwa/astro)
    [ ] name: "Boilerplate Fullstack"
    [ ] icons: 192x192 + 512x512
    [ ] display: "standalone"
    [ ] theme_color: "#534AB7"
    [ ] background_color: "#F8F8F6"

[ ] Service Worker para offline básico:
    └─ Ref: ADR 0024 (Local-First), docs/02-STACK.md L416
    [ ] Cache de assets estáticos (CSS, JS, fuentes)
    [ ] Estrategia: autoUpdate via Workbox

[ ] Verificar:
    └─ Ref: ADR 0022
    [ ] Chrome DevTools → Lighthouse PWA > 80
    [ ] En móvil real: "Añadir a pantalla de inicio" funciona
    [ ] Modo offline: muestra página desde caché
```

---

## M.II — Tauri Mobile setup (ADR 0030 Fase 1)

> **Referencia:** ADR 0030 (Multiplataforma), docs/02-STACK.md L382, docs/03-STRUCTURE.md L574

```
[x] Pre-requisito: Desktop Tauri validado en producción ✅
    └─ Ref: ADR 0030 — web → desktop → mobile

[x] Herramientas para Android:
    └─ Ref: docs/02-STACK.md L382 — Tauri Mobile
    [ ] Android Studio instalado (Requerido para ejecución local)
    [ ] SDK Android 24+ (Android 7.0)
    [ ] JAVA_HOME configurado
    [x] @tauri-apps/cli configurado en el workspace

[ ] Herramientas para iOS:
    └─ Ref: docs/02-STACK.md L382
    [ ] Xcode + Command Line Tools (Solo macOS)
    [ ] Apple Developer account
    [ ] CocoaPods instalado

[x] Crear apps/mobile/src-tauri/:
    └─ Ref: docs/03-STRUCTURE.md L574
    [x] apps/mobile/src-tauri/Cargo.toml
    [x] apps/mobile/src-tauri/tauri.conf.json
    [x] apps/mobile/src-tauri/capabilities/mobile.json
        └─ Ref: docs/03-STRUCTURE.md L553 — capabilities
    [x] apps/mobile/src-tauri/src/lib.rs
    [x] Lógica de comandos y estado compartida con Desktop

[ ] Inicializar proyectos nativos (Ejecutar localmente):
    [ ] npx tauri android init → genera apps/mobile/gen/android/
    [ ] npx tauri ios init     → genera apps/mobile/gen/apple/

[x] Configurar live reload:
    └─ Ref: docs/03-STRUCTURE.md L550
    [x] En tauri.conf.json: devUrl apunta a la IP local (no localhost)
    [x] Configurado para apuntar a apps/web/dist

[ ] Verificar:
    [ ] npx tauri android dev → app abre en emulador o dispositivo
    [ ] La UI de Svelte 5 se muestra correctamente
        └─ Ref: docs/02-STACK.md L375
```

---

## M.III — Bridge con crates Rust en Mobile

> **Referencia:** ADR 0030 (Multiplataforma), ADR 0024 (Local-First), ADR 0004 (SQLite), ADR 0008 (PASETO), docs/02-STACK.md L155-170, L203-226, L382

```
[ ] Los mismos comandos Tauri del desktop funcionan en mobile: 
    └─ Ref: ADR 0030 — reutilización de comandos
    [ ] commands/auth.rs: login, logout, get_current_user
    [ ] commands/users.rs: list_users, get_user, create_user

[ ] SQLite local en el dispositivo (ADR 0024 — Local-First): 
    └─ Ref: ADR 0024, ADR 0004, docs/02-STACK.md L155-170
    [ ] crates/database prepared for targets móviles
    [ ] Base de datos: boilerplate-mobile.db en el directorio de datos del sistema
    [ ] Mismas 6 migraciones ejecutadas al primer arranque
        └─ Ref: docs/01-ARCHITECTURE.md L139-164
    [ ] Los datos persisten entre sesiones (almacenamiento del app)

[ ] Auth storage en mobile: 
    └─ Ref: ADR 0008, docs/02-STACK.md L382
    [ ] Android: EncryptedSharedPreferences via tauri-plugin-store
    [ ] iOS: Keychain via tauri-plugin-store
    [ ] PASETO_SECRET generado al instalar (único por dispositivo)
        └─ Ref: ADR 0008

[ ] Sincronización opcional (ADR 0024 — Local-First): 
    [ ] Estructura lista para sync_queue local
    [ ] Works offline — datos siempre disponibles

[ ] Verificar:
    └─ Ref: ADR 0024
    [ ] Lógica de AppState adaptada para Mobile
    [ ] Generación de secreto único funcional en store móvil
```

---

## M.IV — Build + distribución

> **Referencia:** ADR 0030 (Multiplataforma), ADR 0013 (Build), docs/02-STACK.md L382, L413

### Android

```
[ ] npx tauri android build → genera .apk y .aab
    └─ Ref: docs/02-STACK.md L382
[ ] Tamaño del APK < 15MB (objetivo) — verificar con:
    └─ Ref: ADR 0030, ADR 0013 — bundle minimalista
    [ ] aapt dump badging release.apk | grep "package"
    [ ] Tamaño del bundle en Play Console
[ ] Firma de código: keystore Android
[ ] Distribución en Google Play Store
```

### iOS

```
[ ] npx tauri ios build → genera .ipa
    └─ Ref: docs/02-STACK.md L382
[ ] Tamaño del IPA < 15MB
    └─ Ref: ADR 0030, ADR 0013
[ ] Apple Developer certificate + provisioning profile
[ ] Distribución en App Store
[ ] TestFlight para beta testing
```

### CI para mobile

```
[x] .github/workflows/mobile.yml: ✅
    └─ Ref: docs/03-STRUCTURE.md L571-573 — CI/CD
    [x] Trigger: push de tags v*-mobile
    [x] Builds Android en ubuntu-latest
    [x] Builds iOS en macos-latest
        └─ Ref: ADR 0030 — multiplataforma
    [x] Artefactos: .apk, .ipa subidos a GitHub Releases (Draft)
```

---

## M.V — Fase 3: KMP + UniFFI (CONDICIONAL)

> **Referencia:** ADR 0030 (Multiplataforma), docs/03-STRUCTURE.md L195-198, L237

> **Criterio de activación:** listas de >10.000 elementos lentas,
> animaciones <60Hz constantes, o procesamiento offline masivo que requiera
> UI Compose nativa. NO implementar sin criterio medido en producción real.

```
[ ] Crear crates/mobile-bridge: 
    └─ Ref: docs/02-STACK.md L382, ADR 0031
    [ ] Configuración de UniFFI (Kotlin + Swift)
    [ ] Scaffolding automático en build.rs
    [ ] Exportación de modelos de dominio (Email validation)

[ ] Configurar targets nativos: 
    [ ] aarch64-linux-android (Android)
    [ ] aarch64-apple-ios (iOS)
    [ ] x86_64-apple-ios (Simulator)

[ ] Generación de bindings (Manual/CI): 
    [ ] uniffi-bindgen-rs integrado
    [ ] Estructura lista para integrar en Android Studio / Xcode

[ ] Verificar:
    [ ] cargo check -p mobile-bridge
    [ ] uniffi scaffolding generado en target/

[ ] Ejemplo de anotación UniFFI en crates/mobile-bridge:
    #[uniffi::export]
    pub fn validate_email(email: &str) -> Result<String, DomainError> {
        Email::new(email).map(|e| e.as_str().to_string())
    }
    └─ Ref: docs/02-STACK.md L88 — Email::new()

[ ] El núcleo Rust (domain, application, auth) NO CAMBIA
    └─ Ref: ADR 0030 — código Rust reutilizable
    → Solo se añaden anotaciones #[uniffi::export]
    → La UI cambia de WebView a Compose Multiplatform
    → El servidor Axum no cambia absolutamente nada
        └─ Ref: docs/01-ARCHITECTURE.md L194-196

[ ] NO implementar KMP sin criterio medido en producción real
    └─ Ref: ADR 0030
```

---

## Verificaciones de mobile

```bash
# 1. La web es responsive
# Abrir en Chrome DevTools → modo móvil → sin scroll horizontal

# 2. PWA instalable
# Chrome Android → "Añadir a pantalla de inicio" → app funciona

# 3. Tauri Android arranca
npx tauri android dev
# → App abre en emulador, UI visible ✓

# 4. Login funciona sin internet
# Activar modo avión en el dispositivo
# Intentar login → funciona (SQLite local) ✓

# 5. El mismo código de validación
# Email inválido → mismo error en servidor, desktop y mobile ✓
```

---

## Diagrama de Flujo de Mobile

```
┌─────────────────────────────────────────────────────────────────────────┐
│  M.I — PWA + RESPONSIVIDAD (Base)                                      │
│  ├─ Meta viewport en BaseLayout.astro                                   │
│  ├─ Touch targets > 44×44px                                             │
│  ├─ Responsive: sidebar, UserTable, CommandPalette                    │
│  ├─ PWA manifest (192x192, 512x512)                                     │
│  ├─ Service Worker (network-first, cache-fallback)                    │
│  └─ Lighthouse PWA > 80                                                 │
│     └─ Ref: ADR 0030, 0022, 0024                                       │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  M.II — TAURI MOBILE SETUP (Fase 1)                                    │
│  ├─ Pre-requisito: Desktop Tauri validado                               │
│  ├─ Herramientas: Android Studio, Xcode, CocoaPods                     │
│  ├─ Crear apps/mobile/src-tauri/                                        │
│  ├─ Capabilities: mobile.json (permisos granulares)                      │
│  ├─ npx tauri android init / ios init                                   │
│  └─ Live reload: devUrl apunta a IP local                              │
│     └─ Ref: ADR 0030, docs/03-STRUCTURE.md L574                         │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  M.III — BRIDGE CON CRATES RUST                                        │
│  ├─ Mismos comandos Tauri del desktop                                   │
│  ├─ SQLite local: Android/iOS nativo                                    │
│  ├─ Auth storage: EncryptedSharedPreferences / Keychain               │
│  ├─ PASETO_SECRET único por dispositivo                                 │
│  ├─ Sincronización opcional (sync_queue)                              │
│  └─ Works offline — datos siempre disponibles                          │
│     └─ Ref: ADR 0030, 0024, 0004, 0008                                  │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  M.IV — BUILD + DISTRIBUCIÓN                                          │
│  ├─ Android: .apk, .aab (< 15MB)                                        │
│  ├─ iOS: .ipa (< 15MB)                                                  │
│  ├─ Firma de código: keystores, certificados                            │
│  ├─ Distribución: Play Store, App Store, TestFlight                     │
│  └─ CI: GitHub Actions matrix [ubuntu-latest, macos-latest]             │
│     └─ Ref: ADR 0030, 0013, docs/03-STRUCTURE.md L571-573               │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  M.V — FASE 3: KMP + UNIFFI (Condicional)                              │
│  ├─ Criterio: >10.000 elementos lentos, animaciones <60Hz             │
│  ├─ #[uniffi::export] en crates/domain                                  │
│  ├─ uniffi-bindgen genera bindings Kotlin/Swift                       │
│  ├─ Compose Multiplatform como UI nativa                                 │
│  ├─ Núcleo Rust (domain) NO cambia                                      │
│  └─ NO implementar sin criterio medido                                  │
│     └─ Ref: ADR 0030, docs/03-STRUCTURE.md L195-198                     │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Documentación Oficial de Referencia

| Herramienta | URL | Útil para |
|-------------|-----|-----------|
| **Tauri Mobile** | https://tauri.app/v1/guides/building/mobile/ | iOS, Android, build, dev |
| **Tauri v2 Plugins** | https://beta.tauri.app/features/ | Biometric, barcode, haptics, deep-link |
| **tauri-plugin-store** | https://github.com/tauri-apps/plugins-workspace/tree/plugins/store | Secure storage móvil |
| **tauri-plugin-biometric** | https://github.com/tauri-apps/plugins-workspace/tree/plugins/biometric | FaceID, huella digital |
| **UniFFI** | https://github.com/mozilla/uniffi-rs | Bindings Kotlin/Swift desde Rust |
| **Compose Multiplatform** | https://www.jetbrains.com/lp/compose-mpp/ | UI nativa compartida |
| **Lighthouse PWA** | https://developer.chrome.com/docs/lighthouse/pwa | Auditar PWA |
| **Android Studio** | https://developer.android.com/studio | Dev environment Android |
| **Xcode** | https://developer.apple.com/xcode/ | Dev environment iOS |
| **App Store Connect** | https://appstoreconnect.apple.com/ | Distribución iOS |
| **Google Play Console** | https://play.google.com/console | Distribución Android |

---

## Troubleshooting — Mobile

### M.I — PWA + Responsividad

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| UI no cabe en pantalla | Sin viewport meta tag | Añadir `<meta name="viewport" ...>` — Ref: docs/03-STRUCTURE.md L450-453 |
| Botones pequeños en móvil | Touch targets < 44×44px | Usar `min-h-11 min-w-11` en Tailwind — Ref: ADR 0022 |
| PWA no instalable | Manifest mal configurado | Verificar `manifest.json` — Ref: docs/02-STACK.md L392-393 |
| Offline no funciona | Service Worker no registrado | Revisar SW en `astro.config.mjs` — Ref: ADR 0024 |
| Lighthouse PWA < 80 | Faltan iconos o theme_color | Completar manifest — Ref: ADR 0022 |

### M.II — Tauri Mobile Setup

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| npx tauri android init falla | SDK Android no encontrado | Configurar `ANDROID_HOME` — Ref: docs/02-STACK.md L382 |
| npx tauri ios init falla | Xcode Command Line Tools | `xcode-select --install` — Ref: docs/02-STACK.md L382 |
| App no abre en emulador | devUrl apunta a localhost | Cambiar a IP local en `tauri.conf.json` — Ref: docs/03-STRUCTURE.md L550 |
| Live reload no funciona | Dispositivo en otra red | Conectar a misma WiFi — Ref: docs/03-STRUCTURE.md L550 |
| Capabilities denegadas | Permisos no configurados | Revisar `capabilities/mobile.json` — Ref: docs/03-STRUCTURE.md L553 |

### M.III — Bridge con Rust

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| SQLite no compila para Android | Target no instalado | `rustup target add aarch64-linux-android` — Ref: ADR 0004 |
| SQLite no compila para iOS | Target no instalado | `rustup target add aarch64-apple-ios` — Ref: ADR 0004 |
| Token no persiste | tauri-plugin-store no configurado | Añadir plugin en `lib.rs` — Ref: docs/02-STACK.md L382 |
| Comando no existe | No registrado en invoke_handler | Añadir a `generate_handler![]` — Ref: docs/03-STRUCTURE.md L554 |
| Sync no funciona | Sin conexión o queue mal | Verificar `sync_queue` tabla — Ref: ADR 0024 |

### M.IV — Build y Distribución

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| APK > 15MB | Debug build | Usar `--release` — Ref: ADR 0013 |
| Keystore error | Certificado no configurado | Configurar `android.signingConfigs` — Ref: docs/02-STACK.md L382 |
| iOS signing failed | Provisioning profile mal | Revisar Apple Developer Portal — Ref: docs/02-STACK.md L382 |
| App rechazada en Play Store | Permisos sensibles | Revisar `capabilities/mobile.json` — Ref: docs/03-STRUCTURE.md L553 |
| CI falla en iOS | macOS runner sin Xcode | Usar `macos-latest` — Ref: docs/03-STRUCTURE.md L571-573 |

### M.V — KMP + UniFFI (Fase 3)

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| uniffi-bindgen falla | Funciones no exportables | Usar tipos simples en `#[uniffi::export]` — Ref: docs/03-STRUCTURE.md L195-198 |
| Kotlin bindings no generan | uniffi.toml mal | Configurar `bindings.kotlin` — Ref: docs/03-STRUCTURE.md L195-198 |
| UI lenta aún con KMP | Problema en lógica, no UI | Revisar algoritmos en crates/domain — Ref: docs/01-ARCHITECTURE.md L117-136 |

---

**Nota:** Si un error persiste, revisar los ADRs 0030 (Multiplataforma), 0024 (Local-First), 0004 (SQLite), 0008 (PASETO) que son los más relevantes para mobile.
---

## Notas importantes

- Mobile solo después de que Desktop Tauri esté validado — ADR 0030
- El criterio de KMP (Fase 3) es rendimiento medido, no especulación
- Las apps pesan <15MB — ideal para instalación con datos móviles en Bolivia
- Un solo codebase Rust + Svelte 5 corre en Web, Desktop y Mobile
