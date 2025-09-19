@echo off
REM INSTALLATION CONFIGURATION MOBAXTERM ULTIME
REM Script automatique pour appliquer la personnalisation

echo.
echo ========================================================
echo    🎯 INSTALLATION MOBAXTERM ULTIME EN COURS...
echo ========================================================
echo.

REM Fermer MobaXterm s'il est ouvert
echo 🔄 Fermeture de MobaXterm...
taskkill /f /im MobaXterm.exe >nul 2>&1
taskkill /f /im MobaXterm_Personal.exe >nul 2>&1
timeout /t 2 >nul

REM Sauvegarder l'ancienne configuration
echo 💾 Sauvegarde de l'ancienne configuration...
if exist "%USERPROFILE%\Documents\MobaXterm\MobaXterm.ini" (
    copy "%USERPROFILE%\Documents\MobaXterm\MobaXterm.ini" "%USERPROFILE%\Documents\MobaXterm\MobaXterm.ini.backup" >nul
    echo ✅ Ancienne config sauvegardée
) else (
    echo ℹ️  Aucune configuration existante trouvée
)

REM Créer le dossier MobaXterm s'il n'existe pas
if not exist "%USERPROFILE%\Documents\MobaXterm" (
    mkdir "%USERPROFILE%\Documents\MobaXterm"
    echo ✅ Dossier MobaXterm créé
)

REM Copier la nouvelle configuration
echo 🎨 Installation de la configuration ultime...
copy "MobaXterm_Ultimate.ini" "%USERPROFILE%\Documents\MobaXterm\MobaXterm.ini" >nul
if %errorlevel% equ 0 (
    echo ✅ Configuration ultime installée avec succès !
) else (
    echo ❌ Erreur lors de l'installation
    pause
    exit /b 1
)

REM Installation des thèmes personnalisés
echo 🌈 Installation des thèmes personnalisés...
if not exist "%USERPROFILE%\Documents\MobaXterm\Themes" (
    mkdir "%USERPROFILE%\Documents\MobaXterm\Themes"
)

REM Créer le thème Matrix Hacker
echo # Thème Matrix Hacker Ultime > "%USERPROFILE%\Documents\MobaXterm\Themes\MatrixHacker.mxttheme"
echo [Colors] >> "%USERPROFILE%\Documents\MobaXterm\Themes\MatrixHacker.mxttheme"
echo DefaultForeground=0,255,0 >> "%USERPROFILE%\Documents\MobaXterm\Themes\MatrixHacker.mxttheme"
echo DefaultBackground=0,0,0 >> "%USERPROFILE%\Documents\MobaXterm\Themes\MatrixHacker.mxttheme"
echo CursorColor=255,255,0 >> "%USERPROFILE%\Documents\MobaXterm\Themes\MatrixHacker.mxttheme"

echo ✅ Thème Matrix Hacker installé

REM Créer des macros personnalisées
echo ⚙️ Installation des macros personnalisées...
if not exist "%USERPROFILE%\Documents\MobaXterm\Macros" (
    mkdir "%USERPROFILE%\Documents\MobaXterm\Macros"
)

REM Macro de connexion rapide
echo # Connexion VPS Ultra-Rapide > "%USERPROFILE%\Documents\MobaXterm\Macros\ConnexionVPS.txt"
echo ssh root@52.10.137.225 >> "%USERPROFILE%\Documents\MobaXterm\Macros\ConnexionVPS.txt"
echo cd /opt >> "%USERPROFILE%\Documents\MobaXterm\Macros\ConnexionVPS.txt"
echo echo "🚀 CONNEXION SÉCURISÉE ÉTABLIE !" >> "%USERPROFILE%\Documents\MobaXterm\Macros\ConnexionVPS.txt"

REM Macro monitoring système
echo # Monitoring Système Avancé > "%USERPROFILE%\Documents\MobaXterm\Macros\Monitoring.txt"
echo htop >> "%USERPROFILE%\Documents\MobaXterm\Macros\Monitoring.txt"

echo ✅ Macros personnalisées installées

REM Configuration des raccourcis
echo ⌨️ Configuration des raccourcis ultime...
reg add "HKCU\Software\Mobatek\MobaXterm" /v "Hotkey_F1" /t REG_SZ /d "ConnexionVPS" /f >nul 2>&1
reg add "HKCU\Software\Mobatek\MobaXterm" /v "Hotkey_F2" /t REG_SZ /d "Monitoring" /f >nul 2>&1
reg add "HKCU\Software\Mobatek\MobaXterm" /v "Hotkey_F3" /t REG_SZ /d "htop" /f >nul 2>&1
reg add "HKCU\Software\Mobatek\MobaXterm" /v "Hotkey_F4" /t REG_SZ /d "docker ps" /f >nul 2>&1

echo ✅ Raccourcis configurés (F1-F4)

echo.
echo ========================================================
echo    🎉 MOBAXTERM ULTIME INSTALLÉ AVEC SUCCÈS !
echo ========================================================
echo.
echo 🎨 FONCTIONNALITÉS INSTALLÉES :
echo    ✅ Thème Matrix Hacker (vert sur noir)
echo    ✅ Police Consolas optimisée
echo    ✅ Transparence 15%%
echo    ✅ Curseur jaune clignotant
echo    ✅ Session VPS pré-configurée
echo    ✅ Macros de connexion rapide
echo    ✅ Raccourcis F1-F4 configurés
echo.
echo ⌨️ RACCOURCIS DISPONIBLES :
echo    F1 → Connexion VPS rapide
echo    F2 → Monitoring système
echo    F3 → Htop (processus)
echo    F4 → Docker status
echo.
echo 🚀 DÉMARRAGE :
echo    Lancez MobaXterm pour voir la magie !
echo.
pause