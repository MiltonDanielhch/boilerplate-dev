# ADR F-005 — Futuro: Kubernetes (Orquestación a Escala)

| Campo | Valor |
|-------|-------|
| **Estado** | 🔮 Futuro — activar cuando VPS dedicado ya no sea suficiente |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0014 (Deploy), ADR 0031 (Escalamiento), ADR 0025 (NATS) |

---

## Contexto

Este ADR documenta la migración a **Kubernetes** cuando la arquitectura de VPS dedicados con
Kamal ya no sea suficiente para la escala operativa. Kubernetes proporciona orquestación
automática, auto-scaling, y gestión de múltiples servicios a escala.

**NO activar hasta que:**
- El sistema requiera >10 servicios independientes
- Auto-scaling automático sea crítico (tráfico muy variable)
- Multi-region deploy sea necesario
- El equipo tenga expertise DevOps/SRE dedicado

**Para el 99% de proyectos, Kamal + VPS es suficiente hasta 1M+ usuarios.**

---

## Decisión futura

Evaluar **Kubernetes** (K8s) con **K3s** (distribución ligera) o **managed K8s** (EKS, GKE, DOKS)
cuando la complejidad operativa de múltiples VPS sea mayor que la de K8s.

### Arquitectura K8s

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Ingress Controller                          │
│                    (NGINX / Traefik / Cilium)                       │
└─────────────────────────────┬───────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌───────────────┐    ┌───────────────┐    ┌───────────────┐
│   API Pods    │    │  Worker Pods  │    │   NATS Pods   │
│  (Axum HTTP)  │    │  (Consumers)  │    │  (JetStream)  │
│  HPA: 3-20    │    │   HPA: 2-50   │    │   StatefulSet │
└───────┬───────┘    └───────┬───────┘    └───────┬───────┘
        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     Persistent Volume Claims                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                   │
│  │  SQLite/    │  │  NATS       │  │  Prometheus │                   │
│  │  PostgreSQL │  │  Storage    │  │  Data       │                   │
│  │  (PVC)      │  │  (PVC)      │  │  (PVC)      │                   │
│  └─────────────┘  └─────────────┘  └─────────────┘                   │
└─────────────────────────────────────────────────────────────────────┘
```

### Flujo de deploy

```
Developer push
       │
       ▼
┌──────────────┐
│  CI/CD       │
│  (GitHub     │
│   Actions)   │
└──────┬───────┘
       │
       ▼
┌──────────────┐     ┌──────────────┐
│  Build       │────►│  Registry    │
│  Image       │     │  (ECR/GCR/   │
│              │     │   DOCR)      │
└──────────────┘     └──────┬───────┘
                            │
                            ▼
┌─────────────────────────────────────────┐
│  kubectl apply -f k8s/                 │
│  │                                      │
│  ├──► Deployment/api (RollingUpdate)   │
│  ├──► Deployment/worker (HPA)          │
│  ├──► StatefulSet/nats                   │
│  └──► Service/Ingress                  │
└─────────────────────────────────────────┘
```

---

## Cuándo activar

| Criterio | Umbral |
|----------|--------|
| Servicios | >10 microservicios independientes |
| Auto-scaling | Necesidad de escalar 0→100 pods automáticamente por carga |
| Multi-region | Deploy obligatorio en 3+ regiones geográficas |
| Team size | Equipo DevOps/SRE dedicado (>=2 personas) |
| Complejidad VPS | Gestión manual de >20 VPS es más costosa que K8s |
| Costo | Presupuesto cloud >$500/mes justifica managed K8s |

---

## Implementación

### Opción A: K3s (on-premise / VPS propios)

Para mantener control de costos con VPS dedicados.

```bash
# 1. Crear cluster K3s (1 master, 2+ workers)
# Master node
curl -sfL https://get.k3s.io | sh -

# Workers (obtener token del master)
curl -sfL https://get.k3s.io | K3S_URL=https://master:6443 K3S_TOKEN=xxx sh -

