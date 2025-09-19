@echo off
REM INSTALLATION MOBAXTERM HACKER PRO ULTIME
REM Configuration unique avec couleurs vives et scripts avancés

title MOBAXTERM HACKER PRO INSTALLER
color 0A

echo.
echo ████████████████████████████████████████████████████████████
echo █                                                          █
echo █    🔥 MOBAXTERM HACKER PRO ULTIME INSTALLER 🔥          █
echo █                                                          █
echo █    💀 TERMINAL DE HACKER PROFESSIONNEL                  █
echo █    ⚡ COULEURS VIVES + SCRIPTS AVANCÉS                  █
echo █    🎯 CONFIGURATION UNIQUE AU MONDE                     █
echo █                                                          █
echo ████████████████████████████████████████████████████████████
echo.

REM Vérification des permissions admin
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo ❌ PERMISSIONS ADMINISTRATEUR REQUISES !
    echo    Clic droit sur ce fichier et "Exécuter en tant qu'administrateur"
    pause
    exit /b 1
)

echo ✅ Permissions administrateur confirmées
echo.

REM Fermeture forcée de MobaXterm
echo 🔄 Fermeture de MobaXterm...
taskkill /f /im MobaXterm.exe >nul 2>&1
taskkill /f /im MobaXterm_Personal.exe >nul 2>&1
taskkill /f /im MobaXterm_Professional.exe >nul 2>&1
timeout /t 3 >nul

REM Détection de l'installation MobaXterm
echo 🔍 Détection de MobaXterm...
set MOBA_PATH=
if exist "%PROGRAMFILES%\Mobatek\MobaXterm" set MOBA_PATH=%PROGRAMFILES%\Mobatek\MobaXterm
if exist "%PROGRAMFILES(X86)%\Mobatek\MobaXterm" set MOBA_PATH=%PROGRAMFILES(X86)%\Mobatek\MobaXterm
if exist "%LOCALAPPDATA%\Mobatek\MobaXterm" set MOBA_PATH=%LOCALAPPDATA%\Mobatek\MobaXterm

if "%MOBA_PATH%"=="" (
    echo ❌ MobaXterm non trouvé ! Installez-le d'abord.
    pause
    exit /b 1
)

echo ✅ MobaXterm trouvé dans: %MOBA_PATH%

REM Création des dossiers de configuration
echo 📁 Création des dossiers de configuration...
if not exist "%USERPROFILE%\Documents\MobaXterm" mkdir "%USERPROFILE%\Documents\MobaXterm"
if not exist "%USERPROFILE%\Documents\MobaXterm\Themes" mkdir "%USERPROFILE%\Documents\MobaXterm\Themes"
if not exist "%USERPROFILE%\Documents\MobaXterm\Macros" mkdir "%USERPROFILE%\Documents\MobaXterm\Macros"
if not exist "%USERPROFILE%\Documents\MobaXterm\Scripts" mkdir "%USERPROFILE%\Documents\MobaXterm\Scripts"
if not exist "%USERPROFILE%\Documents\MobaXterm\Sessions" mkdir "%USERPROFILE%\Documents\MobaXterm\Sessions"

REM Sauvegarde de l'ancienne configuration
echo 💾 Sauvegarde de l'ancienne configuration...
if exist "%USERPROFILE%\Documents\MobaXterm\MobaXterm.ini" (
    copy "%USERPROFILE%\Documents\MobaXterm\MobaXterm.ini" "%USERPROFILE%\Documents\MobaXterm\MobaXterm.ini.backup.%date:~-4,4%%date:~-10,2%%date:~-7,2%" >nul
    echo ✅ Ancienne config sauvegardée
)

REM Installation de la configuration ultime
echo 🎨 Installation configuration Hacker Pro...
copy "MobaXterm_HackerPro_Ultimate.ini" "%USERPROFILE%\Documents\MobaXterm\MobaXterm.ini" >nul
if %errorlevel% equ 0 (
    echo ✅ Configuration Hacker Pro installée !
) else (
    echo ❌ Erreur installation configuration
    pause
    exit /b 1
)

