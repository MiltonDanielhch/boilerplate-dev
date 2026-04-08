# ADR 0020 — Storage: Tigris S3 + Presigned URLs

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0004 (Litestream usa Tigris para backups de SQLite), ADR 0014 (infraestructura) |

---

## Contexto

El proyecto necesita almacenar archivos binarios — imágenes, documentos, y backups de la base
de datos — de forma persistente. Los requisitos son:

- **Egress de $0** — no pagar por cada descarga de archivo
- **Compatibilidad S3** — usar el estándar de la industria para no quedar atrapados en un proveedor
- **Baja latencia** — acceso a archivos comparable al acceso a la DB local
- **Sin configuración de CDN manual** — los archivos deben estar cerca de los usuarios automáticamente

---

## Decisión

Usar **Tigris** como motor de almacenamiento de objetos. Compatible con la API de S3
pero sin costos de egress y con replicación global automática.

### Configuración en Rust (SDK de AWS con endpoint de Tigris)

```toml
# crates/storage/Cargo.toml
aws-config = "1.1"
aws-sdk-s3 = "1.1"
```

```rust
// crates/storage/src/client.rs
pub async fn create_storage_client(config: &AppConfig) -> Client {
    let aws_config = aws_config::from_env()
        .endpoint_url(&config.aws_endpoint_url_s3)
        .region(Region::new("auto")) // Tigris gestiona la región automáticamente
        .load()
        .await;

    Client::new(&aws_config)
}
```

### Puerto de almacenamiento (arquitectura hexagonal — ADR 0001)

```rust
// crates/domain/src/ports/storage_repository.rs
pub trait StorageRepository: Send + Sync {
    async fn upload(
        &self,
        path:         &str,
        body:         Vec<u8>,
        content_type: &str,
    ) -> Result<String, StorageError>;

    async fn get_presigned_url(
        &self,
        path:       &str,
        expires_in: Duration,
    ) -> Result<String, StorageError>;

    async fn delete(&self, path: &str) -> Result<(), StorageError>;
}
```

### Adaptador Tigris/S3

```rust
// crates/storage/src/tigris_repository.rs
impl StorageRepository for TigrisRepository {
    async fn upload(
        &self, path: &str, body: Vec<u8>, content_type: &str,
    ) -> Result<String, StorageError> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(path)
            .body(body.into())
            .content_type(content_type)
            .send()
            .await
            .map_err(StorageError::from)?;

        Ok(format!("https://{}.fly.storage.tigris.dev/{}", self.bucket, path))
    }

    async fn get_presigned_url(
        &self, path: &str, expires_in: Duration,
    ) -> Result<String, StorageError> {
        // Los archivos privados NUNCA son públicos por error — siempre Presigned URLs
        let presigned = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(path)
            .presigned(PresigningConfig::expires_in(expires_in)?)
            .await
            .map_err(StorageError::from)?;

        Ok(presigned.uri().to_string())
    }

    async fn delete(&self, path: &str) -> Result<(), StorageError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(path)
            .send()
            .await
            .map_err(StorageError::from)?;
        Ok(())
    }
}
```

### Convención de nombres de buckets

```
boilerplate-{env}-{tipo}

boilerplate-production-assets    ← imágenes y documentos de usuarios
boilerplate-production-backups   ← backups de Litestream (SQLite WAL)
boilerplate-staging-assets
```

### Variables de entorno

```bash
# .env.example
AWS_ENDPOINT_URL_S3=https://fly.storage.tigris.dev
AWS_ACCESS_KEY_ID=tid_xxxxxxxxxxxx
AWS_SECRET_ACCESS_KEY=tsec_xxxxxxxxxxxx
STORAGE_BUCKET=boilerplate-production-assets
LITESTREAM_BUCKET=boilerplate-production-backups
```

### Plan B: Cloudflare R2

Si Tigris no está disponible, migrar a R2 requiere solo cambiar una variable:

```bash
# Mismo SDK, misma API S3 — cambiar solo el endpoint
AWS_ENDPOINT_URL_S3=https://xxxx.r2.cloudflarestorage.com
```

No hay cambios en el código Rust ni en los tests.

---

## Alternativas consideradas

| Opción | Motivo de descarte |
|--------|--------------------|
| AWS S3 | Costos de egress prohibitivos; configuración de IAM compleja |
| Cloudflare R2 | Excelente opción — es el plan B documentado si Tigris falla |
| MinIO self-hosted | Requiere gestionar otro servicio en el VPS — consume RAM y disco |
| Local filesystem | Inviable para contenedores — los archivos se pierden en cada redeploy |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para maximizar la robustez y eficiencia del almacenamiento de objetos:

| Herramienta | Propósito en el Storage |
| :--- | :--- |
| **`mime_guess`** | **Detección de Tipo:** Determina automáticamente el `Content-Type` de los archivos por su extensión. |
| **`image`** | **Procesamiento de Imágenes:** Permite redimensionar, comprimir y optimizar imágenes antes de subirlas. |
| **`tokio-util::io::ReaderStream`** | **Streaming Eficiente:** Manejo de archivos grandes sin cargar todo el contenido en memoria. |
| **`sentry`** | **Monitoreo de Errores:** Captura fallos en las operaciones de S3 y los reporta a Sentry con contexto. |

---

## Consecuencias

### ✅ Positivas

- Sin costos de egress — el escalamiento de descargas es predecible financieramente
- API S3 estándar — si Tigris desaparece, se cambia solo el endpoint en `.env`
- Presigned URLs — los archivos privados nunca se exponen públicamente por error
- Litestream usa el mismo servicio — un solo proveedor para datos y archivos
- Las subidas usan Presigned URLs — los archivos nunca pasan por el servidor Rust

### ⚠️ Negativas / Trade-offs

- Tigris es un servicio joven comparado con AWS S3 o Cloudflare R2
  → Plan B documentado: Cloudflare R2 — cambiar solo `AWS_ENDPOINT_URL_S3` en `.env`
  → El puerto `StorageRepository` garantiza que el código Rust no cambia en ningún caso
- Para archivos muy grandes (GBs por archivo), la consistencia eventual puede notarse
  → Para el MVP, los archivos son documentos y fotos (<50MB) — la consistencia es suficiente
  → Para archivos >100MB: usar Presigned URLs con upload directo y multipart upload
- Depende de la disponibilidad de Tigris
  → Monitorear con Healthchecks.io — si Tigris está caído, los uploads fallan con error claro

### Decisiones derivadas

- Las subidas desde el frontend usan Presigned URLs — sin pasar por el servidor Rust
- El bucket de backups es exclusivo para Litestream — separado del bucket de assets
- Cloudflare R2 está documentado como plan B — migración requiere solo cambiar una variable de entorno
- Los nombres de buckets siguen el patrón `boilerplate-{env}-{tipo}` — inamovible para no romper Litestream
