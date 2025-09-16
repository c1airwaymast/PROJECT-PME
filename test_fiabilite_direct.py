#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Test de fiabilité direct sans importer le script principal
"""

import time
import json
import requests
import threading
from collections import defaultdict, deque

# Rate limiter simple
class RateLimiter:
    def __init__(self, max_requests_per_second=1):
        self.max_requests = max_requests_per_second
        self.requests = deque()
        self.lock = threading.Lock()
    
    def wait_if_needed(self):
        with self.lock:
            now = time.time()
            while self.requests and self.requests[0] <= now - 1:
                self.requests.popleft()
            
            if len(self.requests) >= self.max_requests:
                sleep_time = 1 - (now - self.requests[0])
                if sleep_time > 0:
                    time.sleep(sleep_time)
                    now = time.time()
                    while self.requests and self.requests[0] <= now - 1:
                        self.requests.popleft()
            
            self.requests.append(now)

rate_limiter = RateLimiter(max_requests_per_second=1)

def test_email_gmass(email, api_key):
    """Tester un email avec l'API GMASS"""
    try:
        rate_limiter.wait_if_needed()
        
        headers = {
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
            'Accept': 'application/json, text/plain, */*',
        }
        
        url = f'https://verify.gmass.co/verify?email={email}&key={api_key}'
        start_time = time.time()
        response = requests.get(url, headers=headers, timeout=10)
        end_time = time.time()
        
        if response.status_code == 200:
            data = json.loads(response.text)
            return {
                'email': email,
                'success': True,
                'api_success': data.get('Success', False),
                'valid': data.get('Valid', False),
                'status': data.get('Status', 'Unknown'),
                'smtp_code': data.get('SMTPCode', 0),
                'response_time': end_time - start_time,
                'error': None,
                'raw_response': response.text[:200]
            }
        else:
            return {
                'email': email,
                'success': False,
                'api_success': False,
                'valid': False,
                'status': f'HTTP_{response.status_code}',
                'smtp_code': 0,
                'response_time': end_time - start_time,
                'error': f'HTTP {response.status_code}',
                'raw_response': response.text[:200]
            }
            
    except Exception as e:
        return {
            'email': email,
            'success': False,
            'api_success': False,
            'valid': False,
            'status': 'ERROR',
            'smtp_code': 0,
            'response_time': 0,
            'error': str(e),
            'raw_response': ''
        }

