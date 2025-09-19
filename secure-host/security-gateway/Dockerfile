# GATEWAY DE SÉCURITÉ ULTRA-AVANCÉ
FROM nginx:alpine

# Installer les outils de sécurité
RUN apk add --no-cache \
    fail2ban \
    iptables \
    geoip \
    geoip-dev \
    openssl \
    curl \
    jq

# Copier la configuration sécurisée
COPY nginx.conf /etc/nginx/nginx.conf
COPY security-rules.conf /etc/nginx/conf.d/security.conf
COPY fail2ban/ /etc/fail2ban/
COPY scripts/ /usr/local/bin/

# Créer utilisateur non-root
RUN addgroup -g 1000 nginx-sec && \
    adduser -D -s /bin/sh -u 1000 -G nginx-sec nginx-sec

# Permissions sécurisées
RUN chmod +x /usr/local/bin/*.sh && \
    chown -R nginx-sec:nginx-sec /var/cache/nginx /var/log/nginx

# Supprimer les packages de build
RUN apk del geoip-dev

# Exposer seulement les ports nécessaires
EXPOSE 80 443

# Démarrer avec l'utilisateur non-root
USER nginx-sec

# Healthcheck
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost/health || exit 1

# Script de démarrage sécurisé
ENTRYPOINT ["/usr/local/bin/start-secure.sh"]