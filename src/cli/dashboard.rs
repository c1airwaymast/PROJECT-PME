use anyhow::Result;
use crossterm::{
    cursor,
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use std::io::{self, Write};
use tokio::time::Instant;

pub struct Dashboard {
    start_time: Instant,
    last_stats_update: Instant,
    // Métriques en temps réel
    emails_sent: u64,
    emails_delivered: u64,
    emails_bounced: u64,
    emails_clicked: u64,
    emails_opened: u64,
    current_rate: f64,
    active_campaigns: u32,
    active_smtp_count: u32,
}

impl Dashboard {
    pub fn new() -> Result<Self> {
        Ok(Self {
            start_time: Instant::now(),
            last_stats_update: Instant::now(),
            emails_sent: 0,
            emails_delivered: 0,
            emails_bounced: 0,
            emails_clicked: 0,
            emails_opened: 0,
            current_rate: 0.0,
            active_campaigns: 0,
            active_smtp_count: 4,
        })
    }
    
    pub async fn render(&mut self) -> Result<()> {
        // Mettre à jour les stats simulées (remplacer par vraies données)
        self.update_simulated_stats();
        
        execute!(
            io::stdout(),
            cursor::MoveTo(0, 8),
            SetForegroundColor(Color::Cyan),
            Print("╔══════════════════════════════════════════════════════════════════════════════╗\n"),
            Print("║                           📊 TABLEAU DE BORD PRINCIPAL                      ║\n"),
            Print("╠══════════════════════════════════════════════════════════════════════════════╣\n"),
            ResetColor
        )?;
        
        // Ligne 1: Métriques principales
        execute!(
            io::stdout(),
            Print("║ "),
            SetForegroundColor(Color::Green),
            Print(format!("✉️  Envoyés: {:>8}", self.format_number(self.emails_sent))),
            ResetColor,
            Print(" │ "),
            SetForegroundColor(Color::Blue),
            Print(format!("📬 Délivrés: {:>7}", self.format_number(self.emails_delivered))),
            ResetColor,
            Print(" │ "),
            SetForegroundColor(Color::Red),
            Print(format!("❌ Rebonds: {:>7}", self.format_number(self.emails_bounced))),
            ResetColor,
            Print(" ║\n")
        )?;
        
        // Ligne 2: Engagement
        execute!(
            io::stdout(),
            Print("║ "),
            SetForegroundColor(Color::Yellow),
            Print(format!("👁️  Ouverts: {:>8}", self.format_number(self.emails_opened))),
            ResetColor,
            Print(" │ "),
            SetForegroundColor(Color::Magenta),
            Print(format!("🖱️  Clics: {:>9}", self.format_number(self.emails_clicked))),
            ResetColor,
            Print(" │ "),
            SetForegroundColor(Color::Cyan),
            Print(format!("⚡ Vitesse: {:>6}/h", self.format_number(self.current_rate as u64))),
            ResetColor,
            Print(" ║\n")
        )?;
        
        // Séparateur
        execute!(
            io::stdout(),
            SetForegroundColor(Color::DarkGrey),
            Print("╠══════════════════════════════════════════════════════════════════════════════╣\n"),
            ResetColor
        )?;
        
        // Taux de réussite et performance
        let delivery_rate = if self.emails_sent > 0 {
            (self.emails_delivered as f64 / self.emails_sent as f64) * 100.0
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
            Print(format!("📈 Taux livraison: {:>5.1}%", delivery_rate)),
            ResetColor,
            Print(" │ "),
            SetForegroundColor(Color::Blue),
            Print(format!("👁️  Taux ouverture: {:>5.1}%", open_rate)),
            ResetColor,
            Print(" │ "),
            SetForegroundColor(Color::Magenta),
            Print(format!("🖱️  Taux clic: {:>5.1}%", click_rate)),
            ResetColor,
            Print(" ║\n")
        )?;
        
        // État des campagnes
        execute!(
            io::stdout(),
            Print("║ "),
            SetForegroundColor(Color::Yellow),
            Print(format!("🚀 Campagnes actives: {:>3}", self.active_campaigns)),
            ResetColor,
            Print(" │ "),
            SetForegroundColor(Color::Green),
            Print(format!("🔧 SMTP actifs: {:>6}", self.active_smtp_count)),
            ResetColor,
            Print(" │ "),
            SetForegroundColor(Color::Cyan),
            Print(format!("⏱️  Uptime: {:>8}", self.format_uptime())),
            ResetColor,
            Print(" ║\n")
        )?;
        
        // Graphique ASCII simple de performance
        execute!(
            io::stdout(),
            SetForegroundColor(Color::DarkGrey),
            Print("╠══════════════════════════════════════════════════════════════════════════════╣\n"),
            ResetColor,
            Print("║                          📈 PERFORMANCE (dernière heure)                   ║\n"),
            Print("║                                                                              ║\n")
        )?;
        
        // Graphique ASCII simple
        let performance_bars = self.generate_performance_graph();
        for bar in performance_bars {
            execute!(
                io::stdout(),
                Print(format!("║ {} ║\n", bar))
            )?;
        }
        
        execute!(
            io::stdout(),
            Print("║                                                                              ║\n"),
            SetForegroundColor(Color::DarkGrey),
            Print("╚══════════════════════════════════════════════════════════════════════════════╝\n"),
            ResetColor
        )?;
        
        io::stdout().flush()?;
        Ok(())
    }
    
    fn update_simulated_stats(&mut self) {
        // Simulation de données réelles (à remplacer)
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Augmenter progressivement les métriques
        if self.last_stats_update.elapsed().as_secs() >= 1 {
            self.emails_sent += rng.gen_range(0..5);
            self.emails_delivered += rng.gen_range(0..4);
            self.emails_bounced += rng.gen_range(0..1);
            self.emails_opened += rng.gen_range(0..2);
            self.emails_clicked += rng.gen_range(0..1);
            self.current_rate = rng.gen_range(150.0..250.0);
            self.active_campaigns = rng.gen_range(1..4);
            
            self.last_stats_update = Instant::now();
        }
    }
    
    fn format_number(&self, num: u64) -> String {
        if num >= 1_000_000 {
            format!("{:.1}M", num as f64 / 1_000_000.0)
        } else if num >= 1_000 {
            format!("{:.1}K", num as f64 / 1_000.0)
        } else {
            num.to_string()
        }
    }
    
    fn format_uptime(&self) -> String {
        let uptime = self.start_time.elapsed();
        let hours = uptime.as_secs() / 3600;
        let minutes = (uptime.as_secs() % 3600) / 60;
        let seconds = uptime.as_secs() % 60;
        
        if hours > 0 {
            format!("{}h{:02}m", hours, minutes)
        } else if minutes > 0 {
            format!("{}m{:02}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }
    
    fn generate_performance_graph(&self) -> Vec<String> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Générer un graphique ASCII simple
        let mut bars = Vec::new();
        
        for hour in 0..12 {
            let value = rng.gen_range(20..100);
            let bar_length = (value as f64 / 100.0 * 50.0) as usize;
            let bar = "█".repeat(bar_length);
            let spaces = " ".repeat(50 - bar_length);
            
            bars.push(format!(
                "{:02}h │{}{}│ {}%", 
                (chrono::Local::now().hour() as i32 - 11 + hour).max(0) % 24,
                bar, 
                spaces, 
                value
            ));
        }
        
        bars
    }
}