# -*- coding: utf-8 -*-
#!/usr/bin/python3
# Ces outils prennent du temps, soyez patient
# Modernisé pour Python 3 avec debouncing amélioré
# Créé le 18 janvier 2022, modifié le 16 septembre 2025

import requests
import re
import os
import sys
import random
import time
import threading
from multiprocessing.dummy import Pool
from time import time as timer
from colorama import Fore, init
from collections import deque
from datetime import datetime, timedelta

# Initialiser colorama pour Windows
init(autoreset=True)

# Configuration du debouncing adaptatif intelligent
class AdaptiveRateLimiter:
    def __init__(self, initial_rate=5, min_rate=1, max_rate=15):
        self.current_rate = initial_rate
        self.min_rate = min_rate
        self.max_rate = max_rate
        self.requests = deque()
        self.lock = threading.Lock()
        
        # Statistiques pour l'adaptation
        self.success_count = 0
        self.timeout_count = 0
        self.error_count = 0
        self.total_requests = 0
        self.last_adjustment = time.time()
        self.adjustment_interval = 30  # Ajuster toutes les 30 secondes
        
        print(f"🧠 Rate Limiter Adaptatif initialisé: {self.current_rate} req/sec")
    
    def wait_if_needed(self):
        with self.lock:
            now = time.time()
            
            # Nettoyer les anciennes requêtes
            while self.requests and self.requests[0] <= now - 1:
                self.requests.popleft()
            
            # Attendre si nécessaire selon le taux actuel
            if len(self.requests) >= self.current_rate:
                sleep_time = 1 - (now - self.requests[0])
                if sleep_time > 0:
                    time.sleep(sleep_time)
                    # Nettoyer à nouveau
                    now = time.time()
                    while self.requests and self.requests[0] <= now - 1:
                        self.requests.popleft()
            
            # Enregistrer cette requête
            self.requests.append(now)
    
    def report_success(self):
        """Signaler une requête réussie"""
        with self.lock:
            self.success_count += 1
            self.total_requests += 1
            self._maybe_adjust_rate()
    
    def report_timeout(self):
        """Signaler un timeout - ralentir"""
        with self.lock:
            self.timeout_count += 1
            self.total_requests += 1
            
            # Ralentir immédiatement en cas de timeout
            if self.current_rate > self.min_rate:
                old_rate = self.current_rate
                self.current_rate = max(self.min_rate, self.current_rate - 2)
                print(f"⚠️ TIMEOUT détecté ! Ralentissement: {old_rate} → {self.current_rate} req/sec")
            
            self._maybe_adjust_rate()
    
    def report_error(self):
        """Signaler une erreur"""
        with self.lock:
            self.error_count += 1
            self.total_requests += 1
            self._maybe_adjust_rate()
    
    def _maybe_adjust_rate(self):
        """Ajuster le taux selon les performances"""
        now = time.time()
        
        # Ajuster seulement si assez de temps s'est écoulé et qu'on a des données
        if (now - self.last_adjustment < self.adjustment_interval or 
            self.total_requests < 20):
            return
        
        # Calculer les taux de succès et d'erreur
        success_rate = self.success_count / self.total_requests
        timeout_rate = self.timeout_count / self.total_requests
        error_rate = self.error_count / self.total_requests
        
        old_rate = self.current_rate
        
        # Logique d'ajustement
        if timeout_rate > 0.1:  # Plus de 10% de timeouts
            # Ralentir agressivement
            self.current_rate = max(self.min_rate, self.current_rate - 3)
            print(f"🐌 Trop de timeouts ({timeout_rate*100:.1f}%) ! Ralentissement: {old_rate} → {self.current_rate} req/sec")
            
        elif timeout_rate > 0.05:  # Plus de 5% de timeouts
            # Ralentir modérément
            self.current_rate = max(self.min_rate, self.current_rate - 1)
            print(f"⚠️ Timeouts détectés ({timeout_rate*100:.1f}%) ! Ajustement: {old_rate} → {self.current_rate} req/sec")
            
        elif success_rate > 0.9 and timeout_rate < 0.02:  # Plus de 90% de succès, moins de 2% timeouts
            # Accélérer prudemment
            self.current_rate = min(self.max_rate, self.current_rate + 1)
            print(f"🚀 Bonnes performances ({success_rate*100:.1f}% succès) ! Accélération: {old_rate} → {self.current_rate} req/sec")
        
        # Réinitialiser les compteurs
        self.success_count = 0
        self.timeout_count = 0
        self.error_count = 0
        self.total_requests = 0
        self.last_adjustment = now
    
    def get_stats(self):
        """Obtenir les statistiques actuelles"""
        with self.lock:
            return {
                'current_rate': self.current_rate,
                'total_requests': self.total_requests,
                'success_count': self.success_count,
                'timeout_count': self.timeout_count,
                'error_count': self.error_count
            }

