#!/bin/bash

# SCRIPT D'INSTALLATION AUTOMATIQUE DES IPs
# Configure automatiquement 50+ IPs pour rotation

echo "🌍 CONFIGURATION AUTOMATIQUE DES IPs"
echo "===================================="

# Variables de configuration
VPN_DIR="/etc/openvpn/configs"
PROXY_DIR="/etc/proxies"
IP_LIST="/tmp/available_ips.txt"

# Créer les répertoires
mkdir -p "$VPN_DIR" "$PROXY_DIR"

# OPTION 1: Configuration NordVPN
setup_nordvpn() {
    echo "📥 Configuration NordVPN..."
    
    # Télécharger les configs
    wget -q https://downloads.nordcdn.com/configs/archives/servers/ovpn.zip -O /tmp/nord.zip
    unzip -q /tmp/nord.zip -d /tmp/nordvpn/
    
    # Sélectionner 50 serveurs dans différents pays
    countries=("netherlands" "sweden" "switzerland" "romania" "bulgaria" "estonia" "latvia" "lithuania" "norway" "finland")
    
    for country in "${countries[@]}"; do
        find /tmp/nordvpn -name "*${country}*" -type f | head -5 | while read config; do
            cp "$config" "$VPN_DIR/"
            echo "✅ Ajouté: $(basename "$config")"
        done
    done
    
    rm -rf /tmp/nord.zip /tmp/nordvpn
    echo "✅ NordVPN configuré: $(ls "$VPN_DIR" | wc -l) serveurs"
}

# OPTION 2: Configuration VPS multiples
setup_multiple_vps() {
    echo "🖥️ Configuration VPS multiples..."
    
    # Providers et régions
    declare -A providers=(
        ["digitalocean"]="nyc1,sfo3,ams3,sgp1,lon1,fra1,tor1,blr1"
        ["vultr"]="ewr,ord,dfw,sea,lax,atl,mia,sjc"
        ["linode"]="us-east,us-west,eu-west,ap-south"
    )
    
    for provider in "${!providers[@]}"; do
        echo "📍 Configuration $provider..."
        IFS=',' read -ra regions <<< "${providers[$provider]}"
        
        for region in "${regions[@]}"; do
            case $provider in
                "digitalocean")
                    if command -v doctl &> /dev/null; then
                        doctl compute droplet create "secure-$region" \
                            --size s-1vcpu-1gb \
                            --image ubuntu-20-04-x64 \
                            --region "$region" \
                            --ssh-keys "$SSH_KEY_ID" \
                            --wait
                    fi
                    ;;
                "vultr")
                    if command -v vultr-cli &> /dev/null; then
                        vultr-cli instance create \
                            --region "$region" \
                            --plan vc2-1c-1gb \
                            --os 387 \
                            --label "secure-$region"
                    fi
                    ;;
                "linode")
                    if [ -n "$LINODE_TOKEN" ]; then
                        curl -H "Authorization: Bearer $LINODE_TOKEN" \
                             -H "Content-Type: application/json" \
                             -X POST \
                             -d "{\"type\":\"g6-nanode-1\",\"region\":\"$region\",\"image\":\"linode/ubuntu20.04\",\"label\":\"secure-$region\"}" \
                             https://api.linode.com/v4/linode/instances
                    fi
                    ;;
            esac
            
            echo "✅ VPS créé dans $region"
            sleep 2
        done
    done
}

