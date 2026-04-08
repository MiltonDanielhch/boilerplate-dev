# ADR 0024 — Local-First: SQLite Wasm + Sync Queue

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado — implementación en Fase 2 |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0004 (SQLite), ADR 0022 (Astro + Svelte 5) |

---

## Contexto

Las aplicaciones cloud-first dependen de una conexión constante. Si el internet falla,
la app muere. Para proyectos que operan en entornos con conectividad intermitente —
como Bolivia — esto es inaceptable.

Necesitamos que:

- **La latencia sea cero** — el usuario no espera a que un servidor responda para ver sus datos
- **Los datos sean del usuario primero** — el dispositivo es la fuente de verdad principal
- **La sincronización sea silenciosa** — el sistema detecta cuando hay internet y sincroniza en segundo plano

---

## Decisión

Adoptar una arquitectura **Local-First** usando **SQLite vía WebAssembly** en el navegador
y **replicación bidireccional** con el servidor de Rust.

### Componentes técnicos

| Capa | Tecnología | Rol |
|------|-----------|-----|
| Persistencia en cliente | **SQLite Wasm + OPFS** | SQLite nativo en el navegador con rendimiento de disco real. |
| Sincronización | **PowerSync / Zero** | Sincronización automática de tablas entre Rust y el Cliente. |
| Resolución de conflictos | Last-Write-Wins por timestamp | Merge automático cuando el usuario vuelve online |
| Intercepción offline | Service Workers (Astro) | Sirven la app completa sin red |

### Flujo de datos

```
Usuario escribe dato
  → Guardado instantáneo en SQLite local (Wasm) — latencia <1ms
  → UI actualizada inmediatamente (Optimistic UI)
  → Background task intenta sincronizar con Axum (Rust)
      ├── Si tiene éxito → dato marcado como "Sincronizado" ✓
      └── Si falla (sin internet) → encolado para reintento automático
```

### Estructura en el frontend

```typescript
// apps/web/src/lib/sync/local-db.ts
import { createDbWorker } from 'wa-sqlite/dist/wa-sqlite-async.mjs';

export class LocalDatabase {
    private worker: Awaited<ReturnType<typeof createDbWorker>>;

    async init() {
        this.worker = await createDbWorker(
            ['wa-sqlite-async.wasm'],
            'SharedArrayBuffer' in globalThis ? 'SharedBufferSource' : 'MemorySource',
        );
        await this.worker.db.exec(`
            CREATE TABLE IF NOT EXISTS sync_queue (
                id         TEXT PRIMARY KEY,
                table_name TEXT NOT NULL,
                operation  TEXT NOT NULL,  -- INSERT | UPDATE | DELETE
                payload    TEXT NOT NULL,  -- JSON
                created_at TEXT NOT NULL,
                synced_at  TEXT            -- NULL = pendiente
            );
        `);
    }

    async enqueueSync(table: string, op: string, payload: unknown) {
        await this.worker.db.exec(
            `INSERT INTO sync_queue (id, table_name, operation, payload, created_at)
             VALUES (?, ?, ?, ?, datetime('now'))`,
            [crypto.randomUUID(), table, op, JSON.stringify(payload)],
        );
    }
}
```

### Service Worker para offline completo

```typescript
// apps/web/src/service-worker.ts
const CACHE    = 'boilerplate-v1';
const PRECACHE = ['/', '/login', '/dashboard'];

self.addEventListener('install', (e: ExtendableEvent) => {
    e.waitUntil(caches.open(CACHE).then(c => c.addAll(PRECACHE)));
});

self.addEventListener('fetch', (e: FetchEvent) => {
    e.respondWith(
        fetch(e.request).catch(() => caches.match(e.request) as Promise<Response>),
    );
});
```

---

## Estrategia de resolución de conflictos

Para el MVP se usa **Last-Write-Wins por timestamp** — simple y suficiente para un solo
usuario o equipos pequeños.

| Escenario | Resolución MVP |
|-----------|---------------|
| Mismo usuario, mismos datos, sin internet | Last-Write-Wins por `updated_at` |
| Dos usuarios editando el mismo registro | El último en sincronizar gana |
| Eliminación vs edición simultánea | La eliminación tiene precedencia |

---

## Alternativas consideradas

| Opción | Motivo de descarte |
|--------|--------------------|
| Solo cloud (sin offline) | Inaceptable en entornos con conectividad intermitente |
| IndexedDB puro | Sin SQL — consultas complejas requieren código tedioso |
| PouchDB + CouchDB | Stack adicional — CouchDB consume RAM que no tenemos en el VPS |
| PWA sin persistencia local | Funciona offline pero sin datos — experiencia degradada |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para industrializar la sincronización y el rendimiento offline:

| Herramienta | Propósito en Local-First |
| :--- | :--- |
| **`PowerSync`** | **Sync Engine:** Elimina la necesidad de programar colas de sincronización manuales. |
| **`OPFS`** | **Rendimiento:** Permite a SQLite usar el sistema de archivos privado del navegador para velocidad máxima. |
| **`Automerge-repo`** | **Conflict Resolution:** Implementa CRDTs para fusionar cambios de múltiples dispositivos sin pérdida de datos. |
| **`TanStack Query`** | **Sync State:** Para mostrar visualmente al usuario si sus datos están "Guardados localmente" o "Sincronizados". |

---

## Consecuencias

### ✅ Positivas

- La app funciona completamente sin internet — los datos están en el dispositivo
- Latencia de UI cercana a cero — todas las operaciones son locales primero
- SQLite en cliente y servidor — misma sintaxis, mismo modelo mental
- La sincronización es transparente para el usuario

### ⚠️ Negativas / Trade-offs

- `wa-sqlite` requiere WebAssembly — no compatible con navegadores muy antiguos
  → Mitigación: los navegadores modernos (Chrome 89+, Firefox 89+, Safari 15+) lo soportan
  → Para navegadores sin Wasm: fallback a modo online puro — la app sigue funcionando
- Los conflictos de sincronización requieren estrategia explícita
  → Last-Write-Wins es simple pero puede perder datos en edición concurrente
  → Mitigación: el campo `updated_at` con timestamp UTC garantiza el orden correcto
  → En Fase 3: evaluar CRDTs si se necesita resolución sin pérdida de datos
- La cola de sincronización puede crecer si el usuario nunca vuelve a estar online
  → Límite de retención de 30 días en `sync_queue` sin sincronizar — limpiado por `CleanupJob` (ADR 0018)

### Decisiones derivadas

- Todos los modelos con soporte offline deben tener campo `updated_at` con timestamp UTC
- La cola de sincronización se limpia automáticamente tras 30 días — via `CleanupJob`
- El indicador de estado de sincronización es obligatorio en la UI — el usuario siempre sabe
- `apps/desktop/` con Tauri (ADR 0030) se beneficia directamente: SQLite nativa en el dispositivo
