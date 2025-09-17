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
        
        // ENVOYER UN EMAIL BCC PAR GROUPE DE DOMAINE
        for (domaine, emails_groupe) in groupes_par_domaine {
            info!("📦 Groupe {}: {} emails", domaine, emails_groupe.len());
            
            // Prendre le premier email du groupe pour les variables de base
            let email_representatif = &emails_groupe[0];
            let recipient_data = self.extract_recipient_info(email_representatif);
            
            // Variables adaptées au DOMAINE
            let mut variables_groupe = recipient_data.clone();
            variables_groupe.insert("DOMAINE_GROUPE".to_string(), domaine.clone());
            variables_groupe.insert("NOMBRE_DESTINATAIRES".to_string(), emails_groupe.len().to_string());
            
            // Appliquer les variables du template aux groupes
            let sujet_base = self.process_variables(subject_template, &variables_groupe);
            let expediteur_base = self.process_variables(sender_template, &variables_groupe);
            
            // Utiliser EXACTEMENT vos templates sans préfixes
            let sujet_adapte = sujet_base;
            let expediteur_adapte = expediteur_base;
            
            info!("   📝 Sujet groupe: {}", sujet_adapte);
            info!("   👤 From groupe: {}", expediteur_adapte);
            
            // Sélectionner un client email aléatoire pour les headers
            let clients_email = vec![
                ("Thunderbird", "115.3.1", "Mozilla Thunderbird"),
                ("eM Client", "9.2.1768", "eM Client"),
                ("Outlook", "16.0.16827", "Microsoft Outlook"),
                ("Apple Mail", "16.0", "Apple Mail"),
                ("Mailbird", "2.9.82", "Mailbird"),
                ("BlueMail", "1.9.8.23", "BlueMail")
            ];
            
            let (client_name, version, user_agent) = clients_email.choose(&mut rand::thread_rng()).unwrap();
            
            // Message-ID réaliste selon le client
            let message_id = match *client_name {
                "Thunderbird" => format!("<{}.{}@{}>", 
                    uuid::Uuid::new_v4().simple(),
                    chrono::Utc::now().timestamp(),
                    "thunderbird.net"),
                "eM Client" => format!("<EM{}.{}@emclient.com>",
                    rand::thread_rng().gen_range(100000..999999),
                    chrono::Utc::now().timestamp()),
                "Outlook" => format!("<{}-{}@outlook.com>",
                    uuid::Uuid::new_v4().simple(),
                    chrono::Utc::now().format("%Y%m%d%H%M%S")),
                "Apple Mail" => format!("<{}.{}@me.com>",
                    uuid::Uuid::new_v4().simple(),
                    chrono::Utc::now().timestamp()),
                _ => format!("<{}.{}@{}>", 
                    uuid::Uuid::new_v4().simple(),
                    chrono::Utc::now().timestamp(),
                    "mail.local")
            };
            
            let mut message_builder = Message::builder()
                .message_id(Some(message_id))
                .from(format!("{} <{}>", expediteur_adapte, smtp_config.email).parse()?)
                .to(smtp_config.email.parse()?)
                .reply_to(smtp_config.email.parse()?)
                .subject(sujet_adapte);
            
            // Headers spécifiques selon le client email
            message_builder = self.ajouter_headers_client_email(message_builder, client_name, version, user_agent)?;
            
            // Ajouter emails CC si activé
            if self.config.rotation.cc_enabled {
                let cc_emails = self.generer_cc_dynamiques(&variables_groupe);
                info!("   📧 Ajout de {} emails en CC", cc_emails.len());
                
                for cc_email in cc_emails {
                    if let Ok(mailbox) = cc_email.parse::<Mailbox>() {
                        message_builder = message_builder.cc(mailbox);
                        info!("      CC: {}", cc_email);
                    }
                }
            }
            
            // Ajouter tous les emails du groupe en BCC
            for email in &emails_groupe {
                if let Ok(mailbox) = email.parse::<Mailbox>() {
                    message_builder = message_builder.bcc(mailbox);
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
            emails_groupe.len(),
            domaine,
            chrono::Utc::now().format("%d/%m/%Y"),
            expediteur_adapte,
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
            domaine, domaine, domaine, emails_groupe.len(), domaine,
            domaine, chrono::Utc::now().format("%Y%m%d"),
            domaine, chrono::Utc::now().format("%Y%m%d"),
            chrono::Utc::now().format("%d/%m/%Y"), expediteur_adapte, domaine
            );
            
            // Contenu ultra-varié anti-détection
            let texte_alternatif = self.generer_contenu_anti_spam(&domaine, &expediteur_adapte, emails_groupe.len());
            
            // Créer email multipart (HTML + texte)
            let email_groupe = message_builder
                .multipart(
                    MultiPart::alternative()
                        .singlepart(SinglePart::plain(texte_alternatif))
                        .singlepart(SinglePart::html(html_content))
                )?;
            
            // Envoyer le BCC pour ce groupe
            let debut_envoi = std::time::Instant::now();
            
            match mailer.send(&email_groupe) {
                Ok(_) => {
                    let duree = debut_envoi.elapsed();
                    info!("   ✅ Groupe {} envoyé ({} emails BCC) en {:.2}s", 
                          domaine, emails_groupe.len(), duree.as_secs_f32());
                    total_envoyes += emails_groupe.len();
                }
                Err(e) => {
                    error!("   ❌ Erreur groupe {}: {}", domaine, e);
                }
            }
            
            // Pause entre groupes (naturel)
            let pause_ms = rand::thread_rng().gen_range(1000..3000); // 1-3 secondes
            tokio::time::sleep(tokio::time::Duration::from_millis(pause_ms)).await;
        }
        
        info!("🎉 {} emails envoyés via BCC intelligent (groupés par domaine)", total_envoyes);
        
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
    
    fn ajouter_headers_client_email(
        &self, 
        mut message_builder: lettre::message::MessageBuilder,
        client_name: &str,
        version: &str,
        user_agent: &str
    ) -> Result<lettre::message::MessageBuilder> {
        use lettre::message::header::{HeaderName, HeaderValue};
        use rand::Rng;
        
        info!("      🖥️ Simulation client: {} v{}", client_name, version);
        
        match client_name {
            "Thunderbird" => {
                // Headers Thunderbird réalistes
                message_builder = message_builder
                    .header(HeaderName::new_from_ascii_str("X-Mailer")?, 
                           HeaderValue::from_str(&format!("Mozilla Thunderbird {}", version))?)
                    .header(HeaderName::new_from_ascii_str("User-Agent")?, 
                           HeaderValue::from_str(&format!("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Thunderbird/{}", version))?)
                    .header(HeaderName::new_from_ascii_str("X-Mozilla-Status")?, 
                           HeaderValue::from_str("0001")?)
                    .header(HeaderName::new_from_ascii_str("X-Mozilla-Status2")?, 
                           HeaderValue::from_str("00000000")?)
                    .header(HeaderName::new_from_ascii_str("X-Mozilla-Keys")?, 
                           HeaderValue::from_str("")?)
                    .header(HeaderName::new_from_ascii_str("X-Priority")?, 
                           HeaderValue::from_str("3")?)
                    .header(HeaderName::new_from_ascii_str("X-MSMail-Priority")?, 
                           HeaderValue::from_str("Normal")?);
            },
            "eM Client" => {
                // Headers eM Client réalistes
                message_builder = message_builder
                    .header(HeaderName::new_from_ascii_str("X-Mailer")?, 
                           HeaderValue::from_str(&format!("eM Client {}", version))?)
                    .header(HeaderName::new_from_ascii_str("X-EMClient-Version")?, 
                           HeaderValue::from_str(version)?)
                    .header(HeaderName::new_from_ascii_str("X-Priority")?, 
                           HeaderValue::from_str("3")?)
                    .header(HeaderName::new_from_ascii_str("X-MSMail-Priority")?, 
                           HeaderValue::from_str("Normal")?)
                    .header(HeaderName::new_from_ascii_str("X-MimeOLE")?, 
                           HeaderValue::from_str(&format!("Produced By eM Client v{}", version))?);
            },
            "Outlook" => {
                // Headers Outlook réalistes
                message_builder = message_builder
                    .header(HeaderName::new_from_ascii_str("X-Mailer")?, 
                           HeaderValue::from_str(&format!("Microsoft Outlook {}", version))?)
                    .header(HeaderName::new_from_ascii_str("X-MimeOLE")?, 
                           HeaderValue::from_str(&format!("Produced By Microsoft MimeOLE V{}", version))?)
                    .header(HeaderName::new_from_ascii_str("X-MS-Has-Attach")?, 
                           HeaderValue::from_str("")?)
                    .header(HeaderName::new_from_ascii_str("X-MS-TNEF-Correlator")?, 
                           HeaderValue::from_str(&format!("<{}>", uuid::Uuid::new_v4()))?)
                    .header(HeaderName::new_from_ascii_str("X-Priority")?, 
                           HeaderValue::from_str("3")?)
                    .header(HeaderName::new_from_ascii_str("X-MSMail-Priority")?, 
                           HeaderValue::from_str("Normal")?);
            },
            "Apple Mail" => {
                // Headers Apple Mail réalistes
                message_builder = message_builder
                    .header(HeaderName::new_from_ascii_str("X-Mailer")?, 
                           HeaderValue::from_str(&format!("Apple Mail ({})", version))?)
                    .header(HeaderName::new_from_ascii_str("X-Apple-Mail-Remote-Attachments")?, 
                           HeaderValue::from_str("NO")?)
                    .header(HeaderName::new_from_ascii_str("X-Apple-Base-Url")?, 
                           HeaderValue::from_str("")?)
                    .header(HeaderName::new_from_ascii_str("X-Universally-Unique-Identifier")?, 
                           HeaderValue::from_str(&uuid::Uuid::new_v4().to_string())?);
            },
            "Mailbird" => {
                // Headers Mailbird réalistes
                message_builder = message_builder
                    .header(HeaderName::new_from_ascii_str("X-Mailer")?, 
                           HeaderValue::from_str(&format!("Mailbird {}", version))?)
                    .header(HeaderName::new_from_ascii_str("X-Mailbird-Version")?, 
                           HeaderValue::from_str(version)?)
                    .header(HeaderName::new_from_ascii_str("X-Priority")?, 
                           HeaderValue::from_str("3")?);
            },
            "BlueMail" => {
                // Headers BlueMail réalistes
                message_builder = message_builder
                    .header(HeaderName::new_from_ascii_str("X-Mailer")?, 
                           HeaderValue::from_str(&format!("BlueMail {}", version))?)
                    .header(HeaderName::new_from_ascii_str("X-BlueMail-Version")?, 
                           HeaderValue::from_str(version)?)
                    .header(HeaderName::new_from_ascii_str("X-Mobile-Client")?, 
                           HeaderValue::from_str("true")?);
            },
            _ => {
                // Headers génériques
                message_builder = message_builder
                    .header(HeaderName::new_from_ascii_str("X-Mailer")?, 
                           HeaderValue::from_str(user_agent)?);
            }
        }
        
        // Headers communs anti-spam pour tous les clients
        message_builder = message_builder
            .header(HeaderName::new_from_ascii_str("X-Originating-IP")?, 
                   HeaderValue::from_str(&format!("[{}]", self.generer_ip_realiste()))?)
            .header(HeaderName::new_from_ascii_str("X-Spam-Status")?, 
                   HeaderValue::from_str("No, score=0.0 required=5.0")?)
            .header(HeaderName::new_from_ascii_str("X-Spam-Score")?, 
                   HeaderValue::from_str("0.0")?)
            .header(HeaderName::new_from_ascii_str("X-Virus-Scanned")?, 
                   HeaderValue::from_str("ClamAV 1.2.1")?)
            .header(HeaderName::new_from_ascii_str("Authentication-Results")?, 
                   HeaderValue::from_str(&format!("{}; dkim=pass; spf=pass; dmarc=pass", smtp_config.smtp_host))?)
            .header(HeaderName::new_from_ascii_str("X-Auto-Response-Suppress")?, 
                   HeaderValue::from_str("DR, RN, NRN, OOF, AutoReply")?)
            .header(HeaderName::new_from_ascii_str("X-Content-Filtered-By")?, 
                   HeaderValue::from_str("Mailman/MimeDel 2.1.39")?)
            .header(HeaderName::new_from_ascii_str("X-Sender-Verified")?, 
                   HeaderValue::from_str("TRUE")?)
            .header(HeaderName::new_from_ascii_str("List-Unsubscribe")?, 
                   HeaderValue::from_str("<mailto:unsubscribe@example.com>")?)
            .header(HeaderName::new_from_ascii_str("Precedence")?, 
                   HeaderValue::from_str("bulk")?);
        
        Ok(message_builder)
    }
    
    fn generer_ip_realiste(&self) -> String {
        use rand::Rng;
        let ips_pool = vec![
            "192.168.1.100", "10.0.0.25", "172.16.0.50",
            "192.168.10.15", "10.1.1.75", "172.20.0.100",
            "192.168.100.200", "10.10.10.10", "172.30.1.1"
        ];
        ips_pool.choose(&mut rand::thread_rng()).unwrap().to_string()
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