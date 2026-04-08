# Roadmap — Fase 3: KMP + Mobile Nativo (Kotlin Multiplatform)

> **Objetivo:** Implementar aplicación mobile nativa de clase mundial cuando Tauri Mobile no alcance el rendimiento requerido.
>
> **Stack:** Kotlin Multiplatform (KMP) · UniFFI · Jetpack Compose · SwiftUI
>
> **ADRs:** F-003 (KMP + UniFFI), ADR 0030 (Multiplataforma Tridente), ADR 0031 (Nivel 5)
>
> **Criterio de activación:** Listas de >10.000 elementos con scroll <60fps, animaciones <60Hz constantes en dispositivos medianos, o procesamiento offline masivo que requiera UI nativa 120Hz.
>
> **NO implementar sin métricas reales de producción.** KMP tiene overhead significativo de setup (Android Studio + Xcode).

---

## Estados

```
[ ] Pendiente   [~] En progreso   [x] Completado   [!] Bloqueado
```

---

## Progreso

| Fase | Nombre | Progreso |
|------|--------|----------|
| F3.1 | Setup KMP + UniFFI | 0% |
| F3.2 | Crate domain con anotaciones UniFFI | 0% |
| F3.3 | Android (Jetpack Compose) | 0% |
| F3.4 | iOS (SwiftUI) | 0% |
| F3.5 | Desktop KMP (opcional) | 0% |
| F3.6 | Verificaciones | 0% |

---

## Prerrequisitos estrictos

> **NO saltarse estos pasos.** Fase 3 requiere madurez total del ecosistema.

```
[ ] Fase 1 completada y estable por >90 días
    [ ] MVP web en producción con usuarios reales
    [ ] Todos los roadmaps 01-07 completados ✅
    └─ Ref: ADR 0031 — madurez antes de KMP

[ ] Fase 2 completada y estable por >60 días
    [ ] NATS procesando >10.000 mensajes/hora sin pérdida
    [ ] Workers escalan horizontalmente sin problemas
    [ ] 50-ROADMAP-FASE2.md completado ✅
    └─ Ref: ADR 0031 Nivel 4 → 5

[ ] Criterio de rendimiento MEDIDO en producción:
    [ ] Listas de >10.000 elementos: scroll <60fps en Tauri Mobile
    [ ] Animaciones: <60Hz constantes en dispositivos medianos (no flagship)
    [ ] Procesamiento offline masivo que requiere UI nativa 120Hz
    └─ Medir con: Chrome DevTools Performance, Android Studio GPU profiling
    └─ Ref: ADR 0030 Fase 3, ADR 0031 Nivel 5

[ ] Recursos de equipo disponibles:
    [ ] Developer con experiencia en Kotlin y Android Studio
    [ ] Accesso a Mac para builds de iOS (Xcode requerido)
    [ ] Tiempo: 3-4 semanas dedicadas (setup + implementación)
    └─ Ref: ADR F-003 — overhead de setup no trivial

[ ] Infraestructura lista:
    [ ] VPS ya no es limitante (presupuesto permite escalar)
    [ ] CI/CD soporta builds de Android e iOS
    └─ Ref: ADR 0031 — escalar vertical primero
```

**Si algún prerrequisito NO se cumple:** Seguir optimizando Tauri Mobile. KMP es costoso.

---

## F3.1 — Setup KMP + UniFFI

> **Referencia:** ADR F-003, ADR 0030 Fase 3, ADR 0031 Nivel 5

```
[ ] Instalar herramientas de desarrollo:
    [ ] Android Studio Hedgehog o superior
    [ ] Xcode 15+ (solo en Mac para iOS)
    [ ] Kotlin 1.9+
    [ ] Rust toolchain con targets:
        [ ] rustup target add aarch64-linux-android
        [ ] rustup target add armv7-linux-androideabi
        [ ] rustup target add aarch64-apple-ios
        [ ] rustup target add x86_64-apple-ios

[ ] Instalar UniFFI bindgen:
    [ ] cargo install uniffi_bindgen --version 0.28.0
    [ ] Verificar: uniffi-bindgen --version

[ ] Configurar Android NDK:
    [ ] Descargar NDK 26+ desde Android Studio
    [ ] Configurar ANDROID_NDK_HOME
    [ ] Verificar que rust-lldb funciona
```

