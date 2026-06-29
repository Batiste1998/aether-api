# aether-api

Backend de **Quiz d'Æther** — un jeu de quiz de culture générale dont les questions sont générées par une IA (OpenAI GPT-4o).

Stack : **Rust** · [Axum](https://github.com/tokio-rs/axum) · [SQLx](https://github.com/launchbadge/sqlx) · PostgreSQL 16 · JWT · Argon2.

## Prérequis

- Rust (via [rustup](https://rustup.rs))
- Docker (pour PostgreSQL)

## Démarrage

```bash
docker compose up -d            # PostgreSQL sur le port 5433
cp .env.example .env            # puis renseigner OPENAI_API_KEY
cargo run                       # migrations auto + API sur :8080
```

```bash
curl http://localhost:8080/health
```

## Principales routes

| Méthode | Route | Rôle |
|---------|-------|------|
| POST | `/auth/register`, `/auth/login` | Authentification (JWT) |
| GET  | `/categories` | Catégories de thèmes suggérées |
| POST | `/quiz` | Génère un quiz (IA) et ouvre une session |
| POST | `/sessions/{id}/answers` | Enregistre une réponse, calcule le score |
| POST | `/sessions/{id}/finish` | Clôt la session, renvoie le rang |
| GET  | `/sessions` | Historique du joueur |
| GET  | `/leaderboard` | Classement des meilleurs scores |

## Structure

```
src/
├── main.rs        # bootstrap : config, pool DB, migrations, serveur
├── ai.rs          # génération de quiz via GPT-4o (sortie JSON structurée)
├── quiz.rs        # génération, jeu, scoring, classement, historique
├── rules.rs       # règles pures (scoring) + tests unitaires
├── reference.rs   # catégories
├── auth/          # inscription, connexion, JWT, extracteur AuthUser
├── error.rs · config.rs · state.rs
migrations/        # schéma SQL versionné (quiz, questions, choix, sessions…)
```

## Qualité

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

CI GitHub Actions à chaque push (fmt, clippy, tests, build release).
