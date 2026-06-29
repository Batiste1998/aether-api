-- Permet l'upsert d'objets par nom quand le Maître du Jeu en fait gagner au joueur.
ALTER TABLE objet ADD CONSTRAINT objet_nom_unique UNIQUE (nom);
