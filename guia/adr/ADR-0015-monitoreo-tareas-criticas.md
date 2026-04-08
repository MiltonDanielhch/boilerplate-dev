# ADR 0015 — Monitoreo: Healthchecks.io + Patrón Dead Man's Switch

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0004 (Litestream backups), ADR 0018 (Apalis jobs), ADR 0014 (Caddy TLS) |

---

## Contexto

En un VPS de $5 sin equipo de monitoreo 24/7, necesitamos saber si una tarea crítica dejó
de correr — antes de que el problema se vuelva un desastre.

El monitoreo tradicional (un agente que pregunta "¿estás vivo?") tiene un punto ciego fatal:
si el VPS se apaga o pierde internet, el agente interno tampoco puede avisar.

Necesitamos:

- **Alertas si el VPS muere completamente** — no solo si un proceso falla internamente
- **Costo $0** — capa gratuita suficiente para el inicio del proyecto
- **Impacto en RAM/CPU: 0** — no instalar nada en el servidor

---

## Decisión

Usar **Healthchecks.io** con el patrón de **"interruptor del hombre muerto"**: el servidor
avisa que completó una tarea; si el aviso no llega, Healthchecks asume que algo falló y alerta.

### El patrón: ping al completar, silencio = alerta

```bash
# Patrón general — ping SOLO si la tarea tuvo éxito
./tarea_critica.sh && curl -fsS -m 10 --retry 5 https://hc-ping.com/{uuid}

# Si la tarea falla → no hay ping → Healthchecks alerta por email/Telegram/WhatsApp
```

### Tarea 1 — Backup de Litestream (ADR 0004)

```bash
# Verificar que el último snapshot es del día de hoy
# Correr como cron cada hora via CleanupJob (ADR 0018)
litestream snapshots s3://bucket/boilerplate/db | grep -q "$(date +%Y-%m-%d)" \
  && curl -fsS -m 10 https://hc-ping.com/${HC_LITESTREAM_UUID}
```

### Tarea 2 — Worker de Apalis (ADR 0018)

```rust
// apps/api/src/jobs/worker.rs
// Al completar un ciclo de procesamiento — heartbeat cada 5 minutos
async fn worker_heartbeat(hc_url: &str) {
    if let Err(e) = reqwest::get(hc_url).await {
        tracing::warn!(error = ?e, "failed to ping healthcheck — non-fatal");
    }
}
```

### Tarea 3 — Certificado TLS (ADR 0014)

```bash
# Cron diario — verificar que el certificado no expira en menos de 30 días
openssl s_client -connect tudominio.com:443 2>/dev/null \
  | openssl x509 -noout -checkend 2592000 \
  && curl -fsS https://hc-ping.com/${HC_TLS_UUID}
```

### Tarea 4 — Deploy exitoso

```makefile
# justfile
deploy:
    just audit
    just test
    kamal deploy
    curl -fsS ${HC_DEPLOY_UUID:+https://hc-ping.com/$HC_DEPLOY_UUID} || true
```

### Configuración recomendada de checks

| Check | Período | Grace period | Alerta si no llega |
|-------|---------|-------------|-------------------|
| Litestream backup | 1h | 15min | Email + Telegram |
| Apalis worker | 5min | 2min | Email urgente |
| TLS certificado | 24h | 2h | Email |
| Deploy exitoso | Manual | — | Solo historial |

### Variables de entorno

```bash
# .env.example — opcionales, el sistema arranca sin ellos
HC_LITESTREAM_UUID=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
HC_DEPLOY_UUID=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
HC_TLS_UUID=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
```

---

## Alternativas consideradas

| Opción | Motivo de descarte |
|--------|--------------------|
| UptimeRobot | Solo monitorea URLs HTTP — no detecta si un cron job dejó de correr |
| Prometheus + Alertmanager | ~500MB de RAM — inviable en VPS de $5 |
| Script propio de alertas | Mismo punto ciego: si el VPS muere, el script también |
| Better Uptime | Funcionalidad similar pero sin capa gratuita suficiente |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para centralizar las alertas y mejorar el diagnóstico de fallos:

| Herramienta | Propósito en el Monitoreo |
| :--- | :--- |
| **Sentry Crons** | **Unificación:** Permite usar el SDK de Sentry (ADR 0016) para hacer check-ins de tareas y ver errores en un solo dashboard. |
| **Better Stack** | **Observabilidad:** Una alternativa moderna a Healthchecks.io que combina uptime, logs y pings en una sola plataforma. |
| **`reqwest`** | **Cliente Robusto:** En Rust, configurar timeouts agresivos para que el ping de monitoreo nunca bloquee la lógica de negocio. |
| **Telegram Bot API** | **Alertas Directas:** Configurar Healthchecks.io para enviar notificaciones push inmediatas a un canal de Telegram privado. |

---

## Consecuencias

### ✅ Positivas

- Impacto en RAM/CPU: cero — es un `curl` de una línea al final de cada tarea
- Detecta fallos de VPS completo — no solo de procesos individuales
- Capa gratuita de Healthchecks.io cubre hasta 20 checks — suficiente para el MVP
- Alertas por email, Telegram, Slack, WhatsApp sin configuración adicional

### ⚠️ Negativas / Trade-offs

- Dependencia de un servicio externo — si Healthchecks.io cae, no hay alertas
  → Mitigación: para las tareas MÁS críticas (Litestream), configurar un segundo
    check en UptimeRobot o Cronitor como backup
  → Healthchecks.io tiene SLA 99.9% — las caídas son raras y breves
  → Estado de Healthchecks.io: `curl https://healthchecks.io/api/v3/status/`
- No reemplaza logs detallados — solo indica si la tarea corrió o no, no por qué falló
  → Los logs JSON de tracing (ADR 0016) dan el detalle de por qué falló
  → La combinación ping de HC + logs locales cubre el 100% de los casos de debugging

### Decisiones derivadas

- Las UUIDs de los checks se guardan en `.env.local` como `HC_*` — nunca hardcodeadas
- El check de Litestream es el más crítico — alerta inmediata sin grace period largo
- `just deploy` hace ping al check de deploy como último paso — historial de deploys automático
- Los checks son opcionales en desarrollo — el sistema funciona sin `HC_*` configurados
