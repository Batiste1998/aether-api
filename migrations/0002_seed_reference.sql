-- ============ Données de référence (classes, objets, compétences) ============

INSERT INTO classe (nom, description, pv_base, force_base, intelligence_base, agilite_base) VALUES
    ('Guerrier', 'Combattant robuste, expert du corps à corps.', 130, 16, 6,  9),
    ('Mage',     'Manieur d''arcanes, fragile mais dévastateur.',  80,  6, 16, 8),
    ('Rôdeur',   'Pisteur agile, redoutable à distance.',         100, 10, 9, 16);

INSERT INTO competence (nom, description, type, cout_mana, degats) VALUES
    ('Coup puissant',   'Une attaque physique surpuissante.',          'attaque', 0,  25),
    ('Boule de feu',    'Projette une déflagration ardente.',          'attaque', 15, 35),
    ('Tir précis',      'Une flèche visant un point vital.',           'attaque', 0,  20),
    ('Soin mineur',     'Restaure une partie des points de vie.',      'soin',    10, 0),
    ('Cri de guerre',   'Augmente temporairement la force.',           'buff',    0,  0),
    ('Crochetage',      'Permet d''ouvrir serrures et coffres.',       'utilitaire', 0, 0);

-- Liaison classes <-> compétences (niveau de déblocage)
INSERT INTO classe_competence (id_classe, id_competence, niveau_requis) VALUES
    (1, 1, 1), (1, 5, 3),          -- Guerrier : Coup puissant, Cri de guerre
    (2, 2, 1), (2, 4, 2),          -- Mage : Boule de feu, Soin mineur
    (3, 3, 1), (3, 6, 1);          -- Rôdeur : Tir précis, Crochetage

INSERT INTO objet (nom, description, type, effet, valeur) VALUES
    ('Épée courte',     'Une lame d''acier équilibrée.',            'arme',        '+5 force',     30),
    ('Bâton d''apprenti','Un bâton canalisant la magie.',           'arme',        '+5 intelligence', 30),
    ('Arc de chasse',   'Un arc en bois solide.',                   'arme',        '+5 agilité',   30),
    ('Armure de cuir',  'Protection légère et souple.',             'armure',      '+10 PV max',   40),
    ('Potion de soin',  'Restaure 30 points de vie.',               'consommable', '+30 PV',       15),
    ('Clé rouillée',    'Ouvre une serrure quelque part…',          'cle',         NULL,           0);
