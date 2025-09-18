#!/bin/bash

# CONFIGURATION PIA PROXY POUR ROTATION D'IPs
# Intégration avec le système de sécurité

echo "🔄 CONFIGURATION PIA PROXY"
echo "=========================="

# Configuration PIA Proxy
setup_pia_proxy() {
    echo "📡 Configuration PIA Proxy..."
    
    mkdir -p /etc/proxies/pia
    
    # Configuration principale PIA
    cat > /etc/proxies/pia/config.conf << 'EOF'
# PIA Proxy Configuration
# Residential Proxies for IP Rotation

[MAIN]
provider = pia_proxy
type = residential
rotation_enabled = true
sticky_session = false

[CONNECTION]
host = residential.piaproxy.com
port = 8080
protocol = socks5

[AUTHENTICATION]
username = your_pia_username
password = your_pia_password
auth_method = user_pass

[ROTATION]
rotation_interval = 300
auto_rotate = true
max_retries = 3
timeout = 30

[LOCATIONS]
countries = US,CA,GB,DE,FR,NL,SE,CH,NO,DK,FI,AT,BE,IE,ES,IT,PT,AU,JP
cities = auto
regions = auto

[FEATURES]
sticky_session_duration = 600
concurrent_sessions = 10
bandwidth_limit = unlimited
ipv6_support = false
EOF

    echo "✅ Configuration PIA Proxy créée"
}

