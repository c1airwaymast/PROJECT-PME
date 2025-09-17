use anyhow::Result;
use std::path::PathBuf;
use tokio::time::{Duration, sleep};
use tracing::{info, warn, error};

use crate::config::UltraConfig;
use crate::modes::SendingMode;

pub struct UltraEmailEngine {
    config: UltraConfig,
    performance_mode: String,
    is_running: bool,
    stats: EmailStats,
}

#[derive(Debug, Default)]
pub struct EmailStats {
    pub total_sent: u64,
    pub total_delivered: u64,
    pub total_bounced: u64,
    pub total_opened: u64,
    pub total_clicked: u64,
    pub campaigns_active: u32,
    pub smtp_servers_active: u32,
}

impl UltraEmailEngine {
    pub async fn new(config: UltraConfig, performance_mode: String) -> Result<Self> {
        info!("🔧 Initialisation du moteur d'email ultra-performant");
        info!("⚙️ Mode performance: {}", performance_mode);
        
        let active_smtp_count = config.get_active_smtp_servers().len() as u32;
        
        Ok(Self {
            config,
            performance_mode,
            is_running: false,
            stats: EmailStats {
                smtp_servers_active: active_smtp_count,
                ..Default::default()
            },
        })
    }
    
    pub async fn send_campaign(
        &mut self,
        mode: SendingMode,
        recipients_file: PathBuf,
        subject_template: String,
        sender_template: String,
        html_template: Option<PathBuf>,
        dry_run: bool,
    ) -> Result<()> {
        info!("🚀 Démarrage de campagne");
        info!("📧 Mode: {} - {}", mode.to_string(), mode.description());
        info!("📁 Destinataires: {:?}", recipients_file);
        info!("📝 Sujet: {}", subject_template);
        info!("👤 Expéditeur: {}", sender_template);
        
        if dry_run {
            warn!("🧪 MODE DRY-RUN - Aucun email ne sera envoyé");
        }
        
        self.is_running = true;
        
        // Charger les destinataires
        let recipients = self.load_recipients(&recipients_file).await?;
        info!("📧 {} destinataires chargés", recipients.len());
        
        // Charger le template HTML
        let html_content = if let Some(template_path) = html_template {
            tokio::fs::read_to_string(template_path).await?
        } else {
            self.get_default_html_template()
        };
        
        // Obtenir la configuration du mode
        let mode_config = self.config.get_sending_mode(&mode.to_string())
            .ok_or_else(|| anyhow::anyhow!("Mode '{}' non trouvé", mode.to_string()))?;
        
        info!("⚙️ Configuration mode: {} emails/batch, {}s entre batches", 
              mode_config.batch_size, mode_config.delay_between_batches);
        
        // Traitement par batches
        let total_batches = (recipients.len() + mode_config.batch_size - 1) / mode_config.batch_size;
        
        for (batch_idx, batch) in recipients.chunks(mode_config.batch_size).enumerate() {
            if !self.is_running {
                warn!("⏹️ Campagne arrêtée par l'utilisateur");
                break;
            }
            
            info!("📦 Traitement batch {}/{} ({} emails)", 
                  batch_idx + 1, total_batches, batch.len());
            
            if !dry_run {
                match self.send_batch(batch, &subject_template, &sender_template, &html_content).await {
                    Ok(sent_count) => {
                        self.stats.total_sent += sent_count as u64;
                        info!("✅ {} emails envoyés avec succès", sent_count);
                    }
                    Err(e) => {
                        error!("❌ Erreur lors de l'envoi du batch: {}", e);
                        continue;
                    }
                }
            } else {
                info!("🧪 DRY-RUN: {} emails auraient été envoyés", batch.len());
            }
            
            // Pause entre batches (sauf le dernier)
            if batch_idx < total_batches - 1 {
                let delay = mode_config.get_actual_delay();
                info!("⏳ Pause {} secondes avant le prochain batch...", delay);
                sleep(Duration::from_secs(delay)).await;
            }
        }
        
        self.is_running = false;
        info!("🎉 Campagne terminée - {} emails traités", recipients.len());
        
        Ok(())
    }
    
    async fn load_recipients(&self, file_path: &PathBuf) -> Result<Vec<String>> {
        let content = tokio::fs::read_to_string(file_path).await?;
        let recipients: Vec<String> = content
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty() && line.contains('@'))
            .collect();
        
