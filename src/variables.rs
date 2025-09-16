use std::collections::HashMap;
use chrono::Utc;
use regex::Regex;

pub struct VariableProcessor {
    patterns: HashMap<String, Regex>,
}

impl VariableProcessor {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();
        
        // Patterns pour les variables
        patterns.insert("NOM".to_string(), Regex::new(r"\[NOM\]").unwrap());
        patterns.insert("PRENOM".to_string(), Regex::new(r"\[PRENOM\]").unwrap());
        patterns.insert("ENTREPRISE".to_string(), Regex::new(r"\[ENTREPRISE\]").unwrap());
        patterns.insert("VILLE".to_string(), Regex::new(r"\[VILLE\]").unwrap());
        patterns.insert("DATE".to_string(), Regex::new(r"\[DATE\]").unwrap());
        patterns.insert("HEURE".to_string(), Regex::new(r"\[HEURE\]").unwrap());
        
        Self { patterns }
    }
    
    pub fn process_template(&self, template: &str, recipient_data: &RecipientData) -> String {
        let mut result = template.to_string();
        
        // Variables de base
        result = result.replace("[NOM]", &recipient_data.nom);
        result = result.replace("[PRENOM]", &recipient_data.prenom);
        result = result.replace("[ENTREPRISE]", &recipient_data.entreprise);
        result = result.replace("[VILLE]", &recipient_data.ville);
        
        // Variables temporelles
        let now = Utc::now();
        result = result.replace("[DATE]", &now.format("%d/%m/%Y").to_string());
        result = result.replace("[HEURE]", &now.format("%H:%M").to_string());
        
        // Variables calculées
        result = result.replace("[INITIALES]", &format!("{}.{}", 
                                                        recipient_data.prenom.chars().next().unwrap_or('X'),
                                                        recipient_data.nom.chars().next().unwrap_or('X')));
        
        result
    }
    
    pub fn extract_recipient_data(&self, email: &str) -> RecipientData {
        // Extraction basique (à améliorer avec vraies données)
        let parts: Vec<&str> = email.split('@').collect();
        let local_part = parts.get(0).unwrap_or(&"user");
        
        RecipientData {
            email: email.to_string(),
            nom: "Client".to_string(),
            prenom: local_part.to_string(),
            entreprise: "Entreprise".to_string(),
            ville: "Paris".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RecipientData {
    pub email: String,
    pub nom: String,
    pub prenom: String,
    pub entreprise: String,
    pub ville: String,
}