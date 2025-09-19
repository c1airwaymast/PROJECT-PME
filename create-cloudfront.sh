#!/bin/bash

# CRÉATION CLOUDFRONT AUTOMATIQUE
# Configuration complète en une commande

echo "☁️ CRÉATION CLOUDFRONT AUTOMATIQUE"
echo "=================================="

# Variables
TARGET_ORIGIN="airwaymast.org"
DOMAINS=("secures.sbs" "vantagenode.sbs")

# Vérifier AWS CLI
if ! command -v aws &> /dev/null; then
    echo "📦 Installation AWS CLI..."
    curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
    unzip awscliv2.zip
    sudo ./aws/install
fi

# Configuration AWS (si pas déjà fait)
echo "🔑 Vérification configuration AWS..."
if ! aws sts get-caller-identity &> /dev/null; then
    echo "Configuration AWS requise:"
    aws configure
fi

# Étape 1: Créer le certificat SSL
echo "🔒 Création certificat SSL..."
CERT_ARN=$(aws acm request-certificate \
    --domain-name "${DOMAINS[0]}" \
    --subject-alternative-names "${DOMAINS[1]}" \
    --validation-method DNS \
    --region us-east-1 \
    --query 'CertificateArn' \
    --output text)

echo "✅ Certificat demandé: $CERT_ARN"
echo "$CERT_ARN" > cert_arn.txt

# Attendre validation certificat
echo "⏳ Validation du certificat nécessaire..."
echo "📋 Allez dans AWS ACM Console pour valider:"
echo "   https://console.aws.amazon.com/acm/home?region=us-east-1"
echo ""
echo "Appuyez sur Entrée quand le certificat est validé..."
read

# Étape 2: Créer la distribution CloudFront
echo "☁️ Création distribution CloudFront..."

# Configuration JSON
cat > distribution-config.json << EOF
{
    "CallerReference": "redirect-$(date +%s)",
    "Comment": "Redirecteur automatique vers $TARGET_ORIGIN",
    "DefaultCacheBehavior": {
        "TargetOriginId": "origin-$TARGET_ORIGIN",
        "ViewerProtocolPolicy": "redirect-to-https",
        "AllowedMethods": {
            "Quantity": 7,
            "Items": ["GET", "HEAD", "OPTIONS", "PUT", "POST", "PATCH", "DELETE"],
            "CachedMethods": {
                "Quantity": 2,
                "Items": ["GET", "HEAD"]
            }
        },
        "ForwardedValues": {
            "QueryString": true,
            "Cookies": {
                "Forward": "all"
            },
            "Headers": {
                "Quantity": 3,
                "Items": ["Host", "CloudFront-Forwarded-Proto", "User-Agent"]
            }
        },
        "MinTTL": 0,
        "DefaultTTL": 0,
        "MaxTTL": 0,
        "Compress": true
    },
    "Origins": {
        "Quantity": 1,
        "Items": [
            {
                "Id": "origin-$TARGET_ORIGIN",
                "DomainName": "$TARGET_ORIGIN",
                "CustomOriginConfig": {
                    "HTTPPort": 80,
                    "HTTPSPort": 443,
                    "OriginProtocolPolicy": "http-only",
                    "OriginSslProtocols": {
                        "Quantity": 3,
                        "Items": ["TLSv1", "TLSv1.1", "TLSv1.2"]
                    }
                }
            }
        ]
    },
    "Aliases": {
        "Quantity": 2,
        "Items": ["${DOMAINS[0]}", "${DOMAINS[1]}"]
    },
    "ViewerCertificate": {
        "ACMCertificateArn": "$CERT_ARN",
        "SSLSupportMethod": "sni-only",
        "MinimumProtocolVersion": "TLSv1.2_2021"
    },
    "Enabled": true,
    "PriceClass": "PriceClass_All"
}
EOF

# Créer la distribution
DISTRIBUTION_ID=$(aws cloudfront create-distribution \
    --distribution-config file://distribution-config.json \
    --query 'Distribution.Id' \
    --output text)

if [ $? -eq 0 ]; then
    echo "✅ Distribution créée: $DISTRIBUTION_ID"
    
    # Obtenir le domaine CloudFront
    CLOUDFRONT_DOMAIN=$(aws cloudfront get-distribution \
        --id $DISTRIBUTION_ID \
        --query 'Distribution.DomainName' \
        --output text)
    
    echo "✅ Domaine CloudFront: $CLOUDFRONT_DOMAIN"
    
    # Sauvegarder les infos
    echo "$DISTRIBUTION_ID" > distribution_id.txt
    echo "$CLOUDFRONT_DOMAIN" > cloudfront_domain.txt
    
    # Afficher les instructions DNS
    echo ""
    echo "🎉 CLOUDFRONT CRÉÉ AVEC SUCCÈS !"
    echo "==============================="
    echo ""
    echo "📊 INFORMATIONS :"
    echo "   Distribution ID: $DISTRIBUTION_ID"
    echo "   CloudFront Domain: $CLOUDFRONT_DOMAIN"
    echo "   Certificat SSL: $CERT_ARN"
    echo "   Target Origin: $TARGET_ORIGIN"
    echo ""
    echo "🌐 CONFIGURATION DNS REQUISE :"
    echo "=============================="
    for domain in "${DOMAINS[@]}"; do
        echo ""
        echo "Domaine: $domain"
        echo "Type: CNAME"
        echo "Nom: @ (ou $domain)"
        echo "Valeur: $CLOUDFRONT_DOMAIN"
        echo "TTL: 300"
    done
    echo ""
    echo "⏰ DÉLAIS :"
    echo "   CloudFront: 15-20 minutes pour activation"
    echo "   DNS: 1-48 heures selon provider"
    echo ""
    echo "🔗 GESTION :"
    echo "   Console: https://console.aws.amazon.com/cloudfront"
    echo "   Distribution: $DISTRIBUTION_ID"
    echo ""
    echo "💰 COÛT ESTIMÉ :"
    echo "   1M requêtes ≈ 0.85$"
    echo "   10GB trafic ≈ 0.85$"
    echo "   Free Tier: 1TB gratuit les 12 premiers mois"
    echo ""
    echo "🎯 TEST :"
    echo "   Attendez 20 minutes puis testez:"
    echo "   https://${DOMAINS[0]}"
    echo "   https://${DOMAINS[1]}"
    echo ""
    echo "✅ VOTRE REDIRECTEUR CDN EST PRÊT !"
    
else
    echo "❌ Erreur création distribution"
    exit 1
fi

# Nettoyage
rm -f distribution-config.json awscliv2.zip
rm -rf aws/