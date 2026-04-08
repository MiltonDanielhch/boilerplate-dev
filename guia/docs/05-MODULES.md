# Catálogo de Módulos Implementables

> **Referencia:** Para crear nuevos roadmaps, usar `00-ROADMAP-TEMPLATE.md`
>
> **Convención de numeración:**
> - 01-09: Core (Génesis, Backend, Frontend, Auth, Landing, Infra)
> - 10-49: Módulos de negocio post-MVP (VPS $5 compatibles)
> - 50-59: Fase 2 (NATS + Workers, VPS $10-20)
> - 60-69: Fase 3 (KMP + Mobile Nativo, no VPS $5)
> - 70-79: Futura / Enterprise (no VPS $5)
> - 80-99: Módulos específicos (Admin, etc.)

---

## Módulos Core (01-09) — MVP

| # | Módulo | Estado | Descripción | VPS $5 |
|---|--------|--------|-------------|--------|
| 01 | MASTER | ✅ | Índice de todos los roadmaps | ✅ |
| 02 | GÉNESIS | ✅ | Workspace + tooling inicial | ✅ |
| 03 | BACKEND | ✅ | Dominio + DB + API Axum | ✅ |
| 04 | FRONTEND | ✅ | Astro + Svelte + Dashboard | ✅ |
| 05 | AUTH FULLSTACK | ✅ | Login/Registro back+front | ✅ |
| 06 | LANDING | ✅ | Landing page + leads | ✅ |
| 07 | INFRA | ✅ | Deploy + Caddy + Kamal | ✅ |
| 08 | TAURI DESKTOP | ⏳ | Desktop app (Tauri 2.0) | ✅ |
| 09 | MOBILE | ⏳ | Mobile Tauri + KMP | ✅ Tauri / 🔴 KMP |

---

## Módulos de Negocio (10-49) — Post-MVP VPS $5

Estos módulos se pueden implementar manteniendo VPS $5. No requieren infraestructura adicional.

### Pagos y Facturación (10-19)

| # | Módulo | Descripción | Complejidad | ADRs |
|---|--------|-------------|-------------|------|
| 10 | **Stripe Integration** | Pagos con tarjetas, suscripciones, webhooks | Media | 0006, 0008 |
| 11 | **Mercado Pago** | Pagos LATAM (Argentina, Brasil, México, Chile) | Media | 0006, 0008 |
| 12 | **PayPal Integration** | Pagos internacionales alternativos | Baja | 0006 |
| 13 | **Invoicing** | Facturas PDF, envío por email, historial | Media | 0019 |
| 14 | **Subscriptions** | Suscripciones recurrentes, trials, cancelaciones | Alta | 0006, 0018 |
| 15 | **Usage Billing** | Cobro por uso (API calls, storage, etc.) | Alta | 0016 |
| 16 | **Coupons & Discounts** | Códigos promocionales, descuentos por volumen | Media | 0006 |
| 17 | **Affiliate System** | Referidos, comisiones, tracking | Alta | 0007 |
| 18 | **Invoices Export** | Exportar a contabilidad (QuickBooks, etc.) | Media | — |
| 19 | **Multi-currency** | Soporte USD, EUR, ARS, MXN con conversión | Media | 0002 |

### Comunicaciones (20-29)

| # | Módulo | Descripción | Complejidad | ADRs |
|---|--------|-------------|-------------|------|
| 20 | **Email Marketing** | Campaigns, templates, segmentation | Media | 0019 |
| 21 | **Push Notifications** | FCM (Android) + APNs (iOS) | Media | 0024 |
| 22 | **SMS Notifications** | Twilio / MessageBird para 2FA y alerts | Baja | 0008 |
| 23 | **In-App Messaging** | Chat interno entre usuarios | Alta | 0025 |
| 24 | **Notification Center** | Centro de notificaciones in-app, badges | Media | 0022 |
| 25 | **WebSockets Realtime** | Chat en vivo, notificaciones instantáneas | Alta | 0025, F-007 |
| 26 | **Email Templates CMS** | Editor de templates drag-and-drop | Alta | 0022 |
| 27 | **Scheduled Emails** | Drip campaigns, reminders programados | Media | 0018 |
| 28 | **Webhooks Outbound** | Webhooks configurables por usuario | Media | 0025 |
| 29 | **Slack/Discord Integration** | Notificaciones a canales | Baja | 0028 |

### Usuarios y Social (30-39)

