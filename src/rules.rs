//! Règles de jeu pures (sans I/O), aisément testables.

/// Niveau du personnage en fonction de son expérience (1 niveau tous les 100 XP).
pub fn niveau_pour_xp(xp: i32) -> i32 {
    1 + xp.max(0) / 100
}

/// PV maximum : base de la classe + 20 par niveau au-delà du premier.
pub fn pv_max_pour(pv_base: i32, niveau: i32) -> i32 {
    pv_base + (niveau.max(1) - 1) * 20
}

/// Normalise le type d'objet renvoyé par l'IA vers l'une des valeurs autorisées.
pub fn normaliser_type_objet(t: Option<&str>) -> &'static str {
    match t {
        Some("arme") => "arme",
        Some("armure") => "armure",
        Some("cle") => "cle",
        _ => "consommable",
    }
}

/// Extrait le nombre de PV restaurés par un consommable depuis son texte d'effet.
/// Renvoie 0 si l'effet ne concerne pas les points de vie.
pub fn soin_depuis_effet(effet: &str) -> i32 {
    let lower = effet.to_lowercase();
    if !(lower.contains("pv") || lower.contains("vie")) {
        return 0;
    }
    let mut num = String::new();
    for ch in effet.chars() {
        if ch.is_ascii_digit() {
            num.push(ch);
        } else if !num.is_empty() {
            break;
        }
    }
    num.parse().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn niveau_progression() {
        assert_eq!(niveau_pour_xp(0), 1);
        assert_eq!(niveau_pour_xp(99), 1);
        assert_eq!(niveau_pour_xp(100), 2);
        assert_eq!(niveau_pour_xp(250), 3);
        assert_eq!(niveau_pour_xp(-50), 1); // XP négative bornée
    }

    #[test]
    fn pv_max_augmente_avec_le_niveau() {
        assert_eq!(pv_max_pour(100, 1), 100);
        assert_eq!(pv_max_pour(100, 2), 120);
        assert_eq!(pv_max_pour(130, 3), 170);
        assert_eq!(pv_max_pour(100, 0), 100); // niveau borné à 1
    }

    #[test]
    fn type_objet_normalise() {
        assert_eq!(normaliser_type_objet(Some("arme")), "arme");
        assert_eq!(normaliser_type_objet(Some("armure")), "armure");
        assert_eq!(normaliser_type_objet(Some("cle")), "cle");
        assert_eq!(normaliser_type_objet(Some("potion")), "consommable");
        assert_eq!(normaliser_type_objet(None), "consommable");
    }

    #[test]
    fn soin_extrait_les_pv() {
        assert_eq!(soin_depuis_effet("+30 PV"), 30);
        assert_eq!(soin_depuis_effet("Restaure 25 points de vie"), 25);
        assert_eq!(soin_depuis_effet("+5 force"), 0); // ne concerne pas les PV
        assert_eq!(soin_depuis_effet(""), 0);
    }
}
