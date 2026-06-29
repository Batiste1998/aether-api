-- ============ Chroniques d'Æther — schéma initial ============

CREATE TABLE classe (
    id_classe          SERIAL PRIMARY KEY,
    nom                VARCHAR(50)  NOT NULL UNIQUE,
    description        TEXT,
    pv_base            INT  NOT NULL DEFAULT 100,
    force_base         INT  NOT NULL DEFAULT 10,
    intelligence_base  INT  NOT NULL DEFAULT 10,
    agilite_base       INT  NOT NULL DEFAULT 10
);

CREATE TABLE competence (
    id_competence  SERIAL PRIMARY KEY,
    nom            VARCHAR(80) NOT NULL,
    description    TEXT,
    type           VARCHAR(20) NOT NULL
                   CHECK (type IN ('attaque','soin','buff','utilitaire')),
    cout_mana      INT NOT NULL DEFAULT 0,
    degats         INT NOT NULL DEFAULT 0
);

CREATE TABLE utilisateur (
    id_utilisateur  SERIAL PRIMARY KEY,
    pseudo          VARCHAR(50)  NOT NULL UNIQUE,
    email           VARCHAR(255) NOT NULL UNIQUE,
    mot_de_passe    VARCHAR(255) NOT NULL,   -- hash Argon2id
    role            VARCHAR(20)  NOT NULL DEFAULT 'joueur'
                    CHECK (role IN ('joueur','admin')),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE personnage (
    id_personnage  SERIAL PRIMARY KEY,
    nom            VARCHAR(80) NOT NULL,
    niveau         INT NOT NULL DEFAULT 1 CHECK (niveau >= 1),
    xp             INT NOT NULL DEFAULT 0,
    pv_actuels     INT NOT NULL,
    pv_max         INT NOT NULL,
    or_pieces      INT NOT NULL DEFAULT 0,
    histoire       TEXT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    id_utilisateur INT NOT NULL REFERENCES utilisateur(id_utilisateur) ON DELETE CASCADE,
    id_classe      INT NOT NULL REFERENCES classe(id_classe)
);

CREATE TABLE objet (
    id_objet     SERIAL PRIMARY KEY,
    nom          VARCHAR(80) NOT NULL,
    description  TEXT,
    type         VARCHAR(20) NOT NULL
                 CHECK (type IN ('arme','armure','consommable','cle')),
    effet        VARCHAR(120),
    valeur       INT NOT NULL DEFAULT 0
);

CREATE TABLE partie (
    id_partie     SERIAL PRIMARY KEY,
    titre         VARCHAR(120) NOT NULL,
    statut        VARCHAR(20)  NOT NULL DEFAULT 'en_cours'
                  CHECK (statut IN ('en_cours','terminee','abandonnee')),
    theme         VARCHAR(80),
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    id_personnage INT NOT NULL REFERENCES personnage(id_personnage) ON DELETE CASCADE
);

CREATE TABLE tour (
    id_tour        SERIAL PRIMARY KEY,
    numero         INT NOT NULL,
    action_joueur  TEXT,
    narration_ia   TEXT NOT NULL,
    etat_jeu       JSONB,          -- snapshot d'état renvoyé par l'IA
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    id_partie      INT NOT NULL REFERENCES partie(id_partie) ON DELETE CASCADE,
    UNIQUE (id_partie, numero)
);

CREATE TABLE quete (
    id_quete       SERIAL PRIMARY KEY,
    titre          VARCHAR(120) NOT NULL,
    description    TEXT,
    statut         VARCHAR(20) NOT NULL DEFAULT 'active'
                   CHECK (statut IN ('active','reussie','echouee')),
    recompense_xp  INT NOT NULL DEFAULT 0,
    recompense_or  INT NOT NULL DEFAULT 0,
    id_partie      INT NOT NULL REFERENCES partie(id_partie) ON DELETE CASCADE
);

CREATE TABLE pnj (
    id_pnj      SERIAL PRIMARY KEY,
    nom         VARCHAR(80) NOT NULL,
    description TEXT,
    attitude    VARCHAR(20) DEFAULT 'neutre',
    id_partie   INT NOT NULL REFERENCES partie(id_partie) ON DELETE CASCADE
);

-- ===== Tables de jointure (associations N,N) =====
CREATE TABLE inventaire (
    id_personnage  INT NOT NULL REFERENCES personnage(id_personnage) ON DELETE CASCADE,
    id_objet       INT NOT NULL REFERENCES objet(id_objet),
    quantite       INT NOT NULL DEFAULT 1 CHECK (quantite > 0),
    equipe         BOOLEAN NOT NULL DEFAULT false,
    PRIMARY KEY (id_personnage, id_objet)
);

CREATE TABLE classe_competence (
    id_classe      INT NOT NULL REFERENCES classe(id_classe) ON DELETE CASCADE,
    id_competence  INT NOT NULL REFERENCES competence(id_competence) ON DELETE CASCADE,
    niveau_requis  INT NOT NULL DEFAULT 1,
    PRIMARY KEY (id_classe, id_competence)
);

-- ===== Index utiles =====
CREATE INDEX idx_personnage_user  ON personnage(id_utilisateur);
CREATE INDEX idx_partie_perso     ON partie(id_personnage);
CREATE INDEX idx_tour_partie      ON tour(id_partie);