| # | Módulo | Descripción | Complejidad | ADRs |
|---|--------|-------------|-------------|------|
| 30 | **User Profiles** | Perfiles públicos/privados, avatars, bio | Baja | 0006 |
| 31 | **Social Login** | Google, GitHub, Apple OAuth | Media | 0008 |
| 32 | **Teams/Organizations** | Multi-tenancy básico (equipos) | Alta | 0006 |
| 33 | **Follow System** | Seguir usuarios, timeline | Media | 0004 |
| 34 | **User Invitations** | Invitar a equipo, referral links | Media | 0006 |
| 35 | **Activity Feed** | Feed de actividad propia y de seguidos | Media | 0004 |
| 36 | **Gamification** | Badges, points, leaderboards | Media | 0004 |
| 37 | **User Reviews** | Reviews/ratings entre usuarios | Media | 0006 |
| 38 | **Onboarding Flow** | Tutorial interactivo, checklists | Baja | 0022 |
| 39 | **User Preferences** | Settings avanzados de usuario | Baja | 0002 |

### Contenido y CMS (40-49)

| # | Módulo | Descripción | Complejidad | ADRs |
|---|--------|-------------|-------------|------|
| 40 | **Blog System** | Posts, categorías, tags, comments | Media | 0004 |
| 41 | **File Uploads** | Upload avanzado con S3/Tigris | Media | 0020 |
| 42 | **Image Processing** | Thumbnails, watermarks, resizing | Media | 0020 |
| 43 | **Document Management** | PDFs, versioning, sharing | Alta | 0020 |
| 44 | **Search Basic** | Full-text search con SQLite FTS5 | Media | 0004 |
| 45 | **Comments System** | Comentarios anidados, moderación | Media | 0006 |
| 46 | **Content Moderation** | Flagging, auto-moderation, reports | Alta | 0006 |
| 47 | **SEO Tools** | Sitemap dinámico, meta tags, OpenGraph | Baja | 0029 |
| 48 | **Multi-language CMS** | Contenido en múltiples idiomas | Media | 0023 |
| 49 | **Media Library** | Galería de imágenes/videos, organización | Media | 0020 |

---

## Módulos Fase 2 (50-59) — NATS + Workers

Requieren NATS JetStream. VPS $10-20 (no $5). Ver `50-ROADMAP-FASE2.md`.

| # | Módulo | Descripción | Complejidad |
|---|--------|-------------|-------------|
| 50 | **NATS Core** | Infraestructura + workers desacoplados | Alta |
| 51 | **Background Jobs** | Jobs pesados sin bloquear API | Media |
| 52 | **Event System** | Eventos entre módulos async | Media |
| 53 | **Real-time Sync** | Sincronización multi-device | Alta |
| 54 | **Queue System** | Colas de trabajo con prioridad | Media |
| 55 | **Scheduler** | Cron jobs distribuidos | Media |
| 56 | **Import/Export Bulk** | Importar/exportar datos masivos | Alta |
| 57 | **Data Pipeline** | ETL para analytics | Alta |
| 58 | **Webhook Processing** | Webhooks async, retries | Media |
| 59 | **Notification Queue** | Notificaciones con prioridad | Media |

---

## Módulos Fase 3 (60-69) — KMP + Mobile Nativo

Requieren equipo mobile nativo. NO son VPS $5. Ver `60-ROADMAP-FASE3.md`.

| # | Módulo | Descripción | Complejidad |
|---|--------|-------------|-------------|
| 60 | **KMP Core** | UniFFI bindings, domain compartido | Alta |
| 61 | **Android App** | Jetpack Compose, Play Store | Alta |
| 62 | **iOS App** | SwiftUI, App Store | Alta |
| 63 | **Offline-First** | SQLite local, sync cuando online | Alta |
| 64 | **Biometric Auth** | Face ID / Fingerprint | Media |
| 65 | **Push Native** | FCM + APNs nativo | Media |
| 66 | **Mobile Analytics** | Firebase Analytics, Crashlytics | Baja |
| 67 | **Deep Links** | Universal links, app links | Media |
| 68 | **Mobile Payments** | In-app purchases, StoreKit | Alta |
| 69 | **AR Features** | Realidad aumentada (ARKit/ARCore) | Muy Alta |

---

## Módulos Futura / Enterprise (70-79)

Escalamiento masivo. NO son VPS $5. Ver `70-ROADMAP-FUTURA.md`.

