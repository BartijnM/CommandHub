# Huidige voortgang

Bijgewerkt: 2026-11-03.

## Actief

[F07 — macOS-ondersteuning](features/F07-macos-ondersteuning.md), op branch feature/macos-ondersteuning vóór F04. De eigenaar meldt na de refactor terug op develop te zijn gegaan en deze branch te hebben aangemaakt. Er zijn door de assistent geen Git-commando’s uitgevoerd.

## Aanwezig

De bestaande hoofdapp ondersteunt configuratie, toolontdekking en parameterloze HTTP-uitvoering met timeout, output-/capaciteitslimieten, annulering en shutdown. F07 hergebruikt de Unix-procesgroepen op macOS en voegt een native processtatuscontrole voor tests toe. De CI-matrix bevat nu Ubuntu, Windows en macOS. De macOS-voorbeeldconfiguratie staat in config/tools.macos.json en wordt expliciet via COMMANDHUB_CONFIG gekozen.

De macOS-helper gebruikt proc_pidinfo: een zombie of verdwenen PID geldt als gestopt; API-/toegangsfouten blijven fouten. Nieuwe tests bewaken classificatie, een echte proceslevenscyclus, voorbeeldvalidatie en native echo-uitvoering. De bestaande procesboom-/TCP-tests lopen ongewijzigd mee op macOS.

## Verificatie

Alleen cargo fmt --check en cargo metadata --locked --offline --no-deps --format-version 1 zijn lokaal uitgevoerd en geslaagd. Dit is geen compilatie of gedragstest van de macOS-code. Op verzoek van de eigenaar worden Clippy, tests en build via GitHub Actions uitgevoerd. macOS is lokaal niet beschikbaar. Er zijn nog geen CI-resultaten voor F07; de feature is niet klaar.

Historisch: de refactorcommit 105e69b had volgens de aangeleverde screenshot geslaagde Ubuntu- en Windows-jobs (42 s en 1 m 47 s, totaal 1 m 50 s). De laatste lokale suite vóór F07 telde 64 passed en 4 proceshelpers. Die resultaten bewijzen niet de huidige macOS-uitbreiding. Zie de [F03-notitie](../notities/F03-procesuitvoering.md) voor eerdere verificatie.

## Open en volgende stap

De eigenaar commit en pusht F07 en deelt de Actions-resultaten. Controleer formatter, Clippy, tests en build voor alle drie platforms, los fouten op en werk het featureplan en de [F07-notitie](../notities/F07-macos-ondersteuning.md) bij met het echte bewijs. Daarna review/merge; vervolgens F04. macOS wordt pas als geverifieerd beschreven na een geslaagde job.

De overige macOS-voorbeeldtools en een schone installatie zijn nog niet apart getest. De volledige keten/demonstratie volgt bij F06. Bestaande open administratieve punten voor F00/F01 en exacte mergestatus zijn niet via GitHub gecontroleerd. Geen Git-commando’s, commits, pushes of merges uitgevoerd.
