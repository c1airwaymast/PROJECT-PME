use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Test direct Gmail SMTP");
    
    // Vos vrais identifiants Gmail
    let email = "uwiragiyevi12@gmail.com";
    let password = "lope skuq zrio batj";
    
    println!("📧 Test avec: {}", email);
    
    // Test de connexion SMTP
    let creds = Credentials::new(email.to_string(), password.to_string());
    
    let mailer = SmtpTransport::relay("smtp.gmail.com")?
        .credentials(creds)
        .port(587)
        .timeout(Some(std::time::Duration::from_secs(30)))
        .build();
    
    // Test avec un email simple vers vous-même
    let email_test = Message::builder()
        .from(format!("Test Système <{}>", email).parse()?)
        .to(email.parse()?)
        .subject("Test Rust Email Sender")
        .body("Test de connexion SMTP depuis Rust. Si vous recevez ceci, ça marche !".to_string())?;
    
    println!("📤 Envoi du test...");
    
    match mailer.send(&email_test) {
        Ok(_) => {
            println!("✅ EMAIL ENVOYÉ AVEC SUCCÈS !");
            println!("📬 Vérifiez votre boîte: {}", email);
        }
        Err(e) => {
            println!("❌ Erreur: {}", e);
            
            // Diagnostics détaillés
            if e.to_string().contains("authentication") {
                println!("🔑 Problème d'authentification - Vérifiez:");
                println!("   1. Mot de passe d'application Gmail activé");
                println!("   2. Authentification 2 facteurs activée");
                println!("   3. Mot de passe correct: {}", password);
            } else if e.to_string().contains("SSL") || e.to_string().contains("TLS") {
                println!("🔒 Problème SSL/TLS - Gmail nécessite STARTTLS");
            } else {
                println!("🌐 Problème réseau ou configuration");
            }
        }
    }
    
    Ok(())
}