# Instance globale du rate limiter adaptatif
rate_limiter = AdaptiveRateLimiter(initial_rate=5, min_rate=1, max_rate=12)  # Débute à 5 req/sec, s'adapte entre 1-12								

def Banner():
    clear = '\x1b[0m'
    colors = [36, 32, 34, 35, 31, 37]

    x = '''
    _____  
^..^     \\9
(oo)_____/ 
   WW  WW
1 . EXTRAIRE LES EMAILS
2 . VALIDATEUR GMASS.CO (avec debouncing amélioré)               
'''
    for N, line in enumerate(x.split('\n')):
        sys.stdout.write('\x1b[1;%dm%s%s\n' % (random.choice(colors), line, clear))
        time.sleep(0.02)

# Compteurs de progression
class ProgressTracker:
    def __init__(self):
        self.total = 0
        self.processed = 0
        self.valid = 0
        self.invalid = 0
        self.errors = 0
        self.lock = threading.Lock()
    
    def update(self, status):
        with self.lock:
            self.processed += 1
            if status == 'valid':
                self.valid += 1
            elif status == 'invalid':
                self.invalid += 1
            elif status == 'error':
                self.errors += 1
            
            if self.processed % 10 == 0:  # Afficher le progrès tous les 10 emails
                print(f"{Fore.CYAN}Progrès: {self.processed}/{self.total} - Valides: {self.valid}, Invalides: {self.invalid}, Erreurs: {self.errors}")

progress_tracker = ProgressTracker()

Banner()

choose = input(':~# \033[34mChoisissez\033[32m un numéro : ')

def Extract():
    try:
        fichier = input(' \033[34mFichier à extraire\033[32m (emails) : ')
        if not os.path.exists(fichier):
            print(f"{Fore.RED}Erreur: Le fichier {fichier} n'existe pas!")
            return
        
        emails_extraits = set()  # Utiliser un set pour éviter les doublons
        
        with open(fichier, 'r', encoding='utf-8', errors='ignore') as f:
            contenu = f.read()
            # Pattern d'email amélioré
            pattern_email = r'\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b'
            emails_trouves = re.findall(pattern_email, contenu)
            
            for email in emails_trouves:
                emails_extraits.add(email.lower())  # Normaliser en minuscules
        
        if emails_extraits:
            with open('ResMail.txt', 'w', encoding='utf-8') as f:
                for email in sorted(emails_extraits):
                    f.write(email + '\n')
                    print(f"{Fore.GREEN}EMAIL EXTRAIT : {email}")
            
            print(f"{Fore.CYAN}Total: {len(emails_extraits)} emails uniques extraits dans ResMail.txt")
        else:
            print(f"{Fore.RED}Aucun email trouvé dans le fichier!")
            
    except Exception as e:
        print(f"{Fore.RED}Erreur lors de l'extraction: {str(e)}")
        pass

def Gmass(email):
    """Valider un email avec GMASS.co avec debouncing intégré"""
    try:
        # Appliquer le rate limiting
        rate_limiter.wait_if_needed()
        
        headers = {
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36',
            'Accept': 'application/json, text/plain, */*',
            'Accept-Language': 'fr-FR,fr;q=0.9,en;q=0.8',
            'Accept-Encoding': 'gzip, deflate, br',
            'Connection': 'keep-alive',
            'Referer': 'https://gmass.co/',
            'Cookie': 'GMassUniqueID=558d3296-b37b-4cb8-8c7a-3e188c22e793; GMassAffiliateID='
        }
        
        # Clé API personnalisée fournie par l'utilisateur
        api_key = '5449b291-3f72-498d-9316-362f4ec7168b'
        url = f'https://verify.gmass.co/verify?email={email}&key={api_key}'
        
        # Timeout plus long pour éviter les erreurs de connexion
        response = requests.get(url, headers=headers, timeout=30)
        response.raise_for_status()
        
        result = response.text
        
        # Analyser la réponse JSON
        try:
            import json
            data = json.loads(result)
            
            # Format de réponse GMASS actuel
            if 'Success' in data:
                success = data.get('Success', False)
                valid = data.get('Valid', False)
                status = data.get('Status', 'Unknown')
                smtp_code = data.get('SMTPCode', 0)
                
                if success and valid:
                    print(f"{Fore.YELLOW}[ VALIDE --> ] {Fore.GREEN}{email}")
                    with open('Mail_OK.txt', 'a', encoding='utf-8') as f:
                        f.write(email + '\n')
                    progress_tracker.update('valid')
                    rate_limiter.report_success()  # Signaler le succès
                    return 'valid'
                    
                elif success and not valid:
                    print(f"{Fore.YELLOW}[ INVALIDE --> ] {Fore.RED}{email} ({status})")
                    with open('Mail_FAILED.txt', 'a', encoding='utf-8') as f:
                        f.write(email + '\n')
                    progress_tracker.update('invalid')
                    rate_limiter.report_success()  # C'est un succès API même si email invalide
                    return 'invalid'
                    
                else:
                    print(f"{Fore.YELLOW}[ ERREUR --> ] {Fore.MAGENTA}{email} (Status: {status}, SMTP: {smtp_code})")
                    with open('Mail_ERROR.txt', 'a', encoding='utf-8') as f:
                        f.write(f"{email} - Status: {status}, SMTP: {smtp_code}\n")
                    progress_tracker.update('error')
                    rate_limiter.report_error()  # Signaler l'erreur
                    return 'error'
                    
            # Ancien format de réponse avec StatusCode
            elif 'StatusCode' in data:
                status_code = data.get('StatusCode', 0)
                valid = data.get('Valid', False)
                status = data.get('Status', 'Unknown')
                
                if status_code == 250 and valid:
                    print(f"{Fore.YELLOW}[ VALIDE --> ] {Fore.GREEN}{email}")
                    with open('Mail_OK.txt', 'a', encoding='utf-8') as f:
                        f.write(email + '\n')
                    progress_tracker.update('valid')
                    return 'valid'
                    
                elif status_code == 550 or not valid:
                    print(f"{Fore.YELLOW}[ INVALIDE --> ] {Fore.RED}{email} ({status})")
                    with open('Mail_FAILED.txt', 'a', encoding='utf-8') as f:
                        f.write(email + '\n')
                    progress_tracker.update('invalid')
                    return 'invalid'
                    
                else:
                    print(f"{Fore.YELLOW}[ INCONNU --> ] {Fore.MAGENTA}{email} (Status: {status}, Code: {status_code})")
                    with open('Mail_UNKNOWN.txt', 'a', encoding='utf-8') as f:
                        f.write(f"{email} - Status: {status}, Code: {status_code}\n")
                    progress_tracker.update('error')
                    return 'unknown'
                    
            # Ancien format de réponse (fallback)
            elif '"SMTPCode":250' in result:
                print(f"{Fore.YELLOW}[ VALIDE --> ] {Fore.GREEN}{email}")
                with open('Mail_OK.txt', 'a', encoding='utf-8') as f:
                    f.write(email + '\n')
                progress_tracker.update('valid')
                return 'valid'
                
            elif '"SMTPCode":550' in result:
                print(f"{Fore.YELLOW}[ INVALIDE --> ] {Fore.RED}{email}")
                with open('Mail_FAILED.txt', 'a', encoding='utf-8') as f:
                    f.write(email + '\n')
                progress_tracker.update('invalid')
                return 'invalid'
                
            else:
                print(f"{Fore.YELLOW}[ INCONNU --> ] {Fore.MAGENTA}{email} (Réponse: {result[:100]}...)")
                with open('Mail_UNKNOWN.txt', 'a', encoding='utf-8') as f:
                    f.write(f"{email} - {result[:200]}\n")
                progress_tracker.update('error')
                return 'unknown'
                
        except json.JSONDecodeError:
            # Si ce n'est pas du JSON, utiliser l'ancienne méthode
            if '"SMTPCode":250' in result:
                print(f"{Fore.YELLOW}[ VALIDE --> ] {Fore.GREEN}{email}")
                with open('Mail_OK.txt', 'a', encoding='utf-8') as f:
                    f.write(email + '\n')
                progress_tracker.update('valid')
                return 'valid'
                
            elif '"SMTPCode":550' in result:
                print(f"{Fore.YELLOW}[ INVALIDE --> ] {Fore.RED}{email}")
                with open('Mail_FAILED.txt', 'a', encoding='utf-8') as f:
                    f.write(email + '\n')
                progress_tracker.update('invalid')
                return 'invalid'
                
            else:
                print(f"{Fore.YELLOW}[ INCONNU --> ] {Fore.MAGENTA}{email} (Réponse non-JSON: {result[:100]}...)")
                with open('Mail_UNKNOWN.txt', 'a', encoding='utf-8') as f:
                    f.write(f"{email} - {result[:200]}\n")
                progress_tracker.update('error')
                return 'unknown'
            
    except requests.exceptions.Timeout:
        print(f"{Fore.RED}[ TIMEOUT --> ] {email}")
        with open('Mail_TIMEOUT.txt', 'a', encoding='utf-8') as f:
            f.write(email + '\n')
        progress_tracker.update('error')
        rate_limiter.report_timeout()  # Signaler le timeout au rate limiter
        return 'timeout'
        
    except requests.exceptions.RequestException as e:
        print(f"{Fore.RED}[ ERREUR RÉSEAU --> ] {email} : {str(e)}")
        with open('Mail_ERROR.txt', 'a', encoding='utf-8') as f:
            f.write(f"{email} - {str(e)}\n")
        progress_tracker.update('error')
        rate_limiter.report_error()  # Signaler l'erreur au rate limiter
        return 'error'
        
    except Exception as e:
        print(f"{Fore.RED}[ ERREUR --> ] {email} : {str(e)}")
        with open('Mail_ERROR.txt', 'a', encoding='utf-8') as f:
            f.write(f"{email} - {str(e)}\n")
        progress_tracker.update('error')
        rate_limiter.report_error()  # Signaler l'erreur au rate limiter
        return 'error'