def analyser_fiabilite():
    """Analyser la fiabilité du script GMASS par domaine"""
    
    api_key = "5449b291-3f72-498d-9316-362f4ec7168b"
    
    # Emails de test stratégiques par domaine
    emails_test = {
        'orange.fr': [
            'alain.kleinhans@orange.fr',      # Connu valide
            'test123456789@orange.fr',        # Probablement invalide
            'admin@orange.fr',                # Incertain
        ],
        'gmail.com': [
            'test@gmail.com',                 # Email test classique
            'admin@gmail.com',                # Probablement invalide
            'noreply@gmail.com',              # Incertain
        ],
        'yahoo.fr': [
            'test@yahoo.fr',
            'admin@yahoo.fr',
            'info@yahoo.fr',
        ],
        'hotmail.com': [
            'test@hotmail.com',
            'admin@hotmail.com',
            'support@hotmail.com',
        ],
        'outlook.com': [
            'test@outlook.com',
            'info@outlook.com',
            'noreply@outlook.com',
        ],
        'free.fr': [
            'test@free.fr',
            'admin@free.fr',
            'contact@free.fr',
        ]
    }
    
    print("🔍 ANALYSE DE FIABILITÉ GMASS PAR DOMAINE")
    print("=" * 70)
    print(f"🔑 Clé API: {api_key[:20]}...")
    print(f"📧 Total emails: {sum(len(emails) for emails in emails_test.values())}")
    print(f"🌐 Domaines testés: {len(emails_test)}")
    print(f"⏱️  Rate limiting: 1 requête/seconde")
    print("-" * 70)
    
    resultats_par_domaine = {}
    tous_resultats = []
    
    for domaine, emails in emails_test.items():
        print(f"\n🌐 DOMAINE: {domaine.upper()}")
        print(f"📧 {len(emails)} emails à tester...")
        
        resultats_domaine = []
        
        for i, email in enumerate(emails, 1):
            print(f"  [{i}/{len(emails)}] {email}... ", end="", flush=True)
            
            resultat = test_email_gmass(email, api_key)
            resultats_domaine.append(resultat)
            tous_resultats.append(resultat)
            
            # Affichage du résultat
            if resultat['success'] and resultat['api_success'] and resultat['valid']:
                print(f"✅ VALIDE ({resultat['response_time']:.1f}s)")
            elif resultat['success'] and resultat['api_success'] and not resultat['valid']:
                print(f"❌ INVALIDE [{resultat['status']}] ({resultat['response_time']:.1f}s)")
            elif resultat['success'] and not resultat['api_success']:
                print(f"⚠️  API_FAIL [{resultat['status']}] ({resultat['response_time']:.1f}s)")
            else:
                print(f"🔴 ERREUR [{resultat['error']}]")
        
        resultats_par_domaine[domaine] = resultats_domaine
    
    # ANALYSE GLOBALE
    print("\n" + "=" * 70)
    print("📊 RAPPORT DE FIABILITÉ GLOBAL")
    print("=" * 70)
    
    total_tests = len(tous_resultats)
    connexions_reussies = sum(1 for r in tous_resultats if r['success'])
    api_success = sum(1 for r in tous_resultats if r['success'] and r['api_success'])
    emails_valides = sum(1 for r in tous_resultats if r['success'] and r['api_success'] and r['valid'])
    emails_invalides = sum(1 for r in tous_resultats if r['success'] and r['api_success'] and not r['valid'])
    erreurs_reseau = sum(1 for r in tous_resultats if not r['success'])
    erreurs_api = sum(1 for r in tous_resultats if r['success'] and not r['api_success'])
    
    print(f"\n🌍 FIABILITÉ TECHNIQUE:")
    print(f"  📡 Connexions réussies: {connexions_reussies}/{total_tests} ({connexions_reussies/total_tests*100:.1f}%)")
    print(f"  ✅ API Success: {api_success}/{total_tests} ({api_success/total_tests*100:.1f}%)")
    print(f"  🔴 Erreurs réseau: {erreurs_reseau}/{total_tests} ({erreurs_reseau/total_tests*100:.1f}%)")
    print(f"  ⚠️  Erreurs API: {erreurs_api}/{total_tests} ({erreurs_api/total_tests*100:.1f}%)")
    
    print(f"\n📧 RÉSULTATS DE VALIDATION:")
    print(f"  ✅ Emails VALIDES: {emails_valides}/{total_tests} ({emails_valides/total_tests*100:.1f}%)")
    print(f"  ❌ Emails INVALIDES: {emails_invalides}/{total_tests} ({emails_invalides/total_tests*100:.1f}%)")
    
    # Temps de réponse moyen
    temps_reponse = [r['response_time'] for r in tous_resultats if r['response_time'] > 0]
    if temps_reponse:
        temps_moyen = sum(temps_reponse) / len(temps_reponse)
        print(f"  ⏱️  Temps de réponse moyen: {temps_moyen:.2f}s")
    
    # ANALYSE PAR DOMAINE
    print(f"\n📊 FIABILITÉ PAR DOMAINE:")
    
    domaines_stats = []
    
    for domaine, resultats in resultats_par_domaine.items():
        total_dom = len(resultats)
        connexions_dom = sum(1 for r in resultats if r['success'])
        api_success_dom = sum(1 for r in resultats if r['success'] and r['api_success'])
        valides_dom = sum(1 for r in resultats if r['success'] and r['api_success'] and r['valid'])
        invalides_dom = sum(1 for r in resultats if r['success'] and r['api_success'] and not r['valid'])
        
        fiabilite_technique = connexions_dom / total_dom * 100
        fiabilite_api = api_success_dom / total_dom * 100
        
        temps_dom = [r['response_time'] for r in resultats if r['response_time'] > 0]
        temps_moyen_dom = sum(temps_dom) / len(temps_dom) if temps_dom else 0
        
        domaines_stats.append({
            'domaine': domaine,
            'fiabilite_technique': fiabilite_technique,
            'fiabilite_api': fiabilite_api,
            'valides': valides_dom,
            'invalides': invalides_dom,
            'total': total_dom,
            'temps_moyen': temps_moyen_dom
        })
        
        print(f"\n  🌐 {domaine.upper()}:")
        print(f"    📡 Fiabilité technique: {fiabilite_technique:.1f}%")
        print(f"    ✅ Fiabilité API: {fiabilite_api:.1f}%")
        print(f"    📧 Valides: {valides_dom}/{total_dom} ({valides_dom/total_dom*100:.1f}%)")
        print(f"    ❌ Invalides: {invalides_dom}/{total_dom} ({invalides_dom/total_dom*100:.1f}%)")
        print(f"    ⏱️  Temps moyen: {temps_moyen_dom:.2f}s")
        
        # Détail des statuts pour ce domaine
        statuts = defaultdict(int)
        for r in resultats:
            statuts[r['status']] += 1
        print(f"    📋 Statuts: {dict(statuts)}")
    
    # CLASSEMENT DES DOMAINES
    print(f"\n🏆 CLASSEMENT PAR FIABILITÉ TECHNIQUE:")
    domaines_tries = sorted(domaines_stats, key=lambda x: x['fiabilite_technique'], reverse=True)
    
    for i, stats in enumerate(domaines_tries, 1):
        emoji = "🥇" if i == 1 else "🥈" if i == 2 else "🥉" if i == 3 else "📊"
        print(f"  {emoji} {i}. {stats['domaine']}: {stats['fiabilite_technique']:.1f}% (API: {stats['fiabilite_api']:.1f}%)")
    
    # RECOMMANDATIONS FINALES
    print(f"\n💡 RECOMMANDATIONS:")
    
    fiabilite_globale = api_success / total_tests * 100
    
    if fiabilite_globale >= 90:
        niveau = "🟢 EXCELLENTE"
    elif fiabilite_globale >= 75:
        niveau = "🟡 BONNE"
    elif fiabilite_globale >= 50:
        niveau = "🟠 MOYENNE"
    else:
        niveau = "🔴 FAIBLE"
    
    print(f"  {niveau} fiabilité globale: {fiabilite_globale:.1f}%")
    print(f"  📊 Le script GMASS est fiable à {fiabilite_globale:.1f}% pour valider les emails")
    
    if fiabilite_globale >= 80:
        print(f"  ✅ Recommandé pour utilisation en production")
    elif fiabilite_globale >= 60:
        print(f"  ⚠️  Utilisable avec précaution, vérifier les résultats importants")
    else:
        print(f"  ❌ Non recommandé, trop d'erreurs")
    
    print(f"\n🎯 CONCLUSION:")
    print(f"  Le script fonctionne avec une fiabilité de {fiabilite_globale:.1f}%")
    print(f"  Meilleurs domaines: {', '.join([s['domaine'] for s in domaines_tries[:3]])}")
    
    return fiabilite_globale, domaines_stats

if __name__ == "__main__":
    analyser_fiabilite()