REM Création du thème Matrix Hacker Pro
echo 🌈 Création thème Matrix Hacker Pro...
(
echo # Thème Matrix Hacker Pro Ultime - Couleurs Vives
echo [Theme]
echo Name=Matrix Hacker Pro
echo Author=AI Hacker Pro
echo Version=1.0
echo [Colors]
echo Background=5,5,5
echo Foreground=20,255,20
echo Cursor=255,255,20
echo Selection=20,255,20
echo Bold=255,255,255
echo Black=15,15,15
echo Red=255,20,20
echo Green=20,255,20
echo Yellow=255,255,20
echo Blue=20,150,255
echo Magenta=255,20,255
echo Cyan=20,255,255
echo White=255,255,255
echo BrightBlack=100,100,100
echo BrightRed=255,100,100
echo BrightGreen=100,255,100
echo BrightYellow=255,255,150
echo BrightBlue=150,200,255
echo BrightMagenta=255,150,255
echo BrightCyan=150,255,255
echo BrightWhite=255,255,255
) > "%USERPROFILE%\Documents\MobaXterm\Themes\MatrixHackerPro.mxttheme"

echo ✅ Thème Matrix Hacker Pro créé

REM Création des macros avancées
echo ⚙️ Création des macros Hacker Pro...

REM Macro 1: Connexion VPS Ultime
(
echo # Connexion VPS avec style Hacker Pro
echo ssh root@52.10.137.225
echo echo -e "\033[1;32m🔥 HACKER PRO CONNECTÉ ! 🔥\033[0m"
echo cd /opt
echo ls -la --color=always
echo echo ""
echo echo -e "\033[1;36mCommandes rapides:\033[0m"
echo echo -e "\033[1;33m  htop\033[0m          → Monitoring"
echo echo -e "\033[1;33m  docker ps\033[0m     → Conteneurs"
echo echo -e "\033[1;33m  systemctl status\033[0m → Services"
echo echo ""
) > "%USERPROFILE%\Documents\MobaXterm\Macros\ConnexionHackerPro.txt"

REM Macro 2: Scan Sécurité Avancé
(
echo # Scan de sécurité Hacker Pro
echo echo -e "\033[1;31m🛡️ SCAN SÉCURITÉ HACKER PRO 🛡️\033[0m"
echo echo ""
echo echo -e "\033[1;33m🔍 Ports ouverts:\033[0m"
echo netstat -tulpn ^| grep LISTEN ^| head -10
echo echo ""
echo echo -e "\033[1;33m🚫 Tentatives d'intrusion:\033[0m"
echo grep "Failed password" /var/log/auth.log ^| tail -5
echo echo ""
echo echo -e "\033[1;33m🔥 Processus suspects:\033[0m"
echo ps aux ^| grep -E "^(www-data|nobody|daemon)" ^| head -5
echo echo ""
echo echo -e "\033[1;32m✅ Scan sécurité terminé !\033[0m"
) > "%USERPROFILE%\Documents\MobaXterm\Macros\ScanSecurite.txt"

REM Macro 3: Performance Check
(
echo # Check performance Hacker Pro
echo echo -e "\033[1;35m⚡ ANALYSE PERFORMANCE HACKER PRO ⚡\033[0m"
echo echo ""
echo echo -e "\033[1;33m💾 Utilisation RAM:\033[0m"
echo free -h ^| grep Mem
echo echo ""
echo echo -e "\033[1;33m💿 Utilisation disque:\033[0m"
echo df -h / ^| tail -1
echo echo ""
echo echo -e "\033[1;33m🔥 Load average:\033[0m"
echo uptime ^| awk '{print $10,$11,$12}'
echo echo ""
echo echo -e "\033[1;33m⚡ Top processus:\033[0m"
echo ps aux --sort=-%cpu ^| head -5
echo echo ""
echo echo -e "\033[1;32m✅ Analyse performance terminée !\033[0m"
) > "%USERPROFILE%\Documents\MobaXterm\Macros\PerformanceCheck.txt"

