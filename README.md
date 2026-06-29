# aether-api

Backend de **Chroniques d'Æther** — RPG textuel web dont le Maître du Jeu est propulsé par une IA (OpenAI GPT-4o).

Stack : **Rust** · [Axum](https://github.com/tokio-rs/axum) · [SQLx](https://github.com/launchbadge/sqlx) · PostgreSQL 16 · JWT · Argon2.

## Prérequis

- Rust (via [rustup](https://rustup.rs))
- Docker (pour PostgreSQL)

## Démarrage

```bash
# 1. Lancer la base de données
docker compose up -d

# 2. Copier la configuration
cp .env.example .env

# 3. Lancer l'API (les migrations s'exécutent au démarrage)
cargo run
```

L'API écoute sur `http://localhost:8080`.

```bash
curl http://localhost:8080/health
# {"service":"aether-api","status":"ok"}
```

## Structure

```
src/
├── main.rs        # bootstrap : config, pool DB, migrations, serveur
├── config.rs      # configuration depuis l'environnement
├── state.rs       # état partagé (pool, config)
├── error.rs       # erreur applicative -> réponse HTTP JSON
└── routes/        # handlers HTTP
migrations/        # schéma SQL versionné (issu du MPD du cadrage)
```

## Feuille de route

- [x] Fondation : serveur, pool DB, migrations, `/health`
- [x] Authentification (inscription / connexion, JWT, Argon2)
- [x] CRUD personnage (isolé par utilisateur)
- [x] Parties & boucle de jeu (Maître du Jeu GPT-4o, état persisté)
- [ ] Frontend React
