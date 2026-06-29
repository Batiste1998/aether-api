//! Règles de jeu pures (sans I/O), aisément testables.

/// Durée allouée pour répondre à une question, en millisecondes.
pub const TEMPS_PAR_QUESTION_MS: i32 = 15_000;

/// Nombre de questions par quiz, borné entre 3 et 10.
pub fn borne_nb_questions(n: i32) -> i32 {
    n.clamp(3, 10)
}

/// Points gagnés pour une réponse.
///
/// - 0 si la réponse est fausse.
/// - Sinon : 500 points de base + jusqu'à 500 de bonus de rapidité
///   + 50 par palier de série (plafonné à 10 d'affilée).
pub fn points_reponse(correcte: bool, temps_ms: i32, serie: i32) -> i32 {
    if !correcte {
        return 0;
    }
    let temps = temps_ms.clamp(0, TEMPS_PAR_QUESTION_MS) as f64;
    let ratio_restant = 1.0 - temps / TEMPS_PAR_QUESTION_MS as f64;
    let base = 500.0;
    let bonus_rapidite = 500.0 * ratio_restant;
    let bonus_serie = serie.clamp(0, 10) as f64 * 50.0;
    (base + bonus_rapidite + bonus_serie).round() as i32
}

/// Valide qu'une difficulté est reconnue.
pub fn difficulte_valide(d: &str) -> bool {
    matches!(d, "facile" | "moyen" | "difficile")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nb_questions_borne() {
        assert_eq!(borne_nb_questions(1), 3);
        assert_eq!(borne_nb_questions(5), 5);
        assert_eq!(borne_nb_questions(50), 10);
    }

    #[test]
    fn reponse_fausse_zero() {
        assert_eq!(points_reponse(false, 0, 5), 0);
    }

    #[test]
    fn reponse_instantanee_max() {
        // Réponse immédiate, sans série : 500 base + 500 rapidité.
        assert_eq!(points_reponse(true, 0, 0), 1000);
    }

    #[test]
    fn reponse_lente_base_seule() {
        // Tout le temps consommé : plus de bonus de rapidité.
        assert_eq!(points_reponse(true, TEMPS_PAR_QUESTION_MS, 0), 500);
    }

    #[test]
    fn bonus_de_serie_plafonne() {
        // Série de 4 -> +200 ; lent -> base 500 seule + 200 = 700.
        assert_eq!(points_reponse(true, TEMPS_PAR_QUESTION_MS, 4), 700);
        // Série énorme plafonnée à 10 -> +500.
        assert_eq!(points_reponse(true, TEMPS_PAR_QUESTION_MS, 99), 1000);
    }

    #[test]
    fn difficulte_reconnue() {
        assert!(difficulte_valide("facile"));
        assert!(difficulte_valide("moyen"));
        assert!(difficulte_valide("difficile"));
        assert!(!difficulte_valide("extreme"));
    }
}
