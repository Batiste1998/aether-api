# Document de cadrage — Quiz d'Æther

Document de cadrage du projet (module M2 Dev Ynov).

- **`blactot-batiste-m2-dev-ynov-connect.pdf`** — le document rendu (14 pages).
- **`cadrage.html`** — la source ; le PDF en est généré.

## Régénérer le PDF

```bash
"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
  --headless --disable-gpu --no-pdf-header-footer \
  --print-to-pdf="blactot-batiste-m2-dev-ynov-connect.pdf" \
  "file://$PWD/cadrage.html"
```

## Sommaire

1. Brief projet — présentation, arborescence, wireframes, fonctionnalités
2. Modélisation de la base de données — MCD (Merise), MLD, MPD (SQL PostgreSQL)
3. Stack technique & justifications
4. Fonctionnalité IA — génération de quiz par GPT-4o
