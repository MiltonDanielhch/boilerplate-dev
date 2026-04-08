# ADR VPS — Protocolo de los 5 Pilares

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0014 (Deploy Podman+Caddy+Kamal) · ADR 0004 (Litestream) · ADR 0015 (Healthchecks.io) |

---

## Contexto

Un VPS de $5 (usualmente 1 vCPU y 1GB RAM) es un entorno hostil. Sin preparación adecuada:

- **OOM (Out of Memory):** el sistema matará la base de datos o la API ante cualquier pico de carga
- **Ataques de fuerza bruta:** en menos de 10 minutos de estar en internet, bots intentarán entrar por SSH
- **Fricción de despliegue:** sin un reverse proxy eficiente, gestionar certificados SSL y puertos se vuelve una pesadilla

La arquitectura del proyecto (ADR 0014) ya resuelve el deploy con Kamal y Caddy. Este ADR documenta los pasos de preparación del servidor Linux "desnudo" que deben ejecutarse **antes** del primer `kamal setup`.

---

## Decisión

Adoptar el **Protocolo de los 5 Pilares** como estándar de configuración inicial del VPS. Transforma un Linux desnudo en una plataforma de alto rendimiento de forma reproducible y documentada.

---

## Pilar 1 — Gestión de Memoria Virtual (Swap)

**Problema:** Rust es eficiente, pero picos de tráfico o tareas de fondo (Litestream snapshot, Apalis jobs pesados) pueden exceder la RAM física. Sin Swap, el kernel OOM-killer termina procesos críticos.

**Acción:** Crear un archivo de intercambio de 2GB obligatoriamente.

```bash
# Crear el archivo de swap
sudo fallocate -l 2G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile

# Hacer el swap permanente tras reinicios
echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab

# Verificar
free -h
# Debe mostrar: Swap: 2.0G
swapon --show
# Debe mostrar /swapfile
```

**Parámetro opcional — swappiness:**

```bash
# El kernel usa swap cuando la RAM libre cae al 10% (recomendado para servidores)
echo 'vm.swappiness=10' | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

---

## Pilar 2 — Acceso de Confianza Cero (Zero Trust SSH)

**Problema:** el acceso por password es vulnerable a ataques de diccionario. El root comprometido = servidor comprometido.

**Acción:** prohibir `RootLogin` y `PasswordAuthentication`. Solo llaves ED25519.

### Paso 1 — Crear usuario de deploy

```bash
# En el servidor (como root, por última vez)
adduser deploy
usermod -aG sudo deploy

# Copiar tu llave pública al usuario deploy
# (desde tu laptop de desarrollo)
ssh-copy-id -i ~/.ssh/id_ed25519.pub deploy@IP_DEL_VPS

# Verificar que funciona ANTES de deshabilitar password
ssh deploy@IP_DEL_VPS
# → debe entrar sin pedir password
```

### Paso 2 — Generar llave ED25519 si no tienes una

```bash
# En tu laptop de desarrollo
ssh-keygen -t ed25519 -C "laboratorio3030-vps" -f ~/.ssh/id_ed25519
# Guardar una copia del par de llaves en lugar seguro (password manager, etc.)
```

### Paso 3 — Deshabilitar acceso inseguro

```bash
# En el servidor
sudo nano /etc/ssh/sshd_config

# Cambiar estas líneas:
PermitRootLogin no
PasswordAuthentication no
PubkeyAuthentication yes
AuthorizedKeysFile .ssh/authorized_keys

# Reiniciar SSH (sin cerrar la sesión actual)
sudo systemctl restart sshd

# En OTRA terminal, verificar que sigue funcionando
ssh deploy@IP_DEL_VPS
# → entra con llave ✓
```

### Paso 4 — Fail2Ban (protección adicional)

```bash
sudo apt install fail2ban -y

# Configuración para SSH
sudo tee /etc/fail2ban/jail.local << 'EOF'
[sshd]
enabled  = true
port     = ssh
maxretry = 5
bantime  = 3600
findtime = 600
EOF

sudo systemctl enable fail2ban
sudo systemctl start fail2ban

