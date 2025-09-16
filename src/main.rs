use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{info, warn, error};

mod config;
mod email_engine;
mod security;
mod performance;
mod modes;
mod headers;
mod variables;
mod monitoring;

use config::UltraConfig;
use email_engine::UltraEmailEngine;
use modes::SendingMode;

#[derive(Parser)]
#[command(name = "ultra-email-sender")]
#[command(about = "Système d'envoi d'emails ultra-sécurisé et robuste")]
#[command(version = "1.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Fichier de configuration
    #[arg(short, long, default_value = "config.yaml")]
    config: PathBuf,
    
    /// Niveau de log (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,
    
    /// Mode de performance (eco, balanced, turbo, custom)
    #[arg(short, long, default_value = "balanced")]
    performance: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Lancer une campagne d'envoi
    Send {
        /// Mode d'envoi
        #[arg(short, long, default_value = "natural")]
        mode: String,
        
        /// Fichier des destinataires
        #[arg(short, long)]
        recipients: PathBuf,
        
        /// Sujet du message (avec variables dynamiques)
        #[arg(short, long)]
        subject: String,
        
        /// Nom de l'expéditeur (avec variables dynamiques)
        #[arg(long)]
        sender_name: String,
        
        /// Template HTML
        #[arg(short, long)]
        template: Option<PathBuf>,
        
        /// Dry run (simulation sans envoi)
        #[arg(long)]
        dry_run: bool,
    },
    
    /// Tester la configuration
    Test {
        /// Test spécifique (smtp, headers, variables, performance)
        #[arg(short, long)]
        test_type: Option<String>,
    },
    
    /// Monitorer les performances en temps réel
    Monitor,
    
    /// Configurer interactivement
    Configure,
    
    /// Afficher les statistiques
    Stats {
        /// Période (today, week, month, all)
        #[arg(short, long, default_value = "today")]
        period: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialiser le logging
    init_logging(&cli.log_level)?;
    
    info!("🚀 Ultra Email Sender v1.0.0 - Démarrage");
    info!("📁 Configuration: {:?}", cli.config);
    info!("⚡ Performance: {}", cli.performance);
    
    // Charger la configuration
    let config = UltraConfig::load(&cli.config).await?;
    info!("✅ Configuration chargée avec succès");
    
    // Initialiser le moteur d'email
    let mut engine = UltraEmailEngine::new(config, cli.performance).await?;
    info!("🔧 Moteur d'email initialisé");
    
    // Exécuter la commande
    match cli.command {
        Commands::Send { 
            mode, 
            recipients, 
            subject, 
            sender_name, 
            template, 
            dry_run 
        } => {
            info!("📧 Mode d'envoi: {}", mode);
            info!("📝 Destinataires: {:?}", recipients);
            info!("📨 Sujet: {}", subject);
            
            let sending_mode = SendingMode::from_string(&mode)?;
            
            engine.send_campaign(
                sending_mode,
                recipients,
                subject,
                sender_name,
                template,
                dry_run
            ).await?;
        }
        
        Commands::Test { test_type } => {
            info!("🧪 Test en cours...");
            engine.run_tests(test_type).await?;
        }
        
        Commands::Monitor => {
            info!("📊 Démarrage du monitoring");
            engine.start_monitoring().await?;
        }
        
        Commands::Configure => {
            info!("⚙️ Configuration interactive");
            engine.interactive_config().await?;
        }
        
        Commands::Stats { period } => {
            info!("📈 Statistiques: {}", period);
            engine.show_stats(&period).await?;
        }
    }
    
    info!("✅ Opération terminée avec succès");
    Ok(())
}

fn init_logging(level: &str) -> Result<()> {
    use tracing_subscriber::{fmt, EnvFilter};
    
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level));
    
    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();
    
    Ok(())
}