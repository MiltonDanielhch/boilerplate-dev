# ADR F-003 — Futuro: Kotlin Multiplatform + UniFFI (Mobile Nativo)

| Campo | Valor |
|-------|-------|
| **Estado** | 🔮 Futuro — activar en ADR 0031 Nivel 5 |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0030 (Multiplataforma Tridente), ADR 0031 (Nivel 5), ADR 0001 (Hexagonal), ADR 0024 (Local-First) |

---

## Contexto

Este ADR documenta la implementación de **Kotlin Multiplatform (KMP)** con **UniFFI** para crear
aplicaciones mobile nativas (Android e iOS) cuando Tauri Mobile no alcance el rendimiento requerido.

**Activar cuando (ADR 0031 Nivel 5):**
- Listas de >10.000 elementos con scroll <60fps en Tauri Mobile
- Animaciones <60Hz constantes en dispositivos medianos (no flagship)
- Procesamiento offline masivo que requiere UI nativa 120Hz
- Métricas reales de producción confirman que WebView es el cuello de botella

**NO activar especulativamente.** KMP tiene overhead significativo:
- Requiere Android Studio + Xcode (setup no trivial)
- Curva de aprendizaje de Kotlin + Jetpack Compose + SwiftUI
- Equipo necesita expertise en mobile nativo

---

## Decisión futura

Usar **Kotlin Multiplatform** con **UniFFI** para exponer los crates de dominio Rust a Kotlin y Swift.

### Arquitectura

```
crates/domain (Rust) — sin cambios en la lógica
    ↓ UniFFI bindings generados automáticamente
    ↓ uniffi-bindgen
─────────────────────────────────────
domain.kt              domain.swift
    ↓                      ↓
Jetpack Compose      SwiftUI
(Android)            (iOS)
```

### UniFFI: Bindings automáticos

```rust
// crates/domain/src/user.rs — solo añadir anotaciones
#[derive(uniffi::Record)]
pub struct User {
    pub id: String,
    pub email: String,
}

#[uniffi::export]
pub fn validate_email(email: &str) -> Result<String, DomainError> {
    Email::new(email).map(|e| e.as_str().to_string())
}
```

```kotlin
// Android — generado automáticamente por UniFFI
val result = Domain.validateEmail("user@example.com")
```

```swift
// iOS — generado automáticamente por UniFFI
let result = Domain.validateEmail(email: "user@example.com")
```

---

## Cuándo activar

| Criterio | Umbral |
|----------|--------|
| Scroll performance | <60fps en listas de >10.000 elementos en Tauri Mobile |
| Animaciones | <60Hz constantes en dispositivos medianos |
| Procesamiento offline | UI de progreso nativa para >1.000 registros |
| Madurez Fase 2 | >60 días estable con NATS procesando >10k mensajes/hora |

---

## Implementación

### 1. Setup de desarrollo

```bash
# Instalar UniFFI
cargo install uniffi_bindgen

# Añadir targets mobile
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add aarch64-apple-ios
rustup target add x86_64-apple-ios

# Android Studio Hedgehog+ y Xcode 15+ (solo Mac para iOS)
```

### 2. Configurar crate domain

```toml
# crates/domain/Cargo.toml
[dependencies]
uniffi = { version = "0.25", features = ["build"] }

[build-dependencies]
uniffi_build = "0.25"
```

```rust
// crates/domain/build.rs
fn main() {
    uniffi_build::generate_scaffolding("./src/domain.udl")
        .expect("UDL generation failed");
}
```

```text
// crates/domain/src/domain.udl
namespace boilerplate {
    string validate_email(string email);
    User create_user(string email, string password);
    
    sequence<User> list_users(u32 limit, u32 offset);
    User get_user(string id);
    void delete_user(string id);
};

[Record]
dictionary User {
    string id;
    string email;
    boolean is_active;
};

[Error]
enum DomainError {
    "InvalidEmail",
    "UserNotFound",
    "ValidationFailed",
};
```

### 3. Generar bindings

```bash
# Kotlin para Android
uniffi-bindgen generate --language kotlin \
    crates/domain/src/domain.udl \
    --out-dir apps/android/generated/

# Swift para iOS
uniffi-bindgen generate --language swift \
    crates/domain/src/domain.udl \
    --out-dir apps/ios/generated/
```

### 4. Android — Jetpack Compose

```kotlin
// apps/android/src/main/kotlin/ui/screens/LoginScreen.kt
@Composable
fun LoginScreen(
    onLoginSuccess: () -> Unit
) {
    var email by remember { mutableStateOf("") }
    var error by remember { mutableStateOf<String?>(null) }
    
    Column {
        TextField(
            value = email,
            onValueChange = { 
                email = it
                // Validación vía Rust
                error = try {
                    Domain.validateEmail(it)
                    null
                } catch (e: DomainException) {
                    e.message
                }
            },
            label = { Text("Email") },
            isError = error != null
        )
        
        error?.let { Text(it, color = MaterialTheme.colorScheme.error) }
        
        Button(
            onClick = { 
                val user = Domain.createUser(email, password)
                saveToken(user.id) // Secure Storage
                onLoginSuccess()
            },
            enabled = error == null && email.isNotEmpty()
        ) {
            Text("Login")
        }
    }
}
```

### 5. iOS — SwiftUI