# Verificar
sudo fail2ban-client status sshd
```

---

## Pilar 3 — El Portero Automático (Caddy Server)

**Problema:** Nginx requiere configuración manual de SSL, renovación de certificados y múltiples archivos de configuración. Cada nuevo dominio es un proceso.

**Decisión:** Caddy como Reverse Proxy. Gestiona SSL automáticamente con Let's Encrypt. 3-5 líneas de Caddyfile para apuntar un dominio nuevo.

> **Nota:** Caddy se configura en detalle en `ROADMAP-INFRA.md` (Bloque INF.II) y en ADR 0014.
> Este pilar cubre solo la instalación inicial en el VPS.

```bash
# Instalar Caddy
sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https curl
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' \
  | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' \
  | sudo tee /etc/apt/sources.list.d/caddy-stable.list
sudo apt update && sudo apt install caddy -y

# Verificar
caddy version
# → v2.x.x

# Caddy se instala como servicio de systemd automáticamente
sudo systemctl status caddy
# → active (running)
```

**Caddyfile mínimo para producción:**

```
# /etc/caddy/Caddyfile
tudominio.com {
    reverse_proxy localhost:8080
    encode gzip zstd
    header {
        Strict-Transport-Security "max-age=31536000; includeSubDomains"
        X-Content-Type-Options    "nosniff"
        X-Frame-Options           "DENY"
        -Server
    }
}
```

```bash
# Recargar después de editar el Caddyfile
sudo systemctl reload caddy

# Los certificados SSL se obtienen y renuevan automáticamente
# No hay nada más que hacer para SSL — para siempre
```

---

## Pilar 4 — Blindaje de Red (UFW)

**Problema:** todos los puertos están abiertos por defecto. Un servicio mal configurado en cualquier puerto es una vulnerabilidad.

**Acción:** firewall restrictivo — solo los puertos necesarios.

```bash
# Instalar UFW si no está
sudo apt install ufw -y

# Política por defecto: denegar todo entrante
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Solo abrir los puertos necesarios
sudo ufw allow 22/tcp    # SSH — NO cerrar este antes de habilitar
sudo ufw allow 80/tcp    # HTTP (Caddy redirige a HTTPS)
sudo ufw allow 443/tcp   # HTTPS

# Habilitar el firewall
sudo ufw enable
# → Firewall is active and enabled on system startup

# Verificar
sudo ufw status verbose
```

**Output esperado:**

```
Status: active

To                         Action      From
--                         ------      ----
22/tcp                     ALLOW IN    Anywhere
80/tcp                     ALLOW IN    Anywhere
443/tcp                    ALLOW IN    Anywhere
```

> ⚠️ **Nunca cerrar el puerto 22 antes de verificar que SSH con llave funciona.**
> Si pierdes el acceso SSH, necesitarás acceder por consola del proveedor del VPS.

---

## Pilar 5 — Persistencia de Procesos (Systemd)

**Problema:** sin Systemd, si el binario de Rust falla o el servidor se reinicia, hay que entrar manualmente a levantarlo. Con `nohup` o `screen` se pierde el control de logs y reinicio automático.

**Decisión:** Kamal ya gestiona el ciclo de vida del contenedor vía Podman (ADR 0014). Este pilar documenta el servicio Systemd para Caddy (ya instalado) y el arranque automático de Podman.

```bash
# Habilitar inicio automático al arrancar del servidor
sudo systemctl enable caddy
sudo systemctl enable podman

# Verificar estado de servicios
sudo systemctl status caddy
sudo systemctl status podman

# Para el binario de Rust via Kamal — Kamal gestiona esto automáticamente
# pero el volumen de datos debe sobrevivir reinicios
sudo mkdir -p /data/boilerplate
sudo chown deploy:deploy /data/boilerplate
# Este directorio está declarado en kamal/deploy.yml como volume
```

**Si en algún caso se necesita un servicio Systemd manual:**

```bash
sudo tee /etc/systemd/system/boilerplate.service << 'EOF'
[Unit]
Description=Boilerplate API
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=deploy
WorkingDirectory=/home/deploy/boilerplate
EnvironmentFile=/home/deploy/.env
ExecStart=/home/deploy/boilerplate/api
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable boilerplate
sudo systemctl start boilerplate
```

---

## Script de configuración inicial completo

Ejecutar como root en el VPS recién creado:

```bash
#!/bin/bash
# setup-vps.sh — Protocolo de los 5 Pilares
# Ejecutar: bash setup-vps.sh
# Tiempo estimado: ~5 minutos

