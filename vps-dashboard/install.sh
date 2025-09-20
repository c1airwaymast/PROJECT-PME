#!/bin/bash

# 🔥 INSTALLATION ULTRA-RAPIDE DU DASHBOARD VPS 🔥

set -e

echo "🚀 Installation du Dashboard VPS Ultra-Performant..."
echo "================================================="

# Couleurs pour l'affichage
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Vérification des droits
if [[ $EUID -ne 0 ]]; then
   print_warning "Ce script peut nécessiter des droits sudo pour certaines opérations."
fi

# Détection de l'OS
if [[ -f /etc/os-release ]]; then
    . /etc/os-release
    OS=$NAME
    VER=$VERSION_ID
else
    print_error "Impossible de détecter l'OS"
    exit 1
fi

print_status "OS détecté: $OS $VER"

# Installation de Python et pip si nécessaire
print_status "Vérification de Python..."
if ! command -v python3 &> /dev/null; then
    print_status "Installation de Python3..."
    if [[ "$OS" == *"Ubuntu"* ]] || [[ "$OS" == *"Debian"* ]]; then
        sudo apt update
        sudo apt install -y python3 python3-pip python3-venv
    elif [[ "$OS" == *"CentOS"* ]] || [[ "$OS" == *"Red Hat"* ]]; then
        sudo yum install -y python3 python3-pip
    elif [[ "$OS" == *"Fedora"* ]]; then
        sudo dnf install -y python3 python3-pip
    else
        print_error "OS non supporté pour l'installation automatique"
        exit 1
    fi
else
    print_success "Python3 déjà installé"
fi

# Vérification de pip
if ! command -v pip3 &> /dev/null; then
    print_status "Installation de pip3..."
    curl https://bootstrap.pypa.io/get-pip.py | python3
else
    print_success "pip3 déjà installé"
fi

# Création de l'environnement virtuel
print_status "Création de l'environnement virtuel..."
cd /workspace/vps-dashboard
python3 -m venv venv
source venv/bin/activate

# Installation des dépendances
print_status "Installation des dépendances Python..."
pip install --upgrade pip
pip install -r requirements.txt

# Installation des dépendances système supplémentaires
print_status "Installation des dépendances système..."
if [[ "$OS" == *"Ubuntu"* ]] || [[ "$OS" == *"Debian"* ]]; then
    sudo apt install -y htop iotop nethogs lsof
elif [[ "$OS" == *"CentOS"* ]] || [[ "$OS" == *"Red Hat"* ]]; then
    sudo yum install -y htop iotop nethogs lsof
elif [[ "$OS" == *"Fedora"* ]]; then
    sudo dnf install -y htop iotop nethogs lsof
fi

# Création du service systemd
print_status "Création du service systemd..."
sudo tee /etc/systemd/system/vps-dashboard.service > /dev/null <<EOF
[Unit]
Description=VPS Dashboard Ultra-Performant
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=/workspace/vps-dashboard
Environment=PATH=/workspace/vps-dashboard/venv/bin
ExecStart=/workspace/vps-dashboard/venv/bin/python backend/server.py
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
EOF

# Activation du service
print_status "Activation du service..."
sudo systemctl daemon-reload
sudo systemctl enable vps-dashboard
sudo systemctl start vps-dashboard

# Configuration du firewall (si ufw est installé)
if command -v ufw &> /dev/null; then
    print_status "Configuration du firewall..."
    sudo ufw allow 8765/tcp comment "VPS Dashboard WebSocket"
    sudo ufw allow 8080/tcp comment "VPS Dashboard HTTP"
fi

# Création du serveur HTTP simple pour servir les fichiers statiques
print_status "Création du serveur HTTP..."
cat > start_http_server.py << 'EOF'
#!/usr/bin/env python3
import http.server
import socketserver
import os

PORT = 8080
DIRECTORY = "/workspace/vps-dashboard"

class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=DIRECTORY, **kwargs)

os.chdir(DIRECTORY)
with socketserver.TCPServer(("", PORT), Handler) as httpd:
    print(f"🌐 Serveur HTTP démarré sur http://localhost:{PORT}")
    print(f"📁 Répertoire: {DIRECTORY}")
    httpd.serve_forever()
EOF

chmod +x start_http_server.py

# Création du service HTTP
sudo tee /etc/systemd/system/vps-dashboard-http.service > /dev/null <<EOF
[Unit]
Description=VPS Dashboard HTTP Server
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=/workspace/vps-dashboard
ExecStart=/usr/bin/python3 /workspace/vps-dashboard/start_http_server.py
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable vps-dashboard-http
sudo systemctl start vps-dashboard-http

# Script de démarrage rapide
cat > start.sh << 'EOF'
#!/bin/bash
echo "🔥 Démarrage du Dashboard VPS Ultra-Performant..."