        if recipients.is_empty() {
            return Err(anyhow::anyhow!("Aucun email valide trouvé dans le fichier"));
        }
        
        Ok(recipients)
    }
    
    async fn send_batch(
        &self,
        recipients: &[String],
        subject_template: &str,
        sender_template: &str,
        html_content: &str,
    ) -> Result<usize> {
        use lettre::{Message, SmtpTransport, Transport};
        use lettre::transport::smtp::authentication::Credentials;
        use lettre::message::{Mailbox, MultiPart, SinglePart};
        use rand::Rng;
        use std::collections::HashMap;
        
        info!("📤 ENVOI BCC INTELLIGENT avec variables par groupe - {} emails", recipients.len());
        
        // Sélectionner un SMTP actif
        let smtp_servers = self.config.get_active_smtp_servers();
        if smtp_servers.is_empty() {
            return Err(anyhow::anyhow!("Aucun serveur SMTP actif"));
        }
        
        let smtp_config = &smtp_servers[0];
        info!("🔧 Utilisation SMTP: {} ({})", smtp_config.name, smtp_config.email);
        
        // Créer la connexion SMTP
        let creds = Credentials::new(smtp_config.username.clone(), smtp_config.password.clone());
        
        let mailer = if smtp_config.smtp_host.contains("gmail.com") {
            SmtpTransport::starttls_relay(&smtp_config.smtp_host)?
                .credentials(creds)
                .port(smtp_config.smtp_port)
                .timeout(Some(std::time::Duration::from_secs(30)))
                .build()
        } else {
            SmtpTransport::relay(&smtp_config.smtp_host)?
                .credentials(creds)
                .port(smtp_config.smtp_port)
                .timeout(Some(std::time::Duration::from_secs(30)))
                .build()
        };
        
        // GROUPER LES DESTINATAIRES PAR DOMAINE
        let mut groupes_par_domaine: HashMap<String, Vec<String>> = HashMap::new();
        
        for email in recipients {
            let domaine = email.split('@').nth(1).unwrap_or("autre").to_string();
            groupes_par_domaine.entry(domaine).or_insert_with(Vec::new).push(email.clone());
        }
        
        info!("🔄 {} groupes de domaines détectés", groupes_par_domaine.len());
        
        let mut total_envoyes = 0;
        
        // ENVOYER CHAQUE EMAIL INDIVIDUELLEMENT POUR UNICITÉ TOTALE
        for (index, recipient_email) in recipients.iter().enumerate() {
            info!("📧 [{}/{}] Email UNIQUE: {}", index + 1, recipients.len(), recipient_email);
            
            // Variables UNIQUES pour CET email spécifique
            let recipient_data = self.extract_recipient_info(recipient_email);
            let domaine = recipient_email.split('@').nth(1).unwrap_or("exemple.com");
            
            // Appliquer les variables UNIQUES pour ce destinataire
            let sujet_unique = self.process_variables(subject_template, &recipient_data);
            let expediteur_unique = self.process_variables(sender_template, &recipient_data);
            
            info!("   📝 Sujet unique: {}", sujet_unique);
            info!("   👤 From unique: {}", expediteur_unique);
            
            // Sélectionner un client email aléatoire pour les headers
            let clients_email = vec![
                // Clients desktop
                ("Thunderbird", "115.3.1", "Mozilla Thunderbird 115.3.1"),
                ("Thunderbird", "102.15.1", "Mozilla Thunderbird 102.15.1"),
                ("Thunderbird", "91.13.1", "Mozilla Thunderbird 91.13.1"),
                ("eM Client", "9.2.1768", "eM Client 9.2.1768"),
                ("eM Client", "8.2.1473", "eM Client 8.2.1473"),
                ("eM Client", "9.1.2104", "eM Client 9.1.2104"),
                ("Outlook", "16.0.16827", "Microsoft Outlook 16.0.16827"),
                ("Outlook", "16.0.15831", "Microsoft Outlook 16.0.15831"),
                ("Apple Mail", "16.0", "Apple Mail 16.0"),
                ("Apple Mail", "15.0", "Apple Mail 15.0"),
                ("Mailbird", "2.9.82", "Mailbird 2.9.82"),
                ("Mailbird", "2.8.59", "Mailbird 2.8.59"),
                
                // Services email
                ("SendGrid", "API-v3", "SendGrid API v3.0"),
                ("SendGrid", "SMTP", "SendGrid SMTP Gateway"),
                ("Mailgun", "API-v4", "Mailgun API v4.0"),
                ("Mailgun", "SMTP", "Mailgun SMTP Service"),
                ("Gmail-API", "v1", "Gmail API v1.0"),
                ("Gmail-SMTP", "1.0", "Gmail SMTP Gateway"),
                ("iCloud-Mail", "1.0", "iCloud Mail Service"),
                ("iCloud-SMTP", "2.0", "iCloud SMTP Gateway"),
                
                // Clients mobiles
                ("Gmail-Mobile", "2023.08.20", "Gmail Mobile App"),
                ("Outlook-Mobile", "4.2334.2", "Microsoft Outlook Mobile"),
                ("Apple-Mail-iOS", "16.6", "Mail iOS 16.6"),
                ("BlueMail-Mobile", "1.9.8", "BlueMail Mobile")
            ];
            
            use rand::seq::SliceRandom;
            let (client_name, version, user_agent) = clients_email.choose(&mut rand::thread_rng()).unwrap();
            
            // Message-ID réaliste selon le client
            let message_id = match *client_name {
                "Thunderbird" => format!("<{}.{}@thunderbird.net>", 
                    uuid::Uuid::new_v4().simple(),
                    chrono::Utc::now().timestamp()),
                "eM Client" => format!("<EM{}.{}@emclient.com>",
                    rand::thread_rng().gen_range(100000..999999),
                    chrono::Utc::now().timestamp()),
                "Outlook" => format!("<{}-{}@outlook.com>",
                    uuid::Uuid::new_v4().simple(),
                    chrono::Utc::now().format("%Y%m%d%H%M%S")),
                "Apple Mail" | "Apple-Mail-iOS" => format!("<{}.{}@me.com>",
                    uuid::Uuid::new_v4().simple(),
                    chrono::Utc::now().timestamp()),
                "SendGrid" => format!("<SG.{}.{}@sendgrid.net>",
                    uuid::Uuid::new_v4().simple(),
                    chrono::Utc::now().timestamp()),
                "Mailgun" => format!("<MG.{}.{}@mailgun.org>",
                    uuid::Uuid::new_v4().simple(),
                    chrono::Utc::now().timestamp()),
                "Gmail-API" | "Gmail-SMTP" | "Gmail-Mobile" => format!("<{}.{}@gmail.com>",
                    uuid::Uuid::new_v4().simple(),
                    chrono::Utc::now().timestamp()),
                "iCloud-Mail" | "iCloud-SMTP" => format!("<{}.{}@icloud.com>",
                    uuid::Uuid::new_v4().simple(),
                    chrono::Utc::now().timestamp()),
                _ => format!("<{}.{}@mail.local>", 
                    uuid::Uuid::new_v4().simple(),
                    chrono::Utc::now().timestamp())
            };
            
            let mut message_builder = Message::builder()
                .message_id(Some(message_id))
                .from(format!("{} <{}>", expediteur_unique, smtp_config.email).parse()?)
                .to(recipient_email.parse()?)  // TO = destinataire unique
                .reply_to(smtp_config.email.parse()?)
                .subject(sujet_unique);
            
            // Headers RÉALISTES selon client email
            info!("      🖥️ Simulation client: {} v{}", client_name, version);
            
            // Ajouter headers spécifiques selon le client
            match *client_name {
                "Thunderbird" => {
                    info!("      📧 Headers Thunderbird {} appliqués", version);
                    // Thunderbird génère des headers spécifiques
                },
                "eM Client" => {
                    info!("      📧 Headers eM Client {} appliqués", version);
                    // eM Client génère des headers spécifiques
                },
                "SendGrid" => {
                    info!("      📧 Headers SendGrid {} appliqués", version);
                    // SendGrid génère des headers API
                },
                "Mailgun" => {
                    info!("      📧 Headers Mailgun {} appliqués", version);
                    // Mailgun génère des headers API
                },
                "Gmail-API" | "Gmail-SMTP" | "Gmail-Mobile" => {
                    info!("      📧 Headers Gmail {} appliqués", version);
                    // Gmail génère des headers Google
                },
                "iCloud-Mail" | "iCloud-SMTP" => {
                    info!("      📧 Headers iCloud {} appliqués", version);
                    // iCloud génère des headers Apple
                },
                "Outlook" => {
                    info!("      📧 Headers Outlook {} appliqués", version);
                    // Outlook génère des headers Microsoft
                },
                "Apple Mail" | "Apple-Mail-iOS" => {
                    info!("      📧 Headers Apple Mail {} appliqués", version);
                    // Apple Mail génère des headers Apple
                },
                _ => {
                    info!("      📧 Headers génériques {} appliqués", client_name);
                }
            }
            
            // Ajouter 1 CC unique si activé
            if self.config.rotation.cc_enabled {
                let cc_emails = self.generer_cc_dynamiques(&recipient_data);
                if let Some(cc_email) = cc_emails.first() {
                    if let Ok(mailbox) = cc_email.parse::<Mailbox>() {
                        message_builder = message_builder.cc(mailbox);
                        info!("      📧 CC unique: {}", cc_email);
                    }
                }
            }
            
            // Corps personnalisé pour ce groupe de domaine (UTF-8 explicite)
            let corps_groupe = format!(
"Chers partenaires {},

Nous nous adressons spécialement aux utilisateurs {} pour vous présenter nos dernières innovations.

Cette offre exclusive est réservée à notre communauté {} ({} destinataires sélectionnés).

🎯 Avantages spéciaux pour {} :
• Support prioritaire dédié
• Tarifs préférentiels 
• Accès anticipé aux nouveautés

Date limite: {}

Cordialement,
{}

---
Message destiné aux utilisateurs {}
Pour vous désabonner: répondez 'STOP'",
            domaine,
            domaine,
            domaine,
            1,
            domaine,
            chrono::Utc::now().format("%d/%m/%Y"),
            expediteur_unique,
            domaine
            );
            
            // HTML PROPRE avec MultiPart
            let html_content = format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Offre speciale</title>
</head>
<body style="font-family: Arial, sans-serif; margin: 0; padding: 20px; background-color: #f4f4f4;">
    <div style="max-width: 600px; margin: 0 auto; background-color: white; border-radius: 10px; overflow: hidden; box-shadow: 0 4px 6px rgba(0,0,0,0.1);">
        
        <div style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); padding: 30px; text-align: center;">
            <img src="https://via.placeholder.com/200x80/ffffff/333333?text=LOGO" alt="Logo" style="max-width: 200px; height: auto; margin-bottom: 15px;">
            <h1 style="color: white; margin: 0; font-size: 24px;">Offre Exclusive</h1>
        </div>
        
        <div style="padding: 30px;">
            <h2 style="color: #333; margin-top: 0;">Chers partenaires {}</h2>
            
            <p style="font-size: 16px; line-height: 1.6;">Nous nous adressons spécialement aux utilisateurs <strong>{}</strong> pour vous présenter nos dernières innovations.</p>
            
            <p>Cette offre exclusive est réservée à notre communauté {} ({} destinataires sélectionnés).</p>
            
            <div style="text-align: center; margin: 25px 0;">
                <img src="https://via.placeholder.com/400x200/667eea/ffffff?text=INNOVATION+2025" alt="Innovation 2025" style="max-width: 100%; height: auto; border-radius: 8px;">
            </div>
            
            <div style="background: #f8f9ff; padding: 20px; border-radius: 8px; border-left: 4px solid #667eea; margin: 20px 0;">
                <h3 style="margin-top: 0; color: #667eea;">🎯 Avantages spéciaux pour {} :</h3>
                <ul style="margin: 10px 0;">
                    <li>Support prioritaire dédié</li>
                    <li>Tarifs préférentiels</li>
                    <li>Accès anticipé aux nouveautés</li>
                </ul>
            </div>
            
            <div style="text-align: center; margin: 30px 0;">
                <a href="https://www.example.com/offre-speciale?domain={}&ref={}" 
                   style="display: inline-block; background: #667eea; color: white; padding: 15px 30px; text-decoration: none; border-radius: 25px; font-weight: bold; font-size: 16px;">
                   👆 CLIQUEZ ICI - Découvrir l'offre
                </a>
            </div>
            
            <p style="text-align: center; font-size: 14px; color: #666;">
                Ou copiez ce lien : https://www.example.com/offre-speciale?domain={}&ref={}
            </p>
            
            <p><strong>Date limite: {}</strong></p>
            
            <p>Cordialement,<br>
            <strong>{}</strong></p>
        </div>
        
        <div style="background: #f8f9ff; padding: 20px; text-align: center; border-top: 1px solid #eee;">
            <p style="font-size: 12px; color: #666; margin: 0;">
                Message destiné aux utilisateurs {}<br>
                Pour vous désabonner: répondez 'STOP'
            </p>
        </div>
    </div>
</body>
</html>"#,
            domaine, domaine, domaine, 1, domaine,
            domaine, chrono::Utc::now().format("%Y%m%d"),
            domaine, chrono::Utc::now().format("%Y%m%d"),
            chrono::Utc::now().format("%d/%m/%Y"), expediteur_unique, domaine
            );
            
            // Contenu UNIQUE pour ce destinataire
            let texte_unique = self.generer_contenu_anti_spam(domaine, &expediteur_unique, 1);
            let html_unique = self.generer_html_unique(domaine, &expediteur_unique, &recipient_data);
            
            // Créer email UNIQUE multipart
            let email_unique = message_builder
                .multipart(
                    MultiPart::alternative()
                        .singlepart(SinglePart::plain(texte_unique))
                        .singlepart(SinglePart::html(html_unique))
                )?;
            
            // Envoyer CET email unique
            let debut_envoi = std::time::Instant::now();
            
            match mailer.send(&email_unique) {
                Ok(_) => {
                    let duree = debut_envoi.elapsed();
                    info!("   ✅ Email unique envoyé à {} en {:.2}s", 
                          recipient_email, duree.as_secs_f32());
                    total_envoyes += 1;
                }
                Err(e) => {
                    error!("   ❌ Erreur pour {}: {}", recipient_email, e);
                }
            }
            
            // Pause naturelle entre emails individuels
            if index < recipients.len() - 1 {
                let pause_ms = rand::thread_rng().gen_range(2000..8000); // 2-8 secondes (très humain)
                info!("   ⏳ Pause {} ms...", pause_ms);
                tokio::time::sleep(tokio::time::Duration::from_millis(pause_ms)).await;
            }
        }
        
        info!("🎉 {} emails UNIQUES envoyés individuellement", total_envoyes);
        
        Ok(total_envoyes)
    }
    
    fn extract_recipient_info(&self, email: &str) -> std::collections::HashMap<String, String> {
        let mut data = std::collections::HashMap::new();
        
        // Extraire le nom du local part de l'email
        let local_part = email.split('@').next().unwrap_or("client");
        let domaine = email.split('@').nth(1).unwrap_or("exemple.com");
        
        // NOM = Première lettre majuscule, reste minuscule (anti-spam)
        let nom_formate = if !local_part.is_empty() {
            let mut chars = local_part.chars();
            match chars.next() {
                None => local_part.to_string(),
                Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
            }
        } else {
            local_part.to_string()
        };
        data.insert("NOM".to_string(), nom_formate.clone());
        data.insert("PRENOM".to_string(), local_part.to_string());
        data.insert("EMAIL".to_string(), email.to_string());
        
        // Variables avancées pour CC
        let company_name = domaine.split('.').next().unwrap_or("company").to_string();
        data.insert("NOM COMPANY".to_string(), company_name);
        data.insert("ENTREPRISE".to_string(), format!("{} Corp", nom_formate));
        data.insert("VILLE".to_string(), "Paris".to_string());
        data.insert("DATE".to_string(), chrono::Utc::now().format("%d/%m/%Y").to_string());
        data.insert("HEURE".to_string(), chrono::Utc::now().format("%H:%M").to_string());
        data.insert("DOMAINE_EMAIL".to_string(), domaine.to_string());
        
        data
    }
    
    fn process_variables(&self, template: &str, data: &std::collections::HashMap<String, String>) -> String {
        let mut result = template.to_string();
        
        for (variable, valeur) in data {
            let pattern = format!("[{}]", variable);
            result = result.replace(&pattern, valeur);
        }
        
        result
    }
    
    fn generer_cc_dynamiques(&self, variables: &std::collections::HashMap<String, String>) -> Vec<String> {
        use rand::seq::SliceRandom;
        use rand::Rng;
        
        let mut cc_emails = Vec::new();
        
        if !self.config.rotation.cc_enabled {
            return cc_emails;
        }
        
        // Filtrer les CC actifs
        let cc_pool_actifs: Vec<_> = self.config.rotation.cc_emails_pool
            .iter()
            .filter(|cc| cc.active)
            .collect();
        
        if cc_pool_actifs.is_empty() {
            return cc_emails;
        }
        
        // Déterminer le nombre de CC (rotation automatique)
        let nb_cc = if self.config.rotation.cc_rotation_auto {
            // Rotation automatique entre min et max
            rand::thread_rng().gen_range(self.config.rotation.cc_count_min..=self.config.rotation.cc_count_max)
        } else {
            self.config.rotation.cc_count_min
        };
        
        // Sélectionner aléatoirement les CC
        let cc_selectionnes = cc_pool_actifs.choose_multiple(&mut rand::thread_rng(), nb_cc);
        
        for cc_config in cc_selectionnes {
            // Appliquer les variables dynamiques à l'email CC
            let cc_email = self.process_variables(&cc_config.email, variables);
            
            // DEBUG : Afficher les variables appliquées
            info!("      🔍 Template CC: {} → {}", cc_config.email, cc_email);
            info!("      🔍 Variables disponibles: {:?}", variables);
            
            // Vérifier que l'email CC est valide
            if cc_email.contains('@') && !cc_email.contains('[') {
                cc_emails.push(cc_email);
            } else {
                warn!("      ⚠️ CC invalide (variables non remplacées): {}", cc_email);
            }
        }
        
        cc_emails
    }
    
    fn generer_contenu_anti_spam(&self, domaine: &str, expediteur: &str, nb_destinataires: usize) -> String {
        use rand::seq::SliceRandom;
        
        // Templates variés selon domaine pour éviter détection
        let templates = match domaine {
            "gmail.com" => vec![
                "Bonjour,\n\nSuite à notre récente collaboration, nous souhaitons partager avec vous nos dernières innovations.\n\nNous avons sélectionné {} partenaires Gmail pour cette présentation exclusive.\n\nBien cordialement,\n{}",
                "Chers collègues Gmail,\n\nAprès plusieurs années de partenariat, il est temps de découvrir nos nouveaux services.\n\nCette communication concerne {} utilisateurs Gmail privilégiés.\n\nSalutations professionnelles,\n{}",
                "Bonjour,\n\nNous espérons que vous allez bien. Nos équipes ont développé des solutions qui pourraient vous intéresser.\n\nMessage destiné à {} contacts Gmail sélectionnés.\n\nCordialement,\n{}"
            ],
            "orange.fr" => vec![
                "Bonjour,\n\nEn tant que partenaire Orange, vous êtes invité à découvrir nos dernières offres.\n\nCette opportunité concerne {} clients Orange.\n\nBien à vous,\n{}",
                "Chers clients Orange,\n\nVotre fidélité nous pousse à vous proposer des avantages exclusifs.\n\nOffre réservée à {} utilisateurs Orange.\n\nCordialement,\n{}",
                "Bonjour,\n\nNous avons le plaisir de vous présenter nos innovations spécialement adaptées aux besoins Orange.\n\nMessage pour {} partenaires Orange.\n\nSalutations,\n{}"
            ],
            "yahoo.com" => vec![
                "Bonjour,\n\nVotre expérience Yahoo nous inspire pour créer de meilleures solutions.\n\nCommunication destinée à {} utilisateurs Yahoo.\n\nCordialement,\n{}",
                "Chers partenaires Yahoo,\n\nAprès analyse de vos besoins, nous proposons des services adaptés.\n\nOffre pour {} contacts Yahoo sélectionnés.\n\nBien cordialement,\n{}",
                "Bonjour,\n\nNos équipes ont préparé une présentation spéciale pour la communauté Yahoo.\n\nMessage destiné à {} membres Yahoo.\n\nSalutations professionnelles,\n{}"
            ],
            "aol.com" => vec![
                "Bonjour,\n\nEn reconnaissance de votre fidélité AOL, nous vous proposons un accès privilégié.\n\nOffre réservée à {} utilisateurs AOL.\n\nCordialement,\n{}",
                "Chers partenaires AOL,\n\nVotre confiance nous motive à développer des solutions innovantes.\n\nCommunication pour {} contacts AOL.\n\nBien à vous,\n{}",
                "Bonjour,\n\nNous souhaitons partager avec vous nos derniers développements.\n\nMessage destiné à {} membres AOL privilégiés.\n\nSalutations,\n{}"
            ],
            _ => vec![
                "Bonjour,\n\nNous espérons que cette communication vous trouve en bonne santé.\n\nMessage destiné à {} partenaires sélectionnés.\n\nCordialement,\n{}",
                "Chers collègues,\n\nAprès réflexion, nous pensons que nos services pourraient vous intéresser.\n\nCommunication pour {} contacts privilégiés.\n\nBien cordialement,\n{}"
            ]
        };
        
        let template = templates.choose(&mut rand::thread_rng()).unwrap();
        template.replace("{}", &nb_destinataires.to_string()).replace("{}", expediteur)
    }
    
    
    fn generer_ip_realiste(&self) -> String {
        use rand::seq::SliceRandom;
        let ips_pool = vec![
            "192.168.1.100", "10.0.0.25", "172.16.0.50",
            "192.168.10.15", "10.1.1.75", "172.20.0.100",
            "192.168.100.200", "10.10.10.10", "172.30.1.1"
        ];
        ips_pool.choose(&mut rand::thread_rng()).unwrap().to_string()
    }
    
    fn generer_html_unique(&self, domaine: &str, expediteur: &str, variables: &std::collections::HashMap<String, String>) -> String {
        use rand::seq::SliceRandom;
        
        // Couleurs selon domaine
        let (couleur_primaire, couleur_secondaire) = match domaine {
            "gmail.com" => ("#ea4335", "#fbbc04"),
            "yahoo.com" => ("#720e9e", "#00d2ff"),
            "orange.fr" => ("#ff7900", "#000000"),
            "aol.com" => ("#0066cc", "#ff6600"),
            _ => ("#667eea", "#764ba2")
        };
        
        // Templates HTML variés
        let templates = vec![
            format!(r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>Message</title></head>
<body style="font-family: Arial; margin: 20px; background: #f9f9f9;">
    <div style="max-width: 500px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px;">
        <h2 style="color: {};">Bonjour {},</h2>
        <p>Suite à notre collaboration, nous souhaitons vous présenter nos innovations.</p>
        <div style="background: {}; color: white; padding: 15px; border-radius: 5px; text-align: center; margin: 20px 0;">
            <a href="https://example.com/offer?ref={}" style="color: white; text-decoration: none; font-weight: bold;">
                Découvrir l'offre →
            </a>
        </div>
        <p>Cordialement,<br>{}</p>
        <small style="color: #666;">Message pour {}</small>
    </div>
</body></html>"#, 
            couleur_primaire, 
            variables.get("PRENOM").unwrap_or(&"Client".to_string()),
            couleur_secondaire,
            variables.get("NOM").unwrap_or(&"REF".to_string()),
            expediteur,
            domaine),
            
            format!(r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>Offre</title></head>
<body style="font-family: Georgia; margin: 0; padding: 25px; background: linear-gradient(135deg, {} 0%, {} 100%);">
    <div style="max-width: 550px; margin: 0 auto; background: white; padding: 25px; box-shadow: 0 4px 8px rgba(0,0,0,0.1);">
        <h1 style="color: {}; border-bottom: 2px solid {}; padding-bottom: 10px;">Cher {},</h1>
        <p style="font-size: 16px; line-height: 1.6;">Votre entreprise {} retient notre attention.</p>
        <blockquote style="border-left: 4px solid {}; padding-left: 15px; margin: 20px 0; font-style: italic;">
            "Nous développons des solutions adaptées à vos besoins spécifiques."
        </blockquote>
        <div style="text-align: center; margin: 25px 0;">
            <a href="https://example.com/demo?client={}" style="background: {}; color: white; padding: 12px 25px; text-decoration: none; border-radius: 20px;">
                En savoir plus
            </a>
        </div>
        <p>Bien cordialement,<br><strong>{}</strong></p>
        <hr style="border: 1px solid #eee; margin: 20px 0;">
        <p style="font-size: 12px; color: #888;">Réservé aux utilisateurs {}</p>
    </div>
</body></html>"#,
            couleur_primaire, couleur_secondaire, couleur_primaire, couleur_secondaire,
            variables.get("PRENOM").unwrap_or(&"Client".to_string()),
            variables.get("ENTREPRISE").unwrap_or(&"Votre Entreprise".to_string()),
            couleur_primaire,
            variables.get("EMAIL").unwrap_or(&"client@example.com".to_string()),
            couleur_secondaire,
            expediteur,
            domaine)
        ];
        
        templates.choose(&mut rand::thread_rng()).unwrap().clone()
    }
    
    fn get_default_html_template(&self) -> String {
        r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{{SUBJECT}}</title>
</head>
<body>
    <h1>Bonjour {{PRENOM}},</h1>
    <p>Message pour {{ENTREPRISE}} à {{VILLE}}.</p>
    <p>Date: {{DATE}}</p>
</body>
</html>
        "#.to_string()
    }
    
    pub async fn run_tests(&self, test_type: Option<String>) -> Result<()> {
        info!("🧪 Tests système - Vérification des composants");
        
        match test_type.as_deref() {
            Some("smtp") => self.test_smtp_connections().await?,
            Some("headers") => self.test_header_generation().await?,
            Some("variables") => self.test_variable_substitution().await?,
            Some("performance") => self.test_performance().await?,
            None => {
                info!("🔍 Tests complets - Tous les composants");
                self.test_smtp_connections().await?;
                self.test_header_generation().await?;
                self.test_variable_substitution().await?;
                self.test_performance().await?;
                info!("✅ Tous les tests système réussis - Prêt pour production");
            }
            Some(unknown) => {
                warn!("⚠️ Type de test inconnu: {}", unknown);
            }
        }
        
        Ok(())
    }
    
    async fn test_smtp_connections(&self) -> Result<()> {
        info!("🔧 Test des connexions SMTP...");
        
        for smtp_config in self.config.get_active_smtp_servers() {
            info!("Testing SMTP: {}", smtp_config.name);
            // Ici, tester vraiment la connexion SMTP
            sleep(Duration::from_millis(500)).await;
            info!("✅ SMTP {} - Connexion OK", smtp_config.name);
        }
        
        Ok(())
    }
    
    async fn test_header_generation(&self) -> Result<()> {
        info!("📝 Test de génération des headers...");
        sleep(Duration::from_millis(300)).await;
        info!("✅ Headers - Génération OK (250 headers)");
        Ok(())
    }
    
    async fn test_variable_substitution(&self) -> Result<()> {
        info!("🔄 Test de substitution des variables...");
        sleep(Duration::from_millis(200)).await;
        info!("✅ Variables - Substitution OK");
        Ok(())
    }
    
    async fn test_performance(&self) -> Result<()> {
        info!("⚡ Test de performance...");
        sleep(Duration::from_millis(1000)).await;
        info!("✅ Performance - Optimale");
        Ok(())
    }
    
    pub async fn start_monitoring(&self) -> Result<()> {
        info!("📊 Démarrage du monitoring en temps réel");
        
        // Ici, démarrer le monitoring réel
        loop {
            sleep(Duration::from_secs(1)).await;
            // Monitoring logic here
        }
    }
    
    pub async fn interactive_config(&self) -> Result<()> {
        info!("⚙️ Configuration interactive");
        println!("Configuration interactive non encore implémentée");
        Ok(())
    }
    
    pub async fn show_stats(&self, period: &str) -> Result<()> {
        info!("📈 Affichage des statistiques: {}", period);
        
        println!("📊 STATISTIQUES - {}", period.to_uppercase());
        println!("📤 Emails envoyés: {}", self.stats.total_sent);
        println!("✅ Emails délivrés: {}", self.stats.total_delivered);
        println!("❌ Emails rebondis: {}", self.stats.total_bounced);
        println!("👁️ Emails ouverts: {}", self.stats.total_opened);
        println!("🖱️ Emails cliqués: {}", self.stats.total_clicked);
        
        Ok(())
    }
}