set -e  # Detener si algo falla

echo "🚀 Iniciando Protocolo de los 5 Pilares..."

# Actualizar sistema
apt update && apt upgrade -y

# ── Pilar 1: Swap 2GB ──────────────────────────────────────────────────────
echo "📦 Pilar 1: Configurando Swap 2GB..."
if [ ! -f /swapfile ]; then
    fallocate -l 2G /swapfile
    chmod 600 /swapfile
    mkswap /swapfile
    swapon /swapfile
    echo '/swapfile none swap sw 0 0' >> /etc/fstab
    echo 'vm.swappiness=10' >> /etc/sysctl.conf
    sysctl -p
    echo "✅ Swap configurado: $(free -h | grep Swap)"
else
    echo "⏭️  Swap ya existe, saltando"
fi

# ── Pilar 2: Seguridad SSH ─────────────────────────────────────────────────
echo "🔐 Pilar 2: Creando usuario deploy..."
if ! id "deploy" &>/dev/null; then
    adduser --disabled-password --gecos "" deploy
    usermod -aG sudo deploy
    mkdir -p /home/deploy/.ssh
    chmod 700 /home/deploy/.ssh
    echo "⚠️  IMPORTANTE: Copia tu llave pública a /home/deploy/.ssh/authorized_keys"
    echo "   Comando desde tu laptop: ssh-copy-id deploy@$(hostname -I | awk '{print $1}')"
fi

# Fail2Ban
echo "🛡️  Instalando Fail2Ban..."
apt install -y fail2ban
tee /etc/fail2ban/jail.local << 'EOF'
[sshd]
enabled  = true
maxretry = 5
bantime  = 3600
findtime = 600
EOF
systemctl enable fail2ban && systemctl start fail2ban

# ── Pilar 3: Caddy ──────────────────────────────────────────────────────────
echo "🌐 Pilar 3: Instalando Caddy..."
apt install -y debian-keyring debian-archive-keyring apt-transport-https curl
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' \
    | gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' \
    | tee /etc/apt/sources.list.d/caddy-stable.list
apt update && apt install -y caddy
systemctl enable caddy
echo "✅ Caddy $(caddy version) instalado"

# ── Pilar 4: UFW ────────────────────────────────────────────────────────────
echo "🔒 Pilar 4: Configurando UFW..."
apt install -y ufw
ufw default deny incoming
ufw default allow outgoing
ufw allow 22/tcp
ufw allow 80/tcp
ufw allow 443/tcp
ufw --force enable
echo "✅ Firewall activo: $(ufw status | head -1)"

# ── Pilar 5: Systemd ────────────────────────────────────────────────────────
echo "⚙️  Pilar 5: Configurando persistencia..."
apt install -y podman
systemctl enable podman
mkdir -p /data/boilerplate
chown deploy:deploy /data/boilerplate

# Actualizaciones automáticas de seguridad
apt install -y unattended-upgrades
dpkg-reconfigure -f noninteractive unattended-upgrades

echo ""
echo "✅ =========================================="
echo "✅  Protocolo de los 5 Pilares completado"
echo "✅ =========================================="
echo ""
echo "PRÓXIMOS PASOS MANUALES:"
echo "1. Copiar llave SSH: ssh-copy-id deploy@$(hostname -I | awk '{print $1}')"
echo "2. Verificar acceso: ssh deploy@$(hostname -I | awk '{print $1}')"
echo "3. Deshabilitar password SSH (solo después de verificar llave):"
echo "   Editar /etc/ssh/sshd_config → PasswordAuthentication no"
echo "   sudo systemctl restart sshd"
echo "4. Editar /etc/caddy/Caddyfile con tu dominio"
echo "5. Ejecutar: kamal setup (desde tu laptop)"
echo ""
echo "Swap:     $(free -h | grep Swap | awk '{print $2}')"
echo "Firewall: $(ufw status | head -1)"
echo "Caddy:    $(caddy version)"
```

---

## Verificación post-instalación

```bash
# Ejecutar desde tu laptop después de configurar el VPS

