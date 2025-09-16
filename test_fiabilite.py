#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Test de fiabilité par domaine d'email
"""

import sys
import time
import json
import requests
from collections import defaultdict

sys.path.append('/workspace')
from gmass import rate_limiter

def test_domaine(email, api_key):
    """Tester un email et retourner le résultat détaillé"""
    try:
        rate_limiter.wait_if_needed()
        
        headers = {
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
            'Accept': 'application/json, text/plain, */*',
        }
        
        url = f'https://verify.gmass.co/verify?email={email}&key={api_key}'
        response = requests.get(url, headers=headers, timeout=15)
        
        if response.status_code == 200:
            data = json.loads(response.text)
            return {
                'email': email,
                'success': data.get('Success', False),
                'valid': data.get('Valid', False),
                'status': data.get('Status', 'Unknown'),
                'smtp_code': data.get('SMTPCode', 0),
                'response_time': response.elapsed.total_seconds(),
                'error': None
            }
        else:
            return {
                'email': email,
                'success': False,
                'valid': False,
                'status': f'HTTP_{response.status_code}',
                'smtp_code': 0,
                'response_time': response.elapsed.total_seconds(),
                'error': f'HTTP {response.status_code}'
            }
            
    except Exception as e:
        return {
            'email': email,
            'success': False,
            'valid': False,
            'status': 'ERROR',
            'smtp_code': 0,
            'response_time': 0,
            'error': str(e)
        }

def analyser_fiabilite():
    """Analyser la fiabilité par domaine"""
    
    api_key = "5449b291-3f72-498d-9316-362f4ec7168b"
    
    # Emails de test par domaine (mélange d'emails valides et invalides probables)
    emails_test = {
        'orange.fr': [
            'alain.kleinhans@orange.fr',  # Testé valide
            'af.beaussier@orange.fr',     # Testé valide
            'test12345nonexistent@orange.fr',  # Probablement invalide
            'admin@orange.fr'             # Incertain
        ],
        'gmail.com': [
            'test@gmail.com',             # Email test standard
            'admin@gmail.com',            # Probablement invalide
            'noreply@gmail.com',          # Incertain
            'support@gmail.com'           # Incertain
        ],
        'yahoo.fr': [
            'test@yahoo.fr',
            'admin@yahoo.fr',
            'info@yahoo.fr',
            'contact@yahoo.fr'
        ],
        'hotmail.com': [
            'test@hotmail.com',
            'admin@hotmail.com',
            'info@hotmail.com',
            'support@hotmail.com'
        ],
        'outlook.com': [
            'test@outlook.com',
            'admin@outlook.com',
            'info@outlook.com',
            'noreply@outlook.com'
        ],
        'free.fr': [
            'test@free.fr',
            'admin@free.fr',
            'info@free.fr',
            'contact@free.fr'
        ]
    }
    
    print("🔍 ANALYSE DE FIABILITÉ PAR DOMAINE")
    print("=" * 60)
    print(f"🔑 Clé API: {api_key}")
    print(f"📧 Total emails à tester: {sum(len(emails) for emails in emails_test.values())}")
    print(f"🌐 Domaines: {len(emails_test)}")
    print("⏱️  Rate limiting: 1 req/sec")
    print("-" * 60)
    
    resultats_par_domaine = defaultdict(list)
    tous_resultats = []
    
    for domaine, emails in emails_test.items():
        print(f"\n🌐 Test du domaine: {domaine}")
        print(f"📧 {len(emails)} emails à tester")
        
        for i, email in enumerate(emails, 1):
            print(f"  [{i}/{len(emails)}] Test: {email}")
            
            resultat = test_domaine(email, api_key)
            resultats_par_domaine[domaine].append(resultat)
            tous_resultats.append(resultat)
            
            # Afficher le résultat
            if resultat['success'] and resultat['valid']:
                print(f"    ✅ VALIDE ({resultat['response_time']:.2f}s)")
            elif resultat['success'] and not resultat['valid']:
                print(f"    ❌ INVALIDE - {resultat['status']} ({resultat['response_time']:.2f}s)")
            else:
                print(f"    ⚠️  ERREUR - {resultat['error']} ({resultat['response_time']:.2f}s)")
    
    # Analyser les résultats
    print("\n" + "=" * 60)
    print("📊 RAPPORT DE FIABILITÉ")
    print("=" * 60)
    
    # Statistiques globales
    total_tests = len(tous_resultats)
    total_success = sum(1 for r in tous_resultats if r['success'])
    total_valid = sum(1 for r in tous_resultats if r['success'] and r['valid'])
    total_invalid = sum(1 for r in tous_resultats if r['success'] and not r['valid'])
    total_errors = sum(1 for r in tous_resultats if not r['success'])
    
    print(f"\n🌍 FIABILITÉ GLOBALE:")
    print(f"  📡 Réponses API réussies: {total_success}/{total_tests} ({total_success/total_tests*100:.1f}%)")
    print(f"  ✅ Emails validés: {total_valid}/{total_tests} ({total_valid/total_tests*100:.1f}%)")
    print(f"  ❌ Emails invalidés: {total_invalid}/{total_tests} ({total_invalid/total_tests*100:.1f}%)")
    print(f"  ⚠️  Erreurs techniques: {total_errors}/{total_tests} ({total_errors/total_tests*100:.1f}%)")
    
    # Statistiques par domaine
    print(f"\n📧 FIABILITÉ PAR DOMAINE:")
    for domaine, resultats in resultats_par_domaine.items():
        total_dom = len(resultats)
        success_dom = sum(1 for r in resultats if r['success'])
        valid_dom = sum(1 for r in resultats if r['success'] and r['valid'])
        invalid_dom = sum(1 for r in resultats if r['success'] and not r['valid'])
        error_dom = sum(1 for r in resultats if not r['success'])
        
        avg_time = sum(r['response_time'] for r in resultats if r['response_time'] > 0) / max(1, sum(1 for r in resultats if r['response_time'] > 0))
        
        print(f"\n  🌐 {domaine}:")
        print(f"    📡 Fiabilité API: {success_dom}/{total_dom} ({success_dom/total_dom*100:.1f}%)")
        print(f"    ✅ Emails valides: {valid_dom}/{total_dom} ({valid_dom/total_dom*100:.1f}%)")
        print(f"    ❌ Emails invalides: {invalid_dom}/{total_dom} ({invalid_dom/total_dom*100:.1f}%)")
        print(f"    ⚠️  Erreurs: {error_dom}/{total_dom} ({error_dom/total_dom*100:.1f}%)")
        print(f"    ⏱️  Temps moyen: {avg_time:.2f}s")
        
        # Détail des statuts
        statuts = defaultdict(int)
        for r in resultats:
            if r['success']:
                statuts[r['status']] += 1
            else:
                statuts['ERROR'] += 1
        
        print(f"    📋 Statuts: {dict(statuts)}")
    
    # Recommandations
    print(f"\n💡 RECOMMANDATIONS:")
    
    fiabilite_globale = total_success / total_tests * 100
    if fiabilite_globale >= 90:
        print("  🟢 EXCELLENTE fiabilité (≥90%)")
    elif fiabilite_globale >= 75:
        print("  🟡 BONNE fiabilité (75-89%)")
    elif fiabilite_globale >= 50:
        print("  🟠 FIABILITÉ MOYENNE (50-74%)")
    else:
        print("  🔴 FAIBLE fiabilité (<50%)")
    
    print(f"  📊 Le script est fiable à {fiabilite_globale:.1f}% pour obtenir une réponse de l'API")
    
    # Domaines les plus fiables
    domaines_tries = sorted(
        resultats_par_domaine.items(), 
        key=lambda x: sum(1 for r in x[1] if r['success']) / len(x[1]), 
        reverse=True
    )
    
    print(f"\n🏆 CLASSEMENT DES DOMAINES (par fiabilité):")
    for i, (domaine, resultats) in enumerate(domaines_tries, 1):
        fiabilite = sum(1 for r in resultats if r['success']) / len(resultats) * 100
        print(f"  {i}. {domaine}: {fiabilite:.1f}%")

if __name__ == "__main__":
    analyser_fiabilite()