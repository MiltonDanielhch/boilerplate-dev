# ADR 0034 — Sincronización Offline-First: Web + Desktop + Mobile

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado — implementación por fases según necesidad real |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0001 (Arquitectura Hexagonal) · ADR 0004 (SQLite WAL) · ADR 0024 (Local-First) · ADR 0030 (Multiplataforma) · ADR 0033 (MySQL) · ADR F-002 (PostgreSQL) |

---

## Contexto

El proyecto corre en tres plataformas: Web (Astro + Svelte 5), Desktop (Tauri) y Mobile
(Tauri Mobile). En cada una hay datos locales que deben:

1. **Funcionar sin internet** — el usuario guarda, edita y consulta sin conexión
2. **Sincronizarse cuando vuelve la red** — sin perder datos ni duplicar
3. **Resolver conflictos** — cuando dos dispositivos editaron lo mismo mientras estaban offline
4. **Ser agnósticos al backend** — funcionar con SQLite, MySQL o PostgreSQL en el servidor

Este es el problema más complejo del stack. Este ADR lo aborda con honestidad sobre
su complejidad y propone una estrategia por fases que no sobreingeniería el MVP.

---

## La verdad sobre sincronización offline — sin suavizar

Sincronización bidireccional con resolución de conflictos es uno de los problemas
más difíciles de la ingeniería de software. Empresas como Figma, Linear y Notion
tienen equipos dedicados solo a esto. Antes de elegir una estrategia, hay que
entender el espacio del problema:

### Los tres estados posibles de un dato

```
SERVIDOR ──────────────────────────────────────────
     │
     │  último estado conocido: "Pedro"
     │
     ├── DISPOSITIVO A (online)   → cambia a "Pedro Mamani"   ← sube al servidor
     │
     ├── DISPOSITIVO B (offline)  → cambia a "Pedro Quispe"   ← no puede subir
     │
     └── DISPOSITIVO C (offline)  → elimina el registro       ← no puede subir
```

Cuando B y C vuelven a conectarse: ¿quién tiene razón?
No hay respuesta técnica correcta universal — depende del dominio de negocio.

### Las cuatro estrategias de resolución de conflictos

| Estrategia | Cuándo usar | Complejidad |
|-----------|-------------|-------------|
| **Server Wins** | El servidor siempre tiene razón — descarta cambios locales | ⭐ Baja |
| **Last Write Wins (LWW)** | El cambio con timestamp más reciente gana | ⭐⭐ Media |
| **Client Wins** | Los cambios locales siempre se aplican — merge manual posterior | ⭐⭐ Media |
| **CRDTs** | Algoritmos matemáticos que fusionan automáticamente | ⭐⭐⭐⭐⭐ Muy alta |

**Para el MVP de este proyecto: Last Write Wins con `updated_at`.**
CRDTs son la solución correcta a largo plazo pero requieren 2-3 meses de ingeniería
dedicada. LWW cubre el 95% de los casos de uso reales con el 10% del trabajo.

---

## Arquitectura general — la misma en las tres plataformas

```
┌──────────────────────────────────────────────────────┐
│                    SERVIDOR (Axum)                    │
│  DB: SQLite WAL / MySQL / PostgreSQL                  │
│  Endpoint: POST /api/v1/sync/push                     │
│  Endpoint: GET  /api/v1/sync/pull?since={timestamp}   │
└──────────────────┬───────────────────────────────────┘
                   │ HTTPS + PASETO Bearer (ADR 0008)
        ┌──────────┴──────────┐
        │                     │
┌───────▼──────┐    ┌─────────▼────────┐    ┌──────────────────┐
│   Web (WASM) │    │  Desktop (Tauri)  │    │  Mobile (Tauri)  │
│              │    │                  │    │                  │
│ SQLite en    │    │ SQLite local     │    │ SQLite local     │
│ OPFS del     │    │ en el disco      │    │ en el dispositivo│
│ navegador    │    │                  │    │                  │
│              │    │                  │    │                  │
│ sync_queue   │    │ sync_queue       │    │ sync_queue       │
│ (tabla local)│    │ (tabla local)    │    │ (tabla local)    │
└──────────────┘    └──────────────────┘    └──────────────────┘
```

**Principio fundamental:** cada cliente tiene su propia copia de SQLite con los
datos relevantes para ese usuario. El servidor es la fuente de verdad final.
Los clientes son optimistas — escriben localmente primero, sincronizan después.