**Verificación F3.1:** `cargo build --target aarch64-linux-android` compila sin errores.

---

## F3.2 — Crate domain con anotaciones UniFFI

> **Referencia:** ADR F-003, ADR 0030 Fase 3
>
> Los crates de dominio existentes se mantienen intactos — solo se añaden anotaciones.

```
[ ] Añadir UniFFI a crates/domain/Cargo.toml:
    [ ] uniffi = { version = "0.25", features = ["build"] }
    [ ] [build-dependencies] uniffi_build = "0.25"

[ ] Crear crates/domain/build.rs:
    [ ] uniffi_build::generate_scaffolding("./src/domain.udl")

[ ] Crear crates/domain/src/domain.udl:
    [ ] Definir interfaces UDL para funciones públicas
    [ ] namespace boilerplate {
        string validate_email(string email);
        User create_user(string email, string password);
    }

[ ] Añadir macros UniFFI a structs de dominio:
    ```rust
    #[derive(uniffi::Record)]
    pub struct User {
        pub id: String,
        pub email: String,
    }
    ```

[ ] Generar bindings:
    [ ] uniffi-bindgen generate --language kotlin crates/domain/src/domain.udl
    [ ] uniffi-bindgen generate --language swift crates/domain/src/domain.udl

[ ] Verificar bindings generados:
    [ ] domain.kt para Android
    [ ] domain.swift para iOS
```

**Verificación F3.2:** `cargo build` genera bindings sin errores.

---

## F3.3 — Android (Jetpack Compose)

> **Referencia:** ADR F-003, ADR 0030 Fase 3
>
> Reutiliza toda la lógica de negocio vía UniFFI bindings.

```
[ ] Crear apps/android/:
    [ ] Android Studio → New Project → Empty Compose Activity
    [ ] Mover a apps/android/ en el monorepo

[ ] Configurar build.gradle.kts:
    [ ] Añadir dependencia a librería Rust generada:
        ```kotlin
        dependencies {
            implementation(files("../../crates/domain/generated/domain.jar"))
        }
        ```
    [ ] Configurar NDK y JNI:
        [ ] android { ndkVersion = "26.1.10909125" }

[ ] Implementar arquitectura Android:
    [ ] Domain layer: usa funciones del crate domain via UniFFI
    [ ] UI layer: Jetpack Compose con Material3
    [ ] State: ViewModel + StateFlow

[ ] Pantallas principales (reutilizar diseño de web):
    [ ] LoginScreen.kt — validación de email vía domain.validate_email()
    [ ] DashboardScreen.kt — lista de usuarios con lazy loading
    [ ] UserFormScreen.kt — crear/editar usuarios

[ ] Implementar SQLite local (ADR 0024):
    [ ] Room Database para caché offline
    [ ] Sync con backend cuando hay conexión

[ ] Implementar autenticación:
    [ ] Secure Storage para PASETO token
    [ ] BiometricPrompt para login rápido
```

**Verificación F3.3:** App Android corre en emulador, login funciona, lista scroll >60fps.

---

## F3.4 — iOS (SwiftUI)

> **Referencia:** ADR F-003, ADR 0030 Fase 3
>
> **Requiere Mac con Xcode.** No hay alternativa para builds de iOS.

```
[ ] Crear apps/ios/:
    [ ] Xcode → New Project → iOS App
    [ ] Mover a apps/ios/ en el monorepo

[ ] Integrar bindings Swift:
    [ ] Añadir domain.swift generado al proyecto
    [ ] Configurar Framework de Rust como dependencia

[ ] Implementar arquitectura iOS:
    [ ] Domain: usa structs y funciones del domain.swift
    [ ] UI: SwiftUI con vistas declarativas
    [ ] State: @StateObject y @ObservedObject

[ ] Pantallas principales:
    [ ] LoginView.swift — validación vía Domain.validateEmail()
    [ ] DashboardView.swift — List con lazy loading nativo
    [ ] UserFormView.swift — Form con validación

[ ] Implementar Core Data (opcional) o SQLite directo:
    [ ] Caché offline para datos de usuario
    [ ] Sync con backend

[ ] Implementar autenticación:
    [ ] Keychain para PASETO token
    [ ] FaceID/TouchID con LocalAuthentication
```

