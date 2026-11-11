---
id: F07
prioriteit: Must
status: Klaar
milestone: "Platformuitbreiding"
---

# F07 — macOS-ondersteuning

## Doel en scope

Voeg macOS toe als ondersteund platform naast Windows en Linux, vóór F04 volgens mijn planning. Het ID is F07 zodat bestaande feature-ID's niet veranderen. Branch: `feature/macos-ondersteuning`. Afhankelijkheid: F03. Requirements: R14, R19 en R17 voor de gebruiksinstructies.

De bestaande Unix-executor gebruikt ook op macOS procesgroepen. Deze uitbreiding voegt een macOS-processtatuscontrole voor tests, een native voorbeeldconfiguratie en een macOS-CI-job toe. Geen nieuwe HTTP-routes, dynamische parameters, MCP-integratie of andere Unix-platforms.

## Todo

- [x] Bestaande Unix-procesuitvoering en platformafhankelijke tests nalopen.
- [x] macOS-processtatus via `proc_pidinfo` toevoegen, met expliciet onderscheid tussen actieve processen, zombies, verdwenen processen en fouten.
- [x] Pure classificatietests en een macOS-specifieke proceslevenscyclustest toevoegen.
- [x] Voorbeeldconfiguratie en een macOS-uitvoeringstest met `/bin/echo` toevoegen.
- [x] `macos-latest` opnemen in dezelfde CI-matrix als Ubuntu en Windows.
- [x] Documentatie, voortgang en featurenotitie bijwerken.
- [x] CI van deze branch op alle drie platforms controleren; screenshot toont drie geslaagde jobs; resultaten vastgelegd in de featurenotitie.

## Acceptatiecriteria

- [x] Formatter, Clippy, volledige testsuite en build slagen op macOS via GitHub Actions.
- [x] De bestaande procesboomtests slagen ook op macOS: timeout, outputlimiet, annulering, shutdown, future-drop, taakafbreking en een eerder afgerond direct child.
- [x] De macOS-statuscontrole herkent een actief en gestopt childproces; echte API-/toegangsfouten worden niet als beëindiging verborgen.
- [x] De macOS-voorbeeldconfiguratie valideert en `echo_ready` levert het verwachte resultaat op.
- [x] Ubuntu- en Windows-jobs blijven slagen.

## Verificatie en afronding

Lokaal op Windows zijn alleen `cargo fmt --check` en `cargo metadata --locked --offline --no-deps --format-version 1` gecontroleerd. Geen tests, Clippy of build uitgevoerd: ik voer de verificatie via GitHub Actions uit. macOS is lokaal niet beschikbaar. De latere screenshot bevestigt de CI op macOS (56 s), Ubuntu (44 s) en Windows (1 m 54 s). De workflow bevat formatter, Clippy, tests en build. Exacte CI-testtotalen en commit-/run-ID zijn niet zichtbaar. De platformacceptatiecriteria zijn daarmee aangetoond. De bestaande afrondingschecklist bevestigt review en merge naar develop.

- [x] CI-resultaten vastgelegd.
- [x] PR beoordeeld en gemerged naar develop; pas daarna status Klaar.

[Featurenotitie](../../notities/F07-macos-ondersteuning.md).


## Oplevercontext op 2026-11-11

De bestaande checklist bevestigt merge naar develop. De huidige macOS-configuratie is bij F06 uitgebreid tot achttien tools, inclusief Python-tools. De CI gebruikt het platformprofiel voor de Inspector-basistest. De volledige actuele toolset staat in de [configuratiehandleiding](../../config/README.md).
