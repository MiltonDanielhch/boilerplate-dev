# ADR F-006 — Futuro: CDN Global (CloudFront / Cloudflare)

| Campo | Valor |
|-------|-------|
| **Estado** | 🔮 Futuro — activar cuando usuarios globales >50% del tráfico |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0020 (S3/Tigris), ADR 0014 (Deploy), ADR 0022 (Frontend) |

---

## Contexto

Este ADR documenta la implementación de un **CDN (Content Delivery Network)** global cuando
la aplicación tiene usuarios distribuidos geográficamente y la latencia desde el VPS central
afecta la experiencia de usuario.

**Activar cuando:**
- >50% del tráfico proviene de fuera de la región del VPS
- Latencia >200ms para usuarios en continentes distintos
- Assets estáticos (JS, CSS, imágenes) representan >50% del bandwidth
- Necesidad de edge caching y DDoS protection

**NO activar en MVP.** Caddy en el VPS + caché de assets es suficiente hasta ~100k usuarios globales.

---

## Decisión futura

Usar **Cloudflare** (preferido) o **AWS CloudFront** como CDN global.

### Arquitectura

```
Usuario en Asia ──┐
                  │
Usuario en EU ────┼──► CDN Edge Location ──► Cache Hit? ──► Usuario (rápido)
                  │        (Cloudflare)           │
Usuario en US ────┘                             │
                                                  │ Miss
                                                  ▼
                                           VPS Origin (Caddy)
                                           ┌─────────────────┐
                                           │  apps/web/      │
                                           │  (Astro +       │
                                           │   Svelte)       │
                                           └─────────────────┘
```

### Flujo de requests

| Escenario | Sin CDN | Con CDN |
|-----------|---------|---------|
| 1er request | 300ms (desde Asia) | 50ms (edge Asia) + 250ms (fetch origin) |
| 2do request | 300ms | 10ms (cache hit) |
| Assets estáticos | Siempre desde VPS | Cache 24h en edge |
| DDoS | VPS saturado | Absorbido por CDN |

---

## Cuándo activar

| Criterio | Umbral |
|----------|--------|
| Distribución geográfica | >50% tráfico fuera de región del VPS |
| Latencia percebida | >200ms TTFB para usuarios lejanos |
| Assets estáticos | >50% del bandwidth total |
| Seguridad | Necesidad de WAF y DDoS protection |
| Presupuesto | $20-200/mes para CDN justificable |

---

## Implementación

### Opción A: Cloudflare (Recomendado)

Mejor para proyectos que no usan AWS.

```
1. Registrar dominio en Cloudflare (o transferir NS)
2. Configurar DNS:
   
   Type: A
   Name: @
   IPv4 address: <VPS_IP>
   Proxy status: Proxied (naranja)
   
3. Configurar Page Rules:
   
   URL: *tudominio.com/assets/*
   Settings:
     - Cache Level: Cache Everything
     - Edge Cache TTL: 24 hours
     - Browser Cache TTL: 4 hours
     
4. Configurar Cache Rules:
   
   URL: *tudominio.com/api/*
   Settings:
     - Cache Level: Bypass (nunca cachear API)
```

**Configuración Astro:**

```javascript
// astro.config.mjs
export default defineConfig({
  build: {
    assets: 'assets',
    // Cloudflare cache-busting con hash
    assetsPrefix: 'https://cdn.tudominio.com'
  }
});
```

### Opción B: AWS CloudFront

Si ya se usa AWS (EKS, RDS, etc.).

```
1. Crear Distribution en CloudFront
2. Origin: VPS IP (o ALB si ya hay K8s)
3. Behaviors:
   
   Path pattern: /assets/*
   Viewer protocol policy: Redirect HTTP to HTTPS
   Cache policy: Managed-CachingOptimized
   
   Path pattern: /api/*
   Cache policy: Managed-CachingDisabled
   
4. SSL: ACM certificate para tudominio.com
5. Route53: Alias record apuntando a CloudFront
```

**Costo estimado:**