def Main():
    try:
        if choose == '1':
            Extract()
            
        elif choose == '2':
            fichier_emails = input("\n\033[91mListe d'emails\033[97m:~# \033[97m")
            
            if not os.path.exists(fichier_emails):
                print(f"{Fore.RED}Erreur: Le fichier {fichier_emails} n'existe pas!")
                return
            
            # Lire les emails
            with open(fichier_emails, 'r', encoding='utf-8', errors='ignore') as f:
                emails = [line.strip() for line in f if line.strip() and '@' in line]
            
            if not emails:
                print(f"{Fore.RED}Aucun email valide trouvé dans le fichier!")
                return
            
            print(f"{Fore.CYAN}Nombre d'emails à valider: {len(emails)}")
            
            # Initialiser le tracker de progression
            progress_tracker.total = len(emails)
            
            # Demander le nombre de threads
            try:
                nb_threads = input(f"{Fore.YELLOW}Nombre de threads (recommandé: 5-10, défaut: 5): ").strip()
                nb_threads = int(nb_threads) if nb_threads else 5
                nb_threads = max(1, min(nb_threads, 20))  # Limiter entre 1 et 20
            except ValueError:
                nb_threads = 5
            
            print(f"{Fore.CYAN}Démarrage de la validation avec {nb_threads} threads...")
            print(f"{Fore.YELLOW}🧠 Rate Limiter ADAPTATIF activé - S'ajuste automatiquement selon les performances !")
            print(f"{Fore.CYAN}   • Débute à 5 req/sec")
            print(f"{Fore.CYAN}   • Ralentit si timeouts détectés (min: 1 req/sec)")
            print(f"{Fore.CYAN}   • Accélère si tout va bien (max: 12 req/sec)")
            
            # Nettoyer les anciens fichiers de résultats
            for fichier in ['Mail_OK.txt', 'Mail_FAILED.txt', 'Mail_UNKNOWN.txt', 'Mail_TIMEOUT.txt', 'Mail_ERROR.txt']:
                if os.path.exists(fichier):
                    os.remove(fichier)
            
            # Démarrer la validation avec pool de threads
            start_time = timer()
            
            with Pool(nb_threads) as pool:
                resultats = pool.map(Gmass, emails)
            
            end_time = timer()
            
            # Afficher les statistiques finales
            print(f"\n{Fore.CYAN}=== RAPPORT FINAL ===")
            print(f"{Fore.GREEN}Emails valides: {progress_tracker.valid}")
            print(f"{Fore.RED}Emails invalides: {progress_tracker.invalid}")
            print(f"{Fore.MAGENTA}Erreurs/Timeouts: {progress_tracker.errors}")
            print(f"{Fore.YELLOW}Total traité: {progress_tracker.processed}/{progress_tracker.total}")
            print(f"{Fore.CYAN}Temps total: {end_time - start_time:.2f} secondes")
            
            # Statistiques du rate limiter adaptatif
            stats = rate_limiter.get_stats()
            vitesse_finale = progress_tracker.processed / (end_time - start_time) if (end_time - start_time) > 0 else 0
            print(f"\n{Fore.CYAN}=== PERFORMANCE ADAPTATIVE ===")
            print(f"{Fore.YELLOW}🧠 Taux final: {rate_limiter.current_rate} req/sec")
            print(f"{Fore.YELLOW}⚡ Vitesse moyenne: {vitesse_finale:.1f} emails/sec")
            
            # Estimation pour 3500 emails
            if vitesse_finale > 0:
                temps_3500 = 3500 / vitesse_finale
                minutes = int(temps_3500 // 60)
                secondes = int(temps_3500 % 60)
                print(f"{Fore.CYAN}📊 Estimation pour 3500 emails: ~{minutes}m {secondes}s")
            
            if progress_tracker.valid > 0:
                print(f"{Fore.GREEN}✓ Emails valides sauvés dans: Mail_OK.txt")
            if progress_tracker.invalid > 0:
                print(f"{Fore.RED}✗ Emails invalides sauvés dans: Mail_FAILED.txt")
            if progress_tracker.errors > 0:
                print(f"{Fore.MAGENTA}! Erreurs sauvées dans: Mail_ERROR.txt, Mail_TIMEOUT.txt, Mail_UNKNOWN.txt")
                
        else:
            print(f"{Fore.RED}Choix invalide! Veuillez choisir 1 ou 2.")
            
    except KeyboardInterrupt:
        print(f"\n{Fore.YELLOW}Interruption par l'utilisateur. Arrêt en cours...")
        
    except Exception as e:
        print(f"{Fore.RED}Erreur dans Main(): {str(e)}")

if __name__ == '__main__':
    Main()
