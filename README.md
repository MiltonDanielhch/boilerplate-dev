# boilerplate

> Arquitectura hexagonal en Rust + Astro/Svelte — MVP web lista para producción.

---

## 🏛️ Arquitectura

**Monolito modular** con fronteras físicas por `Cargo.toml`.

```
┌─────────────────────────────────────────────────────────────┐
│  apps/                                                      │
│  ├── api/          Axum — API REST                           │
│  ├── web/          Astro SSR + Svelte 5                    │
│  └── mailer/       Resend — envío de emails                │
│                                                             │
│  crates/                                                    │
│  ├── domain/       Núcleo puro (sin dependencias externas) │
│  ├── application/  Casos de uso                            │
│  ├── infrastructure/ HTTP, config, router                    │
│  ├── database/     SQLx + Moka (único con SQL)              │
│  ├── auth/         Argon2id + PASETO v4                     │
│  ├── mailer/       Puerto Mailer + Resend                   │
│  └── storage/      Puerto Storage + Tigris                  │
└─────────────────────────────────────────────────────────────┘
```

**Stack:** Rust 1.94 · Axum 0.8 · SQLx · SQLite · PASETO · Astro · Svelte 5

---

## 🚀 Inicio Rápido

```bash
# 1. Instalar toolchain
mise install

# 2. Setup completo
just setup

# 3. Desarrollo
just dev
```

---

## 📚 Documentación

- [`guia/docs/00-GUIA-USO.md`](guia/docs/00-GUIA-USO.md) — Cómo usar esta guía
- [`guia/roadmap/02-ROADMAP-GENESIS.md`](guia/roadmap/02-ROADMAP-GENESIS.md) — Fase actual
- [`guia/adr/`](guia/adr/) — Decisiones arquitectónicas

---

## 📜 Licencia

MIT © 2026 Laboratorio 3030