| Proveedor | Costo | Límites |
|-----------|-------|---------|
| Cloudflare Free | $0 | Sin limites, sin bandwidth charges |
| Cloudflare Pro | $20/mes | WAF, analytics avanzadas |
| Cloudflare Business | $200/mes | SLA 100%, support prioritario |
| AWS CloudFront | ~$0.085/GB | Pago por uso, más complejo |

### Configuración Caddy (Origin)

```caddyfile
# Caddyfile en VPS
tudominio.com {
    # Headers para CDN cache control
    header {
        # Assets: cache 24h en CDN, 4h en browser
        @assets {
            path /assets/*
        }
        header @assets Cache-Control "public, max-age=14400, s-maxage=86400"
        
        # API: nunca cachear
        @api {
            path /api/*
        }
        header @api Cache-Control "no-store, no-cache, must-revalidate"
    }
    
    reverse_proxy localhost:8080
}
```

---

## Features adicionales del CDN

### 1. DDoS Protection

Cloudflare absorbe ataques automáticamente en la capa gratuita.

### 2. WAF (Web Application Firewall)

```
Cloudflare Pro ($20/mes):
  - OWASP Core Ruleset
  - Rate limiting por IP
  - Bot management básico
  - Bloqueo de países
```

### 3. Edge Workers (opcional)

```javascript
// Cloudflare Worker: A/B testing en edge
addEventListener('fetch', event => {
  event.respondWith(handleRequest(event.request))
})

async function handleRequest(request) {
  const url = new URL(request.url)
  
  // A/B test: 50% a versión nueva
  const cookie = request.headers.get('Cookie')
  if (!cookie || !cookie.includes('version=')) {
    const version = Math.random() < 0.5 ? 'v1' : 'v2'
    url.hostname = version === 'v2' ? 'v2.tudominio.com' : url.hostname
    
    const response = await fetch(url, request)
    response.headers.append('Set-Cookie', `version=${version}; Path=/`)
    return response
  }
  
  return fetch(url, request)
}
```

### 4. Image Optimization

Cloudflare Pro incluye Polish (optimización automática de imágenes).

```html
<!-- Sin CDN -->
<img src="/assets/hero.jpg" width="1200" height="600">

<!-- Con Cloudflare Polish -->
<img src="/assets/hero.jpg?w=800&q=80" width="800">
<!-- Cloudflare sirve WebP/AVIF automáticamente si el browser lo soporta -->
```

---

## Consecuencias

### ✅ Positivas

- **Latencia global:** 50-100ms TTFB en cualquier continente
- **DDoS protection:** CDN absorbe ataques antes de llegar al VPS
- **Bandwidth savings:** 60-80% menos tráfico al VPS (assets cacheados)
- **SEO:** Mejor Core Web Vitals por latencia reducida
- **Costo:** Cloudflare Free es literalmente gratis sin límites

### ⚠️ Negativas / Trade-offs

- **Cache invalidation:** Deploys requieren purgar caché del CDN
- **SSL complejidad:** Necesita certificado válido para el dominio en el CDN
- **Debugging:** Requests pasan por CDN → logs del VPS no ven IP real (usar CF-Connecting-IP)
- **Vendor lock-in:** Migrar de Cloudflare requiere cambiar DNS
- **API caching:** Riesgo de cachear respuestas dinámicas accidentalmente

### Decisiones derivadas

- Usar Cloudflare Free hasta que se necesite WAF ($20/mes)
- Nunca cachear `/api/*` — solo assets estáticos
- Configurar `CF-Connecting-IP` en logs para ver IP real del usuario
- Cache-busting con hash en nombres de archivo (Astro lo hace automático)

---

## Estado actual

**No implementar.** Mantener Caddy directo hasta que:
1. Usuarios globales representen >50% del tráfico
2. Métricas de Sentry muestren latencia >200ms para usuarios lejanos
3. Assets estáticos consuman >50% del bandwidth

**Path recomendado:**
1. Fase 1-2: Caddy directo (suficiente)
2. Fase 3+: Cloudflare Free (sin costo, instantáneo)
3. Fase 4+: Cloudflare Pro ($20/mes) si se necesita WAF

Ver ADR 0014 para Caddy y ADR 0022 para optimización de assets.
