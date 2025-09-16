#!/usr/bin/env python3
import requests
import json
import time

def test_debug():
    """Test de debug pour identifier le problème"""
    
    email = "alain.kleinhans@orange.fr"
    api_key = "5449b291-3f72-498d-9316-362f4ec7168b"
    
    headers = {
        'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
        'Accept': 'application/json, text/plain, */*',
    }
    
    url = f'https://verify.gmass.co/verify?email={email}&key={api_key}'
    
    print(f"🧪 Debug test avec: {email}")
    print(f"🔗 URL: {url}")
    
    try:
        print("⏳ Envoi de la requête...")
        start_time = time.time()
        
        response = requests.get(url, headers=headers, timeout=15)
        
        end_time = time.time()
        print(f"✅ Réponse reçue en {end_time - start_time:.2f}s")
        print(f"📊 Status: {response.status_code}")
        
        result = response.text
        print(f"📄 Réponse brute: {result}")
        
        # Essayer de parser en JSON
        try:
            data = json.loads(result)
            print(f"📋 JSON parsé: {data}")
            
            status_code = data.get('StatusCode', 0)
            valid = data.get('Valid', False)
            status = data.get('Status', 'Unknown')
            
            print(f"🔍 StatusCode: {status_code}")
            print(f"🔍 Valid: {valid}")
            print(f"🔍 Status: {status}")
            
            if status_code == 250 and valid:
                print("✅ Email VALIDE")
            elif status_code == 550 or not valid:
                print("❌ Email INVALIDE")
            else:
                print("❓ Statut INCONNU")
                
        except json.JSONDecodeError as e:
            print(f"❌ Erreur JSON: {e}")
            
    except requests.exceptions.Timeout:
        print("⏰ TIMEOUT")
    except Exception as e:
        print(f"❌ Erreur: {e}")

if __name__ == "__main__":
    test_debug()