| # | Módulo | Descripción | VPS |
|---|--------|-------------|-----|
| 70 | **OTel Stack** | Loki + Tempo + Grafana | $40+ |
| 71 | **Redis Cluster** | Caché distribuida L1+L2 | $20+ |
| 72 | **CDN Global** | Cloudflare / CloudFront | $0-20 |
| 73 | **PostgreSQL** | DB con réplicas | $40+ |
| 74 | **SurrealDB** | Multi-modelo (grafo + docs) | $40+ |
| 75 | **Kubernetes** | Orquestación K3s/managed | $100+ |
| 76 | **Elasticsearch** | Búsqueda avanzada | $100+ |
| 77 | **Advanced Analytics** | ClickHouse, BigQuery | $100+ |
| 78 | **Multi-region** | Deploy en 3+ regiones | $200+ |
| 79 | **Compliance** | GDPR, SOC2, HIPAA | — |

---

## Módulos Específicos (80-99)

| # | Módulo | Descripción | VPS $5 |
|---|--------|-------------|--------|
| 80 | **Admin Dashboard** | Gestión users, analytics, CMS | ✅ |
| 81 | **API Rate Limits** | Tiers: Free, Pro, Enterprise | ✅ |
| 82 | **Developer Portal** | Docs, API keys, playground | ✅ |
| 83 | **Partner API** | API para integraciones externas | ✅ |
| 84 | **White Label** | Custom branding, domains | ✅ |
| 85 | **Advanced RBAC** | Permisos granulares, ABAC | ✅ |
| 86 | **Audit System** | Logs de auditoría completo | ✅ |
| 87 | **Data Export** | GDPR right to data portability | ✅ |
| 88 | **A/B Testing** | Experiments, feature flags | ✅ |
| 89 | **AI Features** | OpenAI/Anthropic integration | ⚠️ Costo API |
| 90 | **ML Models** | Modelos entrenados propios | 🔴 No |
| 91 | **Video Streaming** | WebRTC, live streaming | 🔴 No |
| 92 | **Voice/SIP** | Llamadas VoIP, telefonía | 🔴 No |
| 93 | **IoT Integration** | MQTT, device management | 🟡 Medium |
| 94 | **Blockchain** | Web3, wallets, smart contracts | 🔴 No |
| 95 | **Fraud Detection** | ML para detección de fraude | 🔴 No |
| 96 | **CDN Uploads** | Uploads directos a S3/CloudFront | ✅ |
| 97 | **Serverless Functions** | Lambda/Edge functions | ⚠️ Vendor |
| 98 | **GraphQL API** | GraphQL además de REST | ✅ |
| 99 | **gRPC Services** | gRPC interno (si Fase 2) | 🟡 NATS |

---

## Matriz de Decisión — ¿Qué módulo implementar?

```
¿Necesitas monetización?
    Sí → 10-19 (Pagos)
    No ↓
¿Necesitas comunicación con usuarios?
    Sí → 20-29 (Comunicaciones)
    No ↓
¿Necesitas features sociales?
    Sí → 30-39 (Usuarios/Social)
    No ↓
¿Necesitas CMS avanzado?
    Sí → 40-49 (Contenido)
    No ↓
¿Necesitas workers async?
    Sí → 50-59 (Fase 2)
    No ↓
¿Necesitas mobile nativo 120Hz?
    Sí → 60-69 (Fase 3)
    No ↓
¿Tienes >100k users?
    Sí → 70-79 (Futura)
    No ↓
Todo listo → Mantén VPS $5, enfócate en core product
```

---

## Prioridad Sugerida para Startups

### Fase MVP (meses 1-6)
1. ✅ Core (01-09) — MVP básico
2. 10 Stripe — Monetización
3. 20 Email Marketing — Retención
4. 30 User Profiles — Engagement
5. 80 Admin Dashboard — Operaciones

### Growth (meses 6-12)
6. 50 NATS Core — Foundation Fase 2
7. 51 Background Jobs — Escalar procesamiento
8. 14 Subscriptions — Modelo recurrente
9. 21 Push Notifications — Retención mobile
10. 32 Teams/Organizations — B2B expansion

### Scale (año 2+)
11. 60 KMP Core — Si mobile nativo es crítico
12. 70+ Futura — Según métricas reales

---

## Cómo Crear un Nuevo Roadmap

1. **Elegir número:** Siguiente disponible en la categoría
2. **Copiar template:** `cp 00-ROADMAP-TEMPLATE.md XX-ROADMAP-MODULO.md`
3. **Rellenar campos:** Según el módulo específico
4. **Actualizar MASTER:** Añadir a la tabla de "Roadmaps Post-MVP"
5. **Actualizar este doc:** Marcar módulo como "📋 Roadmap creado"

---

**Última actualización:** 2026-04-04
**Próxima revisión:** Cuando se creen roadmaps 14-20
**Responsable:** Staff Engineer + Product Owner
