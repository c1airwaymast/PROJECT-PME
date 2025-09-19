#!/bin/bash

# CRÉATION CLOUDFRONT AUTOMATIQUE POUR REDIRECTIONS
# Script ultra-simple pour créer votre CDN de redirection

echo "🚀 CRÉATION CLOUDFRONT AUTOMATIQUE"
echo "=================================="

# Variables de configuration
TARGET_DOMAIN="airwaymast.org"
YOUR_DOMAINS=("secures.sbs" "vantagenode.sbs")

# Fonction de logging
log() { echo "✅ $1"; }
error() { echo "❌ $1"; }

# Vérifier AWS CLI
check_aws_cli() {
    if ! command -v aws &> /dev/null; then
        error "AWS CLI non installé. Installation..."
        curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
        unzip awscliv2.zip
        sudo ./aws/install
        log "AWS CLI installé"
    fi
}

# Configuration AWS
setup_aws() {
    echo "🔑 Configuration AWS..."
    echo "Entrez vos clés AWS (disponibles dans IAM):"
    aws configure
}

# Créer le certificat SSL
create_ssl_certificate() {
    log "🔒 Création certificat SSL..."
    
    # Créer la liste des domaines
    DOMAIN_LIST=""
    for domain in "${YOUR_DOMAINS[@]}"; do
        DOMAIN_LIST="$DOMAIN_LIST Name=$domain"
    done
    
    # Demander le certificat
    CERT_ARN=$(aws acm request-certificate \
        --domain-name "${YOUR_DOMAINS[0]}" \
        --subject-alternative-names "${YOUR_DOMAINS[@]:1}" \
        --validation-method DNS \
        --region us-east-1 \
        --query 'CertificateArn' \
        --output text)
    
    if [ $? -eq 0 ]; then
        log "Certificat SSL demandé: $CERT_ARN"
        echo "📋 IMPORTANT: Validez le certificat dans AWS Console ACM"
        echo "   https://console.aws.amazon.com/acm/home?region=us-east-1"
    else
        error "Erreur création certificat"
    fi
    
    echo "$CERT_ARN" > cert_arn.txt
}

# Créer la distribution CloudFront
create_cloudfront() {
    log "☁️ Création distribution CloudFront..."
    
    # Configuration JSON pour CloudFront
    cat > cloudfront-config.json << EOF
{
    "CallerReference": "redirect-$(date +%s)",
    "Comment": "Redirecteur sécurisé vers $TARGET_DOMAIN",
    "DefaultCacheBehavior": {
        "TargetOriginId": "redirect-origin",
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
                "Forward": "none"
            },
            "Headers": {
                "Quantity": 1,
                "Items": ["Host"]
            }
        },
        "MinTTL": 0,
        "DefaultTTL": 0,
        "MaxTTL": 0
    },
    "Origins": {
        "Quantity": 1,
        "Items": [
            {
                "Id": "redirect-origin",
                "DomainName": "$TARGET_DOMAIN",
                "CustomOriginConfig": {
                    "HTTPPort": 80,
                    "HTTPSPort": 443,
                    "OriginProtocolPolicy": "http-only"
                }
            }
        ]
    },
    "Enabled": true,
    "PriceClass": "PriceClass_All"
}
EOF

    # Créer la distribution
    DISTRIBUTION_ID=$(aws cloudfront create-distribution \
        --distribution-config file://cloudfront-config.json \
        --query 'Distribution.Id' \
        --output text)
    
    if [ $? -eq 0 ]; then
        log "Distribution CloudFront créée: $DISTRIBUTION_ID"
        
        # Obtenir le nom de domaine CloudFront
        CLOUDFRONT_DOMAIN=$(aws cloudfront get-distribution \
            --id $DISTRIBUTION_ID \
            --query 'Distribution.DomainName' \
            --output text)
        
        log "Domaine CloudFront: $CLOUDFRONT_DOMAIN"
        
        echo "$DISTRIBUTION_ID" > distribution_id.txt
        echo "$CLOUDFRONT_DOMAIN" > cloudfront_domain.txt
    else
        error "Erreur création distribution"
    fi
}

# Configuration DNS automatique
setup_dns() {
    log "🌐 Configuration DNS..."
    
    CLOUDFRONT_DOMAIN=$(cat cloudfront_domain.txt)
    
    echo ""
    echo "📋 CONFIGURATION DNS REQUISE :"
    echo "=============================="
    echo ""
    for domain in "${YOUR_DOMAINS[@]}"; do
        echo "Domaine: $domain"
        echo "Type: CNAME"
        echo "Valeur: $CLOUDFRONT_DOMAIN"
        echo ""
    done
    
    echo "⚠️  Configurez ces enregistrements dans votre gestionnaire DNS"
    echo "   (Cloudflare, Namecheap, GoDaddy, etc.)"
}

# Affichage final
show_results() {
    DISTRIBUTION_ID=$(cat distribution_id.txt)
    CLOUDFRONT_DOMAIN=$(cat cloudfront_domain.txt)
    
    echo ""
    echo "🎉 CLOUDFRONT CRÉÉ AVEC SUCCÈS !"
    echo "================================"
    echo ""
    echo "📊 INFORMATIONS :"
    echo "   Distribution ID: $DISTRIBUTION_ID"
    echo "   CloudFront Domain: $CLOUDFRONT_DOMAIN"
    echo "   Target: $TARGET_DOMAIN"
    echo ""
    echo "🌐 VOS DOMAINES :"
    for domain in "${YOUR_DOMAINS[@]}"; do
        echo "   $domain → $TARGET_DOMAIN"
    done
    echo ""
    echo "⏰ DÉLAI DE PROPAGATION :"
    echo "   CloudFront: 15-20 minutes"
    echo "   DNS: 1-48 heures"
    echo ""
    echo "🔗 GESTION :"
    echo "   Console: https://console.aws.amazon.com/cloudfront"
    echo "   Distribution: $DISTRIBUTION_ID"
    echo ""
    echo "💰 COÛT ESTIMÉ :"
    echo "   1M requêtes ≈ 0.85$"
    echo "   Trafic sortant ≈ 0.085$/GB"
    echo ""
    echo "✅ VOTRE REDIRECTEUR CDN EST PRÊT !"
}

# Fonction principale
main() {
    check_aws_cli
    setup_aws
    create_ssl_certificate
    
    echo ""
    echo "⏳ Attendez la validation du certificat SSL..."
    echo "   Puis appuyez sur Entrée pour continuer"
    read
    
    create_cloudfront
    setup_dns
    show_results
}

# Exécution
main "$@"