# 2. Configurar kubectl
mkdir -p ~/.kube
cp /etc/rancher/k3s/k3s.yaml ~/.kube/config
kubectl get nodes
# Esperado: 3 nodes Ready
```

### Opción B: Managed Kubernetes (cloud)

Para equipos sin expertise de gestión de infraestructura.

| Proveedor | Servicio | Costo base | Cuándo usar |
|-----------|----------|------------|-------------|
| DigitalOcean | DOKS | ~$24/mes | Proyectos medianos, equipo pequeño |
| AWS | EKS | ~$75/mes | Enterprise, servicios AWS integrados |
| GCP | GKE | ~$75/mes | BigQuery, Dataflow, servicios GCP |
| Hetzner | hcloud + k3s | ~$10/mes | Presupuesto limitado, Europa |

### Configuración de manifests

```yaml
# k8s/namespace.yml
apiVersion: v1
kind: Namespace
metadata:
  name: boilerplate

---
# k8s/deployment-api.yml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: boilerplate-api
  namespace: boilerplate
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: boilerplate-api
  template:
    metadata:
      labels:
        app: boilerplate-api
    spec:
      containers:
        - name: api
          image: ghcr.io/tuusuario/boilerplate:latest
          ports:
            - containerPort: 8080
          env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: boilerplate-secrets
                  key: database-url
            - name: RUST_LOG
              value: "info"
          resources:
            requests:
              memory: "64Mi"
              cpu: "100m"
            limits:
              memory: "256Mi"
              cpu: "500m"
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 10
          readinessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 3
            periodSeconds: 5

---
# k8s/hpa-api.yml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: boilerplate-api-hpa
  namespace: boilerplate
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: boilerplate-api
  minReplicas: 3
  maxReplicas: 20
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: 80
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
        - type: Percent
          value: 100
          periodSeconds: 15
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
        - type: Percent
          value: 10
          periodSeconds: 60

---
# k8s/deployment-worker.yml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: boilerplate-worker
  namespace: boilerplate
spec:
  replicas: 2
  selector:
    matchLabels:
      app: boilerplate-worker
  template:
    metadata:
      labels:
        app: boilerplate-worker
    spec:
      containers:
        - name: worker
          image: ghcr.io/tuusuario/boilerplate-worker:latest
          command: ["./worker"]
          env:
            - name: NATS_URL
              value: "nats://nats:4222"
          resources:
            requests:
              memory: "128Mi"
              cpu: "200m"
            limits:
              memory: "512Mi"
              cpu: "1000m"

---
# k8s/statefulset-nats.yml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: nats
  namespace: boilerplate
spec:
  serviceName: nats
  replicas: 3
  selector:
    matchLabels:
      app: nats
  template:
    metadata:
      labels:
        app: nats
    spec:
      containers:
        - name: nats
          image: nats:2.10-alpine
          command:
            - "nats-server"
            - "-js"
            - "-sd"
            - "/data"
            - "-m"
            - "8222"
          ports:
            - containerPort: 4222
            - containerPort: 8222
          volumeMounts:
            - name: nats-data
              mountPath: /data
  volumeClaimTemplates:
    - metadata:
        name: nats-data
      spec:
        accessModes: ["ReadWriteOnce"]
        resources:
          requests:
            storage: 10Gi

---
# k8s/ingress.yml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: boilerplate-ingress
  namespace: boilerplate
  annotations:
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    cert-manager.io/cluster-issuer: "letsencrypt"
spec:
  tls:
    - hosts:
        - api.tudominio.com
      secretName: boilerplate-tls
  rules:
    - host: api.tudominio.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: boilerplate-api
                port:
                  number: 8080
```

### CI/CD con GitHub Actions

```yaml
# .github/workflows/k8s-deploy.yml
name: Deploy to Kubernetes