```swift
// apps/ios/ios/Views/LoginView.swift
struct LoginView: View {
    @State private var email = ""
    @State private var errorMessage: String?
    @StateObject private var viewModel = LoginViewModel()
    
    var body: some View {
        VStack {
            TextField("Email", text: $email)
                .textFieldStyle(RoundedBorderTextFieldStyle())
                .onChange(of: email) { newValue in
                    // Validación vía Rust
                    do {
                        _ = try Domain.validateEmail(newValue)
                        errorMessage = nil
                    } catch let error as DomainError {
                        errorMessage = error.message
                    } catch {
                        errorMessage = "Unknown error"
                    }
                }
            
            if let error = errorMessage {
                Text(error)
                    .foregroundColor(.red)
            }
            
            Button("Login") {
                do {
                    let user = try Domain.createUser(email: email, password: password)
                    saveToKeychain(user.id) // Keychain
                    viewModel.isLoggedIn = true
                } catch {
                    errorMessage = error.localizedDescription
                }
            }
            .disabled(errorMessage != nil || email.isEmpty)
        }
    }
}
```

### 6. Offline-first (ADR 0024)

```kotlin
// Android — Room para caché offline
@Dao
interface UserDao {
    @Query("SELECT * FROM users WHERE is_synced = 0")
    suspend fun getPendingSync(): List<UserEntity>
    
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insert(user: UserEntity)
}

// Sync cuando hay conexión
class SyncWorker(appContext: Context, params: WorkerParameters) :
    CoroutineWorker(appContext, params) {
    
    override suspend fun doWork(): Result {
        val pending = userDao.getPendingSync()
        pending.forEach { user ->
            Domain.createUser(user.email, user.password)
            userDao.markSynced(user.id)
        }
        return Result.success()
    }
}
```

---

## Infraestructura de build

### Android — Gradle

```kotlin
// apps/android/build.gradle.kts
plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.mozilla.rust-android-gradle") version "0.9.3"
}

android {
    ndkVersion = "26.1.10909125"
    
    defaultConfig {
        minSdk = 26
        targetSdk = 34
    }
}

dependencies {
    // Librería Rust generada
    implementation(files("src/main/jniLibs/domain.jar"))
    
    // Compose
    implementation(platform("androidx.compose:compose-bom:2024.02.00"))
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.material3:material3")
    
    // Room
    implementation("androidx.room:room-runtime:2.6.1")
    kapt("androidx.room:room-compiler:2.6.1")
    implementation("androidx.room:room-ktx:2.6.1")
    
    // Biometric
    implementation("androidx.biometric:biometric:1.1.0")
}

// Compilar Rust para Android
tasks.register<Exec>("buildRust") {
    commandLine("cargo", "ndk", "-t", "arm64-v8a", "-o", "src/main/jniLibs", "build")
}
```

### iOS — Xcode

```bash
# Build script para compilar Rust
#!/bin/bash
set -e

CARGO_TARGET_X86_64_APPLE_IOS_SIMULATOR_SDK=
CARGO_TARGET_AARCH64_APPLE_IOS_SIMULATOR_SDK=

# Compilar para iOS real
cargo build --target aarch64-apple-ios --release

# Compilar para simulador
cargo build --target aarch64-apple-ios-sim --release

# Crear universal binary
lipo -create \
    target/aarch64-apple-ios/release/libdomain.a \
    target/aarch64-apple-ios-sim/release/libdomain.a \
    -output apps/ios/Frameworks/libdomain.a
```

---

## Comparativa con Tauri Mobile

| Aspecto | Tauri Mobile | KMP + UniFFI |
|---------|-------------|--------------|
| **Bundle size** | ~15MB (con WebView) | ~10MB (nativo) |
| **Performance** | ~60fps (WebView limitado) | ~120fps (nativo) |
| **Setup** | npm + cargo | Android Studio + Xcode |
| **UI framework** | Svelte (web tech) | Jetpack Compose / SwiftUI |
| **Reutilización Rust** | Tauri commands | UniFFI bindings directos |
| **Offline storage** | SQLite vía plugin | Room / Core Data nativo |
| **Biometric auth** | Plugin de terceros | APIs nativas directas |
| **Equipo necesario** | 1 fullstack | 2+ especialistas mobile |
| **When to use** | Fase 1-2, <10k items | Fase 3, >10k items, 120Hz |

---

## Consecuencias

### ✅ Positivas

- Rendimiento nativo 120Hz en iOS, 60-120fps en Android
- Reutilización máxima del dominio Rust — cero duplicación de lógica
- UI nativa con Jetpack Compose y SwiftUI — experiencia de clase mundial
- Acceso directo a APIs nativas (biométrico, notificaciones, cámara)
- Offline-first con SQLite nativo (Room/Core Data) — ADR 0024

### ⚠️ Negativas / Trade-offs

- **Overhead de setup:** Android Studio + Xcode + Kotlin + Swift = curva pronunciada
- **Equipo especializado:** Necesita developers con experiencia en mobile nativo
- **Build compleja:** NDK, JNI, cross-compilation, lipo (iOS universal binaries)
- **CI/CD más complejo:** Builds de iOS requieren Mac (GitHub Actions macOS runners más caros)
- **Debugging distribuido:** Problemas pueden estar en Rust, Kotlin, Swift, o los bindings

### Decisiones derivadas

- `apps/android/` y `apps/ios/` se crean **solo** cuando el criterio de rendimiento es real y medido
- Los crates `domain`, `application` nunca tienen dependencias de plataforma — UniFFI es solo anotaciones
- La UI es nativa por plataforma — no se intenta compartir UI entre Android e iOS
- Tauri Mobile sigue siendo la opción por defecto — KMP es la excepción para casos extremos

---

## Estado actual

**No implementar.** Mantener Tauri Mobile (ADR 0030) hasta que:
1. Fase 1 y Fase 2 estén completadas y estables por >60 días
2. Métricas de producción confirmen <60fps en escenarios reales
3. El equipo tenga recursos para el overhead de KMP

Ver 60-ROADMAP-FASE3.md para el checklist completo de implementación.