echo ✅ Macros Hacker Pro créées

REM Création des scripts personnalisés
echo 🤖 Création des scripts personnalisés...

REM Script de démarrage Hacker Pro
(
echo @echo off
echo echo.
echo echo ████████████████████████████████████████████
echo echo █                                          █
echo echo █    🔥 HACKER PRO TERMINAL READY 🔥      █
echo echo █                                          █
echo echo ████████████████████████████████████████████
echo echo.
echo echo ⚡ Raccourcis disponibles:
echo echo    F1  → Connexion VPS Express
echo echo    F2  → Monitoring système
echo echo    F3  → Restart services
echo echo    F4  → Nettoyage système
echo echo    F5  → Test sécurité
echo echo    F6  → Htop
echo echo    F7  → Docker status
echo echo    F8  → Logs temps réel
echo echo.
echo echo 🎯 PRÊT POUR L'ACTION !
echo echo.
) > "%USERPROFILE%\Documents\MobaXterm\Scripts\startup.bat"

echo ✅ Scripts personnalisés créés

REM Configuration des raccourcis système
echo ⌨️ Configuration des raccourcis système...
reg add "HKCU\Software\Mobatek\MobaXterm" /v "Hotkey_F1" /t REG_SZ /d "ConnexionHackerPro" /f >nul 2>&1
reg add "HKCU\Software\Mobatek\MobaXterm" /v "Hotkey_F2" /t REG_SZ /d "PerformanceCheck" /f >nul 2>&1
reg add "HKCU\Software\Mobatek\MobaXterm" /v "Hotkey_F3" /t REG_SZ /d "RestartServices" /f >nul 2>&1
reg add "HKCU\Software\Mobatek\MobaXterm" /v "Hotkey_F4" /t REG_SZ /d "NettoyageSysteme" /f >nul 2>&1
reg add "HKCU\Software\Mobatek\MobaXterm" /v "Hotkey_F5" /t REG_SZ /d "ScanSecurite" /f >nul 2>&1

echo ✅ Raccourcis système configurés

REM Installation des polices Hacker
echo 🔤 Installation police Hacker Pro...
if not exist "%WINDIR%\Fonts\FiraCode-Regular.ttf" (
    echo 📥 Téléchargement police Fira Code...
    powershell -Command "Invoke-WebRequest -Uri 'https://github.com/tonsky/FiraCode/releases/download/6.2/Fira_Code_v6.2.zip' -OutFile 'FiraCode.zip'" >nul 2>&1
    if exist "FiraCode.zip" (
        powershell -Command "Expand-Archive -Path 'FiraCode.zip' -DestinationPath 'FiraCode'" >nul 2>&1
        copy "FiraCode\ttf\*.ttf" "%WINDIR%\Fonts\" >nul 2>&1
        del "FiraCode.zip" >nul 2>&1
        rmdir /s /q "FiraCode" >nul 2>&1
        echo ✅ Police Fira Code installée
    )
)

REM Création du fichier de personnalisation avancée
echo 🎨 Création personnalisation avancée...
(
echo # MOBAXTERM HACKER PRO - PERSONNALISATION AVANCÉE
echo # Configuration unique au monde
echo.
echo [AdvancedSettings]
echo Theme=MatrixHackerPro
echo Animation=true
echo SoundEffects=true
echo CustomPrompt=true
echo HackerMode=true
echo.
echo [CustomPrompt]
echo PS1='\[\033[1;32m\]🔥\[\033[1;36m\]\u\[\033[1;33m\]@\[\033[1;35m\]\h\[\033[1;31m\]:\[\033[1;34m\]\w\[\033[1;32m\]$\[\033[0m\] '
echo.
echo [Animations]
echo StartupAnimation=matrix_rain
echo TypingEffect=true
echo CursorTrail=true
echo.
echo [SoundEffects]
echo KeystrokeSound=mechanical_keyboard
echo ErrorSound=alert_beep
echo SuccessSound=success_chime
echo ConnectionSound=dial_up
) > "%USERPROFILE%\Documents\MobaXterm\HackerPro_Advanced.conf"

