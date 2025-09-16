use anyhow::Result;
use std::io::{self, Write};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use tokio::time::{Duration, Instant};

pub mod dashboard;
pub mod monitoring;
pub mod interactive;
pub mod stats;

use dashboard::Dashboard;
use monitoring::RealTimeMonitor;
use stats::StatsDisplay;

pub struct UltraCLI {
    dashboard: Dashboard,
    monitor: RealTimeMonitor,
    stats: StatsDisplay,
    current_view: CLIView,
    last_update: Instant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CLIView {
    Dashboard,
    RealTimeMonitor,
    EmailStats,
    SMTPStatus,
    Configuration,
    Logs,
    Help,
}

impl UltraCLI {
    pub fn new() -> Result<Self> {
        // Activer le mode raw terminal pour contrôle total
        terminal::enable_raw_mode()?;
        
        Ok(Self {
            dashboard: Dashboard::new()?,
            monitor: RealTimeMonitor::new()?,
            stats: StatsDisplay::new()?,
            current_view: CLIView::Dashboard,
            last_update: Instant::now(),
        })
    }
    
    pub async fn run(&mut self) -> Result<()> {
        self.print_welcome_banner()?;
        
        loop {
            // Mettre à jour l'affichage toutes les 100ms
            if self.last_update.elapsed() >= Duration::from_millis(100) {
                self.refresh_display().await?;
                self.last_update = Instant::now();
            }
            
            // Gérer les événements clavier
            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if self.handle_key_event(key).await? {
                        break; // Sortir si demandé
                    }
                }
            }
            
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        
        self.cleanup()?;
        Ok(())
    }
    
    fn print_welcome_banner(&self) -> Result<()> {
        execute!(
            io::stdout(),
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0),
            SetForegroundColor(Color::Cyan),
            Print("╔══════════════════════════════════════════════════════════════════════════════╗\n"),
            Print("║                    🚀 ULTRA EMAIL SENDER - VPS CLI v1.0                     ║\n"),
            Print("║                          Tableau de Bord Professionnel                      ║\n"),
            Print("╠══════════════════════════════════════════════════════════════════════════════╣\n"),
            SetForegroundColor(Color::Yellow),
            Print("║  [1] 📊 Dashboard    [2] 📈 Monitor    [3] 📧 Stats    [4] 🔧 SMTP Status  ║\n"),
            Print("║  [5] ⚙️  Config      [6] 📝 Logs      [H] ❓ Help      [Q] 🚪 Quit        ║\n"),
            SetForegroundColor(Color::Cyan),
            Print("╚══════════════════════════════════════════════════════════════════════════════╝\n"),
            ResetColor,
            Print("\n")
        )?;
        
