# Validateur d'Emails GMASS - Version Améliorée

## 🚀 Améliorations apportées

### 1. **Modernisation Python 3**
- Migration complète de Python 2.7 vers Python 3
- Remplacement de `raw_input()` par `input()`
- Amélioration de la gestion des encodages (UTF-8)
- Suppression des dépendances obsolètes (`urllib2`)

### 2. **Système de Debouncing Avancé**
- **Rate Limiter intégré** : Maximum 1 requête par seconde
- **Protection contre la surcharge** du serveur GMASS.co
- **Gestion thread-safe** avec verrous (locks)
- **Queue intelligente** pour gérer les requêtes en attente

### 3. **Gestion d'Erreurs Robuste**
- **Timeouts configurables** (30 secondes)
- **Gestion des erreurs réseau** spécifiques
- **Sauvegarde automatique** des erreurs dans des fichiers séparés
- **Messages d'erreur détaillés** en français

### 4. **Suivi de Progression en Temps Réel**
- **Compteurs en direct** : valides, invalides, erreurs
- **Affichage périodique** du progrès (tous les 10 emails)
- **Rapport final détaillé** avec statistiques complètes
- **Temps d'exécution** affiché

### 5. **Threading Optimisé**
- **Nombre de threads configurable** (1-20, recommandé: 5-10)
- **Pool de threads géré** automatiquement
- **Nettoyage automatique** des anciens fichiers de résultats

## 📋 Installation

```bash
pip install -r requirements.txt
```

## 🎯 Utilisation

```bash
python3 gmass.py
```

### Options disponibles :
1. **Extraire les emails** d'un fichier texte
2. **Valider les emails** avec GMASS.co

## 📁 Fichiers de sortie

- `Mail_OK.txt` - Emails valides
- `Mail_FAILED.txt` - Emails invalides
- `Mail_UNKNOWN.txt` - Réponses inconnues
- `Mail_TIMEOUT.txt` - Timeouts
- `Mail_ERROR.txt` - Autres erreurs

## ⚙️ Configuration

- **Rate Limiting** : 1 requête/seconde (modifiable dans le code)
- **Timeout** : 30 secondes par requête
- **Threads** : 5 par défaut (configurable)

## 🔧 Clé API

Remplacez la clé API dans le code par la vôtre :
```python
api_key = 'VOTRE_CLÉ_ICI'  # Ligne 147
```

Obtenez votre clé sur : https://gmass.co/

## 🎨 Interface

- **Messages colorés** avec Colorama
- **Interface en français**
- **Progression visuelle** en temps réel
- **Rapports détaillés**

---

*Version améliorée - Septembre 2025*
*Debouncing intégré pour une utilisation respectueuse de l'API*