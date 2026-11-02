# Werken aan CommandHub

Lees eerst README.md, docs/voortgang.md, docs/requirements.md, docs/architectuur.md en het actieve featurebestand. Lees vervolgens de bijbehorende notitie in notities/.

- Dit is een leerproject. Leg relevante Rust-keuzes begrijpelijk uit.
- Controleer de actuele code en tests; notities zijn geen bewijs dat iets werkt.
- Implementeer alleen de afgesproken feature. Werk in kleine, controleerbare stappen.
- Featurebestanden zijn de bron voor gedetailleerde todo's en acceptatiecriteria.
- Noteer uitleg, keuzes, problemen en verificatie in de notitie van de feature.
- Werk docs/voortgang.md bij met huidige situatie, open punten en volgende stap.
- Markeer alleen werkelijk geverifieerd werk als klaar.
- Houd HTTP/MCP buiten de kern van configuratie, validatie en procesuitvoering.
- Geen shellstrings opbouwen uit invoer. Programma's en argumenten blijven gescheiden.
- De client kiest geen executable; beheerdersconfiguratie bepaalt toegestane tools.
- Log geen geheimen, volledige parameterwaarden of procesoutput.
- Voer passende controles uit: cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings en cargo test --locked.
- Meld wat niet getest kon worden en waarom.
- Geen commit, push, merge of externe publicatie zonder opdracht van de gebruiker.

## Leerbegeleiding

CommandHub is een leerproject en het eerste Rust-project van de eigenaar. Werk daarom in kleine, controleerbare stappen:

- Begin waar mogelijk met een kleine werkende verticale slice: laat één normaal scenario end-to-end werken en test het.
- Breid die werkende basis daarna stap voor stap uit tot de requirements en acceptatiecriteria zijn aangetoond. Stel noodzakelijke veiligheids- en foutafhandeling niet uit tot na het aansluiten van een publieke route.
- Geef bij leerstappen één klein codefragment tegelijk. Benoem het exacte bestand en de plek waar het fragment hoort, en leg de relevante Rust-syntax en ontwerpkeuzes begrijpelijk uit.
- Laat de eigenaar het codefragment zelf invoeren. Wacht op de uitkomst van de gerichte controle voordat de volgende stap wordt gegeven.
- Pas documentatie alleen aan wanneer de eigenaar daar om vraagt. Wanneer documentatie-update is gevraagd, werk dan de actuele voortgang en de notitie van de feature bij.