# OPTION 3: Configuration proxies résidentiels
setup_residential_proxies() {
    echo "🏠 Configuration proxies résidentiels..."
    
    # Bright Data
    if [ -n "$BRIGHT_DATA_USER" ] && [ -n "$BRIGHT_DATA_PASS" ]; then
        cat > "$PROXY_DIR/brightdata.conf" << EOF
# Bright Data Configuration
proxy_host=zproxy.lum-superproxy.io
proxy_port=22225
username=$BRIGHT_DATA_USER
password=$BRIGHT_DATA_PASS
rotation_interval=600
country_pool=US,CA,GB,DE,FR,NL,SE,CH
EOF
        echo "✅ Bright Data configuré"
    fi
    
    # Smartproxy
    if [ -n "$SMART_PROXY_USER" ] && [ -n "$SMART_PROXY_PASS" ]; then
        cat > "$PROXY_DIR/smartproxy.conf" << EOF
# Smartproxy Configuration  
proxy_host=gate.smartproxy.com
proxy_port=10000
username=$SMART_PROXY_USER
password=$SMART_PROXY_PASS
rotation_interval=300
sticky_session=false
EOF
        echo "✅ Smartproxy configuré"
    fi
    
    # ProxyEmpire
    if [ -n "$PROXY_EMPIRE_USER" ] && [ -n "$PROXY_EMPIRE_PASS" ]; then
        cat > "$PROXY_DIR/proxyempire.conf" << EOF
# ProxyEmpire Configuration
proxy_host=rotating-residential.proxies.proxyempire.io
proxy_port=9000
username=$PROXY_EMPIRE_USER
password=$PROXY_EMPIRE_PASS
rotation_interval=180
geo_targeting=true
EOF
        echo "✅ ProxyEmpire configuré"
    fi
}