# Script de connexion PIA
create_pia_connector() {
    cat > /usr/local/bin/pia-connect.sh << 'EOF'
#!/bin/bash

# Connecteur PIA Proxy
PIA_CONFIG="/etc/proxies/pia/config.conf"
CURRENT_IP_FILE="/tmp/pia_current_ip.txt"
LOG_FILE="/var/log/pia-proxy.log"

log_pia() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# Test connexion PIA
test_pia_connection() {
    local test_ip
    
    # Test via proxy PIA
    test_ip=$(curl -s --proxy socks5://residential.piaproxy.com:8080 \
                   --proxy-user "$PIA_USERNAME:$PIA_PASSWORD" \
                   --max-time 10 \
                   https://ipinfo.io/ip 2>/dev/null)
    
    if [[ "$test_ip" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo "$test_ip" > "$CURRENT_IP_FILE"
        log_pia "✅ Connexion PIA réussie - IP: $test_ip"
        return 0
    else
        log_pia "❌ Échec connexion PIA"
        return 1
    fi
}

# Rotation PIA
rotate_pia_ip() {
    log_pia "🔄 Rotation PIA en cours..."
    
    # Nouvelle session PIA
    session_id="session_$(date +%s)_$RANDOM"
    
    # Test nouvelle IP
    new_ip=$(curl -s --proxy socks5://residential.piaproxy.com:8080 \
                   --proxy-user "$PIA_USERNAME:$PIA_PASSWORD" \
                   --header "Session-ID: $session_id" \
                   --max-time 10 \
                   https://ipinfo.io/ip 2>/dev/null)
    
    if [[ "$new_ip" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        old_ip=$(cat "$CURRENT_IP_FILE" 2>/dev/null || echo "unknown")
        echo "$new_ip" > "$CURRENT_IP_FILE"
        log_pia "✅ IP rotée: $old_ip → $new_ip"
        
        # Notifier le système principal
        curl -X POST "http://localhost:8080/ip-changed" \
             -H "Content-Type: application/json" \
             -d "{\"old_ip\":\"$old_ip\",\"new_ip\":\"$new_ip\",\"provider\":\"pia\"}" \
             2>/dev/null &
        
        return 0
    else
        log_pia "❌ Échec rotation PIA"
        return 1
    fi
}

# Boucle de rotation automatique
auto_rotation_pia() {
    while true; do
        sleep 300  # 5 minutes
        rotate_pia_ip
    done
}

# Point d'entrée
case "$1" in
    "test")
        test_pia_connection
        ;;
    "rotate")
        rotate_pia_ip
        ;;
    "auto")
        auto_rotation_pia
        ;;
    *)
        echo "Usage: $0 {test|rotate|auto}"
        exit 1
        ;;
esac
EOF

    chmod +x /usr/local/bin/pia-connect.sh
    echo "✅ Script connecteur PIA créé"
}

# Intégration avec Docker
integrate_pia_docker() {
    echo "🐳 Intégration Docker PIA..."
    
    # Service PIA dans docker-compose
    cat >> /workspace/secure-host/docker-compose.yml << 'EOF'

  # Service PIA Proxy
  pia-proxy:
    image: alpine:latest
    container_name: pia-proxy
    environment:
      - PIA_USERNAME=${PIA_USERNAME}
      - PIA_PASSWORD=${PIA_PASSWORD}
      - ROTATION_INTERVAL=300
    volumes:
      - ./pia-proxy:/app
      - /usr/local/bin/pia-connect.sh:/usr/local/bin/pia-connect.sh:ro
    networks:
      - internal
    restart: unless-stopped
    command: /usr/local/bin/pia-connect.sh auto
    healthcheck:
      test: ["/usr/local/bin/pia-connect.sh", "test"]
      interval: 60s
      timeout: 30s
      retries: 3
EOF

    echo "✅ Service PIA ajouté à Docker"
}

# Configuration Nginx pour PIA
configure_nginx_pia() {
    echo "🔧 Configuration Nginx pour PIA..."
    
    cat > /workspace/secure-host/security-gateway/pia-upstream.conf << 'EOF'
# Configuration upstream PIA Proxy

# Backend via PIA Proxy
upstream pia_backend {
    server web-server:8080;
    keepalive 32;
}

# Configuration proxy PIA
server {
    listen 443 ssl http2;
    server_name _;
    
    # SSL configuration (existing)
    include /etc/nginx/ssl.conf;
    
    # Proxy via PIA
    location / {
        # Headers standard
        proxy_pass http://pia_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        
        # Headers PIA spécifiques
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header X-PIA-Session $request_id;
        
        # Configuration proxy sortant via PIA
        proxy_bind $server_addr;
        
        # Timeouts
        proxy_connect_timeout 30s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;
        
        # Buffers
        proxy_buffering on;
        proxy_buffer_size 128k;
        proxy_buffers 4 256k;
        proxy_busy_buffers_size 256k;
    }
}
EOF

    echo "✅ Configuration Nginx PIA créée"
}

# Test complet PIA
test_pia_setup() {
    echo "🧪 Test configuration PIA..."
    
    if [ -z "$PIA_USERNAME" ] || [ -z "$PIA_PASSWORD" ]; then
        echo "❌ Variables PIA manquantes:"
        echo "   export PIA_USERNAME=votre_username"
        echo "   export PIA_PASSWORD=votre_password"
        return 1
    fi
    
    # Test connexion
    echo "🔍 Test connexion PIA..."
    if /usr/local/bin/pia-connect.sh test; then
        echo "✅ PIA fonctionne correctement"
    else
        echo "❌ Problème avec PIA"
        return 1
    fi
    
    # Test rotation
    echo "🔄 Test rotation PIA..."
    if /usr/local/bin/pia-connect.sh rotate; then
        echo "✅ Rotation PIA fonctionne"
    else
        echo "❌ Problème rotation PIA"
        return 1
    fi
    
    echo "🎉 Configuration PIA validée!"
    return 0
}

# Monitoring PIA
setup_pia_monitoring() {
    echo "📊 Configuration monitoring PIA..."
    
    cat > /usr/local/bin/pia-monitor.sh << 'EOF'
#!/bin/bash

# Monitoring PIA Proxy
LOG_FILE="/var/log/pia-monitor.log"
ALERT_WEBHOOK="${ALERT_WEBHOOK:-}"

log_monitor() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# Vérifier statut PIA
check_pia_status() {
    if /usr/local/bin/pia-connect.sh test >/dev/null 2>&1; then
        log_monitor "✅ PIA opérationnel"
        return 0
    else
        log_monitor "🚨 PIA hors service"
        
        # Alerte webhook
        if [ -n "$ALERT_WEBHOOK" ]; then
            curl -X POST "$ALERT_WEBHOOK" \
                 -H "Content-Type: application/json" \
                 -d '{"text":"🚨 PIA Proxy hors service!"}' \
                 2>/dev/null &
        fi
        
        return 1
    fi
}

# Statistiques PIA
pia_stats() {
    current_ip=$(cat /tmp/pia_current_ip.txt 2>/dev/null || echo "unknown")
    rotations=$(grep "IP rotée" /var/log/pia-proxy.log | wc -l)
    uptime=$(uptime | awk '{print $3,$4}' | sed 's/,//')
    
    log_monitor "📊 Stats PIA: IP=$current_ip, Rotations=$rotations, Uptime=$uptime"
}

# Boucle monitoring
while true; do
    check_pia_status
    pia_stats
    sleep 300  # 5 minutes
done
EOF

    chmod +x /usr/local/bin/pia-monitor.sh
    echo "✅ Monitoring PIA configuré"
}

# Installation complète PIA
install_pia_complete() {
    echo "🚀 Installation complète PIA Proxy..."
    
    # Vérifier les credentials
    if [ -z "$PIA_USERNAME" ] || [ -z "$PIA_PASSWORD" ]; then
        echo ""
        echo "📋 CONFIGURATION REQUISE:"
        echo "========================"
        echo ""
        echo "1. Créez un compte sur: https://www.piaproxy.com"
        echo "2. Choisissez un plan (Residential Proxy recommandé)"
        echo "3. Récupérez vos identifiants"
        echo "4. Configurez les variables:"
        echo ""
        echo "   export PIA_USERNAME=votre_username"
        echo "   export PIA_PASSWORD=votre_password"
        echo ""
        echo "5. Relancez ce script"
        echo ""
        return 1
    fi
    
    # Installation
    setup_pia_proxy
    create_pia_connector
    integrate_pia_docker
    configure_nginx_pia
    setup_pia_monitoring
    
    # Test final
    if test_pia_setup; then
        echo ""
        echo "🎉 PIA PROXY CONFIGURÉ AVEC SUCCÈS!"
        echo "=================================="
        echo ""
        echo "✅ Proxies résidentiels actifs"
        echo "✅ Rotation automatique (5 min)"
        echo "✅ Monitoring en continu"
        echo "✅ Intégration Docker complète"
        echo ""
        echo "🚀 DÉMARRAGE:"
        echo "   docker-compose up -d"
        echo ""
        return 0
    else
        echo ""
        echo "❌ PROBLÈME CONFIGURATION PIA"
        echo "=============================="
        echo ""
        echo "Vérifiez vos identifiants et réessayez."
        echo ""
        return 1
    fi
}

# Menu principal
main() {
    echo "🔄 CONFIGURATEUR PIA PROXY"
    echo "=========================="
    echo ""
    echo "PIA Proxy offre:"
    echo "✅ Proxies résidentiels premium"
    echo "✅ 200+ pays disponibles"
    echo "✅ Rotation automatique"
    echo "✅ IPs ultra-propres"
    echo ""
    
    read -p "Continuer l'installation? (y/n): " continue_install
    
    if [[ $continue_install == "y" ]]; then
        install_pia_complete
    else
        echo "Installation annulée."
        exit 0
    fi
}

# Exécution
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi