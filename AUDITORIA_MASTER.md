# 🛠️ Auditoría de Software — Lab 3030

> Generado: `2026-04-10 16:03`

## Resumen

| Métrica | Valor |
| :--- | :--- |
| **Proyecto** | `boilerplate4` |
| **Líneas de Código (Netas)** | 22317 LoC |
| **Peso Total del Proyecto** | 1.27MB |
| **Timestamp** | 2026-04-10 16:03 |
| **Estado** | Activa |

## Breakdown por Capa

| Capa / Archivo | LoC | Peso | % LoC |
| :--- | ---: | ---: | ---: |
| `guia` | 18713 | 852.61KB | 83.9% ████████████████ |
| `crates` | 2245 | 80.35KB | 10.1% ██ |
| `apps` | 671 | 23.40KB | 3.0%  |
| `data` | 258 | 165.31KB | 1.2%  |
| `audit.py` | 146 | 5.43KB | 0.7%  |
| `Cargo.toml` | 87 | 5.49KB | 0.4%  |
| `deny.toml` | 74 | 3.86KB | 0.3%  |
| `README.md` | 41 | 2.02KB | 0.2%  |
| `lefthook.yml` | 35 | 850.00B | 0.2%  |
| `mise.toml` | 23 | 860.00B | 0.1%  |
| `rust-toolchain.toml` | 13 | 617.00B | 0.1%  |
| `pnpm-workspace.yaml` | 11 | 353.00B | 0.0%  |
| `.env.example` | 0 | 4.90KB | 0.0%  |
| `.github` | 0 | 0.00B | 0.0%  |
| `.sqlx` | 0 | 0.00B | 0.0%  |
| `Cargo.lock` | 0 | 145.38KB | 0.0%  |
| `check_output.txt` | 0 | 866.00B | 0.0%  |
| `infra` | 0 | 0.00B | 0.0%  |
| `justfile` | 0 | 7.66KB | 0.0%  |
| `proto` | 0 | 0.00B | 0.0%  |
| **TOTAL** | **22317** | **1.27MB** | 100% |

## Mapa de Arquitectura

