# Huidige voortgang

Bijgewerkt: 2026-11-03.

## Actief

[F07 — macOS-ondersteuning](features/F07-macos-ondersteuning.md) staat op Review, op branch feature/macos-ondersteuning vóór F04. De aangeleverde screenshot bevestigt geslaagde CI-jobs op macOS, Ubuntu en Windows. De eigenaar meldt na de refactor terug op develop te zijn gegaan en deze branch te hebben aangemaakt. Er zijn door de assistent geen Git-commando’s uitgevoerd.

## Aanwezig

De bestaande hoofdapp ondersteunt configuratie, toolontdekking en parameterloze HTTP-uitvoering met timeout, output-/capaciteitslimieten, annulering en shutdown. F07 hergebruikt de Unix-procesgroepen op macOS en voegt een native processtatuscontrole voor tests toe. De CI-matrix bevat nu Ubuntu, Windows en macOS. De macOS-voorbeeldconfiguratie staat in config/tools.macos.json en wordt expliciet via COMMANDHUB_CONFIG gekozen.

De macOS-helper gebruikt proc_pidinfo: een zombie of verdwenen PID geldt als gestopt; API-/toegangsfouten blijven fouten. Nieuwe tests bewaken classificatie, een echte proceslevenscyclus, voorbeeldvalidatie en native echo-uitvoering. De bestaande procesboom-/TCP-tests lopen ongewijzigd mee op macOS.

## Verificatie

Alleen cargo fmt --check en cargo metadata --locked --offline --no-deps --format-version 1 zijn lokaal uitgevoerd en geslaagd. Dit is geen compilatie of gedragstest van de macOS-code. Op verzoek van de eigenaar worden Clippy, tests en build via GitHub Actions uitgevoerd. macOS is lokaal niet beschikbaar. De later aangeleverde screenshot bevestigt F07 op macOS (56 s), Ubuntu (44 s) en Windows (1 m 54 s). De workflow bevat formatter, Clippy, tests en build. Exacte CI-testtotalen, run-/commit-ID en totale duur zijn niet zichtbaar. De platformacceptatie is aangetoond; review/merge blijft open.

Historisch: de refactorcommit 105e69b had volgens de aangeleverde screenshot geslaagde Ubuntu- en Windows-jobs (42 s en 1 m 47 s, totaal 1 m 50 s). De laatste lokale suite vóór F07 telde 64 passed en 4 proceshelpers. Die resultaten bewijzen niet de huidige macOS-uitbreiding. Zie de [F03-notitie](../notities/F03-procesuitvoering.md) voor eerdere verificatie.

## Open en volgende stap

De eigenaar commit en pusht deze documentatie-update en controleert de CI van de laatste branchstand. Daarna PR-review en merge naar develop, gevolgd door het afsluiten van de branch; vervolgens F04. Markeer F07 pas Klaar na bevestigde merge. Het CI-bewijs staat in de [F07-notitie](../notities/F07-macos-ondersteuning.md).

De overige macOS-voorbeeldtools en een schone installatie zijn nog niet apart getest. De volledige keten/demonstratie volgt bij F06. Bestaande open administratieve punten voor F00/F01 en exacte mergestatus zijn niet via GitHub gecontroleerd. Geen Git-commando’s, commits, pushes of merges uitgevoerd.

Deze sessie: uitsluitend documentatie aangepast op basis van de screenshot. Geen tests opnieuw uitgevoerd en geen Git-commando’s uitgevoerd. De drie waarschuwingen en twee notices veranderen niets aan de groene checks; de notices zijn niet volledig zichtbaar.
