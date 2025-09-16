# 🚀 GUIDE D'INSTALLATION ET TEST

## 📦 **FICHIERS EXPORTÉS**

### 📁 **gmass_adaptatif_complet.zip** (Recommandé)
Contient :
- `gmass.py` - Script principal avec debouncing adaptatif
- `requirements.txt` - Dépendances Python
- `test_emails.txt` - 200 emails Orange pour test
- `lancer_validation.py` - Script de lancement facile
- Documentation complète

### 📁 **gmass_minimal.zip** (Essentiel)
Contient juste :
- `gmass.py` - Script principal
- `requirements.txt` - Dépendances

---

## 🔧 **INSTALLATION**

### 1️⃣ **Extraire le ZIP**
```bash
unzip gmass_adaptatif_complet.zip
cd gmass_adaptatif_complet/
```

### 2️⃣ **Installer les dépendances**
```bash
# Option A : Avec pip (recommandé)
pip install -r requirements.txt

# Option B : Avec apt (Ubuntu/Debian)
sudo apt install python3-requests python3-colorama

# Option C : Environnement virtuel
python3 -m venv venv
source venv/bin/activate  # Linux/Mac
pip install -r requirements.txt
```

### 3️⃣ **Vérifier l'installation**
```bash
python3 -c "import requests, colorama; print('✅ Dépendances OK')"
```

---

## 🧪 **TESTS**

### 🎯 **TEST RAPIDE (5 emails)**
```bash
# Créer un fichier test
echo -e "alain.kleinhans@orange.fr\naf.beaussier@orange.fr\ntest@example.com\nadmin@orange.fr\ninfo@orange.fr" > test_rapide.txt

# Lancer le test
python3 gmass.py
# Choisir : 2
# Fichier : test_rapide.txt  
# Threads : 5
```

### 🎯 **TEST MOYEN (200 emails Orange)**
```bash
# Utiliser le fichier fourni
python3 lancer_validation.py
# Ou manuellement :
python3 gmass.py
# Choisir : 2
# Fichier : test_emails.txt
# Threads : 10
```

### 🎯 **TEST COMPLET (Vos 3500 emails)**
```bash
# 1. Créer votre fichier d'emails
echo "email1@orange.fr" > mes_3500_emails.txt
echo "email2@orange.fr" >> mes_3500_emails.txt
# ... ou copier/coller vos emails

# 2. Lancer la validation
python3 gmass.py
# Choisir : 2
# Fichier : mes_3500_emails.txt
# Threads : 15-20 (recommandé pour gros volumes)
```

---

## ⚙️ **CONFIGURATION**

### 🔑 **Votre clé API est déjà configurée** :
```
5449b291-3f72-498d-9316-362f4ec7168b
```

### 🧠 **Système adaptatif configuré** :
- Départ : 5 req/sec
- Min : 1 req/sec  
- Max : 12 req/sec
- S'ajuste automatiquement !

---

## 📊 **RÉSULTATS ATTENDUS**

### 📁 **Fichiers générés** :
- `Mail_OK.txt` - Emails valides ✅
- `Mail_FAILED.txt` - Emails invalides ❌  
- `Mail_ERROR.txt` - Erreurs diverses ⚠️
- `Mail_TIMEOUT.txt` - Timeouts ⏰
- `Mail_UNKNOWN.txt` - Statuts inconnus ❓

### ⏱️ **Temps estimés** :
- **5 emails** : ~30 secondes
- **200 emails** : ~2-4 minutes  
- **3500 emails** : ~7-12 minutes

---

## 🚨 **DÉPANNAGE**

### ❌ **"Module not found"**
```bash
pip install requests colorama
```

### ❌ **"Permission denied"**
```bash
sudo python3 gmass.py
# ou
chmod +x gmass.py
```

### ❌ **"Timeout" répétés**
Le système va automatiquement ralentir. C'est normal !

### ❌ **"Clé API invalide"**
Vérifiez que votre clé est bien : `5449b291-3f72-498d-9316-362f4ec7168b`

---

## 🎯 **CONSEILS POUR 3500 EMAILS**

1. **🕐 Lancez pendant les heures creuses** (éviter 9h-17h)
2. **🌐 Connexion stable** recommandée
3. **🧵 15-20 threads** pour gros volumes
4. **⏱️ Soyez patient** - Le système s'optimise automatiquement
5. **📊 Surveillez les messages** d'ajustement du rate limiter

---

## ✅ **CHECKLIST AVANT TEST**

- [ ] ZIP extrait
- [ ] Dépendances installées  
- [ ] Clé API configurée
- [ ] Fichier d'emails prêt
- [ ] Connexion internet stable
- [ ] Python 3 installé

**🚀 Vous êtes prêt à tester !**