on:
  push:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build image
        run: |
          docker build -t ghcr.io/${{ github.repository }}:${{ github.sha }} .
          docker push ghcr.io/${{ github.repository }}:${{ github.sha }}

      - name: Update K8s manifests
        run: |
          sed -i "s|image:.*|image: ghcr.io/${{ github.repository }}:${{ github.sha }}|" k8s/deployment-api.yml

      - name: Deploy to K8s
        uses: azure/k8s-deploy@v4
        with:
          manifests: |
            k8s/namespace.yml
            k8s/deployment-api.yml
            k8s/deployment-worker.yml
            k8s/hpa-api.yml
            k8s/service-api.yml
            k8s/ingress.yml
          images: |
            ghcr.io/${{ github.repository }}:${{ github.sha }}
```

---

## Comparativa: Kamal vs Kubernetes

| Aspecto | Kamal + VPS | Kubernetes |
|---------|-------------|------------|
| **Complejidad** | Baja (SSH + Docker) | Alta (YAML, operators, CRDs) |
| **Learning curve** | 1 día | 1-3 meses |
| **Auto-scaling** | Manual (add VPS) | Automático (HPA) |
| **Auto-healing** | Reinicio manual | Automático (liveness probes) |
| **Costo (<100k users)** | $20-80/mes | $100-500/mes |
| **Costo (>1M users)** | $200+/mes | $500+/mes (pero más eficiente) |
| **Multi-region** | Complejo | Nativo con clusters |
| **Rollback** | `kamal rollback` | `kubectl rollout undo` |
| **Ideal para** | Startups, MVPs, SMBs | Enterprise, scale-ups, SaaS multi-tenant |

---

## Consecuencias

### ✅ Positivas

- **Auto-scaling:** Escala automáticamente 3→20 pods según carga de CPU/memoria
- **Auto-healing:** Pods fallidos se reinician automáticamente (liveness probes)
- **Rolling updates:** Deploys sin downtime con verificación de health
- **Resource limits:** Garantía de recursos por pod, isolation entre servicios
- **Ecosistema:** Helm charts, operators, service mesh (Istio, Linkerd)

### ⚠️ Negativas / Trade-offs

- **Complejidad:** YAML everywhere, operators, CRDs — curva de aprendizaje pronunciada
- **Costo base:** Managed K8s tiene costo fijo $75+/mes antes de correr workloads
- **Debugging:** `kubectl logs`, `kubectl exec`, port-forwarding vs SSH simple
- **Networking:** CNI, ingress controllers, service mesh — complejidad de red aumenta
- **Observability:** Necesitas stack completo (Prometheus, Grafana, Loki) — más complejidad
- **Vendor lock-in:** EKS/GKE tienen servicios propietarios difíciles de migrar

### Decisiones derivadas

- **Mantener Kamal hasta el límite:** Es 10x más simple para la mayoría de casos
- **K3s antes de managed:** Si se necesita K8s, probar K3s primero (más barato)
- **Monolito antes de microservicios:** Un solo Deployment con 3 replicas > 10 microservicios
- **Managed K8s:** Si se va a cloud, usar DOKS (más barato) antes que EKS/GKE
- **Litestream → Velero:** Migrar backups de Litestream a Velero para PVCs de K8s

---

## Estado actual

**NO implementar.** Mantener Kamal (ADR 0014) hasta que:
1. El sistema tenga >10 servicios independientes
2. Auto-scaling automático sea crítico para el negocio
3. El equipo tenga expertise DevOps/SRE dedicado
4. El costo de gestionar 20+ VPS manualmente supere el de K8s managed

**Path de transición:**
1. Fase 1-3: Kamal + VPS ($5-40/mes)
2. Fase 4+: Evaluar K3s si se mantiene en VPS dedicados
3. Fase 5+: Managed K8s (DOKS/EKS) si se necesita multi-region

Ver ADR 0014 para el deploy actual con Kamal y ADR 0031 para estrategia de escalamiento.
