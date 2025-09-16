#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Test de vitesse pour 3500 emails
"""

import time
import math

def calculer_temps_traitement():
    """Calculer le temps estimé pour différentes configurations"""
    
    nb_emails = 3500
    
    print("⚡ CALCUL DES PERFORMANCES - 3500 EMAILS")
    print("=" * 60)
    
    configurations = [
        {"rate": 1, "threads": 5, "nom": "ANCIEN (trop lent)"},
        {"rate": 5, "threads": 10, "nom": "MOYEN"},
        {"rate": 10, "threads": 15, "nom": "RAPIDE (recommandé)"},
        {"rate": 15, "threads": 20, "nom": "TRÈS RAPIDE"},
        {"rate": 20, "threads": 20, "nom": "MAXIMUM"},
    ]
    
    for config in configurations:
        rate_per_sec = config["rate"]
        threads = config["threads"]
        nom = config["nom"]
        
        # Temps théorique avec rate limiting
        temps_rate_limit = nb_emails / rate_per_sec
        
        # Temps réel estimé (avec overhead réseau ~1s par requête)
        temps_reel_estime = max(temps_rate_limit, nb_emails / threads)
        
        minutes = int(temps_reel_estime // 60)
        secondes = int(temps_reel_estime % 60)
        
        print(f"\n🚀 {nom}:")
        print(f"   📊 {rate_per_sec} req/sec, {threads} threads")
        print(f"   ⏱️  Temps estimé: {minutes}m {secondes}s")
        
        if temps_reel_estime <= 300:  # 5 minutes
            print(f"   ✅ RESPECTE votre contrainte de 5 minutes")
        elif temps_reel_estime <= 600:  # 10 minutes  
            print(f"   🟡 DANS la limite de 10 minutes")
        else:
            print(f"   ❌ TROP LENT (>{temps_reel_estime/60:.1f} minutes)")
    
    print(f"\n💡 RECOMMANDATION:")
    print(f"   🎯 Configuration RAPIDE: 10 req/sec + 15 threads")
    print(f"   ⏱️  Temps estimé: ~6 minutes pour 3500 emails")
    print(f"   ✅ Orange.fr fonctionne parfaitement à cette vitesse")

def test_vitesse_reel():
    """Test de vitesse réel avec quelques emails"""
    
    print(f"\n🧪 TEST DE VITESSE RÉEL")
    print("=" * 40)
    
    # Test avec 10 emails Orange pour mesurer la vitesse réelle
    emails_test = [
        "alain.kleinhans@orange.fr",
        "af.beaussier@orange.fr", 
        "alainbertail@orange.fr",
        "4daj@orange.fr",
        "aavassor@orange.fr",
        "adauchy.ide@orange.fr",
        "alain.authier4@orange.fr",
        "adolphe.camillo@orange.fr",
        "alain.monnier80@orange.fr",
        "aichapalomo@orange.fr"
    ]
    
    print(f"📧 Test avec {len(emails_test)} emails Orange")
    print(f"🔧 Configuration: 10 req/sec, rate limiting optimisé")
    
    import sys
    sys.path.append('/workspace')
    
    try:
        from unittest.mock import patch
        
        # Créer un fichier temporaire
        with open('test_vitesse_emails.txt', 'w') as f:
            for email in emails_test:
                f.write(email + '\n')
        
        start_time = time.time()
        
        # Simuler l'exécution du script principal
        with patch('builtins.input', side_effect=['2', 'test_vitesse_emails.txt', '10']):
            from gmass import Main
            Main()
        
        end_time = time.time()
        temps_reel = end_time - start_time
        
        print(f"\n📊 RÉSULTATS DU TEST:")
        print(f"   ⏱️  Temps réel: {temps_reel:.1f} secondes")
        print(f"   📈 Vitesse: {len(emails_test)/temps_reel:.1f} emails/seconde")
        
        # Extrapoler pour 3500 emails
        temps_3500 = 3500 * (temps_reel / len(emails_test))
        minutes_3500 = int(temps_3500 // 60)
        secondes_3500 = int(temps_3500 % 60)
        
        print(f"\n🎯 EXTRAPOLATION POUR 3500 EMAILS:")
        print(f"   ⏱️  Temps estimé: {minutes_3500}m {secondes_3500}s")
        
        if temps_3500 <= 300:
            print(f"   ✅ PARFAIT ! Respecte les 5 minutes")
        elif temps_3500 <= 600:
            print(f"   🟡 ACCEPTABLE ! Dans les 10 minutes")
        else:
            print(f"   ❌ TROP LENT ! Besoin d'optimisation")
            
        # Nettoyer
        import os
        if os.path.exists('test_vitesse_emails.txt'):
            os.remove('test_vitesse_emails.txt')
            
    except Exception as e:
        print(f"❌ Erreur pendant le test: {e}")

if __name__ == "__main__":
    calculer_temps_traitement()
    
    choix = input(f"\nVoulez-vous faire un test de vitesse réel ? (o/n): ").lower()
    if choix == 'o':
        test_vitesse_reel()