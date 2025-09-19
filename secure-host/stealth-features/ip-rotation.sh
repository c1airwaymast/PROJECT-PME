#!/bin/bash

# ROTATION AUTOMATIQUE D'IP - INVISIBILITÉ TOTALE
# L'hébergeur change d'IP constamment pour rester introuvable

echo "🔄 SYSTÈME DE ROTATION D'IP ACTIVÉ"
echo "=================================="

# Configuration
VPN_CONFIGS="/etc/openvpn/configs"
CURRENT_IP_FILE="/tmp/current_ip.txt"
IP_HISTORY="/var/log/ip_history.log"
ROTATION_INTERVAL=3600  # 1 heure

# Fonction de logging
log_rotation() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$IP_HISTORY"
}

# Liste des serveurs VPN (ajoutez vos configs)
VPN_SERVERS=(
    "netherlands-01"
    "sweden-02"
    "switzerland-03"
    "romania-04"
    "bulgaria-05"
    "iceland-06"
    "norway-07"
    "finland-08"
    "estonia-09"
    "lithuania-10"
)

# Obtenir l'IP actuelle
get_current_ip() {
    curl -s https://ipinfo.io/ip 2>/dev/null || curl -s https://icanhazip.com 2>/dev/null
}

# Changer de serveur VPN
rotate_vpn() {
    local new_server=${VPN_SERVERS[$RANDOM % ${#VPN_SERVERS[@]}]}
    local old_ip=$(get_current_ip)
    
    log_rotation "🔄 Rotation vers serveur: $new_server"
    
    # Arrêter la connexion VPN actuelle
    pkill openvpn
    sleep 5
    
    # Démarrer nouvelle connexion VPN
    openvpn --config "$VPN_CONFIGS/$new_server.ovpn" --daemon
    sleep 10
    
    # Vérifier la nouvelle IP
    local new_ip=$(get_current_ip)
    
    if [ "$old_ip" != "$new_ip" ]; then
        log_rotation "✅ IP changée: $old_ip → $new_ip"
        echo "$new_ip" > "$CURRENT_IP_FILE"
        
        # Mettre à jour les règles firewall
        update_firewall_rules "$new_ip"
        
        # Notifier les autres services
        notify_ip_change "$old_ip" "$new_ip"
    else
        log_rotation "❌ Échec rotation IP, tentative suivante..."
    fi
}

# Mettre à jour les règles firewall avec la nouvelle IP
update_firewall_rules() {
    local new_ip=$1
    
    # Permettre le trafic sortant depuis la nouvelle IP
    iptables -D OUTPUT -s "$old_ip" -j ACCEPT 2>/dev/null
    iptables -A OUTPUT -s "$new_ip" -j ACCEPT
    
    log_rotation "🔧 Règles firewall mises à jour pour IP: $new_ip"
}

# Notifier les autres services du changement d'IP
notify_ip_change() {
    local old_ip=$1
    local new_ip=$2
    
    # Notifier le monitoring
    curl -X POST "http://security-monitor:8080/ip-changed" \
         -H "Content-Type: application/json" \
         -d "{\"old_ip\":\"$old_ip\",\"new_ip\":\"$new_ip\",\"timestamp\":\"$(date -Iseconds)\"}" 2>/dev/null &
    
    # Mettre à jour le DNS dynamique si configuré
    if [ -n "$DDNS_UPDATE_URL" ]; then
        curl -s "$DDNS_UPDATE_URL&myip=$new_ip" >/dev/null 2>&1 &
    fi
}

# Rotation automatique avec randomisation
auto_rotation() {
    while true; do
        # Intervalle randomisé (±30 minutes)
        random_offset=$((RANDOM % 3600))
        sleep_time=$((ROTATION_INTERVAL + random_offset - 1800))
        
        log_rotation "⏰ Prochaine rotation dans ${sleep_time}s"
        sleep "$sleep_time"
        
        # Vérifier si la rotation est nécessaire
        if should_rotate; then
            rotate_vpn
        fi
    done
}

# Déterminer si une rotation est nécessaire
should_rotate() {
    local current_ip=$(get_current_ip)
    
    # Toujours faire la rotation programmée
    if [ -f "$CURRENT_IP_FILE" ]; then
        local last_ip=$(cat "$CURRENT_IP_FILE")
        if [ "$current_ip" = "$last_ip" ]; then
            return 0  # Rotation nécessaire
        fi
    fi
    
    # Vérifier si l'IP est compromise (dans une blacklist)
    if check_ip_reputation "$current_ip"; then
        log_rotation "🚨 IP compromise détectée, rotation d'urgence"
        return 0
    fi
    
    return 1  # Pas de rotation nécessaire
}

# Vérifier la réputation de l'IP
check_ip_reputation() {
    local ip=$1
    
    # Vérifier contre plusieurs blacklists
    local blacklists=(
        "https://check.torproject.org/api/ip"
        "https://www.abuseipdb.com/check/$ip/json"
    )
    
    for blacklist in "${blacklists[@]}"; do
        if curl -s "$blacklist" | grep -q "\"malicious\":true\|\"tor\":true"; then
            return 0  # IP compromise
        fi
    done
    
    return 1  # IP propre
}

# Rotation d'urgence (appelée en cas de détection)
emergency_rotation() {
    log_rotation "🚨 ROTATION D'URGENCE DÉCLENCHÉE"
    
    # Changer immédiatement de serveur
    rotate_vpn
    
    # Changer aussi les ports d'écoute
    randomize_ports
    
    # Effacer les traces
    clear_traces
}

# Randomiser les ports d'écoute
randomize_ports() {
    local new_port=$((RANDOM % 10000 + 40000))  # Port entre 40000-50000
    
    log_rotation "🔀 Changement de port: 443 → $new_port"
    
    # Mettre à jour la configuration Nginx
    sed -i "s/listen 443/listen $new_port/g" /etc/nginx/nginx.conf
    
    # Redémarrer Nginx
    nginx -s reload
    
    # Mettre à jour le firewall
    iptables -D INPUT -p tcp --dport 443 -j ACCEPT 2>/dev/null
    iptables -A INPUT -p tcp --dport "$new_port" -j ACCEPT
}

# Effacer les traces de l'ancienne IP
clear_traces() {
    log_rotation "🧹 Effacement des traces..."
    
    # Vider les logs temporaires
    > /var/log/nginx/access.log
    > /var/log/nginx/error.log
    
    # Effacer l'historique des connexions
    > /proc/net/nf_conntrack 2>/dev/null
    
    # Redémarrer les services réseau
    systemctl restart networking 2>/dev/null
}

# Surveillance continue de la détection
monitor_detection() {
    while true; do
        # Vérifier si des scans sont détectés
        if tail -100 /var/log/firewall/ultimate.log | grep -q "Scan détecté"; then
            log_rotation "🚨 Scans détectés, rotation préventive"
            emergency_rotation
        fi
        
        # Vérifier la charge réseau (possible DDoS)
        network_load=$(cat /proc/net/dev | grep eth0 | awk '{print $2}' | tail -1)
        if [ "$network_load" -gt 1000000000 ]; then  # 1GB
            log_rotation "🚨 Charge réseau élevée, rotation d'urgence"
            emergency_rotation
        fi
        
        sleep 60  # Vérification chaque minute
    done &
}

# Configuration initiale
setup_rotation() {
    log_rotation "🚀 Configuration du système de rotation d'IP"
    
    # Créer les répertoires nécessaires
    mkdir -p "$VPN_CONFIGS"
    mkdir -p "$(dirname "$IP_HISTORY")"
    
    # Obtenir l'IP initiale
    local initial_ip=$(get_current_ip)
    echo "$initial_ip" > "$CURRENT_IP_FILE"
    log_rotation "📍 IP initiale: $initial_ip"
    
    # Vérifier que les configs VPN existent
    if [ ! -d "$VPN_CONFIGS" ] || [ -z "$(ls -A "$VPN_CONFIGS")" ]; then
        log_rotation "⚠️  ATTENTION: Aucune config VPN trouvée dans $VPN_CONFIGS"
        log_rotation "💡 Ajoutez vos fichiers .ovpn pour activer la rotation"
    fi
}

# Gestion des signaux
trap 'log_rotation "Arrêt du système de rotation..."; pkill openvpn; exit 0' TERM INT

# Point d'entrée principal
main() {
    setup_rotation
    monitor_detection
    auto_rotation
}

# Démarrer si exécuté directement
if [ "${BASH_SOURCE[0]}" = "${0}" ]; then
    main
fi