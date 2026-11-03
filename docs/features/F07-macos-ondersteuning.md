---
id: F07
prioriteit: Must
status: In uitvoering
milestone: "Platformuitbreiding"
---

# F07 — macOS-ondersteuning

## Doel en scope

Voeg macOS toe als ondersteund platform naast Windows en Linux, op verzoek van de eigenaar vóór F04. Het ID is F07 zodat bestaande feature-ID's niet veranderen. Branch: `feature/macos-ondersteuning`. Afhankelijkheid: F03. Requirements: R14, R19 en R17 voor de gebruiksinstructies.

De bestaande Unix-executor gebruikt ook op macOS procesgroepen. Deze uitbreiding voegt een macOS-processtatuscontrole voor tests, een native voorbeeldconfiguratie en een macOS-CI-job toe. Geen nieuwe HTTP-routes, dynamische parameters, MCP-integratie of andere Unix-platforms.

## Todo

- [x] Bestaande Unix-procesuitvoering en platformafhankelijke tests nalopen.
- [x] macOS-processtatus via `proc_pidinfo` toevoegen, met expliciet onderscheid tussen actieve processen, zombies, verdwenen processen en fouten.
- [x] Pure classificatietests en een macOS-specifieke proceslevenscyclustest toevoegen.
- [x] Voorbeeldconfiguratie en een macOS-uitvoeringstest met `/bin/echo` toevoegen.
- [x] `macos-latest` opnemen in dezelfde CI-matrix als Ubuntu en Windows.
- [x] Documentatie, voortgang en featurenotitie bijwerken.
- [ ] CI van deze branch op alle drie platforms controleren; fouten oplossen en resultaten vastleggen.

## Acceptatiecriteria

- [ ] Formatter, Clippy, volledige testsuite en build slagen op macOS via GitHub Actions.
- [ ] De bestaande procesboomtests slagen ook op macOS: timeout, outputlimiet, annulering, shutdown, future-drop, taakafbreking en een eerder afgerond direct child.
- [ ] De macOS-statuscontrole herkent een actief en gestopt childproces; echte API-/toegangsfouten worden niet als beëindiging verborgen.
- [ ] De macOS-voorbeeldconfiguratie valideert en `echo_ready` levert het verwachte resultaat op.
- [ ] Ubuntu- en Windows-jobs blijven slagen.

## Verificatie en afronding

Lokaal op Windows zijn alleen `cargo fmt --check` en `cargo metadata --locked --offline --no-deps --format-version 1` gecontroleerd. Geen tests, Clippy of build uitgevoerd: de eigenaar wil de verificatie via GitHub Actions doen. macOS is lokaal niet beschikbaar. De vinkjes hierboven beschrijven toegevoegde code en documentatie, geen geslaagde platformacceptatie.

- [ ] CI-resultaten vastgelegd.
- [ ] PR beoordeeld en gemerged naar develop; pas daarna status Klaar.

[Featurenotitie](../../notities/F07-macos-ondersteuning.md).
