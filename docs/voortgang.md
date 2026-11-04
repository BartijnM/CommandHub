# Huidige voortgang

Bijgewerkt: 2026-11-04.

## Actief

[F04 — Dynamische parameters en padbeleid](features/F04-parameters.md) staat op Review. Implementatie, lokale controles en CI op alle drie platforms zijn geslaagd. Alleen PR-review/merge naar develop en branchafsluiting resten. Markeer F04 pas Klaar na bevestigde merge. De eigenaar commit deze documentatie en sluit de branch zelf af.

## Aanwezig

Dynamische HTTP-uitvoering met verplichte/onbekende/typecontrole, allowed_values, veilige afzonderlijke argumenten en allowed_roots. Padcanonicalisatie controleert symlinkdoelen binnen toegestane roots. Iedere parameter vereist een placeholder; maximaal één optionele placeholder achteraan, zonder vaste opties. GET /tools toont enum en additionalProperties: false. Eén gedeeld uitvoeringspad voor parameterloze en dynamische tools, met capaciteit, timeout, outputlimiet, annulering, shutdown en logging zonder parameterwaarden of output.

De voorbeeldtools rust_details, git_recent en git_diff_view staan in config/tools.json. De aparte F04-configuratie is verwijderd en de tests lezen de standaardconfiguratie. Zie [handmatig testen](F04-handmatig-testen.md).

## Verificatie

Lokaal Windows: cargo fmt --check en cargo clippy --all-targets --locked -- -D warnings geslaagd; cargo test --locked: 138 passed, 0 failed, 5 ignored (expliciete proceshelpers). Laatste lokale suite: 2,25 s. HTTP-paden, dynamische logging, argumentgrenzen en voorbeeldtools zijn getest.

Screenshot run #39 voor commit 1fb8d71 op feature/F04-parameters: Success op macOS (57 s), Ubuntu (40 s) en Windows (1 m 50 s); totaal 1 m 54 s. De workflow bevat formatter, Clippy, tests en build. Ook de Unix-symlinktest loopt op Ubuntu/macOS mee. Exacte CI-testtotalen zijn niet zichtbaar. De eerdere macOS-run voor 2d02834 kreeg geen runner; deze infrastructuurblokkade is nu vervallen. De eigenaar meldt dat alles werkt.

## Grenzen

Geen sandbox of bescherming tegen wijzigingen tussen padcontrole en gebruik. Alleen bestaande absolute clientpaden; niet-UTF-8-paden worden afgewezen. Roots, bovenliggende directories en toolconfiguratie worden door de beheerder beheerd. Prefixfilters vervangen geen toolspecifieke beoordeling. Geen afzonderlijke Windows-symlink/junctiontest. Deze bewuste grenzen zijn geen nieuwe featureopdracht.

## Volgende stap

Documentatie committen, PR beoordelen en naar develop mergen, daarna branch afsluiten. Na bevestigde merge status/checklist op Klaar zetten. F05 (MCP-integratie) is de volgende feature, pas starten op opdracht. Deze sessie wijzigde uitsluitend documentatie op basis van de screenshot; geen broncodewijzigingen, nieuwe testuitvoering, commits, pushes of merges door de assistent.
