use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ TEST DE VITESSE - 200 EMAILS");
    println!("{}", "=".repeat(50));
    
    // Générer 200 emails de test
    let mut emails_test = Vec::new();
    for i in 1..=200 {
        emails_test.push(format!("test{}@example.com", i));
    }
    
    // Écrire dans un fichier temporaire
    let contenu = emails_test.join("\n");
    tokio::fs::write("test_200_emails.txt", contenu).await?;
    
    println!("📧 200 emails générés");
    println!("🚀 Démarrage du test de vitesse...");
    
    let debut = Instant::now();
    
    // Lancer la commande d'envoi
    let output = std::process::Command::new("./target/release/ultra-email-sender")
        .args(&[
            "send",
            "-m", "fast",  // Mode rapide pour le test
            "-r", "test_200_emails.txt",
            "-s", "Test Vitesse - [EMAIL]",
            "--sender-name", "Test Vitesse - [NOM]",
            "--dry-run"  // Dry run pour mesurer sans envoyer
        ])
        .output()?;
    
    let duree = debut.elapsed();
    
    println!("⏱️ RÉSULTATS DU TEST :");
    println!("   Temps total: {:.2} secondes", duree.as_secs_f32());
    println!("   Vitesse: {:.1} emails/seconde", 200.0 / duree.as_secs_f32());
    println!("   Temps par email: {:.0} ms", duree.as_millis() as f32 / 200.0);
    
    // Extrapoler pour envoi réel
    let temps_reel_estime = duree.as_secs_f32() * 2.5; // Facteur pour envoi réel
    println!("\n📊 ESTIMATION ENVOI RÉEL :");
    println!("   Temps estimé: {:.1} secondes", temps_reel_estime);
    println!("   Vitesse réelle: {:.1} emails/seconde", 200.0 / temps_reel_estime);
    
    if temps_reel_estime <= 60.0 {
        println!("   ✅ EXCELLENT - Moins d'1 minute");
    } else if temps_reel_estime <= 120.0 {
        println!("   🟡 BON - Moins de 2 minutes");
    } else {
        println!("   🔴 LENT - Plus de 2 minutes");
    }
    
    // Calculer pour 3500 emails
    let temps_3500 = (3500.0 / 200.0) * temps_reel_estime;
    let minutes_3500 = temps_3500 / 60.0;
    
    println!("\n🎯 EXTRAPOLATION POUR 3500 EMAILS :");
    println!("   Temps estimé: {:.1} minutes", minutes_3500);
    
    if minutes_3500 <= 10.0 {
        println!("   ✅ PARFAIT - Dans vos 10 minutes !");
    } else if minutes_3500 <= 15.0 {
        println!("   🟡 ACCEPTABLE - Proche de vos 10 minutes");
    } else {
        println!("   🔴 TROP LENT - Au-delà de 15 minutes");
    }
    
    // Nettoyer
    let _ = tokio::fs::remove_file("test_200_emails.txt").await;
    
    println!("\n🎉 Test de vitesse terminé !");
    
    Ok(())
}