---

## Decisión: 3 Fases de implementación

### Fase 1 — Sync básico con cola de operaciones (MVP)
*Implementar ahora. Cubre el 90% de los casos de uso.*

### Fase 2 — Sync en tiempo real con WebSockets
*Cuando el MVP esté validado y se necesite colaboración multi-usuario.*

### Fase 3 — CRDTs para edición colaborativa simultánea
*Solo si múltiples usuarios editan los mismos registros simultáneamente.*

---

## Fase 1 — Cola de Operaciones (Outbox Pattern) — ADR 0034

### 1.1 — Columnas obligatorias en todas las tablas sincronizadas

```sql
-- Agregar a CADA tabla que necesita sincronización
-- Funciona igual en SQLite, MySQL y PostgreSQL

-- Para SQLite (migraciones existentes):
ALTER TABLE users ADD COLUMN sync_version   INTEGER NOT NULL DEFAULT 0;
ALTER TABLE users ADD COLUMN device_id      TEXT;
ALTER TABLE users ADD COLUMN is_synced      INTEGER NOT NULL DEFAULT 1;  -- 1=synced, 0=pending
-- updated_at ya existe en las 6 migraciones base ✓

-- Para MySQL:
ALTER TABLE users ADD COLUMN sync_version  INT         NOT NULL DEFAULT 0;
ALTER TABLE users ADD COLUMN device_id     VARCHAR(36);
ALTER TABLE users ADD COLUMN is_synced     TINYINT(1)  NOT NULL DEFAULT 1;

-- Para PostgreSQL:
ALTER TABLE users ADD COLUMN sync_version  INTEGER NOT NULL DEFAULT 0;
ALTER TABLE users ADD COLUMN device_id     TEXT;
ALTER TABLE users ADD COLUMN is_synced     BOOLEAN NOT NULL DEFAULT TRUE;
```

### 1.2 — Tabla sync_queue en el cliente (cada dispositivo)

```sql
-- data/migrations/{timestamp}_create_sync_queue.sql
-- Esta tabla vive SOLO en el cliente — NO en el servidor

CREATE TABLE IF NOT EXISTS sync_queue (
    id           TEXT     PRIMARY KEY NOT NULL,     -- UUID del cliente
    operation    TEXT     NOT NULL,                  -- CREATE | UPDATE | DELETE
    table_name   TEXT     NOT NULL,                  -- "users", "leads", etc.
    record_id    TEXT     NOT NULL,                  -- ID del registro afectado
    payload      TEXT     NOT NULL,                  -- JSON del estado completo
    device_id    TEXT     NOT NULL,                  -- ID único de este dispositivo
    created_at   DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    synced_at    DATETIME,                           -- NULL = pendiente de sync
    retry_count  INTEGER  NOT NULL DEFAULT 0,
    status       TEXT     NOT NULL DEFAULT 'pending' -- pending | in_flight | synced | failed
);

CREATE INDEX IF NOT EXISTS idx_sync_queue_status   ON sync_queue(status);
CREATE INDEX IF NOT EXISTS idx_sync_queue_table    ON sync_queue(table_name, record_id);
```

### 1.3 — Tabla sync_state en el cliente (cursor de sincronización)

```sql
-- Guarda el "punto de corte" — hasta dónde se sincronizó con el servidor
CREATE TABLE IF NOT EXISTS sync_state (
    table_name       TEXT     PRIMARY KEY NOT NULL,
    last_pulled_at   DATETIME,   -- último timestamp que se bajó del servidor
    last_pushed_at   DATETIME,   -- último push exitoso
    device_id        TEXT     NOT NULL
);
```

### 1.4 — Cómo funciona cada operación en el cliente

