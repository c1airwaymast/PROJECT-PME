# HÉBERGEUR WEB ULTRA-SÉCURISÉ
# Support: PHP, Node.js, Python, HTML/CSS/JS

FROM node:18-alpine

# Installer les environnements multiples
RUN apk add --no-cache \
    php81 \
    php81-fpm \
    php81-mysqli \
    php81-pdo \
    php81-pdo_mysql \
    php81-pdo_pgsql \
    php81-json \
    php81-openssl \
    php81-curl \
    php81-zlib \
    php81-xml \
    php81-phar \
    php81-intl \
    php81-dom \
    php81-xmlreader \
    php81-ctype \
    php81-session \
    php81-mbstring \
    php81-gd \
    python3 \
    python3-pip \
    nginx \
    supervisor \
    curl \
    wget \
    git \
    unzip \
    imagemagick \
    ffmpeg

# Installer Composer pour PHP
RUN curl -sS https://getcomposer.org/installer | php -- --install-dir=/usr/local/bin --filename=composer

# Installer PM2 pour Node.js
RUN npm install -g pm2

# Créer utilisateur non-root
RUN addgroup -g 1000 webuser && \
    adduser -D -s /bin/sh -u 1000 -G webuser webuser

# Configuration Nginx sécurisée
COPY nginx.conf /etc/nginx/nginx.conf
COPY php-fpm.conf /etc/php81/php-fpm.conf
COPY supervisord.conf /etc/supervisor/conf.d/supervisord.conf

# Scripts de démarrage
COPY start-services.sh /usr/local/bin/
COPY health-check.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/*.sh

# Répertoires de travail
RUN mkdir -p /var/www/html /var/log/web /tmp/uploads && \
    chown -R webuser:webuser /var/www/html /var/log/web /tmp/uploads

# Permissions sécurisées
RUN chmod 755 /var/www/html && \
    chmod 750 /var/log/web

# Variables d'environnement
ENV NODE_ENV=production
ENV PHP_FPM_USER=webuser
ENV PHP_FPM_GROUP=webuser

# Exposer les ports
EXPOSE 8080

# Healthcheck
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD /usr/local/bin/health-check.sh

# Utilisateur non-root
USER webuser

# Point d'entrée
ENTRYPOINT ["/usr/local/bin/start-services.sh"]