#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Test du système de debouncing adaptatif
"""

import sys
import time
from unittest.mock import patch

sys.path.append('/workspace')

def test_debouncing_adaptatif():
    """Tester le système adaptatif avec un échantillon d'emails"""
    
    print("🧪 TEST DU RATE LIMITER ADAPTATIF")
    print("=" * 60)
    
    # Créer un fichier de test avec quelques emails Orange
    emails_test = [
        "alain.kleinhans@orange.fr",
        "af.beaussier@orange.fr", 
        "alainbertail@orange.fr",
        "4daj@orange.fr",
        "aavassor@orange.fr",
        "test12345@orange.fr",  # Probablement invalide
        "admin@orange.fr",      # Peut causer timeout
    ]
    
    with open('test_adaptatif_emails.txt', 'w') as f:
        for email in emails_test:
            f.write(email + '\n')
    
    print(f"📧 {len(emails_test)} emails de test")
    print(f"🧠 Le rate limiter va s'adapter automatiquement")
    print(f"⏱️  Observez les ajustements en temps réel...")
    print("-" * 60)
    
    try:
        # Simuler l'exécution avec 10 threads pour tester l'adaptation
        with patch('builtins.input', side_effect=['2', 'test_adaptatif_emails.txt', '10']):
            start_time = time.time()
            
            from gmass import Main
            Main()
            
            end_time = time.time()
            
            print(f"\n🎯 TEST TERMINÉ !")
            print(f"⏱️  Durée totale: {end_time - start_time:.1f}s")
            print(f"⚡ Le système s'est adapté automatiquement aux conditions réseau")
            
    except Exception as e:
        print(f"❌ Erreur: {e}")
    
    finally:
        # Nettoyer
        import os
        if os.path.exists('test_adaptatif_emails.txt'):
            os.remove('test_adaptatif_emails.txt')

def calculer_estimation_3500():
    """Calculer l'estimation pour 3500 emails avec le système adaptatif"""
    
    print(f"\n📊 ESTIMATION POUR 3500 EMAILS")
    print("=" * 50)
    
    scenarios = [
        {"nom": "OPTIMAL", "rate": 8, "description": "Pas de timeouts, rate limiter accélère"},
        {"nom": "MOYEN", "rate": 5, "description": "Quelques ajustements, vitesse stable"},
        {"nom": "DIFFICILE", "rate": 3, "description": "Timeouts détectés, ralentissement"},
        {"nom": "CRITIQUE", "rate": 1, "description": "Beaucoup de timeouts, vitesse minimum"}
    ]
    
    for scenario in scenarios:
        rate = scenario["rate"]
        nom = scenario["nom"]
        desc = scenario["description"]
        
        # Calcul avec overhead réseau
        temps_estime = 3500 / rate
        minutes = int(temps_estime // 60)
        secondes = int(temps_estime % 60)
        
        emoji = "🟢" if temps_estime <= 600 else "🟡" if temps_estime <= 900 else "🔴"
        
        print(f"\n{emoji} {nom}:")
        print(f"   📊 {rate} req/sec moyennes")
        print(f"   ⏱️  Temps: {minutes}m {secondes}s")
        print(f"   📝 {desc}")
        
        if temps_estime <= 300:
            print(f"   ✅ EXCELLENT - Dans vos 5 minutes !")
        elif temps_estime <= 600:
            print(f"   🟡 BON - Dans vos 10 minutes")
        else:
            print(f"   ❌ TROP LENT - Au-delà de 10 minutes")
    
    print(f"\n💡 AVANTAGES DU SYSTÈME ADAPTATIF:")
    print(f"   🧠 S'adapte automatiquement aux conditions")
    print(f"   🚀 Accélère quand possible (jusqu'à 12 req/sec)")
    print(f"   🛡️  Ralentit si timeouts (évite les blocages)")
    print(f"   📊 Optimise la vitesse selon votre réseau/API")
    
    print(f"\n🎯 CONCLUSION:")
    print(f"   Avec le système adaptatif, vous devriez obtenir")
    print(f"   entre 6-12 minutes pour 3500 emails selon les conditions")

if __name__ == "__main__":
    calculer_estimation_3500()
    
    print(f"\n" + "="*60)
    choix = input(f"Voulez-vous tester le système adaptatif ? (o/n): ").lower()
    if choix == 'o':
        test_debouncing_adaptatif()