```text
boilerplate4/
├── .env.example (0 LoC | 4.90KB)
├── .github/ [0.00B]
│   └── workflows/ [0.00B]
├── .sqlx/ [0.00B]
├── Cargo.lock (0 LoC | 145.38KB)
├── Cargo.toml (87 LoC | 5.49KB)
├── README.md (41 LoC | 2.02KB)
├── apps/ [23.40KB]
│   ├── api/ [21.87KB]
│   │   ├── .gitkeep (0 LoC | 0.00B)
│   │   ├── Cargo.toml (44 LoC | 2.49KB)
│   │   └── src/ [19.38KB]
│   │       ├── .gitkeep (0 LoC | 0.00B)
│   │       ├── error.rs (161 LoC | 5.05KB)
│   │       ├── handlers/ [5.79KB]
│   │       │   ├── health.rs (30 LoC | 956.00B)
│   │       │   ├── leads.rs (33 LoC | 1.02KB)
│   │       │   ├── mod.rs (6 LoC | 140.00B)
│   │       │   └── users.rs (112 LoC | 3.70KB)
│   │       ├── main.rs (62 LoC | 2.20KB)
│   │       ├── middleware/ [1.78KB]
│   │       │   ├── mod.rs (5 LoC | 128.00B)
│   │       │   ├── request_id.rs (32 LoC | 891.00B)
│   │       │   └── trace.rs (27 LoC | 805.00B)
│   │       ├── router.rs (48 LoC | 1.69KB)
│   │       ├── setup.rs (53 LoC | 2.15KB)
│   │       └── state.rs (24 LoC | 741.00B)
│   ├── cli/ [1.53KB]
│   │   ├── .gitkeep (0 LoC | 0.00B)
│   │   ├── Cargo.toml (25 LoC | 1.25KB)
│   │   ├── src/ [287.00B]
│   │   │   ├── .gitkeep (0 LoC | 0.00B)
│   │   │   ├── commands/ [0.00B]
│   │   │   ├── main.rs (9 LoC | 287.00B)
│   │   │   └── scaffold/ [0.00B]
│   │   └── templates/ [0.00B]
│   ├── desktop/ [0.00B]
│   │   ├── src/ [0.00B]
│   │   └── src-tauri/ [0.00B]
│   ├── mailer/ [0.00B]
│   │   ├── .gitkeep (0 LoC | 0.00B)
│   │   └── src/ [0.00B]
│   │       ├── .gitkeep (0 LoC | 0.00B)
│   │       ├── partials/ [0.00B]
│   │       └── templates/ [0.00B]
│   └── web/ [0.00B]
│       ├── public/ [0.00B]
│       └── src/ [0.00B]
│           ├── components/ [0.00B]
│           ├── layouts/ [0.00B]
│           ├── lib/ [0.00B]
│           ├── pages/ [0.00B]
│           ├── stores/ [0.00B]
│           └── styles/ [0.00B]
├── audit.py (146 LoC | 5.43KB)
├── check_output.txt (0 LoC | 866.00B)
├── crates/ [80.35KB]
│   ├── application/ [16.15KB]
│   │   ├── .gitkeep (0 LoC | 0.00B)
│   │   ├── Cargo.toml (19 LoC | 1.08KB)
│   │   └── src/ [15.07KB]
│   │       ├── auth/ [6.13KB]
│   │       │   ├── login.rs (58 LoC | 1.71KB)
│   │       │   ├── logout.rs (24 LoC | 824.00B)
│   │       │   ├── mod.rs (14 LoC | 374.00B)
│   │       │   ├── refresh.rs (33 LoC | 1.18KB)
│   │       │   └── register.rs (59 LoC | 2.06KB)
│   │       ├── leads/ [2.38KB]
│   │       │   ├── capture_lead.rs (63 LoC | 2.15KB)
│   │       │   └── mod.rs (8 LoC | 235.00B)
│   │       ├── lib.rs (10 LoC | 257.00B)
│   │       ├── use_cases.rs (48 LoC | 1.53KB)
│   │       └── users/ [4.79KB]
│   │           ├── get_user.rs (25 LoC | 846.00B)
│   │           ├── list_users.rs (35 LoC | 944.00B)
│   │           ├── mod.rs (14 LoC | 468.00B)
│   │           ├── soft_delete_user.rs (37 LoC | 1.28KB)
│   │           └── update_user.rs (46 LoC | 1.31KB)
│   ├── auth/ [2.01KB]
│   │   ├── .gitkeep (0 LoC | 0.00B)
│   │   ├── Cargo.toml (23 LoC | 1.18KB)
│   │   └── src/ [849.00B]
│   │       ├── lib.rs (8 LoC | 236.00B)
│   │       ├── password.rs (12 LoC | 364.00B)
│   │       └── token.rs (9 LoC | 249.00B)
│   ├── database/ [14.76KB]
│   │   ├── .gitkeep (0 LoC | 0.00B)
│   │   ├── Cargo.toml (24 LoC | 1.58KB)
│   │   └── src/ [13.18KB]
│   │       ├── lib.rs (8 LoC | 232.00B)
│   │       ├── models/ [1.12KB]
│   │       │   ├── mod.rs (7 LoC | 248.00B)
│   │       │   └── user_row.rs (27 LoC | 896.00B)
│   │       ├── pool.rs (66 LoC | 2.40KB)
│   │       └── repositories/ [9.44KB]
│   │           ├── mod.rs (9 LoC | 332.00B)
│   │           └── sqlite_user_repository.rs (255 LoC | 9.11KB)
│   ├── domain/ [37.93KB]
│   │   ├── .gitkeep (0 LoC | 0.00B)
│   │   ├── Cargo.toml (21 LoC | 896.00B)
│   │   └── src/ [37.05KB]
│   │       ├── entities/ [11.00KB]
│   │       │   ├── audit_log.rs (66 LoC | 1.91KB)
│   │       │   ├── lead.rs (55 LoC | 1.60KB)
│   │       │   ├── mod.rs (17 LoC | 447.00B)
│   │       │   ├── role.rs (49 LoC | 1.53KB)
│   │       │   ├── session.rs (61 LoC | 1.92KB)
│   │       │   └── user.rs (111 LoC | 3.59KB)
│   │       ├── errors.rs (69 LoC | 2.95KB)
│   │       ├── lib.rs (13 LoC | 579.00B)
│   │       ├── ports/ [10.03KB]
│   │       │   ├── audit_repository.rs (31 LoC | 1.18KB)
│   │       │   ├── lead_repository.rs (21 LoC | 1.03KB)
│   │       │   ├── mailer.rs (52 LoC | 1.50KB)
│   │       │   ├── mod.rs (21 LoC | 785.00B)
│   │       │   ├── session_repository.rs (26 LoC | 1.46KB)
│   │       │   ├── storage_repository.rs (34 LoC | 1.34KB)
│   │       │   ├── token_repository.rs (28 LoC | 1.38KB)
│   │       │   └── user_repository.rs (25 LoC | 1.38KB)
│   │       └── value_objects/ [12.51KB]
│   │           ├── email.rs (90 LoC | 2.99KB)
│   │           ├── mod.rs (15 LoC | 472.00B)
│   │           ├── password_hash.rs (77 LoC | 2.77KB)
│   │           ├── permission.rs (119 LoC | 4.45KB)
│   │           └── user_id.rs (66 LoC | 1.84KB)
│   ├── events/ [2.00KB]
│   │   ├── Cargo.toml (20 LoC | 1.11KB)
│   │   └── src/ [910.00B]
│   │       ├── lib.rs (9 LoC | 329.00B)
│   │       ├── publisher.rs (12 LoC | 289.00B)
│   │       └── subscriber.rs (12 LoC | 292.00B)
│   ├── infrastructure/ [3.66KB]
│   │   ├── .gitkeep (0 LoC | 0.00B)
│   │   ├── Cargo.toml (44 LoC | 2.69KB)
│   │   └── src/ [999.00B]
│   │       ├── config.rs (11 LoC | 235.00B)
│   │       ├── http.rs (9 LoC | 207.00B)
│   │       ├── lib.rs (9 LoC | 285.00B)
│   │       └── router.rs (10 LoC | 272.00B)
│   ├── mailer/ [1.86KB]
│   │   ├── .gitkeep (0 LoC | 0.00B)
│   │   ├── Cargo.toml (18 LoC | 1.04KB)
│   │   └── src/ [842.00B]
│   │       ├── lib.rs (7 LoC | 195.00B)
│   │       ├── ports.rs (8 LoC | 253.00B)
│   │       └── resend_adapter.rs (13 LoC | 394.00B)
│   └── storage/ [1.98KB]
│       ├── .gitkeep (0 LoC | 0.00B)
│       ├── Cargo.toml (19 LoC | 1.05KB)
│       └── src/ [946.00B]
│           ├── lib.rs (7 LoC | 202.00B)
│           ├── ports.rs (12 LoC | 321.00B)
│           └── s3_adapter.rs (17 LoC | 423.00B)
├── data/ [165.31KB]
│   ├── boilerplate.db (0 LoC | 152.00KB)
│   ├── migrations/ [13.31KB]
│   │   ├── 20260305135148_create_users_table.sql (50 LoC | 2.41KB)
│   │   ├── 20260305135149_create_rbac.sql (58 LoC | 2.85KB)
│   │   ├── 20260305135150_create_tokens.sql (31 LoC | 1.51KB)
│   │   ├── 20260305135151_create_audit_logs.sql (33 LoC | 1.75KB)
│   │   ├── 20260305135152_seed_system_data.sql (45 LoC | 2.69KB)
│   │   └── 20260305135153_create_sessions.sql (41 LoC | 2.10KB)
│   └── seeds/ [0.00B]
├── deny.toml (74 LoC | 3.86KB)
├── guia/ [852.61KB]
│   ├── PROMPT_MAESTRO.md (313 LoC | 15.43KB)
│   ├── adr/ [323.67KB]
│   │   ├── ADR-0001-arquitectura-hexagonal.md (172 LoC | 10.55KB)
│   │   ├── ADR-0002-configuracion-tipeada-secretos.md (169 LoC | 7.68KB)
│   │   ├── ADR-0003-stack-backend-rust-axum.md (146 LoC | 6.96KB)
│   │   ├── ADR-0004-persistencia-sqlite-litestream.md (149 LoC | 7.36KB)
│   │   ├── ADR-0005-migraciones-seeding.md (155 LoC | 7.02KB)
│   │   ├── ADR-0006-rbac-sessions-audit.md (276 LoC | 12.62KB)
│   │   ├── ADR-0007-manejo-errores.md (194 LoC | 8.73KB)
│   │   ├── ADR-0008-seguridad-auth-paseto.md (176 LoC | 8.33KB)
│   │   ├── ADR-0009-rate-limiting.md (141 LoC | 6.14KB)
│   │   ├── ADR-0010-testing-calidad.md (322 LoC | 12.74KB)
│   │   ├── ADR-0011-estandares-desarrollo.md (95 LoC | 4.85KB)
│   │   ├── ADR-0012-herramientas-desarrollo.md (190 LoC | 8.21KB)
│   │   ├── ADR-0013-build-externo-binarios.md (125 LoC | 6.19KB)
│   │   ├── ADR-0014-infraestructura-deploy.md (230 LoC | 8.93KB)
│   │   ├── ADR-0015-monitoreo-tareas-criticas.md (112 LoC | 5.61KB)
│   │   ├── ADR-0016-observabilidad-telemetria.md (144 LoC | 7.11KB)
│   │   ├── ADR-0017-cache-moka-decorator.md (176 LoC | 8.54KB)
│   │   ├── ADR-0018-procesamiento-asincrono-apalis.md (192 LoC | 8.23KB)
│   │   ├── ADR-0019-mailer-resend.md (188 LoC | 7.46KB)
│   │   ├── ADR-0020-almacenamiento-tigris.md (159 LoC | 6.78KB)
│   │   ├── ADR-0021-documentacion-openapi-utoipa.md (166 LoC | 7.61KB)
│   │   ├── ADR-0022-frontend-astro-svelte.md (233 LoC | 9.96KB)
│   │   ├── ADR-0023-i18n-adaptacion-regional.md (135 LoC | 6.48KB)
│   │   ├── ADR-0024-filosofia-local-first.md (129 LoC | 6.61KB)
│   │   ├── ADR-0025-eventos-nats-jetstream.md (123 LoC | 5.98KB)
│   │   ├── ADR-0026-mensajeria-nats-publisher.md (131 LoC | 5.25KB)
│   │   ├── ADR-0027-connectrpc-protobuf.md (120 LoC | 5.92KB)
│   │   ├── ADR-0028-sintonia-cli.md (188 LoC | 8.48KB)
│   │   ├── ADR-0029-landing-page-leads.md (244 LoC | 9.51KB)
│   │   ├── ADR-0030-multiplataforma-tridente.md (182 LoC | 8.14KB)
│   │   ├── ADR-0031-estrategia-escalamiento.md (177 LoC | 8.24KB)
│   │   └── futura/ [81.46KB]
│   │       ├── ADR-F001-surrealdb-rocksdb.md (46 LoC | 2.28KB)
│   │       ├── ADR-F002-postgresql-escala-horizontal.md (89 LoC | 3.16KB)
│   │       ├── ADR-F003-kotlin-multiplatform-uniffi.md (318 LoC | 11.04KB)
│   │       ├── ADR-F004-otel-stack-completo.md (376 LoC | 12.72KB)
│   │       ├── ADR-F005-kubernetes-orquestacion.md (391 LoC | 14.28KB)
│   │       ├── ADR-F006-cdn-global.md (198 LoC | 7.42KB)
│   │       ├── ADR-F007-redis-cluster.md (409 LoC | 14.89KB)
│   │       └── ADR-F008-elasticsearch-busqueda.md (437 LoC | 15.67KB)
│   ├── docs/ [202.80KB]
│   │   ├── 00-GUIA-USO.md (182 LoC | 9.36KB)
│   │   ├── 01-ARCHITECTURE.md (255 LoC | 11.15KB)
│   │   ├── 02-STACK.md (436 LoC | 17.46KB)
│   │   ├── 03-STRUCTURE.md (452 LoC | 19.17KB)
│   │   ├── 04-VERIFICATION.md (725 LoC | 30.10KB)
│   │   ├── 05-MODULES.md (201 LoC | 10.50KB)
│   │   ├── ADR-VPS-5-PILARES.md (356 LoC | 15.47KB)
│   │   ├── BRUJULA-COMPLETA.md (244 LoC | 14.75KB)
│   │   ├── DASHBOARD-DISENO.md (292 LoC | 11.10KB)
│   │   ├── INICIO.md (1450 LoC | 50.50KB)
│   │   └── SINTONIA-CLI.md (332 LoC | 13.23KB)
│   └── roadmap/ [310.71KB]
│       ├── 00-ROADMAP-TEMPLATE.md (146 LoC | 4.43KB)
│       ├── 01-ROADMAP-MASTER.md (247 LoC | 11.02KB)
│       ├── 02-ROADMAP-GENESIS.md (421 LoC | 24.42KB)
│       ├── 03-ROADMAP-BACKEND.md (759 LoC | 41.13KB)
│       ├── 04-ROADMAP-FRONTEND.md (470 LoC | 27.27KB)
│       ├── 05-ROADMAP-AUTH-FULLSTACK.md (511 LoC | 29.13KB)
│       ├── 06-ROADMAP-LANDING.md (510 LoC | 27.63KB)
│       ├── 07-ROADMAP-INFRA.md (462 LoC | 25.34KB)
│       ├── 08-ROADMAP-TAURI-DESKTOP.md (446 LoC | 26.03KB)
│       ├── 09-ROADMAP-MOBILE.md (352 LoC | 20.23KB)
│       ├── 50-ROADMAP-FASE2.md (299 LoC | 15.77KB)
│       ├── 60-ROADMAP-FASE3.md (325 LoC | 17.09KB)
│       ├── 70-ROADMAP-FUTURA.md (444 LoC | 20.73KB)
│       └── 80-ROADMAP-ADMIN.md (480 LoC | 20.49KB)
├── infra/ [0.00B]
│   ├── caddy/ [0.00B]
│   ├── docker/ [0.00B]
│   ├── kamal/ [0.00B]
│   └── litestream/ [0.00B]
├── justfile (0 LoC | 7.66KB)
├── lefthook.yml (35 LoC | 850.00B)
├── mise.toml (23 LoC | 860.00B)
├── pnpm-workspace.yaml (11 LoC | 353.00B)
├── proto/ [0.00B]
│   └── user/ [0.00B]
│       └── v1/ [0.00B]
└── rust-toolchain.toml (13 LoC | 617.00B)
```
