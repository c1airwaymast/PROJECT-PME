# 🚀 Guide de Déploiement - Système Anti-Bot

## Étapes de Déploiement Rapide

### 1. Préparation
```bash
# Installer Wrangler CLI
npm install -g wrangler

# Se connecter à Cloudflare
wrangler login
```

### 2. Configuration Obligatoire

#### A. Modifier `wrangler.toml`
```toml
# OBLIGATOIRE: Remplacez ces valeurs
name = "votre-worker-name"  # Nom unique pour votre worker

[[routes]]
pattern = "votre-domaine.com/*"      # Votre domaine
zone_name = "votre-domaine.com"      # Votre zone Cloudflare

[env.production.vars]
TARGET_URL = "https://votre-site-reel.com"  # URL de votre vrai site
```

#### B. Variables Importantes
- `TARGET_URL`: L'URL vers laquelle rediriger le trafic légitime
- `BOT_THRESHOLD`: Seuil de détection (0.7 = 70% par défaut)

### 3. Test Local
```bash
# Tester en local avant déploiement
wrangler dev

# Le worker sera accessible sur http://localhost:8787
```

### 4. Déploiement
```bash
# Déploiement en production
wrangler deploy

# Ou avec environnement spécifique
wrangler deploy --env production
```

## Configuration de votre Domaine

### Option 1: Route Worker (Recommandé)
1. Allez dans le dashboard Cloudflare
2. Sélectionnez votre domaine
3. Onglet "Workers Routes"
4. Ajoutez une route: `votre-domaine.com/*`
5. Sélectionnez votre worker

### Option 2: Sous-domaine Worker
1. Le worker sera accessible sur: `votre-worker-name.votre-compte.workers.dev`
2. Configurez un CNAME dans votre DNS:
   ```
   security CNAME votre-worker-name.votre-compte.workers.dev
   ```

## Architecture de Déploiement

```
Internet → Cloudflare Edge → Security Worker → Votre Site Réel
                                ↓
                         Challenge Page (si bot détecté)
```

### Flux de Traitement
1. **Requête arrive** sur votre domaine
2. **Worker analyse** la requête (User-Agent, IP, headers, etc.)
3. **Si score < 70%**: Trafic transmis à votre site réel
4. **Si score 70-90%**: Challenge humain affiché
5. **Si score > 90%**: Blocage direct
6. **Après vérification**: Cookie placé, trafic transmis

## Variables d'Environnement

### Production
```toml
[env.production.vars]
TARGET_URL = "https://votre-site-production.com"
BOT_THRESHOLD = "0.7"
```

### Staging/Test
```toml
[env.staging.vars]
TARGET_URL = "https://staging.votre-site.com"
BOT_THRESHOLD = "0.6"  # Plus strict pour les tests
```

## Monitoring Post-Déploiement

### Logs en Temps Réel
```bash
# Voir les logs du worker
wrangler tail

# Filtrer par environnement
wrangler tail --env production
```

### Métriques Cloudflare
```bash
# Statistiques d'utilisation
wrangler metrics

# Dashboard Cloudflare
# Analytics → Workers → Votre Worker
```

## Tests de Fonctionnement

### 1. Test Utilisateur Normal
```bash
# Devrait passer sans problème
curl -H "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36" \
     -H "Accept: text/html,application/xhtml+xml" \
     -H "Accept-Language: fr-FR,fr;q=0.9,en;q=0.8" \
     https://votre-domaine.com/
```

### 2. Test Bot Simple
```bash
# Devrait être bloqué ou challengé
curl https://votre-domaine.com/
```

### 3. Test Bot Avancé
```bash
# Devrait être détecté
curl -H "User-Agent: python-requests/2.28.1" \
     https://votre-domaine.com/
```

## Résolution de Problèmes

### Problème: Utilisateurs Légitimes Bloqués
```javascript
// Dans security-worker.js, réduire le seuil
this.BOT_THRESHOLD = 0.8; // Au lieu de 0.7
```

### Problème: Trop de Bots Passent
```javascript
// Augmenter la sensibilité
this.BOT_THRESHOLD = 0.6; // Au lieu de 0.7
```

### Problème: Worker ne Démarre Pas
1. Vérifiez `wrangler.toml`
2. Vérifiez que `TARGET_URL` est défini
3. Vérifiez les routes dans le dashboard

### Problème: Erreur 1101 (Worker Exception)
```bash
# Voir les erreurs détaillées
wrangler tail --format pretty
```

## Sécurité Post-Déploiement

### 1. Headers de Sécurité Ajoutés
- `X-Bot-Score`: Score de détection
- `X-Protected-By`: Identification du système
- `X-Security-Verified`: Pour les requêtes vérifiées

### 2. Monitoring des Attaques
```bash
# Surveiller les tentatives de bot
wrangler tail | grep "Bot Score"
```

### 3. Mise à Jour des Règles
Mettez régulièrement à jour les patterns dans `security-worker.js`:
- Nouveaux User-Agents de bots
- Nouvelles ranges IP suspectes
- Nouveaux patterns d'attaque

## Performance

### Métriques Attendues
- **Latence ajoutée**: <10ms en moyenne
- **CPU utilisé**: <50ms par requête
- **Précision**: >95% pour la détection
- **Faux positifs**: <2% pour les utilisateurs légitimes

### Optimisations
- Cache des vérifications humaines (1 heure)
- Analyse parallèle des critères
- Fallback rapide en cas d'erreur

## Backup et Rollback

### Sauvegarder la Configuration Actuelle
```bash
# Télécharger la configuration actuelle
wrangler download
```

### Rollback Rapide
```bash
# Revenir à la version précédente
wrangler rollback
```

## Support et Maintenance

### Logs Importants à Surveiller
- Scores de détection élevés
- Erreurs de worker
- Temps de réponse anormaux
- Pics de trafic

### Maintenance Régulière
1. **Hebdomadaire**: Vérifier les métriques
2. **Mensuel**: Mettre à jour les patterns de détection  
3. **Trimestriel**: Analyser les faux positifs/négatifs

---

**🎯 Checklist de Déploiement**
- [ ] `wrangler.toml` configuré avec votre domaine
- [ ] `TARGET_URL` définie vers votre vrai site
- [ ] Routes configurées dans Cloudflare
- [ ] Tests effectués en local
- [ ] Déploiement réussi
- [ ] Tests de fonctionnement OK
- [ ] Monitoring activé

**⚠️ Important**: Gardez toujours un accès direct à votre site réel en cas de problème avec le worker!