        io::stdout().flush()?;
        Ok(())
    }
    
    async fn refresh_display(&mut self) -> Result<()> {
        match self.current_view {
            CLIView::Dashboard => self.dashboard.render().await?,
            CLIView::RealTimeMonitor => self.monitor.render().await?,
            CLIView::EmailStats => self.stats.render().await?,
            CLIView::SMTPStatus => self.render_smtp_status().await?,
            CLIView::Configuration => self.render_configuration().await?,
            CLIView::Logs => self.render_logs().await?,
            CLIView::Help => self.render_help()?,
        }
        
        self.render_status_bar().await?;
        Ok(())
    }
    
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(true),
            KeyCode::Char('1') => self.current_view = CLIView::Dashboard,
            KeyCode::Char('2') => self.current_view = CLIView::RealTimeMonitor,
            KeyCode::Char('3') => self.current_view = CLIView::EmailStats,
            KeyCode::Char('4') => self.current_view = CLIView::SMTPStatus,
            KeyCode::Char('5') => self.current_view = CLIView::Configuration,
            KeyCode::Char('6') => self.current_view = CLIView::Logs,
            KeyCode::Char('h') | KeyCode::Char('H') => self.current_view = CLIView::Help,
            KeyCode::Char('r') | KeyCode::Char('R') => {
                // Rafraîchir l'affichage
                self.clear_screen()?;
                self.print_welcome_banner()?;
            }
            KeyCode::Esc => {
                self.current_view = CLIView::Dashboard;
            }
            _ => {}
        }
        
        Ok(false)
    }
    
    async fn render_smtp_status(&self) -> Result<()> {
        execute!(
            io::stdout(),
            cursor::MoveTo(0, 8),
            SetForegroundColor(Color::Green),
            Print("╔══════════════════════════════════════════════════════════════════════════════╗\n"),
            Print("║                            🔧 STATUT DES SERVEURS SMTP                      ║\n"),
            Print("╠══════════════════════════════════════════════════════════════════════════════╣\n"),
            ResetColor
        )?;
        
        // Simuler les statuts SMTP (à remplacer par vraies données)
        let smtp_servers = vec![
            ("iCloud-01", "✅", "Actif", "847/1000", "2.3ms", "99.8%"),
            ("iCloud-02", "✅", "Actif", "523/1000", "1.8ms", "99.9%"),
            ("Gmail-01", "⚠️", "Lent", "234/800", "8.7ms", "97.2%"),
            ("Outlook-01", "❌", "Erreur", "0/500", "timeout", "0%"),
        ];
        
        for (name, status, state, usage, latency, success) in smtp_servers {
            execute!(
                io::stdout(),
                Print(format!(
                    "║ {} {:12} │ {:6} │ {:10} │ {:8} │ {:6} │ {:6} ║\n",
                    status, name, state, usage, latency, success
                ))
            )?;
        }
        
        execute!(
            io::stdout(),
            Print("╚══════════════════════════════════════════════════════════════════════════════╝\n")
        )?;
        
        Ok(())
    }
    
    async fn render_configuration(&self) -> Result<()> {
        execute!(
            io::stdout(),
            cursor::MoveTo(0, 8),
            SetForegroundColor(Color::Blue),
            Print("╔══════════════════════════════════════════════════════════════════════════════╗\n"),
            Print("║                              ⚙️ CONFIGURATION SYSTÈME                       ║\n"),
            Print("╠══════════════════════════════════════════════════════════════════════════════╣\n"),
            ResetColor,
            Print("║                                                                              ║\n"),
            Print("║  📁 Fichier Config    : /etc/ultra-sender/config.yaml                      ║\n"),
            Print("║  🗂️  Templates        : /var/lib/ultra-sender/templates/                    ║\n"),
            Print("║  📊 Base de données   : /var/lib/ultra-sender/data/emails.db               ║\n"),
            Print("║  📝 Logs             : /var/log/ultra-sender/                              ║\n"),
            Print("║                                                                              ║\n"),
            Print("║  🔧 Mode actuel      : Natural (Envoi sécurisé)                           ║\n"),
            Print("║  🧵 Threads          : 4 threads actifs                                    ║\n"),
            Print("║  ⏱️  Rate Limit       : 200 emails/heure                                   ║\n"),
            Print("║  🛡️  Sécurité         : Headers dynamiques + DKIM                          ║\n"),
            Print("║                                                                              ║\n"),
            Print("║  Appuyez sur [E] pour éditer la configuration                              ║\n"),
            Print("║                                                                              ║\n"),
            Print("╚══════════════════════════════════════════════════════════════════════════════╝\n")
        )?;
        
        Ok(())
    }
    
    async fn render_logs(&self) -> Result<()> {
        execute!(
            io::stdout(),
            cursor::MoveTo(0, 8),
            SetForegroundColor(Color::Magenta),
            Print("╔══════════════════════════════════════════════════════════════════════════════╗\n"),
            Print("║                                📝 LOGS EN TEMPS RÉEL                        ║\n"),
            Print("╠══════════════════════════════════════════════════════════════════════════════╣\n"),
            ResetColor
        )?;
        
        // Simuler des logs en temps réel
        let logs = vec![
            ("2025-09-16 14:32:15", "INFO", "✅ Email envoyé à jean.dupont@techcorp.com"),
            ("2025-09-16 14:32:14", "INFO", "📧 Batch 15/50 traité (200 emails)"),
            ("2025-09-16 14:32:12", "WARN", "⚠️ SMTP iCloud-02 lent (3.2s)"),
            ("2025-09-16 14:32:10", "INFO", "🔄 Rotation vers SMTP iCloud-01"),
            ("2025-09-16 14:32:08", "INFO", "✅ Headers générés: 247 X-Headers uniques"),
            ("2025-09-16 14:32:06", "INFO", "📊 Variables appliquées pour marie.martin@commerce.fr"),
            ("2025-09-16 14:32:04", "ERROR", "❌ Timeout SMTP Outlook-01 - Passage au suivant"),
            ("2025-09-16 14:32:02", "INFO", "🚀 Campagne démarrée: 3500 destinataires"),
        ];
        
        for (time, level, message) in logs {
            let color = match level {
                "INFO" => Color::Green,
                "WARN" => Color::Yellow,
                "ERROR" => Color::Red,
                _ => Color::White,
            };
            
            execute!(
                io::stdout(),
                SetForegroundColor(color),
                Print(format!("║ {} [{}] {} ║\n", time, level, message)),
                ResetColor
            )?;
        }
        
        execute!(
            io::stdout(),
            Print("║                                                                              ║\n"),
            Print("║  Appuyez sur [C] pour effacer les logs                                     ║\n"),
            Print("╚══════════════════════════════════════════════════════════════════════════════╝\n")
        )?;
        
        Ok(())
    }
    
    fn render_help(&self) -> Result<()> {
        execute!(
            io::stdout(),
            cursor::MoveTo(0, 8),
            SetForegroundColor(Color::Yellow),
            Print("╔══════════════════════════════════════════════════════════════════════════════╗\n"),
            Print("║                                 ❓ AIDE - RACCOURCIS                         ║\n"),
            Print("╠══════════════════════════════════════════════════════════════════════════════╣\n"),
            ResetColor,
            Print("║                                                                              ║\n"),
            Print("║  🎯 NAVIGATION                                                              ║\n"),
            Print("║  [1-6]     Changer de vue                                                  ║\n"),
            Print("║  [H]       Afficher cette aide                                             ║\n"),
            Print("║  [R]       Rafraîchir l'affichage                                          ║\n"),
            Print("║  [ESC]     Retour au dashboard                                             ║\n"),
            Print("║  [Q]       Quitter l'application                                           ║\n"),
            Print("║                                                                              ║\n"),
            Print("║  🚀 ACTIONS RAPIDES                                                        ║\n"),
            Print("║  [SPACE]   Pause/Reprendre campagne en cours                              ║\n"),
            Print("║  [S]       Arrêter campagne en cours                                       ║\n"),
            Print("║  [E]       Éditer configuration                                            ║\n"),
            Print("║  [C]       Effacer logs                                                    ║\n"),
            Print("║                                                                              ║\n"),
            Print("║  📊 VUES DISPONIBLES                                                       ║\n"),
            Print("║  Dashboard     Vue d'ensemble avec métriques principales                  ║\n"),
            Print("║  Monitor       Surveillance temps réel des envois                         ║\n"),
            Print("║  Stats         Statistiques détaillées des emails                         ║\n"),
            Print("║  SMTP Status   État de tous les serveurs SMTP                            ║\n"),
            Print("║  Config        Configuration système                                       ║\n"),
            Print("║  Logs          Journal des événements                                      ║\n"),
            Print("║                                                                              ║\n"),
            Print("╚══════════════════════════════════════════════════════════════════════════════╝\n")
        )?;
        
        Ok(())
    }
    
    async fn render_status_bar(&self) -> Result<()> {
        let current_time = chrono::Local::now().format("%H:%M:%S").to_string();
        let view_name = match self.current_view {
            CLIView::Dashboard => "📊 Dashboard",
            CLIView::RealTimeMonitor => "📈 Monitor",
            CLIView::EmailStats => "📧 Stats",
            CLIView::SMTPStatus => "🔧 SMTP",
            CLIView::Configuration => "⚙️ Config",
            CLIView::Logs => "📝 Logs",
            CLIView::Help => "❓ Help",
        };
        
        execute!(
            io::stdout(),
            cursor::MoveTo(0, 30),
            SetForegroundColor(Color::DarkGrey),
            Print("╔══════════════════════════════════════════════════════════════════════════════╗\n"),
            Print(format!(
                "║ {} │ ⏰ {} │ 🖥️  VPS-Ultra │ 🔄 Auto-refresh │ [H] Help [Q] Quit ║\n",
                view_name, current_time
            )),
            Print("╚══════════════════════════════════════════════════════════════════════════════╝\n"),
            ResetColor
        )?;
        
        io::stdout().flush()?;
        Ok(())
    }
    
    fn clear_screen(&self) -> Result<()> {
        execute!(
            io::stdout(),
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        Ok(())
    }
    
    fn cleanup(&self) -> Result<()> {
        terminal::disable_raw_mode()?;
        execute!(
            io::stdout(),
            cursor::Show,
            ResetColor,
            Print("\n🚀 Ultra Email Sender fermé proprement.\n"),
            Print("Merci d'avoir utilisé notre système professionnel !\n")
        )?;
        Ok(())
    }
}

impl Drop for UltraCLI {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}