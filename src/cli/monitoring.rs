use anyhow::Result;
use crossterm::{
    cursor,
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use std::io::{self, Write};
use std::collections::VecDeque;
use tokio::time::Instant;

#[derive(Debug, Clone)]
pub struct EmailEvent {
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub event_type: EmailEventType,
    pub email: String,
    pub smtp_server: String,
    pub response_time: u64, // ms
    pub status: String,
}

#[derive(Debug, Clone)]
pub enum EmailEventType {
    Sent,
    Delivered,
    Bounced,
    Opened,
    Clicked,
    Error,
}

pub struct RealTimeMonitor {
    events: VecDeque<EmailEvent>,
    max_events: usize,
    last_update: Instant,
    current_scroll: usize,
}

impl RealTimeMonitor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            events: VecDeque::new(),
            max_events: 100,
            last_update: Instant::now(),
            current_scroll: 0,
        })
    }
    
    pub async fn render(&mut self) -> Result<()> {
        // Simuler de nouveaux événements
        self.simulate_events();
        
        execute!(
            io::stdout(),
            cursor::MoveTo(0, 8),
            SetForegroundColor(Color::Green),
            Print("╔══════════════════════════════════════════════════════════════════════════════╗\n"),
            Print("║                        📈 MONITORING TEMPS RÉEL                             ║\n"),
            Print("╠══════════════════════════════════════════════════════════════════════════════╣\n"),
            ResetColor,
            Print("║ Heure    │ Type      │ Email                    │ SMTP      │ Temps │ État ║\n"),
            SetForegroundColor(Color::DarkGrey),
            Print("╠══════════════════════════════════════════════════════════════════════════════╣\n"),
            ResetColor
        )?;
        
        // Afficher les derniers événements
        let events_to_show = self.events.iter()
            .rev()
            .skip(self.current_scroll)
            .take(15)
            .collect::<Vec<_>>();
        
        for event in events_to_show {
            let (emoji, color) = match event.event_type {
                EmailEventType::Sent => ("📤", Color::Blue),
                EmailEventType::Delivered => ("✅", Color::Green),
                EmailEventType::Bounced => ("❌", Color::Red),
                EmailEventType::Opened => ("👁️", Color::Yellow),
                EmailEventType::Clicked => ("🖱️", Color::Magenta),
                EmailEventType::Error => ("💥", Color::Red),
            };
            
            let time_str = event.timestamp.format("%H:%M:%S").to_string();
            let email_short = self.truncate_email(&event.email, 20);
            let smtp_short = self.truncate_string(&event.smtp_server, 8);
            let response_time = if event.response_time > 0 {
                format!("{}ms", event.response_time)
            } else {
                "-".to_string()
            };
            
            execute!(
                io::stdout(),
                Print("║ "),
                Print(format!("{} ", time_str)),
                Print("│ "),
                SetForegroundColor(color),
                Print(format!("{} {:8}", emoji, self.format_event_type(&event.event_type))),
                ResetColor,
                Print(" │ "),
                Print(format!("{:24}", email_short)),
                Print(" │ "),
                Print(format!("{:9}", smtp_short)),
                Print(" │ "),
                Print(format!("{:5}", response_time)),
                Print(" │ "),
                SetForegroundColor(color),
                Print(format!("{:4}", event.status)),
                ResetColor,
                Print(" ║\n")
            )?;
        }
        
        // Remplir les lignes vides
        for _ in events_to_show.len()..15 {
            execute!(
                io::stdout(),
                Print("║                                                                              ║\n")
            )?;
        }
        
        // Statistiques en temps réel
        execute!(
            io::stdout(),
            SetForegroundColor(Color::DarkGrey),
            Print("╠══════════════════════════════════════════════════════════════════════════════╣\n"),
            ResetColor,
            Print("║                              📊 MÉTRIQUES TEMPS RÉEL                       ║\n"),
            Print("║                                                                              ║\n")
        )?;
        
        // Calculer les taux
        let total_sent = self.emails_sent;
        let delivery_rate = if total_sent > 0 {
            (self.emails_delivered as f64 / total_sent as f64) * 100.0
        } else { 0.0 };
        
        let open_rate = if self.emails_delivered > 0 {
            (self.emails_opened as f64 / self.emails_delivered as f64) * 100.0
        } else { 0.0 };
        
        let click_rate = if self.emails_opened > 0 {
            (self.emails_clicked as f64 / self.emails_opened as f64) * 100.0
        } else { 0.0 };
        
        execute!(
            io::stdout(),
            Print("║ "),
            SetForegroundColor(Color::Green),
            Print(format!("📈 Taux livraison: {:5.1}%", delivery_rate)),
            ResetColor,
            Print(" │ "),
            SetForegroundColor(Color::Blue),
            Print(format!("👁️  Taux ouverture: {:5.1}%", open_rate)),
            ResetColor,
            Print(" │ "),
            SetForegroundColor(Color::Magenta),
            Print(format!("🖱️  Taux clic: {:5.1}%", click_rate)),
            ResetColor,
            Print(" ║\n")
        )?;
        
        // Barre de progression pour campagne active
        if self.active_campaigns > 0 {
            let progress = self.calculate_campaign_progress();
            let progress_bar = self.create_progress_bar(progress, 60);
            
            execute!(
                io::stdout(),
                Print("║                                                                              ║\n"),
                Print("║ 🚀 Campagne en cours:                                                      ║\n"),
                Print(format!("║ {} {:5.1}% │ ETA: {:8} ║\n", progress_bar, progress, self.calculate_eta())),
            )?;
        }
        
        execute!(
            io::stdout(),
            Print("║                                                                              ║\n"),
            SetForegroundColor(Color::DarkGrey),
            Print("║ 📋 [↑/↓] Scroll │ [SPACE] Pause │ [S] Stop │ [R] Refresh                 ║\n"),
            Print("╚══════════════════════════════════════════════════════════════════════════════╝\n"),
            ResetColor
        )?;
        
        io::stdout().flush()?;
        Ok(())
    }
    
    fn simulate_events(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Ajouter quelques événements aléatoires
        if self.last_update.elapsed().as_millis() >= 500 {
            let event_types = vec![
                EmailEventType::Sent,
                EmailEventType::Delivered,
                EmailEventType::Opened,
                EmailEventType::Clicked,
                EmailEventType::Bounced,
            ];
            
            let emails = vec![
                "jean.dupont@techcorp.com",
                "marie.martin@commerce.fr", 
                "pierre.durand@industrie.com",
                "sophie.bernard@services.fr",
                "alain.moreau@startup.io",
            ];
            
            let smtp_servers = vec![
                "iCloud-01", "iCloud-02", "Gmail-01", "Outlook-01"
            ];
            
            if rng.gen_bool(0.7) { // 70% chance d'ajouter un événement
                let event = EmailEvent {
                    timestamp: chrono::Local::now(),
                    event_type: event_types.choose(&mut rng).unwrap().clone(),
                    email: emails.choose(&mut rng).unwrap().to_string(),
                    smtp_server: smtp_servers.choose(&mut rng).unwrap().to_string(),
                    response_time: rng.gen_range(100..5000),
                    status: self.generate_random_status(),
                };
                
                self.add_event(event);
            }
            
            self.last_update = Instant::now();
        }
    }
    
    fn add_event(&mut self, event: EmailEvent) {
        self.events.push_back(event);
        
        // Garder seulement les derniers événements
        while self.events.len() > self.max_events {
            self.events.pop_front();
        }
    }
    
    fn truncate_email(&self, email: &str, max_len: usize) -> String {
        if email.len() <= max_len {
            email.to_string()
        } else {
            format!("{}...", &email[0..max_len-3])
        }
    }
    
    fn truncate_string(&self, s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[0..max_len-3])
        }
    }
    
    fn format_event_type(&self, event_type: &EmailEventType) -> String {
        match event_type {
            EmailEventType::Sent => "Envoyé".to_string(),
            EmailEventType::Delivered => "Livré".to_string(),
            EmailEventType::Bounced => "Rebond".to_string(),
            EmailEventType::Opened => "Ouvert".to_string(),
            EmailEventType::Clicked => "Cliqué".to_string(),
            EmailEventType::Error => "Erreur".to_string(),
        }
    }
    
    fn generate_random_status(&self) -> String {
        use rand::seq::SliceRandom;
        let statuses = vec!["OK", "250", "550", "421", "450", "451", "452", "500"];
        statuses.choose(&mut rand::thread_rng()).unwrap().to_string()
    }
    
    fn calculate_campaign_progress(&self) -> f64 {
        // Simulation de progrès (à remplacer par vraies données)
        use rand::Rng;
        rand::thread_rng().gen_range(15.0..85.0)
    }
    
    fn calculate_eta(&self) -> String {
        use rand::Rng;
        let minutes = rand::thread_rng().gen_range(5..45);
        format!("{}m", minutes)
    }
    
    fn create_progress_bar(&self, progress: f64, width: usize) -> String {
        let filled = ((progress / 100.0) * width as f64) as usize;
        let empty = width - filled;
        
        format!("[{}{}]", 
                "█".repeat(filled),
                "░".repeat(empty))
    }
}