# 1. SSH con llave funciona
ssh deploy@IP_DEL_VPS "echo '✅ SSH con llave OK'"

# 2. Swap activo
ssh deploy@IP_DEL_VPS "free -h | grep Swap"
# → Swap: 2.0G

# 3. Firewall activo
ssh deploy@IP_DEL_VPS "sudo ufw status"
# → Status: active (22, 80, 443)

# 4. Caddy corriendo
ssh deploy@IP_DEL_VPS "sudo systemctl status caddy | grep Active"
# → Active: active (running)

# 5. SSH sin password falla (seguridad verificada)
ssh -o PasswordAuthentication=no deploy@IP_DEL_VPS "echo OK"
# → Si entró sin pedir password → ✅ (la llave funciona)
# → Si pide password → ⚠️ revisar PasswordAuthentication en sshd_config

# 6. Volumen de datos existe
ssh deploy@IP_DEL_VPS "ls -la /data/"
# → drwxr-xr-x deploy deploy /data/boilerplate
```

---

## Alternativas consideradas

| Opción | Motivo de descarte |
|--------|--------------------|
| **Nginx** en lugar de Caddy | Requiere configuración manual de SSL (certbot, cron de renovación, múltiples archivos) — más puntos de fallo |
| **1GB de Swap** | Insuficiente para picos de Litestream + Apalis corriendo simultáneamente |
| **Firewall del proveedor** (Hetzner, DigitalOcean) | UFW local es más consistente — funciona igual en cualquier proveedor |
| **Docker** en lugar de Podman | Docker requiere daemon root — Podman rootless reduce la superficie de ataque |
| **Sin Fail2Ban** | Sin él, los logs de SSH se llenan de intentos en <10 minutos en cualquier IP pública |

---

## Consecuencias

### ✅ Positivas

- **Estabilidad de hierro:** el servidor no se cae por falta de memoria "real" — el Swap actúa como colchón de seguridad para Litestream y Apalis
- **Deploy veloz:** una vez configurado Caddy, apuntar un dominio nuevo toma 10 segundos — editar el Caddyfile y `sudo systemctl reload caddy`
- **Mantenimiento cero de SSL:** los certificados se renuevan automáticamente para siempre — sin cron jobs, sin recordatorios
- **Superficie de ataque mínima:** root deshabilitado + llaves ED25519 + Fail2Ban + UFW = un VPS de $5 con seguridad de producción real
- **Reproducible:** el script de instalación puede ejecutarse en cualquier VPS y da el mismo resultado

### ⚠️ Negativas / Trade-offs

- **Desgaste de disco por Swap:** el uso intensivo de Swap en SSDs puede acortar su vida — no es un problema real en VPS modernos de 2-3 años ni en hosting cloud (los discos son virtuales)
  → Mitigación: `vm.swappiness=10` minimiza el uso de Swap salvo que sea necesario
- **Riesgo de bloqueo SSH:** si se pierden las llaves SSH, se pierde el acceso al servidor
  → Mitigación obligatoria: guardar el par de llaves ED25519 en un password manager (Bitwarden, 1Password) Y en un backup físico offline
  → Plan B: acceso por consola VNC/KVM del proveedor (Hetzner, DigitalOcean, etc.)
- **Caddy y Let's Encrypt requieren dominio real:** no funciona con IP directa
  → Para desarrollo local: usar `localhost` en el Caddyfile sin HTTPS o `caddy reverse-proxy`

### Decisiones derivadas

- El script `setup-vps.sh` debe ejecutarse **antes** de `kamal setup`
- Las llaves SSH se generan en la laptop del desarrollador — nunca en el servidor
- `/data/boilerplate` es el volumen de datos declarado en `infra/kamal/deploy.yml`
- `PasswordAuthentication no` se activa **solo después** de verificar que la llave funciona — el orden importa
- El usuario `deploy` es el que usa Kamal para conectarse al VPS (declarado en `deploy.yml` como `ssh.user: deploy`)
