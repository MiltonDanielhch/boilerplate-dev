# Ubicación: `infra/scripts/vps-setup.sh`
#
# Descripción: Script para configurar seguridad del VPS.
#              Ejecute como root después del deploy inicial.
#
# ADRs relacionados: 0014 (Deploy)

#!/bin/bash
set -e

echo "===.Configuracion de seguridad del VPS ==="

# ============================================
# 1. Actualizar sistema
# ============================================
echo "[1/10] Actualizando sistema..."
apt update && apt upgrade -y

# ============================================
# 2. Crear usuario no-root (si no existe)
# ============================================
echo "[2/10] Creando usuario no-root..."
if ! id -u boilerplate >/dev/null 2>&1; then
    adduser boilerplate
    usermod -aG sudo boilerplate
fi

# ============================================
# 3. Configurar SSH
# ============================================
echo "[3/10] Configurando SSH..."
cat > /etc/ssh/sshd_config.d/hardening.conf <<EOF
# No permitir root login
PermitRootLogin no
# Usar clave solo
PasswordAuthentication no
# Disable empty passwords
PermitEmptyPasswords no
# Max auth tries
MaxAuthTries 3
# Client alive interval
ClientAliveInterval 300
ClientAliveCountMax 2
EOF

systemctl reload sshd

# ============================================
# 4. Configurar firewall (UFW)
# ============================================
echo "[4/10] Configurando firewall..."
apt install -y ufw

# Default:negar entrada, permitir salida
ufw default incoming deny
ufw default allow outgoing

# SSH (limitar intentos)
ufw limit 22/tcp comment 'SSH'

# HTTP/HTTPS
ufw allow 80/tcp comment 'HTTP'
ufw allow 443/tcp comment 'HTTPS'

# Habilitar firewall
echo "y" | ufw enable

# ============================================
# 5. Fail2Ban (proteccion contra fuerza bruta)
# ============================================
echo "[5/10] Instalando Fail2Ban..."
apt install -y fail2ban

# Config fail2ban
cat > /etc/fail2ban/jail.local <<EOF
[DEFAULT]
bantime = 1h
findtime = 10m
maxretry = 3

[sshd]
enabled = true
port = ssh
filter = sshd
logpath = /var/log/auth.log
maxretry = 5

[nginx-http-auth]
enabled = false
EOF

systemctl enable fail2ban
systemctl restart fail2ban

# ============================================
# 6. Configurar kernel hardening
# ============================================
echo "[6/10] Configurando kernel hardening..."
cat >> /etc/sysctl.conf <<EOF

# Proteccion IP
net.ipv4.conf.all.rp_filter = 1
net.ipv4.conf.default.rp_filter = 1

# Ignore ICMP broadcast
net.ipv4.icmp_echo_ignore_broadcasts = 1

# Ignore bogus ICMP errors
net.ipv4.icmp_ignore_bogus_error_responses = 1

# Disable source packet routing
net.ipv4.conf.all.accept_source_route = 0
net.ipv4.conf.default.accept_source_route = 0

# Enable TCP SYN cookies
net.ipv4.tcp_syncookies = 1

# No ARP proxy
net.ipv4.conf.all.proxy_ask = 0
net.ipv4.conf.default.proxy_ask = 0
EOF

sysctl -p

# ============================================
# 7. Configurar logrotate (logs)
# ============================================
echo "[7/10] Configurando logrotate..."
cat > /etc/logrotate.d/boilerplate <<EOF
/var/log/boilerplate/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 0640 root root
}
EOF

# ============================================
# 8. Montar	tmp con noexec
# ============================================
echo "[8/10] Configurando /tmp sin exec..."
cat >> /etc/fstab <<EOF
tmpfs /tmp tmpfs defaults,noexec,nosuid,nodev 0 0
EOF

mount -o remount /tmp

# ============================================
# 9. Configurar automatic updates
# ============================================
echo "[9/10] Configurando automatic updates..."
apt install -y unattended-upgrades

cat > /etc/apt/apt.conf.d/50unattended-upgrades <<EOF
Unattended-Upgrade::Allowed-Origins {
    "${distro_id}:${distro_codename}-security";
    "${distro_id}:${distro_codename}-updates";
};
Unattended-Upgrade::Package-Copy-Origin-List {
    "${distro_id}:${distro_codename}-security";
};
Unattended-Upgrade::Automatic-Reboot "true";
Unattended-Upgrade::Automatic-Reboot-Time "03:00";
EOF

systemctl enable unattended-upgrades

# ============================================
# 10. Failover para DB (si se usa PostgreSQL)
# ============================================
echo "[10/10] Configuracion finalizada!"

echo ""
echo "=== Resumen de seguridad ==="
echo "- Usuario no-root: boilerplate"
echo "- SSH hardening: OK"
echo "- UFW firewall: OK (SSH, HTTP, HTTPS)"
echo "- Fail2Ban: OK"
echo "- Kernel hardening: OK"
echo "- Automatic updates: OK"
echo ""
echo "NOTA: Reinicia el servidor para aplicar todos los cambios"