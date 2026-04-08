# ADR 0027 — Comunicación: ConnectRPC + buf generate

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado — implementación en Fase 2 |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0003 (Axum), ADR 0022 (Frontend Astro+Svelte), ADR 0030 (Tauri/Móvil) |

---

## Contexto

El boilerplate apunta a ser un ecosistema multiplataforma — Web, Desktop con Tauri y App Móvil.
REST sin contratos tipados obliga a escribir DTOs a mano o duplicar código entre plataformas.
Necesitamos un contrato de comunicación que:

- Genere tipos para Rust, TypeScript, Kotlin y Swift desde una sola definición `.proto`
- Sea binario y ligero para conexiones móviles inestables
- Funcione nativamente en navegadores sin necesidad de proxies complejos como Envoy

---

## Decisión

Usar **ConnectRPC** con **Protocol Buffers (proto3)** como estándar de comunicación para
todas las interfaces del proyecto en Fase 2.

En Fase 1 (MVP), el frontend usa tipos generados manualmente compatibles con los endpoints REST.
En Fase 2, `buf generate` reemplaza esos tipos por código generado desde `proto/`.

### El contrato universal

```protobuf
// proto/user/v1/user.proto
syntax = "proto3";
package user.v1;

service UserService {
    rpc GetUser    (GetUserRequest)    returns (GetUserResponse);
    rpc CreateUser (CreateUserRequest) returns (CreateUserResponse);
    rpc ListUsers  (ListUsersRequest)  returns (ListUsersResponse);
}

message User {
    string id              = 1;
    string email           = 2;
    bool   email_verified  = 3;
    string created_at      = 4;
}

message GetUserRequest     { string id    = 1; }
message GetUserResponse    { User   user  = 1; }
message CreateUserRequest  { string email = 1; string password = 2; }
message CreateUserResponse { User   user  = 1; }
message ListUsersRequest   { int32 page = 1; int32 per_page = 2; }
message ListUsersResponse  { repeated User users = 1; int32 total = 2; }
```

### Implementación en el backend (Rust + Axum)

ConnectRPC se monta directamente sobre el Router de Axum — sin servidor gRPC separado:

```rust
// apps/api/src/main.rs
let app = Router::new()
    .nest("/api/v1", rest_router())     // Endpoints REST existentes (Fase 1)
    .nest("/rpc",    connectrpc_router()) // ConnectRPC sobre HTTP/1.1 (Fase 2)
    .layer(auth_layer);
```

### Generación de código multi-plataforma con `buf`

```bash
# buf.gen.yaml configura los targets
buf generate
```

| Plataforma | Generador | Resultado |
|-----------|-----------|-----------|
| Web (Svelte / Astro) | `@connectrpc/connect` | Funciones TypeScript con autocompletado |
| Desktop (Tauri) | `prost` | Structs nativas de Rust |
| Android | `protoc-gen-kotlin` | Clases Kotlin con tipos garantizados |
| iOS | `protoc-gen-swift` | Structs Swift sin capas intermedias |

### Flujo de trabajo en Fase 2

```
1. Editar el archivo .proto con el nuevo campo o servicio
2. buf generate
3. Todos los clientes actualizados automáticamente
4. El compilador marca los usos rotos — error en compile-time, no en runtime
```

### `just types` en Fase 2

```makefile
# justfile — en Fase 2 buf generate reemplaza el generador manual
types:
    buf generate
    @echo "✅ Tipos generados en apps/web/src/lib/types/api.ts"

types-check:
    buf generate
    git diff --exit-code apps/web/src/lib/types/api.ts
```

---

## Alternativas consideradas

| Opción | Motivo de descarte |
|--------|--------------------|
| REST sin contratos tipados | Obliga a escribir y mantener DTOs TypeScript a mano — se desincroniza con el backend |
| gRPC tradicional | Requiere Envoy Proxy para funcionar en navegadores; ConnectRPC lo resuelve nativamente |
| OpenAPI / Swagger | Muy verboso y propenso a errores en los generadores de código |
| GraphQL | Overhead de resolver y N+1 queries sin beneficio claro en este stack |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para industrializar la comunicación y asegurar contratos robustos:

| Herramienta | Propósito en la Comunicación |
| :--- | :--- |
| **`connect-query`** | **Svelte Integration:** Conecta ConnectRPC con TanStack Query para un manejo de estado ultra-fluido. |
| **`protovalidate`** | **Validación Universal:** Define reglas de validación en el `.proto` que se ejecutan en todas las plataformas. |
| **`axum-connect`** | **Rust Integration:** Facilita el montaje de servicios Connect como rutas nativas de Axum. |
| **`buf studio`** | **Debugging UI:** Interfaz visual para testear endpoints RPC sin escribir código de cliente. |

---

## Consecuencias

### ✅ Positivas

- Un solo `.proto` genera código para todas las plataformas — sin duplicación
- Preparado para móvil desde el día uno — el día que se lance la app, la comunicación ya está escrita
- Eficiencia binaria — menor uso de datos y batería en dispositivos móviles
- Contrato estricto — imposible que el frontend pida un campo que el backend no envía

### ⚠️ Negativas / Trade-offs

- Curva de aprendizaje de la sintaxis de Protocol Buffers
  → Mitigación: `buf lint` valida la sintaxis automáticamente; hay un `.proto` de ejemplo
    en `proto/user/v1/user.proto` que sirve como plantilla
  → La sintaxis proto3 es más simple que SQL o TypeScript — se aprende en ~1 hora
- Requiere instalar `buf` en el entorno de desarrollo
  → `just setup` instala `buf` automáticamente junto con las demás herramientas (ADR 0012)
- Los mensajes binarios no son legibles en la pestaña Network del browser sin plugins
  → En Fase 1 los endpoints REST siguen funcionando para debugging manual
  → En Fase 2: usar `buf curl` o la extensión de Chrome para ConnectRPC

### Decisiones derivadas

- `buf` gestiona todas las dependencias de proto y los lints
- Todos los archivos `.proto` siguen el estilo de Google API Design Guide
- El servidor HTTP del backend es el mismo de ADR 0003 — no hay proceso adicional
- `apps/web/src/lib/types/api.ts` es generado por `buf generate` — nunca editado manualmente
