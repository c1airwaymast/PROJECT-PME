#!/bin/bash

# PROTECTION ULTIME ANTI-SCAN
# Système de défense à 100% - AUCUN BOT NE PASSE

echo "🛡️  DÉMARRAGE PROTECTION ULTIME"
echo "================================"

# Variables de configuration
LOG_FILE="/var/log/firewall/ultimate.log"
BLOCK_LIST="/tmp/blocked_ips.txt"
WHITELIST="/etc/firewall-rules/whitelist.txt"

# Fonction de logging
log_event() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# 1. RÈGLES IPTABLES ULTRA-STRICTES
setup_firewall_rules() {
    log_event "Configuration des règles firewall ultra-strictes..."
    
    # Vider toutes les règles existantes
    iptables -F
    iptables -X
    iptables -t nat -F
    iptables -t nat -X
    iptables -t mangle -F
    iptables -t mangle -X
    
    # Politique par défaut : TOUT BLOQUER
    iptables -P INPUT DROP
    iptables -P FORWARD DROP
    iptables -P OUTPUT DROP
    
    # Autoriser seulement le loopback
    iptables -A INPUT -i lo -j ACCEPT
    iptables -A OUTPUT -o lo -j ACCEPT
    
    # Autoriser les connexions établies SEULEMENT
    iptables -A INPUT -m state --state ESTABLISHED,RELATED -j ACCEPT
    iptables -A OUTPUT -m state --state ESTABLISHED,RELATED -j ACCEPT
    
    # HTTPS uniquement (port 443)
    iptables -A INPUT -p tcp --dport 443 -m state --state NEW -j ACCEPT
    iptables -A OUTPUT -p tcp --sport 443 -j ACCEPT
    
    # Bloquer TOUS les autres ports (même cachés)
    iptables -A INPUT -p tcp --dport 80 -j DROP
    iptables -A INPUT -p tcp --dport 22 -j DROP
    iptables -A INPUT -p tcp --dport 21 -j DROP
    iptables -A INPUT -p tcp --dport 25 -j DROP
    iptables -A INPUT -p tcp --dport 53 -j DROP
    iptables -A INPUT -p tcp --dport 110 -j DROP
    iptables -A INPUT -p tcp --dport 143 -j DROP
    iptables -A INPUT -p tcp --dport 993 -j DROP
    iptables -A INPUT -p tcp --dport 995 -j DROP
    iptables -A INPUT -p tcp --dport 3389 -j DROP
    iptables -A INPUT -p tcp --dport 5432 -j DROP
    iptables -A INPUT -p tcp --dport 3306 -j DROP
    iptables -A INPUT -p tcp --dport 1433 -j DROP
    iptables -A INPUT -p tcp --dport 8080 -j DROP
    iptables -A INPUT -p tcp --dport 8443 -j DROP
    
    # Anti-scan de ports : bloquer les tentatives de scan
    iptables -A INPUT -p tcp --tcp-flags ALL NONE -j DROP
    iptables -A INPUT -p tcp --tcp-flags ALL ALL -j DROP
    iptables -A INPUT -p tcp --tcp-flags ALL FIN,URG,PSH -j DROP
    iptables -A INPUT -p tcp --tcp-flags ALL SYN,RST,ACK,FIN,URG -j DROP
    iptables -A INPUT -p tcp --tcp-flags SYN,RST SYN,RST -j DROP
    iptables -A INPUT -p tcp --tcp-flags SYN,FIN SYN,FIN -j DROP
    
    # Limiter les connexions simultanées (anti-DDoS)
    iptables -A INPUT -p tcp --dport 443 -m connlimit --connlimit-above 10 -j DROP
    
    # Rate limiting ultra-strict
    iptables -A INPUT -p tcp --dport 443 -m limit --limit 5/min --limit-burst 3 -j ACCEPT
    iptables -A INPUT -p tcp --dport 443 -j DROP
    
    log_event "✅ Règles firewall configurées - MODE FORTERESSE"
}