**Verificación F3.4:** App corre en simulador iOS, login funciona, rendimiento 120Hz.

---

## F3.5 — Desktop KMP (opcional)

> **Referencia:** ADR 0030, ADR F-003
>
> Desktop KMP es opcional — Tauri ya cubre Desktop. Solo si se necesita consistencia total.

```
[ ] Evaluar necesidad:
    [ ] ¿Tauri Desktop tiene alguna limitación real?
    [ ] ¿Se necesita compartir código UI entre Android y Desktop?
    
[ ] Si se implementa:
    [ ] Compose Multiplatform para Desktop (JVM)
    [ ] Reutiliza el mismo domain.kt de Android
    [ ] UI consistente con Android
```

**Nota:** Tauri Desktop sigue siendo la opción preferida para Desktop (ADR 0030).

---

## F3.6 — Verificaciones

```bash
# 1. UniFFI genera bindings correctamente
cd crates/domain
uniffi-bindgen generate --language kotlin src/domain.udl
uniffi-bindgen generate --language swift src/domain.udl
# Esperado: archivos domain.kt y domain.swift generados sin errores

# 2. Crate domain compila para targets mobile
cargo build --target aarch64-linux-android
cargo build --target aarch64-apple-ios
# Esperado: sin errores de linkeo

# 3. Android app compila
./gradlew :apps:android:assembleDebug
# Esperado: APK generado en apps/android/build/outputs/apk/debug/

# 4. iOS app compila (en Mac)
xcodebuild -project apps/ios/ios.xcodeproj -scheme ios -configuration Debug
# Esperado: Build succeeded

# 5. Rendimiento nativo medido
# Android: GPU profiling >60fps en scroll de lista 10k elementos
# iOS: Core Animation >120Hz en animaciones
# Esperado: cumple criterio de activación con margen

# 6. Lógica de negocio idéntica
# Mismo email inválido → mismo error en Web, Android, iOS
# Esperado: comportamiento idéntico — crates/domain es fuente única de verdad

# 7. Offline-first funciona
# Activar modo avión → login funciona (SQLite local)
# Esperado: comportamiento idéntico a Tauri Mobile — Ref: ADR 0024
```

---

## Diagrama de Flujo Fase 3

```
┌─────────────────────────────────────────────────────────────────┐
│  F3.1 — Setup KMP + UniFFI                                       │
│  ├─ Android Studio + Xcode instalados                            │
│  ├─ Rust targets mobile añadidos                                 │
│  ├─ uniffi-bindgen instalado                                     │
│  └─ Verificar: cargo build --target aarch64-linux-android        │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│  F3.2 — Crate domain con UniFFI                                  │
│  ├─ Añadir uniffi a Cargo.toml                                   │
│  ├─ Crear domain.udl con interfaces                              │
│  ├─ #[derive(uniffi::Record)] en structs                         │
│  ├─ Generar domain.kt y domain.swift                             │
│  └─ Verificar: bindings compilan                                 │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│  F3.3 — Android (Jetpack Compose)                                │
│  ├─ apps/android/ con Compose UI                               │
│  ├─ Domain vía domain.kt (UniFFI)                                │
│  ├─ Pantallas: Login, Dashboard, UserForm                        │
│  ├─ Room DB para offline                                         │
│  ├─ Secure Storage + Biometric                                   │
│  └─ Verificar: >60fps en scroll 10k elementos                    │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│  F3.4 — iOS (SwiftUI)                                            │
│  ├─ apps/ios/ con SwiftUI                                        │
│  ├─ Domain vía domain.swift (UniFFI)                             │
│  ├─ Pantallas: LoginView, DashboardView, UserFormView            │
│  ├─ Core Data / SQLite para offline                              │
│  ├─ Keychain + FaceID/TouchID                                    │
│  └─ Verificar: 120Hz en animaciones                               │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│  F3.5 — Desktop KMP (opcional)                                   │
│  ├─ Solo si Tauri tiene limitaciones                             │
│  ├─ Compose Multiplatform JVM                                    │
│  └─ Reutiliza domain.kt de Android                               │
└─────────────────────────────────────────────────────────────────┘
```

---

## Documentación Oficial de Referencia

