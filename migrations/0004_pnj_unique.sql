-- Évite les PNJ en double quand le Maître du Jeu réintroduit un personnage existant.
-- On supprime d'abord les doublons éventuels (on conserve le plus ancien).
DELETE FROM pnj a
USING pnj b
WHERE a.id_pnj > b.id_pnj
  AND a.id_partie = b.id_partie
  AND a.nom = b.nom;

ALTER TABLE pnj ADD CONSTRAINT pnj_partie_nom_unique UNIQUE (id_partie, nom);