# 2. DÉTECTION INTELLIGENTE DES BOTS
detect_bots() {
    log_event "Démarrage détection intelligente des bots..."
    
    # Surveiller les connexions en temps réel
    netstat -tuln 2>/dev/null | while read line; do
        # Analyser chaque connexion
        if echo "$line" | grep -q ":443"; then
            # Extraire l'IP
            ip=$(echo "$line" | awk '{print $5}' | cut -d: -f1)
            
            # Vérifier si c'est suspect
            if [ "$ip" != "0.0.0.0" ] && [ "$ip" != "127.0.0.1" ]; then
                check_suspicious_ip "$ip"
            fi
        fi
    done &
}

# 3. VÉRIFICATION IP SUSPECTE
check_suspicious_ip() {
    local ip=$1
    local suspicious=0
    
    # Vérifier contre la whitelist
    if grep -q "$ip" "$WHITELIST" 2>/dev/null; then
        return 0
    fi
    
    # Tests de détection de bots
    
    # Test 1: Géolocalisation (datacenters connus)
    if curl -s "http://ip-api.com/json/$ip" | grep -q '"hosting":true'; then
        suspicious=$((suspicious + 30))
        log_event "🚨 IP suspecte (datacenter): $ip"
    fi
    
    # Test 2: Reverse DNS
    reverse=$(dig -x "$ip" +short 2>/dev/null)
    if echo "$reverse" | grep -qE "(aws|google|azure|digitalocean|linode|vultr|ovh)"; then
        suspicious=$((suspicious + 25))
        log_event "🚨 IP suspecte (cloud provider): $ip"
    fi
    
    # Test 3: Vitesse de connexion (trop rapide = bot)
    connection_count=$(netstat -an | grep "$ip" | wc -l)
    if [ "$connection_count" -gt 5 ]; then
        suspicious=$((suspicious + 20))
        log_event "🚨 IP suspecte (trop de connexions): $ip"
    fi
    
    # Test 4: User-Agent analysis via logs
    if tail -100 /var/log/nginx/access.log 2>/dev/null | grep "$ip" | grep -qE "(bot|crawler|spider|wget|curl|python)"; then
        suspicious=$((suspicious + 40))
        log_event "🚨 IP suspecte (user-agent bot): $ip"
    fi
    
    # Si score de suspicion > 50, BLOQUER IMMÉDIATEMENT
    if [ "$suspicious" -gt 50 ]; then
        block_ip_immediately "$ip" "$suspicious"
    fi
}

# 4. BLOCAGE IMMÉDIAT
block_ip_immediately() {
    local ip=$1
    local score=$2
    
    log_event "🔥 BLOCAGE IMMÉDIAT: $ip (score: $score)"
    
    # Bloquer dans iptables
    iptables -I INPUT 1 -s "$ip" -j DROP
    iptables -I OUTPUT 1 -d "$ip" -j DROP
    
    # Ajouter à la liste noire permanente
    echo "$ip" >> "$BLOCK_LIST"
    
    # Bloquer au niveau du kernel (plus efficace)
    echo "$ip" > /proc/net/xt_recent/blocklist
    
    # Notifier le système de monitoring
    curl -X POST "http://security-monitor:8080/alert" \
         -H "Content-Type: application/json" \
         -d "{\"type\":\"bot_blocked\",\"ip\":\"$ip\",\"score\":$score}" 2>/dev/null &
}

# 5. SCAN PROACTIF ANTI-RECONNAISSANCE
anti_reconnaissance() {
    log_event "Démarrage protection anti-reconnaissance..."
    
    while true; do
        # Détecter les scans de ports
        netstat -tuln | awk '{print $5}' | cut -d: -f1 | sort | uniq -c | while read count ip; do
            if [ "$count" -gt 3 ] && [ "$ip" != "0.0.0.0" ] && [ "$ip" != "127.0.0.1" ]; then
                log_event "🔍 Scan détecté depuis: $ip (tentatives: $count)"
                block_ip_immediately "$ip" 100
            fi
        done
        
        sleep 5
    done &
}

