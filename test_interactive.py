#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Test interactif simulé du script principal
"""

import sys
import os
from unittest.mock import patch
import time

# Ajouter le répertoire courant au path
sys.path.append('/workspace')

# Simuler les entrées utilisateur
def simulate_user_input():
    inputs = iter(['2', 'test_simple_emails.txt', '3'])  # Option 2, fichier, 3 threads
    return inputs

def test_simulation():
    print("🧪 SIMULATION DU SCRIPT PRINCIPAL")
    print("=" * 50)
    
    # Simuler les entrées utilisateur
    with patch('builtins.input', side_effect=['2', 'test_simple_emails.txt', '3']):
        try:
            # Importer et exécuter le script principal
            from gmass import Main
            Main()
        except Exception as e:
            print(f"❌ Erreur lors de l'exécution: {e}")
            import traceback
            traceback.print_exc()

if __name__ == "__main__":
    test_simulation()