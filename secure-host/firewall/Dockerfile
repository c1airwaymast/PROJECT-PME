# FIREWALL INTELLIGENT ANTI-SCAN
FROM alpine:latest

# Installer les outils de sécurité réseau
RUN apk add --no-cache \
    iptables \
    ip6tables \
    ipset \
    fail2ban \
    nmap \
    tcpdump \
    netstat-nat \
    conntrack-tools \
    python3 \
    py3-pip \
    curl \
    jq \
    geoip \
    geoip-dev

# Installer les modules Python pour l'IA
RUN pip3 install --no-cache-dir \
    numpy \
    scikit-learn \
    pandas \
    requests \
    psutil

# Copier les scripts de firewall
COPY scripts/ /usr/local/bin/
COPY rules/ /etc/firewall-rules/
COPY ai-detection/ /opt/ai-detection/

# Rendre les scripts exécutables
RUN chmod +x /usr/local/bin/*.sh && \
    chmod +x /opt/ai-detection/*.py

# Créer les répertoires de logs
RUN mkdir -p /var/log/firewall /var/log/intrusion

# Script principal
ENTRYPOINT ["/usr/local/bin/start-firewall.sh"]