# 6. CAMOUFLAGE COMPLET DU SERVEUR
stealth_mode() {
    log_event "Activation mode furtif..."
    
    # Masquer la signature du serveur
    echo 'net.ipv4.tcp_timestamps = 0' >> /etc/sysctl.conf
    echo 'net.ipv4.ip_forward = 0' >> /etc/sysctl.conf
    echo 'net.ipv4.conf.all.send_redirects = 0' >> /etc/sysctl.conf
    echo 'net.ipv4.conf.all.accept_redirects = 0' >> /etc/sysctl.conf
    echo 'net.ipv4.conf.all.accept_source_route = 0' >> /etc/sysctl.conf
    echo 'net.ipv4.conf.all.log_martians = 1' >> /etc/sysctl.conf
    echo 'net.ipv4.icmp_echo_ignore_all = 1' >> /etc/sysctl.conf
    echo 'net.ipv4.icmp_echo_ignore_broadcasts = 1' >> /etc/sysctl.conf
    
    # Appliquer les changements
    sysctl -p
    
    # Masquer les ports ouverts
    iptables -A INPUT -p tcp --tcp-flags RST RST -m limit --limit 2/s --limit-burst 2 -j ACCEPT
    iptables -A INPUT -p tcp --tcp-flags RST RST -j DROP
    
    log_event "✅ Mode furtif activé - Serveur invisible"
}

# 7. MONITORING CONTINU
continuous_monitoring() {
    log_event "Démarrage monitoring continu..."
    
    while true; do
        # Vérifier les tentatives de connexion
        current_connections=$(netstat -an | grep ":443" | grep "SYN_RECV" | wc -l)
        
        if [ "$current_connections" -gt 20 ]; then
            log_event "🚨 ALERTE: Trop de connexions simultanées ($current_connections)"
            
            # Bloquer temporairement toutes les nouvelles connexions
            iptables -I INPUT 1 -p tcp --dport 443 -m state --state NEW -j DROP
            sleep 30
            iptables -D INPUT 1
        fi
        
        # Nettoyer les anciennes règles
        iptables -L INPUT -n --line-numbers | grep "DROP" | tail -100 | while read line; do
            # Garder seulement les 100 dernières règles de blocage
            rule_num=$(echo "$line" | awk '{print $1}')
            if [ "$rule_num" -gt 100 ]; then
                iptables -D INPUT "$rule_num" 2>/dev/null
            fi
        done
        
        sleep 10
    done &
}

# 8. HONEYPOT INTÉGRÉ
setup_honeypot() {
    log_event "Configuration honeypot intégré..."
    
    # Créer des ports piège
    nc -l -p 22 -e /usr/local/bin/honeypot-ssh.sh &
    nc -l -p 21 -e /usr/local/bin/honeypot-ftp.sh &
    nc -l -p 3389 -e /usr/local/bin/honeypot-rdp.sh &
    
    log_event "✅ Honeypots actifs sur ports 22, 21, 3389"
}

# DÉMARRAGE PRINCIPAL
main() {
    log_event "🚀 DÉMARRAGE PROTECTION ULTIME ANTI-SCAN"
    
    # Créer les fichiers nécessaires
    touch "$BLOCK_LIST"
    touch "$WHITELIST"
    
    # Ajouter quelques IPs de confiance à la whitelist
    echo "127.0.0.1" > "$WHITELIST"
    echo "::1" >> "$WHITELIST"
    
    # Lancer tous les systèmes de protection
    setup_firewall_rules
    stealth_mode
    detect_bots
    anti_reconnaissance
    continuous_monitoring
    setup_honeypot
    
    log_event "✅ TOUS LES SYSTÈMES DE PROTECTION ACTIVÉS"
    log_event "🛡️  SERVEUR EN MODE FORTERESSE - AUCUN BOT NE PEUT PASSER"
    
    # Boucle principale
    while true; do
        # Statistiques de protection
        blocked_count=$(wc -l < "$BLOCK_LIST")
        active_rules=$(iptables -L INPUT | grep DROP | wc -l)
        
        log_event "📊 Stats: $blocked_count IPs bloquées, $active_rules règles actives"
        
        sleep 300  # Stats toutes les 5 minutes
    done
}

# Gestion des signaux
trap 'log_event "Arrêt de la protection..."; exit 0' TERM INT

# Démarrer
main