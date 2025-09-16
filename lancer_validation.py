#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Script de lancement pour valider les emails Orange
"""

import sys
import os
from unittest.mock import patch
import time

# Ajouter le répertoire courant au path
sys.path.append('/workspace')

def lancer_validation_orange():
    print("🍊 VALIDATION DES EMAILS ORANGE")
    print("=" * 50)
    print("📧 Fichier: test_emails.txt (200 emails)")
    print("🔑 Clé API: 5449b291-3f72-498d-9316-362f4ec7168b")
    print("⏱️  Rate limiting: 1 requête/seconde")
    print("🧵 Threads recommandés: 5-10")
    print("-" * 50)
    
    choix = input("Voulez-vous continuer ? (o/n): ").lower()
    if choix != 'o':
        print("❌ Annulé par l'utilisateur")
        return
    
    nb_threads = input("Nombre de threads (défaut: 5): ").strip()
    if not nb_threads:
        nb_threads = "5"
    
    print(f"\n🚀 Démarrage de la validation avec {nb_threads} threads...")
    print("⚠️  Cela peut prendre plusieurs minutes (rate limiting actif)")
    
    # Simuler les entrées utilisateur pour le script principal
    with patch('builtins.input', side_effect=['2', 'test_emails.txt', nb_threads]):
        try:
            from gmass import Main
            Main()
        except KeyboardInterrupt:
            print("\n⏹️  Validation interrompue par l'utilisateur")
        except Exception as e:
            print(f"❌ Erreur lors de l'exécution: {e}")

if __name__ == "__main__":
    lancer_validation_orange()