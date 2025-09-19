#!/bin/bash

echo "🛡️  INSTALLATION DU SYSTÈME ANTI-BOT"
echo "====================================="

# Vérifier si Node.js est installé
if ! command -v node &> /dev/null; then
    echo "❌ Node.js n'est pas installé. Installez-le depuis https://nodejs.org"
    exit 1
fi

# Installer Wrangler
echo "📦 Installation de Wrangler..."
npm install -g wrangler

# Vérifier l'installation
if ! command -v wrangler &> /dev/null; then
    echo "❌ Erreur lors de l'installation de Wrangler"
    exit 1
fi

echo "✅ Wrangler installé avec succès"

# Se connecter à Cloudflare
echo "🔑 Connexion à Cloudflare..."
echo "Une page web va s'ouvrir pour vous authentifier"
wrangler login

# Vérifier la connexion
echo "👤 Vérification de la connexion..."
wrangler whoami

echo ""
echo "🎯 PROCHAINES ÉTAPES :"
echo "1. Modifiez le fichier 'wrangler.toml' avec votre domaine"
echo "2. Modifiez TARGET_URL dans wrangler.toml avec l'URL de votre vrai site"
echo "3. Lancez : wrangler deploy"
echo ""
echo "📖 Consultez DEPLOYMENT.md pour les détails complets"