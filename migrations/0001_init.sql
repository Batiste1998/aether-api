-- ============ Quiz d'Æther — schéma initial ============

CREATE TABLE utilisateur (
    id_utilisateur  SERIAL PRIMARY KEY,
    pseudo          VARCHAR(50)  NOT NULL UNIQUE,
    email           VARCHAR(255) NOT NULL UNIQUE,
    mot_de_passe    VARCHAR(255) NOT NULL,   -- hash Argon2id
    role            VARCHAR(20)  NOT NULL DEFAULT 'joueur'
                    CHECK (role IN ('joueur','admin')),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE categorie (
    id_categorie  SERIAL PRIMARY KEY,
    libelle       VARCHAR(80) NOT NULL UNIQUE,
    description   TEXT,
    emoji         VARCHAR(8)
);

CREATE TABLE quiz (
    id_quiz         SERIAL PRIMARY KEY,
    theme           VARCHAR(120) NOT NULL,
    difficulte      VARCHAR(20)  NOT NULL
                    CHECK (difficulte IN ('facile','moyen','difficile')),
    id_categorie    INT REFERENCES categorie(id_categorie),
    id_utilisateur  INT NOT NULL REFERENCES utilisateur(id_utilisateur) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE question (
    id_question  SERIAL PRIMARY KEY,
    id_quiz      INT NOT NULL REFERENCES quiz(id_quiz) ON DELETE CASCADE,
    position     INT NOT NULL,
    intitule     TEXT NOT NULL,
    explication  TEXT,
    UNIQUE (id_quiz, position)
);

CREATE TABLE choix (
    id_choix      SERIAL PRIMARY KEY,
    id_question   INT NOT NULL REFERENCES question(id_question) ON DELETE CASCADE,
    position      INT NOT NULL,
    texte         TEXT NOT NULL,
    est_correcte  BOOLEAN NOT NULL DEFAULT false,
    UNIQUE (id_question, position)
);

CREATE TABLE session (
    id_session      SERIAL PRIMARY KEY,
    id_quiz         INT NOT NULL REFERENCES quiz(id_quiz) ON DELETE CASCADE,
    id_utilisateur  INT NOT NULL REFERENCES utilisateur(id_utilisateur) ON DELETE CASCADE,
    score           INT NOT NULL DEFAULT 0,
    nb_bonnes       INT NOT NULL DEFAULT 0,
    serie_courante  INT NOT NULL DEFAULT 0,
    serie_max       INT NOT NULL DEFAULT 0,
    termine         BOOLEAN NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE reponse (
    id_reponse   SERIAL PRIMARY KEY,
    id_session   INT NOT NULL REFERENCES session(id_session) ON DELETE CASCADE,
    id_question  INT NOT NULL REFERENCES question(id_question),
    id_choix     INT REFERENCES choix(id_choix),
    correcte     BOOLEAN NOT NULL,
    temps_ms     INT NOT NULL,
    points       INT NOT NULL,
    UNIQUE (id_session, id_question)
);

CREATE INDEX idx_question_quiz   ON question(id_quiz);
CREATE INDEX idx_choix_question  ON choix(id_question);
CREATE INDEX idx_session_user    ON session(id_utilisateur);
CREATE INDEX idx_session_score   ON session(score DESC);
CREATE INDEX idx_reponse_session ON reponse(id_session);