| Herramienta | URL | Útil para |
|-------------|-----|-----------|
| **UniFFI** | https://mozilla.github.io/uniffi-rs/ | Bindings Rust ↔ Kotlin/Swift |
| **Kotlin Multiplatform** | https://kotlinlang.org/docs/multiplatform.html | Share code between platforms |
| **Jetpack Compose** | https://developer.android.com/jetpack/compose | Android UI |
| **SwiftUI** | https://developer.apple.com/documentation/swiftui | iOS UI |
| **Rust Mobile** | https://github.com/rust-mobile | Community resources |

---

## Troubleshooting — Fase 3

### F3.1 — Setup

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| `uniffi-bindgen: command not found` | No instalado | `cargo install uniffi_bindgen --version 0.28.0` — Ref: ADR F-003 |
| `error: linker cc not found` | NDK no configurado | Configurar ANDROID_NDK_HOME — Ref: ADR F-003 |
| `xcode-select: error` | Xcode no instalado | Instalar Xcode desde App Store (solo Mac) — Ref: ADR F-003 |
| `could not find native static library` | Targets Rust faltantes | `rustup target add aarch64-linux-android` — Ref: ADR F-003 |

### F3.2 — UniFFI Bindings

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| `UDL syntax error` | Archivo .udl mal formado | Verificar sintaxis UDL — Ref: UniFFI docs |
| `generate_scaffolding failed` | build.rs incorrecto | Verificar ruta al archivo .udl — Ref: ADR F-003 |
| Bindings vacíos | Funciones no exportadas en UDL | Añadir funciones a namespace en .udl — Ref: ADR F-003 |
| `uniffi::Record` no funciona | Feature no activado | Añadir `features = ["build"]` en Cargo.toml — Ref: ADR F-003 |

### F3.3 — Android

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| `ClassNotFoundException: domain` | JAR no vinculado | Añadir `implementation(files(...))` en build.gradle — Ref: ADR F-003 |
| `UnsatisfiedLinkError` | Librería nativa no cargada | Verificar JNI loading en MainActivity — Ref: ADR F-003 |
| Scroll lento (<60fps) | LazyColumn mal configurado | Usar `items()` con key y contentType — Ref: Jetpack Compose docs |
| BiometricPrompt no aparece | Permiso faltante | Añadir `<uses-permission android:name="android.permission.USE_BIOMETRIC"/>` — Ref: ADR F-003 |

### F3.4 — iOS

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| `No such module 'domain'` | Framework no vinculado | Añadir domain.swift al target — Ref: ADR F-003 |
| `dyld: Library not loaded` | Librería Rust no embebida | Añadir al "Embed Libraries" build phase — Ref: ADR F-003 |
| Animaciones <120Hz | CADisplayLink mal configurado | Usar `UIView.animate` con `UIViewAnimationOptions` — Ref: SwiftUI docs |
| FaceID no funciona | Info.plist faltante | Añadir `NSFaceIDUsageDescription` — Ref: ADR F-003 |

---

## Comparativa: Tauri vs KMP

| Aspecto | Tauri Mobile | KMP + UniFFI |
|---------|-------------|--------------|
| **Bundle size** | ~15MB | ~10MB (sin WebView) |
| **Setup** | npm + cargo | Android Studio + Xcode |
| **UI** | Svelte (web tech) | Jetpack Compose / SwiftUI (nativo) |
| **Performance** | ~60fps (WebView) | ~120fps (nativo) |
| **Offline** | SQLite vía plugin | SQLite nativo |
| **Reutilización Rust** | Tauri commands | UniFFI bindings |
| **Costo equipo** | 1 developer fullstack | 2+ developers (Android + iOS) |
| **When to use** | Fase 1-2, <10k items | Fase 3, >10k items, 120Hz necesario |

---

## Notas Importantes

- **KMP es la última opción**, no la primera. Tauri cubre el 90% de casos.
- **Mismo crate domain:** La lógica de negocio no se reescribe — solo se expone vía UniFFI.
- **Criterio concreto:** Sin métricas de <60fps en producción real, NO activar Fase 3.
- **Overhead real:** Android Studio + Xcode + Kotlin + Swift = curva de aprendizaje pronunciada.
- Ver ADR F-003 para detalles arquitectónicos completos de KMP + UniFFI.