# Démarrage des services
sudo systemctl start vps-dashboard
sudo systemctl start vps-dashboard-http

# Attendre que les services démarrent
sleep 3

# Vérification du statut
echo "📊 Statut des services:"
sudo systemctl status vps-dashboard --no-pager -l
sudo systemctl status vps-dashboard-http --no-pager -l

# Affichage des informations de connexion
echo ""
echo "🌐 Dashboard disponible sur:"
echo "   - Interface Web: http://$(hostname -I | awk '{print $1}'):8080/frontend/"
echo "   - WebSocket: ws://$(hostname -I | awk '{print $1}'):8765"
echo ""
echo "🔧 Commandes utiles:"
echo "   - Arrêter: sudo systemctl stop vps-dashboard vps-dashboard-http"
echo "   - Redémarrer: sudo systemctl restart vps-dashboard vps-dashboard-http"
echo "   - Logs: sudo journalctl -u vps-dashboard -f"
echo "   - Debug: sudo journalctl -u vps-dashboard-http -f"
EOF

chmod +x start.sh

# Script d'arrêt
cat > stop.sh << 'EOF'
#!/bin/bash
echo "🛑 Arrêt du Dashboard VPS..."
sudo systemctl stop vps-dashboard
sudo systemctl stop vps-dashboard-http
echo "✅ Services arrêtés"
EOF

chmod +x stop.sh

# Script de désinstallation
cat > uninstall.sh << 'EOF'
#!/bin/bash
echo "🗑️ Désinstallation du Dashboard VPS..."

# Arrêt et désactivation des services
sudo systemctl stop vps-dashboard vps-dashboard-http
sudo systemctl disable vps-dashboard vps-dashboard-http

# Suppression des services
sudo rm -f /etc/systemd/system/vps-dashboard.service
sudo rm -f /etc/systemd/system/vps-dashboard-http.service
sudo systemctl daemon-reload

# Suppression des règles firewall
if command -v ufw &> /dev/null; then
    sudo ufw delete allow 8765/tcp
    sudo ufw delete allow 8080/tcp
fi

echo "✅ Dashboard désinstallé"
echo "💡 Pour supprimer complètement, supprimez le répertoire /workspace/vps-dashboard"
EOF

chmod +x uninstall.sh

# Optimisations système
print_status "Application des optimisations système..."

# Augmentation des limites de fichiers ouverts
echo "* soft nofile 65536" | sudo tee -a /etc/security/limits.conf
echo "* hard nofile 65536" | sudo tee -a /etc/security/limits.conf

# Optimisations réseau
sudo tee -a /etc/sysctl.conf > /dev/null <<EOF

# Optimisations VPS Dashboard
net.core.somaxconn = 65535
net.core.netdev_max_backlog = 5000
net.ipv4.tcp_max_syn_backlog = 65535
net.ipv4.tcp_fin_timeout = 30
net.ipv4.tcp_keepalive_time = 600
EOF

sudo sysctl -p

# Création d'un cron pour le nettoyage automatique
print_status "Configuration du nettoyage automatique..."
(crontab -l 2>/dev/null; echo "0 2 * * * find /workspace/vps-dashboard -name '*.log' -mtime +7 -delete") | crontab -

# Test de connectivité
print_status "Test des services..."
sleep 5

if systemctl is-active --quiet vps-dashboard; then
    print_success "✅ Service backend actif"
else
    print_error "❌ Service backend inactif"
fi

if systemctl is-active --quiet vps-dashboard-http; then
    print_success "✅ Service HTTP actif"
else
    print_error "❌ Service HTTP inactif"
fi

# Informations finales
echo ""
echo "🎉 INSTALLATION TERMINÉE AVEC SUCCÈS!"
echo "====================================="
echo ""
echo "🌐 Accès au Dashboard:"
echo "   URL: http://$(hostname -I | awk '{print $1}'):8080/frontend/"
echo ""
echo "🔧 Gestion des services:"
echo "   Démarrer: ./start.sh"
echo "   Arrêter: ./stop.sh"
echo "   Désinstaller: ./uninstall.sh"
echo ""
echo "📊 Fonctionnalités activées:"
echo "   ✅ Monitoring temps réel CPU/RAM/Disque/Réseau"
echo "   ✅ Mode debug ultra-performant"
echo "   ✅ Graphiques interactifs"
echo "   ✅ Alertes intelligentes"
echo "   ✅ Historique des métriques"
echo "   ✅ Interface responsive"
echo "   ✅ WebSocket temps réel"
echo "   ✅ Auto-démarrage au boot"
echo ""
echo "🚀 Le dashboard est maintenant opérationnel!"
echo "   Appuyez sur Ctrl+D pour activer le mode debug"
echo "   Utilisez F11 pour le mode plein écran"
echo ""
print_success "Profitez de votre monitoring ultra-performant! 🔥"