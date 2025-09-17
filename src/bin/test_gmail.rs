use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 TEST DIRECT GMAIL SMTP");
    
    // Vos vrais identifiants Gmail
    let email = "uwiragiyevi12@gmail.com";
    let password = "lope skuq zrio batj";
    
    println!("📧 Test avec: {}", email);
    println!("🔑 Mot de passe: {}...", &password[0..4]);
    
    // Test de connexion SMTP Gmail
    let creds = Credentials::new(email.to_string(), password.to_string());
    
    println!("🔌 Connexion à smtp.gmail.com:587...");
    
    let mailer = SmtpTransport::starttls_relay("smtp.gmail.com")?
        .credentials(creds)
        .port(587)
        .timeout(Some(std::time::Duration::from_secs(30)))
        .build();
    
    // Email de test simple vers vous-même
    let email_test = Message::builder()
        .from(format!("Test Rust <{}>", email).parse()?)
        .to(email.parse()?)
        .subject("🚀 Test Rust Email Sender - SUCCÈS")
        .body(format!("
🎉 FÉLICITATIONS !

Votre système Rust Email Sender fonctionne parfaitement !

✅ Connexion SMTP Gmail réussie
✅ Authentification validée
✅ Email envoyé depuis: {}
✅ Timestamp: {}

Le système est prêt pour vos campagnes !

---
Test automatique du système Ultra Email Sender v1.0.0
        ", email, chrono::Utc::now().format("%d/%m/%Y %H:%M:%S")))?;
    
    println!("📤 Envoi du test en cours...");
    
    match mailer.send(&email_test) {
        Ok(response) => {
            println!("🎉 ✅ EMAIL ENVOYÉ AVEC SUCCÈS !");
            println!("📬 Vérifiez votre boîte Gmail: {}", email);
            println!("📊 Réponse serveur: {:?}", response);
            println!("⏱️ L'email devrait arriver dans 1-2 minutes");
        }
        Err(e) => {
            println!("❌ ERREUR SMTP: {}", e);
            
            // Diagnostics détaillés
            let error_str = e.to_string();
            if error_str.contains("authentication") || error_str.contains("535") {
                println!("\n🔑 PROBLÈME D'AUTHENTIFICATION:");
                println!("   1. Vérifiez que l'authentification 2FA est activée sur Gmail");
                println!("   2. Générez un 'Mot de passe d'application' spécifique");
                println!("   3. Utilisez ce mot de passe d'app, pas votre mot de passe Gmail normal");
                println!("   4. Mot de passe actuel: {}", password);
            } else if error_str.contains("SSL") || error_str.contains("TLS") {
                println!("\n🔒 PROBLÈME SSL/TLS:");
                println!("   Gmail nécessite STARTTLS sur le port 587");
            } else {
                println!("\n🌐 AUTRE PROBLÈME:");
                println!("   Erreur complète: {}", error_str);
            }
        }
    }
    
    Ok(())
}