```typescript
// apps/web/src/lib/sync/repository.ts
// El mismo patrón en Web (TypeScript) y Tauri (vía IPC)

class SyncableRepository {
    // ── CREAR ────────────────────────────────────────────────────────────────
    async create(table: string, data: Record<string, unknown>): Promise<string> {
        const id = crypto.randomUUID();
        const now = new Date().toISOString();
        const record = { ...data, id, created_at: now, updated_at: now,
                         sync_version: 0, device_id: DEVICE_ID, is_synced: false };

        // 1. Escribir en SQLite local — INMEDIATO, sin esperar red
        await localDB.execute(
            `INSERT INTO ${table} VALUES (${placeholders(record)})`,
            Object.values(record)
        );

        // 2. Encolar para sincronización posterior
        await this.enqueue('CREATE', table, id, record);

        return id; // Respuesta instantánea al usuario
    }

    // ── ACTUALIZAR ───────────────────────────────────────────────────────────
    async update(table: string, id: string, changes: Record<string, unknown>): Promise<void> {
        const now = new Date().toISOString();

        // 1. Actualizar localmente con nuevo timestamp y versión
        await localDB.execute(
            `UPDATE ${table} SET updated_at = ?, is_synced = 0,
             sync_version = sync_version + 1, ${setClause(changes)}
             WHERE id = ?`,
            [now, ...Object.values(changes), id]
        );

        // 2. Encolar — payload completo del registro actualizado
        const record = await localDB.queryOne(`SELECT * FROM ${table} WHERE id = ?`, [id]);
        await this.enqueue('UPDATE', table, id, record);
    }

    // ── ELIMINAR (Soft Delete — nunca DELETE real) ────────────────────────────
    async softDelete(table: string, id: string): Promise<void> {
        const now = new Date().toISOString();

        await localDB.execute(
            `UPDATE ${table} SET deleted_at = ?, is_synced = 0, updated_at = ?
             WHERE id = ?`,
            [now, now, id]
        );

        await this.enqueue('DELETE', table, id, { id, deleted_at: now });
    }

    // ── ENCOLAR ──────────────────────────────────────────────────────────────
    private async enqueue(
        operation: 'CREATE' | 'UPDATE' | 'DELETE',
        table:     string,
        recordId:  string,
        payload:   unknown
    ): Promise<void> {
        await localDB.execute(
            `INSERT INTO sync_queue
             (id, operation, table_name, record_id, payload, device_id, status)
             VALUES (?, ?, ?, ?, ?, ?, 'pending')`,
            [crypto.randomUUID(), operation, table, recordId,
             JSON.stringify(payload), DEVICE_ID]
        );
    }
}
```

### 1.5 — Motor de sincronización en el cliente