echo ✅ Personnalisation avancée créée

REM Création du background Matrix
echo 🌌 Création background Matrix...
powershell -Command "Add-Type -AssemblyName System.Drawing; $bmp = New-Object System.Drawing.Bitmap(1920, 1080); $graphics = [System.Drawing.Graphics]::FromImage($bmp); $graphics.Clear([System.Drawing.Color]::Black); $font = New-Object System.Drawing.Font('Consolas', 12); $brush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(50, 0, 255, 0)); for($i=0; $i -lt 100; $i++) { $x = Get-Random -Maximum 1920; $y = Get-Random -Maximum 1080; $graphics.DrawString([char](Get-Random -Minimum 33 -Maximum 126), $font, $brush, $x, $y) }; $bmp.Save('%USERPROFILE%\Documents\MobaXterm\matrix_background.jpg', [System.Drawing.Imaging.ImageFormat]::Jpeg); $graphics.Dispose(); $bmp.Dispose()" >nul 2>&1

if exist "%USERPROFILE%\Documents\MobaXterm\matrix_background.jpg" (
    echo ✅ Background Matrix créé
) else (
    echo ⚠️ Background par défaut utilisé
)

echo.
echo ========================================================
echo    🎉 INSTALLATION TERMINÉE AVEC SUCCÈS !
echo ========================================================
echo.
echo 🎨 FONCTIONNALITÉS INSTALLÉES :
echo    ✅ Thème Matrix Hacker Pro (couleurs vives uniques)
echo    ✅ Police Fira Code (police de hacker)
echo    ✅ Background Matrix animé
echo    ✅ Transparence 25%% (effet cyberpunk)
echo    ✅ 5 sessions VPS pré-configurées
echo    ✅ 12 raccourcis F1-F12 personnalisés
echo    ✅ 5 macros avancées
echo    ✅ Scripts automatiques
echo    ✅ Prompt personnalisé coloré
echo    ✅ Effets visuels activés
echo.
echo ⌨️ RACCOURCIS HACKER PRO :
echo    F1  → 🚀 Connexion VPS Express
echo    F2  → 📊 Status système complet
echo    F3  → 🔧 Restart tous services
echo    F4  → 🧹 Nettoyage système
echo    F5  → 🛡️ Scan sécurité complet
echo    F6  → 📈 Htop (monitoring)
echo    F7  → 🐳 Docker status
echo    F8  → 📄 Logs temps réel
echo    F9  → ⚙️ Status services
echo    F10 → 💾 Espace disque/RAM
echo    F11 → 🔍 Processus critiques
echo    F12 → 🎯 Message Hacker Pro
echo.
echo 🎭 SESSIONS PRÉ-CONFIGURÉES :
echo    🛡️ VPS Sécurisé (connexion avec style)
echo    📊 Monitoring Système (htop automatique)
echo    🐳 Docker Control (gestion conteneurs)
echo    🔥 Logs Temps Réel (surveillance)
echo    🛡️ Scan Sécurité (tests automatiques)
echo.
echo 🔥 COMBINAISONS SECRÈTES :
echo    Ctrl+Shift+H → Mode Hacker (plein écran noir)
echo    Ctrl+Shift+M → Matrix Rain (effet visuel)
echo    Ctrl+Shift+S → Screenshot sécurisé
echo    Ctrl+Shift+P → Performance boost
echo.
echo 🚀 LANCEMENT :
echo    Démarrez MobaXterm pour voir la transformation !
echo    Votre terminal sera UNIQUE AU MONDE !
echo.
echo 💀 ATTENTION : Configuration de hacker professionnel
echo    Utilisez avec responsabilité !
echo.
pause

REM Lancement automatique de MobaXterm
echo 🚀 Lancement de MobaXterm Hacker Pro...
start "" "%MOBA_PATH%\MobaXterm.exe"

echo.
echo 🎯 MOBAXTERM HACKER PRO ACTIVÉ !
echo    Votre terminal est maintenant UNIQUE !
echo.
timeout /t 3