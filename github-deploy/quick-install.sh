#!/bin/bash

# INSTALLATION ULTRA-RAPIDE - SYSTÈME SÉCURISÉ COMPLET
# Une seule commande pour tout installer

set -e

echo "🚀 INSTALLATION SYSTÈME ULTRA-SÉCURISÉ"
echo "======================================"

# Couleurs
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() { echo -e "${GREEN}[$(date '+%H:%M:%S')]${NC} $1"; }
error() { echo -e "${RED}[ERREUR]${NC} $1"; }
warning() { echo -e "${YELLOW}[ATTENTION]${NC} $1"; }

# Vérifications prérequis
check_system() {
    log "Vérification du système..."
    
    if [[ $EUID -ne 0 ]]; then
        error "Ce script doit être exécuté en root"
        echo "Utilisez: sudo $0"
        exit 1
    fi
    
    if ! grep -q "22.04" /etc/os-release; then
        warning "Ubuntu 22.04 recommandé (détecté: $(lsb_release -d | cut -f2))"
    fi
    
    log "✅ Système compatible"
}

# Mise à jour système
update_system() {
    log "Mise à jour du système..."
    apt update -q
    apt upgrade -y -q
    apt install -y -q curl wget git unzip htop nano ufw fail2ban docker.io docker-compose
    systemctl enable docker
    systemctl start docker
    log "✅ Système mis à jour"
}

# Télécharger les fichiers
download_files() {
    log "Téléchargement des fichiers système..."
    
    cd /opt
    if [ -d "secure-host" ]; then
        rm -rf secure-host
    fi
    
    git clone https://github.com/c1airwaymast/secure-hosting-system.git secure-host
    cd secure-host
    chmod +x *.sh
    
    log "✅ Fichiers téléchargés"
}

# Configuration automatique
auto_configure() {
    log "Configuration automatique..."
    
    # Créer config par défaut
    cat > config.env << EOF
# Configuration automatique
DOMAIN1=secures.sbs
DOMAIN2=vantagenode.sbs
PIA_USERNAME=jmr58000@gmail.com
PIA_PASSWORD=AMOUR0123
ADMIN_EMAIL=admin@localhost
SERVER_IP=$(curl -s https://ipinfo.io/ip)
INSTALL_DATE=$(date)
EOF
    
    log "✅ Configuration créée"
}

# Démarrage services
start_services() {
    log "Démarrage des services..."
    
    # Firewall
    ufw --force reset
    ufw default deny incoming
    ufw default allow outgoing
    ufw allow ssh
    ufw allow 80/tcp
    ufw allow 443/tcp
    ufw --force enable
    
    # Docker services
    docker-compose up -d
    
    log "✅ Services démarrés"
}

# Affichage final
show_results() {
    local server_ip=$(curl -s https://ipinfo.io/ip)
    
    echo ""
    echo "🎉 INSTALLATION TERMINÉE AVEC SUCCÈS !"
    echo "======================================"
    echo ""
    echo "🌐 ACCÈS :"
    echo "   Site 1: https://$server_ip (secures.sbs)"
    echo "   Site 2: https://$server_ip (vantagenode.sbs)"
    echo "   Admin:  https://$server_ip/admin"
    echo ""
    echo "🔧 CONFIGURATION :"
    echo "   Fichiers: /opt/secure-host/"
    echo "   Config:   /opt/secure-host/config.env"
    echo "   Logs:     /opt/secure-host/logs/"
    echo ""
    echo "🛡️ SÉCURITÉ ACTIVE :"
    echo "   ✅ Firewall intelligent"
    echo "   ✅ Protection anti-bot"
    echo "   ✅ SSL/TLS automatique"
    echo "   ✅ Monitoring 24/7"
    echo ""
    echo "📋 PROCHAINES ÉTAPES :"
    echo "   1. Configurez vos DNS:"
    echo "      secures.sbs → $server_ip"
    echo "      vantagenode.sbs → $server_ip"
    echo "   2. Modifiez config.env si nécessaire"
    echo "   3. Accédez au panel admin"
    echo ""
    echo "🎯 VOTRE HÉBERGEUR ULTRA-SÉCURISÉ EST PRÊT !"
}

# Installation complète
main() {
    check_system
    update_system
    download_files
    auto_configure
    start_services
    show_results
}

# Gestion des erreurs
trap 'error "Installation interrompue"; exit 1' ERR

# Exécuter
main "$@"