```typescript
// apps/web/src/lib/sync/engine.ts

class SyncEngine {
    private isRunning = false;
    private intervalId: ReturnType<typeof setInterval> | null = null;

    // Iniciar sync automático cuando hay red
    start(): void {
        window.addEventListener('online', () => this.syncNow());
        // Sync periódico: cada 30s cuando hay conexión
        this.intervalId = setInterval(() => {
            if (navigator.onLine) this.syncNow();
        }, 30_000);
    }

    async syncNow(): Promise<SyncResult> {
        if (this.isRunning) return { pushed: 0, pulled: 0 };
        this.isRunning = true;

        try {
            // Orden obligatorio: primero PUSH (subir cambios locales),
            // luego PULL (bajar cambios del servidor)
            // Esto evita sobrescribir cambios del servidor con los locales
            const pushed = await this.push();
            const pulled = await this.pull();
            return { pushed, pulled };
        } finally {
            this.isRunning = false;
        }
    }

    // ── PUSH — subir cambios locales al servidor ──────────────────────────────
    private async push(): Promise<number> {
        const pending = await localDB.query(
            `SELECT * FROM sync_queue
             WHERE status = 'pending'
             ORDER BY created_at ASC
             LIMIT 50`   // Procesar en lotes de 50
        );

        if (pending.length === 0) return 0;

        // Marcar como in_flight para evitar doble envío
        const ids = pending.map(op => op.id);
        await localDB.execute(
            `UPDATE sync_queue SET status = 'in_flight'
             WHERE id IN (${ids.map(() => '?').join(',')})`,
            ids
        );

        try {
            const response = await fetch('/api/v1/sync/push', {
                method:  'POST',
                headers: {
                    'Content-Type':  'application/json',
                    'Authorization': `Bearer ${getAccessToken()}`,
                    'Idempotency-Key': ids.join(','), // Para reintentos seguros
                },
                body: JSON.stringify({ operations: pending }),
            });

            if (!response.ok) throw new Error(`HTTP ${response.status}`);

            const { accepted, conflicts } = await response.json();

            // Marcar exitosos como synced
            await localDB.execute(
                `UPDATE sync_queue SET status = 'synced', synced_at = CURRENT_TIMESTAMP
                 WHERE id IN (${accepted.map(() => '?').join(',')})`,
                accepted
            );

            // Aplicar resolución de conflictos (LWW)
            for (const conflict of conflicts) {
                await this.resolveConflict(conflict);
            }

            // Actualizar is_synced = 1 en los registros afectados
            for (const op of pending.filter(o => accepted.includes(o.id))) {
                await localDB.execute(
                    `UPDATE ${op.table_name} SET is_synced = 1 WHERE id = ?`,
                    [op.record_id]
                );
            }

            return accepted.length;
        } catch (error) {
            // Revertir a pending para reintentar
            await localDB.execute(
                `UPDATE sync_queue SET status = 'pending', retry_count = retry_count + 1
                 WHERE id IN (${ids.map(() => '?').join(',')}) AND status = 'in_flight'`,
                ids
            );
            // Marcar como failed si superó 5 intentos
            await localDB.execute(
                `UPDATE sync_queue SET status = 'failed'
                 WHERE id IN (${ids.map(() => '?').join(',')}) AND retry_count >= 5`,
                ids
            );
            throw error;
        }
    }

    // ── PULL — bajar cambios del servidor ─────────────────────────────────────
    private async pull(): Promise<number> {
        const syncState = await localDB.queryOne(
            `SELECT last_pulled_at FROM sync_state WHERE table_name = 'all'`
        );
        const since = syncState?.last_pulled_at ?? '1970-01-01T00:00:00Z';

        const response = await fetch(
            `/api/v1/sync/pull?since=${encodeURIComponent(since)}&device_id=${DEVICE_ID}`,
            { headers: { 'Authorization': `Bearer ${getAccessToken()}` } }
        );

        if (!response.ok) throw new Error(`HTTP ${response.status}`);

        const { changes, server_time } = await response.json();

        // Aplicar cada cambio con LWW: gana quien tiene updated_at más reciente
        for (const change of changes) {
            await this.applyServerChange(change);
        }

        // Actualizar cursor
        await localDB.execute(
            `INSERT OR REPLACE INTO sync_state (table_name, last_pulled_at, device_id)
             VALUES ('all', ?, ?)`,
            [server_time, DEVICE_ID]
        );

        return changes.length;
    }

    // ── RESOLVER CONFLICTO con Last Write Wins ────────────────────────────────
    private async resolveConflict(conflict: SyncConflict): Promise<void> {
        const local = await localDB.queryOne(
            `SELECT * FROM ${conflict.table} WHERE id = ?`,
            [conflict.record_id]
        );

        if (!local) return;

        const localTime  = new Date(local.updated_at).getTime();
        const serverTime = new Date(conflict.server_updated_at).getTime();

        if (serverTime >= localTime) {
            // El servidor ganó — actualizar local con datos del servidor
            await this.applyServerChange(conflict);
            console.info(`[Sync] Conflicto resuelto — servidor ganó: ${conflict.table}/${conflict.record_id}`);
        } else {
            // El cliente ganó — el servidor aplicará nuestros datos
            console.info(`[Sync] Conflicto resuelto — cliente ganó: ${conflict.table}/${conflict.record_id}`);
        }
    }

    private async applyServerChange(change: ServerChange): Promise<void> {
        const local = await localDB.queryOne(
            `SELECT updated_at FROM ${change.table} WHERE id = ?`,
            [change.record_id]
        );

        if (local) {
            const localTime  = new Date(local.updated_at).getTime();
            const changeTime = new Date(change.updated_at).getTime();

            // Last Write Wins: solo aplicar si el cambio del servidor es más reciente
            // y el registro local no tiene cambios pendientes de subir
            if (changeTime > localTime) {
                await localDB.execute(
                    `UPDATE ${change.table} SET ${setClause(change.data)},
                     is_synced = 1 WHERE id = ?`,
                    [...Object.values(change.data), change.record_id]
                );
            }
        } else if (change.operation !== 'DELETE') {
            // El registro no existe localmente — insertar
            await localDB.execute(
                `INSERT OR IGNORE INTO ${change.table} (${Object.keys(change.data).join(',')})
                 VALUES (${Object.keys(change.data).map(() => '?').join(',')})`,
                Object.values(change.data)
            );
        }
    }
}
```

### 1.6 — Endpoints de sync en el servidor (Axum)

```rust
// crates/infrastructure/src/http/handlers/sync.rs

/// POST /api/v1/sync/push — recibe operaciones del cliente
#[utoipa::path(
    post,
    path = "/api/v1/sync/push",
    security(("bearer_auth" = [])),
    tag = "sync",
)]
pub async fn push_operations(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Json(body): Json<PushRequest>,
) -> Result<Json<PushResponse>, AppError> {
    let mut accepted:  Vec<String> = Vec::new();
    let mut conflicts: Vec<ConflictInfo> = Vec::new();

    for op in body.operations {
        // Verificar que el usuario tiene permiso sobre este registro
        let can_modify = state.sync_repo
            .user_owns_record(&user_id, &op.table_name, &op.record_id)
            .await?;

        if !can_modify {
            continue; // Silenciosamente ignorar — no revelar si el registro existe
        }

        let server_record = state.sync_repo
            .find_record(&op.table_name, &op.record_id)
            .await?;

        match server_record {
            None => {
                // Registro nuevo — aplicar directamente
                state.sync_repo.apply_operation(&op).await?;
                accepted.push(op.id.clone());
            }
            Some(server) => {
                let client_time = op.payload["updated_at"]
                    .as_str().unwrap_or("").parse::<DateTime<Utc>>().unwrap_or_default();
                let server_time = server.updated_at;

                if client_time >= server_time {
                    // Cliente tiene versión más reciente — aplicar
                    state.sync_repo.apply_operation(&op).await?;
                    accepted.push(op.id.clone());
                } else {
                    // Conflicto — servidor tiene versión más reciente
                    conflicts.push(ConflictInfo {
                        operation_id:       op.id.clone(),
                        table:              op.table_name.clone(),
                        record_id:          op.record_id.clone(),
                        server_updated_at:  server_time.to_rfc3339(),
                        server_data:        server.to_json(),
                    });
                }
            }
        }
    }

    Ok(Json(PushResponse { accepted, conflicts }))
}

/// GET /api/v1/sync/pull?since=2026-01-01T00:00:00Z&device_id=xxx
/// Solo devuelve cambios posteriores al timestamp — eficiencia garantizada
#[utoipa::path(
    get,
    path = "/api/v1/sync/pull",
    security(("bearer_auth" = [])),
    tag = "sync",
)]
pub async fn pull_changes(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Query(params): Query<PullParams>,
) -> Result<Json<PullResponse>, AppError> {
    // Solo devolver registros que pertenecen a este usuario
    // y que fueron modificados DESPUÉS de `since`
    let changes = state.sync_repo
        .get_changes_since(&user_id, &params.since, &params.device_id)
        .await?;

    Ok(Json(PullResponse {
        changes,
        server_time: Utc::now().to_rfc3339(),
    }))
}
```

```rust
// crates/domain/src/ports/sync_repository.rs
pub trait SyncRepository: Send + Sync {
    async fn find_record(&self, table: &str, id: &str)
        -> Result<Option<SyncRecord>, DomainError>;

    async fn apply_operation(&self, op: &SyncOperation)
        -> Result<(), DomainError>;

    async fn get_changes_since(&self, user_id: &UserId, since: &str, device_id: &str)
        -> Result<Vec<ServerChange>, DomainError>;

    async fn user_owns_record(&self, user_id: &UserId, table: &str, record_id: &str)
        -> Result<bool, DomainError>;
}
```

### 1.7 — SQLite en el navegador (Web)

```typescript
// apps/web/src/lib/sync/wasm-db.ts
// SQLite en el browser via OPFS (Origin Private File System)
// Disponible en Chrome 102+, Firefox 111+, Safari 15.2+

import initSqlJs from 'sql.js';
import sqliteWasm from '@sqlite.org/sqlite-wasm';

let db: SQLiteDB | null = null;

export async function initLocalDB(): Promise<SQLiteDB> {
    if (db) return db;

    // Usar OPFS para persistencia entre sesiones del browser
    const sqlite3 = await sqliteWasm({
        locateFile: (file) => `/sqlite-wasm/${file}`,
    });

    if (sqlite3.capi.sqlite3_vfs_find('opfs')) {
        // OPFS disponible — persistencia real entre sesiones
        db = new sqlite3.oo1.OpfsDb('/boilerplate.db');
    } else {
        // Fallback a memoria — datos se pierden al cerrar tab
        db = new sqlite3.oo1.DB(':memory:');
        console.warn('[DB] OPFS no disponible — usando memoria (datos no persisten)');
    }

    await runMigrations(db);
    return db;
}

async function runMigrations(db: SQLiteDB): Promise<void> {
    // Las mismas migraciones del servidor, adaptadas al cliente
    // Solo las tablas necesarias para el usuario actual
    db.exec(`
        CREATE TABLE IF NOT EXISTS users (
            id VARCHAR(36) PRIMARY KEY, email TEXT, updated_at DATETIME,
            sync_version INTEGER DEFAULT 0, is_synced INTEGER DEFAULT 1,
            deleted_at DATETIME
        );
        CREATE TABLE IF NOT EXISTS sync_queue ( /* ver 1.2 */ );
        CREATE TABLE IF NOT EXISTS sync_state ( /* ver 1.3 */ );
    `);
}
```

### 1.8 — SQLite en Desktop y Mobile (Tauri)

En Tauri, SQLite corre nativamente vía `crates/database` — el mismo código Rust
que el servidor, sin diferencias. Los comandos Tauri ya tienen acceso a la DB local.

```rust
// apps/desktop/src-tauri/src/commands/sync.rs

#[tauri::command]
pub async fn sync_now(state: tauri::State<'_, AppState>) -> Result<SyncResult, String> {
    // Paso 1: Push — subir cambios locales pendientes
    let pending_ops = state.sync_repo.get_pending_operations().await
        .map_err(|e| e.to_string())?;

    if !pending_ops.is_empty() {
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/api/v1/sync/push", state.config.server_url))
            .bearer_auth(&state.access_token)
            .json(&PushRequest { operations: pending_ops })
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let result: PushResponse = response.json().await.map_err(|e| e.to_string())?;

        // Aplicar resolución de conflictos LWW
        for conflict in result.conflicts {
            state.sync_repo.resolve_conflict_lww(&conflict).await
                .map_err(|e| e.to_string())?;
        }
    }

    // Paso 2: Pull — bajar cambios del servidor
    let since = state.sync_repo.get_last_pulled_at().await
        .map_err(|e| e.to_string())?;

    let response = reqwest::Client::new()
        .get(&format!("{}/api/v1/sync/pull?since={}&device_id={}",
             state.config.server_url, since, state.device_id))
        .bearer_auth(&state.access_token)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let pull_result: PullResponse = response.json().await.map_err(|e| e.to_string())?;

    for change in &pull_result.changes {
        state.sync_repo.apply_server_change(change).await
            .map_err(|e| e.to_string())?;
    }

    state.sync_repo.update_last_pulled_at(&pull_result.server_time).await
        .map_err(|e| e.to_string())?;

    Ok(SyncResult {
        pushed: 0, // ya procesado arriba
        pulled: pull_result.changes.len(),
    })
}

/// Invocado automáticamente cuando Tauri detecta reconexión de red
#[tauri::command]
pub async fn on_network_restored(
    state: tauri::State<'_, AppState>
) -> Result<(), String> {
    sync_now(state).await.map(|_| ())
}
```

### 1.9 — Indicadores de estado de sync en la UI

```svelte
<!-- apps/web/src/components/ui/SyncStatus.svelte -->
<script lang="ts">
    import { syncEngine } from '$lib/sync/engine';

    let status = $state<'synced' | 'syncing' | 'offline' | 'error'>('synced');
    let pendingCount = $state(0);

    $effect(() => {
        // Actualizar estado según la cola local
        const interval = setInterval(async () => {
            const pending = await localDB.queryOne(
                `SELECT COUNT(*) as count FROM sync_queue WHERE status = 'pending'`
            );
            pendingCount = pending?.count ?? 0;

            if (!navigator.onLine) {
                status = 'offline';
            } else if (pendingCount > 0) {
                status = 'syncing';
            } else {
                status = 'synced';
            }
        }, 3000);
        return () => clearInterval(interval);
    });
</script>

<div class="sync-status" data-status={status}>
    {#if status === 'offline'}
        <span>📵 Sin conexión — {pendingCount} cambios pendientes</span>
    {:else if status === 'syncing'}
        <span>🔄 Sincronizando... ({pendingCount} pendientes)</span>
    {:else if status === 'error'}
        <span>⚠️ Error de sincronización
            <button onclick={() => syncEngine.syncNow()}>Reintentar</button>
        </span>
    {:else}
        <span>✅ Sincronizado</span>
    {/if}
</div>
```

---

## Fase 2 — Sync en tiempo real con WebSockets (cuando sea necesario)

*Activar cuando: múltiples usuarios necesiten ver cambios en tiempo real sin recargar.*

```rust
// crates/infrastructure/src/http/handlers/sync_ws.rs
// WebSocket que notifica a todos los clientes cuando hay cambios

use axum::extract::ws::{WebSocket, WebSocketUpgrade};

pub async fn sync_websocket(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_sync_ws(socket, state, user_id))
}

async fn handle_sync_ws(mut socket: WebSocket, state: AppState, user_id: UserId) {
    // Suscribir a cambios del usuario
    let mut rx = state.sync_broadcaster.subscribe();

    loop {
        tokio::select! {
            // Cuando hay cambios nuevos — notificar al cliente
            Ok(notification) = rx.recv() => {
                if notification.user_id == user_id {
                    let msg = serde_json::to_string(&notification).unwrap();
                    if socket.send(Message::Text(msg)).await.is_err() {
                        break;
                    }
                }
            }
            // Ping periódico para mantener la conexión viva
            _ = tokio::time::sleep(Duration::from_secs(30)) => {
                if socket.send(Message::Ping(vec![])).await.is_err() {
                    break;
                }
            }
        }
    }
}
```

```svelte
<!-- En el cliente — escuchar notificaciones de cambios remotos -->
<script lang="ts">
    let ws: WebSocket;

    onMount(() => {
        ws = new WebSocket(`wss://tudominio.com/api/v1/sync/ws`);
        ws.onmessage = async (event) => {
            const notification = JSON.parse(event.data);
            // Hay cambios en el servidor — hacer pull selectivo
            await syncEngine.pullTable(notification.table);
            // TanStack Query invalida automáticamente las queries afectadas
            queryClient.invalidateQueries({ queryKey: [notification.table] });
        };
        return () => ws?.close();
    });
</script>
```

---

## Fase 3 — CRDTs (solo si edición simultánea de mismo registro)

*Activar cuando: el mismo documento es editado por múltiples usuarios al mismo tiempo.*

Esta fase está fuera del scope del MVP. Las opciones son:

| Opción | Descripción | Self-hosted |
|--------|-------------|-------------|
| **PowerSync** | Sync engine Postgres/MySQL → SQLite cliente. Open Edition gratuita | ✅ Sí |
| **ElectricSQL** | Postgres → SQLite cliente con sync activo-activo | ✅ Sí |
| **cr-sqlite** | Extensión de SQLite que añade CRDTs a nivel de tabla | ✅ Sí |
| **Triplit** | DB full-stack con sync engine (adquirido por Supabase 2025) | ✅/☁️ |

**Recomendación para Fase 3:** PowerSync Open Edition. Soporta Postgres, MySQL y MongoDB
como backend. Los clientes usan SQLite local. Tiene SDK para Web (WASM), Kotlin Multiplatform
y Swift. Se puede integrar con el backend Axum existente sin reescribirlo.

---

## Cómo afecta a cada crate del proyecto

```
crates/domain/      → agregar entidades SyncOperation, SyncRecord, SyncState
                      agregar port SyncRepository
                      SIN cambios en entidades existentes

crates/application/ → agregar use cases PushOperations, PullChanges
                      SIN cambios en use cases existentes

crates/database/    → agregar SqliteSyncRepository (o MySqlSyncRepository)
                      agregar la tabla sync_queue al cliente
                      SIN cambios en repositorios existentes

crates/infrastructure/ → agregar handlers sync.rs
                          agregar GET /api/v1/sync/pull
                          agregar POST /api/v1/sync/push
                          agregar GET /api/v1/sync/ws (Fase 2)

apps/web/           → agregar lib/sync/engine.ts
                      agregar lib/sync/wasm-db.ts
                      agregar components/ui/SyncStatus.svelte

apps/desktop/       → agregar commands/sync.rs
                      configurar auto-sync al reconectar red

apps/mobile/        → igual que desktop
```

---

## Casos especiales

### Eliminación de registros — tombstones

El Soft Delete del proyecto (ADR 0006) ya es la estrategia correcta para sincronización.
`deleted_at` actúa como tombstone — cuando el cliente baja un cambio con `deleted_at != NULL`,
oculta el registro localmente sin eliminarlo del disco. Esto permite recuperar datos
si el delete fue un conflicto.

### Primer sync de un dispositivo nuevo

```typescript
// Al instalar la app o al hacer login por primera vez
async function initialSync(): Promise<void> {
    // Bajar TODOS los datos del usuario (sin filtro de `since`)
    const response = await fetch('/api/v1/sync/pull?since=1970-01-01T00:00:00Z');
    const { changes } = await response.json();
    for (const change of changes) {
        await applyServerChange(change);
    }
}
```

### Autenticación expirada durante sync offline

```typescript
// Si el token expira mientras se está offline:
// Los cambios se siguen guardando en sync_queue con status = 'pending'
// Al reconectar, el refresh token renueva el access token (ADR 0008)
// Y el push/pull se ejecuta normalmente
// El usuario no pierde nada — la cola es durable en SQLite
```

### Datos que NO deben sincronizarse

No todos los datos necesitan sync:

| Dato | ¿Sincronizar? | Motivo |
|------|--------------|--------|
| `users` | ✅ Sí | Datos del perfil del usuario |
| `sessions` | ❌ No | Las sesiones son por dispositivo |
| `audit_logs` | ❌ No (solo servidor) | Insert-only, no editable por cliente |
| `sync_queue` | ❌ No | Solo existe en el cliente |
| `tokens` | ❌ No | Gestión de auth, no datos de usuario |

---

## Alternativas consideradas y por qué no ahora

| Opción | Por qué no ahora |
|--------|-----------------|
| **CRDTs desde el día 1** | 2-3 meses de ingeniería solo para el sync engine — el MVP no lo justifica |
| **PowerSync desde el inicio** | Añade un servicio externo con su propia infraestructura — complejidad innecesaria en MVP |
| **Replicación MySQL master-slave** | Solo sincroniza entre servidores, no hacia los clientes |
| **Firebase Realtime DB** | Vendor lock-in, fuera del stack self-hosted definido |
| **IndexedDB en lugar de SQLite WASM** | Menor compatibilidad con las queries existentes; SQLite WASM da SQL real |

---

## Consecuencias

### ✅ Positivas

- El usuario puede crear, editar y eliminar datos sin internet — experiencia sin interrupciones
- Al volver la red, los cambios se sincronizan automáticamente en segundo plano
- El Outbox Pattern (cola de operaciones) garantiza que ningún cambio se pierde aunque la app se cierre
- Last Write Wins es simple de entender, debuggear y explicar al cliente
- La arquitectura hexagonal hace que todo esto quepa en `crates/database/` y un par de handlers — sin tocar el dominio
- Funciona con SQLite, MySQL y PostgreSQL en el servidor sin cambios en la lógica de sync

### ⚠️ Negativas / Trade-offs

- **LWW puede perder datos en conflictos simultáneos:** si A y B editan el mismo campo al mismo tiempo offline, el cambio del perdedor desaparece sin aviso
  → Mitigación: mostrar `SyncStatus` en la UI que notifica conflictos resueltos
  → Mitigación: guardar la versión "perdedora" en `sync_queue` con status `conflicted` para audit
  → Mitigación: usar Fase 3 (CRDTs) si el dominio lo requiere
- **Latencia de consistencia:** el usuario A puede ver datos desactualizados hasta el próximo pull
  → Mitigación: pull en primer plano al abrir la app, pull automático cada 30s
  → Mitigación: Fase 2 (WebSockets) para tiempo real cuando sea necesario
- **El tamaño de la DB local crece con el tiempo:** SQLite acumula todas las operaciones de sync
  → Mitigación: `CleanupJob` que purga `sync_queue` con status `synced` después de 7 días
- **El primer sync puede ser lento** si hay muchos datos históricos
  → Mitigación: paginar el pull inicial: `?since=...&limit=500&offset=0`

### Decisiones derivadas

- `updated_at` en TODAS las tablas es obligatorio — ya existe en las 6 migraciones base ✓
- Los IDs son generados por el cliente (UUID v4) — el servidor no genera IDs para registros cliente
- El servidor NUNCA confía en timestamps del cliente para seguridad — solo para resolución de conflictos
- `sync_queue` se limpia periódicamente por `CleanupJob` (ADR 0018) — nunca crece sin control
- Fase 2 y 3 se activan solo cuando el MVP web en producción tiene evidencia de la necesidad