# Test de connectivité
test_ip_connectivity() {
    echo "🧪 Test de connectivité des IPs..."
    
    > "$IP_LIST"
    
    # Tester les configs VPN
    if [ -d "$VPN_DIR" ] && [ "$(ls -A "$VPN_DIR")" ]; then
        for config in "$VPN_DIR"/*.ovpn; do
            if [ -f "$config" ]; then
                echo "🔍 Test $(basename "$config")..."
                
                # Démarrer VPN temporairement
                timeout 30 openvpn --config "$config" --daemon --log /tmp/vpn_test.log
                sleep 10
                
                # Obtenir l'IP
                current_ip=$(curl -s --max-time 5 https://ipinfo.io/ip 2>/dev/null)
                
                if [ -n "$current_ip" ] && [[ "$current_ip" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
                    echo "$current_ip|$(basename "$config")" >> "$IP_LIST"
                    echo "✅ IP active: $current_ip"
                else
                    echo "❌ Échec: $(basename "$config")"
                fi
                
                # Arrêter VPN
                pkill openvpn
                sleep 2
            fi
        done
    fi
    
    # Statistiques finales
    total_ips=$(wc -l < "$IP_LIST" 2>/dev/null || echo 0)
    echo ""
    echo "📊 RÉSUMÉ:"
    echo "✅ IPs configurées: $total_ips"
    echo "✅ Fichier de mapping: $IP_LIST"
    
    if [ "$total_ips" -ge 20 ]; then
        echo "🎉 Configuration réussie! Rotation disponible."
    else
        echo "⚠️  Moins de 20 IPs disponibles. Ajoutez plus de sources."
    fi
}

# Génération du fichier de rotation
generate_rotation_config() {
    echo "⚙️ Génération de la configuration de rotation..."
    
    cat > /etc/ip-rotation.conf << EOF
# Configuration de rotation automatique
ROTATION_INTERVAL=3600
EMERGENCY_ROTATION_TRIGGER=10
MIN_IPS_REQUIRED=20
CURRENT_IP_FILE=/tmp/current_ip.txt
IP_HISTORY_FILE=/var/log/ip_history.log
BLACKLIST_CHECK_INTERVAL=300

# Sources d'IPs
VPN_CONFIG_DIR=$VPN_DIR
PROXY_CONFIG_DIR=$PROXY_DIR
IP_LIST_FILE=$IP_LIST

# Monitoring
ENABLE_MONITORING=true
ALERT_WEBHOOK_URL=$ALERT_WEBHOOK
TELEGRAM_BOT_TOKEN=$TELEGRAM_TOKEN
TELEGRAM_CHAT_ID=$TELEGRAM_CHAT
EOF
    
    echo "✅ Configuration de rotation générée"
}

# Installation des dépendances
install_dependencies() {
    echo "📦 Installation des dépendances..."
    
    # Détection de l'OS
    if [ -f /etc/debian_version ]; then
        apt-get update -q
        apt-get install -y openvpn curl wget unzip jq netcat-openbsd
    elif [ -f /etc/redhat-release ]; then
        yum update -q -y
        yum install -y openvpn curl wget unzip jq netcat
    elif [ -f /etc/alpine-release ]; then
        apk update -q
        apk add --no-cache openvpn curl wget unzip jq netcat-openbsd
    fi
    
    echo "✅ Dépendances installées"
}

# Menu principal
main_menu() {
    echo ""
    echo "🎯 CHOISISSEZ VOTRE MÉTHODE D'IPs:"
    echo "1) NordVPN (Recommandé - 3€/mois)"
    echo "2) ExpressVPN (Premium - 6€/mois)" 
    echo "3) VPS Multiples (Propre - 100€/mois)"
    echo "4) Proxies Résidentiels (Ultra-propre - 500€/mois)"
    echo "5) Configuration manuelle"
    echo "0) Quitter"
    echo ""
    read -p "Votre choix (1-5): " choice
    
    case $choice in
        1)
            echo "📋 NORDVPN SÉLECTIONNÉ"
            echo "1. Créez un compte sur https://nordvpn.com"
            echo "2. Téléchargez les configs: https://nordvpn.com/ovpn/"
            echo "3. Placez les fichiers .ovpn dans: $VPN_DIR"
            read -p "Continuer automatiquement? (y/n): " auto
            if [[ $auto == "y" ]]; then
                setup_nordvpn
            fi
            ;;
        2)
            echo "📋 EXPRESSVPN SÉLECTIONNÉ"
            echo "1. Créez un compte sur https://expressvpn.com"
            echo "2. Installez le client: expressvpn setup"
            echo "3. Configurez les serveurs manuellement"
            ;;
        3)
            echo "📋 VPS MULTIPLES SÉLECTIONNÉS"
            echo "Configuration des APIs nécessaires:"
            echo "- DigitalOcean: export DIGITALOCEAN_ACCESS_TOKEN=your_token"
            echo "- Vultr: export VULTR_API_KEY=your_key"  
            echo "- Linode: export LINODE_TOKEN=your_token"
            read -p "APIs configurées? Continuer? (y/n): " apis
            if [[ $apis == "y" ]]; then
                setup_multiple_vps
            fi
            ;;
        4)
            echo "📋 PROXIES RÉSIDENTIELS SÉLECTIONNÉS"
            echo "Variables d'environnement nécessaires:"
            echo "- export BRIGHT_DATA_USER=your_user"
            echo "- export BRIGHT_DATA_PASS=your_pass"
            echo "- export SMART_PROXY_USER=your_user"
            echo "- export SMART_PROXY_PASS=your_pass"
            read -p "Variables configurées? Continuer? (y/n): " vars
            if [[ $vars == "y" ]]; then
                setup_residential_proxies
            fi
            ;;
        5)
            echo "📋 CONFIGURATION MANUELLE"
            echo "Placez vos fichiers .ovpn dans: $VPN_DIR"
            echo "Placez vos configs proxy dans: $PROXY_DIR"
            ;;
        0)
            echo "👋 Au revoir!"
            exit 0
            ;;
        *)
            echo "❌ Choix invalide"
            main_menu
            ;;
    esac
}

# Point d'entrée principal
main() {
    echo "🚀 DÉMARRAGE CONFIGURATION IPs..."
    
    # Vérifier les permissions
    if [[ $EUID -ne 0 ]]; then
        echo "❌ Ce script doit être exécuté en root"
        exit 1
    fi
    
    install_dependencies
    main_menu
    test_ip_connectivity
    generate_rotation_config
    
    echo ""
    echo "🎉 CONFIGURATION TERMINÉE!"
    echo "📁 Configs VPN: $VPN_DIR"
    echo "📁 Configs Proxy: $PROXY_DIR"
    echo "📄 Liste IPs: $IP_LIST"
    echo "⚙️ Config rotation: /etc/ip-rotation.conf"
    echo ""
    echo "🚀 Démarrez maintenant: docker-compose up -d"
}

# Exécution si script appelé directement
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi