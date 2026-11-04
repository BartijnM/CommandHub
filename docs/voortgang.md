# Huidige voortgang

Bijgewerkt: 2026-11-04.

## Actief

[F04 — Dynamische parameters en padbeleid](features/F04-parameters.md) is gereed voor platformverificatie en PR-review. De implementatie en lokale Windows-controles zijn afgerond. F04 blijft op Review totdat macOS-CI en merge zijn bevestigd. Geen commit, push, merge of externe publicatie door de assistent uitgevoerd.

## Aanwezig

POST /tools/{toolname} gebruikt één gedeeld uitvoeringspad voor parameterloze en dynamische tools: JSON-object → registry → validatie → afzonderlijke argumenten → proces → response. Configuratie beschrijft stringparameters met required, allowed_values en allowed_roots. Onbekende velden, ongeldige typen, NUL, verboden prefixes, niet-toegestane waarden en afgewezen paden geven 400 invalid_parameters vóór processtart.

Iedere parameter moet een volledige placeholder hebben. Maximaal één optionele placeholder mag achteraan staan, zonder vaste argumenten met prefix - of /. Padparameters vereisen bestaande absolute paden binnen bestaande absolute directoryroots. Het gecanonicaliseerde pad wordt doorgegeven. GET /tools toont enum voor waardelijsten en additionalProperties: false.

De oude parameterloze productie-ingangen en ParametersNotSupported zijn verwijderd. Bestaande procestests gebruiken testhelpers die naar de huidige productie-ingang delegeren. Timeout, capaciteit, expliciete annulering, shutdown en opruiming worden zo via hetzelfde pad getest als dynamische uitvoering. Logging deelt eveneens dit pad en bevat geen parameterwaarden of procesoutput.

## Verificatie op Windows

Door de assistent daadwerkelijk uitgevoerd na toestemming van de eigenaar:

- cargo fmt --check: geslaagd.
- cargo clippy --all-targets --locked -- -D warnings: geslaagd.
- cargo test --locked: 138 passed, 0 failed, 5 ignored; 2,83 s voor de suite.

De vijf genegeerde functies zijn expliciete childproceshelpers; de nieuwe argument_echo_helper wordt door een actieve test gestart. De echte argumenttest bevestigt behoud van spaties, quotes, Unicode en shelltekens als één argument. Drie tools uit config/tools.json zijn geconfigureerd, gevalideerd en werkelijk uitgevoerd in de tests. HTTP-tests bewijzen geldige/ongeldige padinvoer, resultaat/foutmapping en de exacte bodygrens. Dynamische logging is in een apart proces getest.

## Grenzen en volgende stap

De aangeleverde CI-screenshot voor commit 2d02834 bevestigt Ubuntu en Windows geslaagd. macOS heeft geen runner gekregen en heeft dus geen controles uitgevoerd. De bestaande CI-matrix bevat beide plus Windows, met formatter, Clippy, tests en build. De Unix-symlinktest is lokaal op Windows niet gecompileerd of uitgevoerd; de groene Ubuntu-job van F04 voert deze test via de workflow wel uit. Exacte CI-testtotalen zijn niet zichtbaar. macOS moet nog volgen. Een afzonderlijke Windows-symlink/junctiontest is niet toegevoegd.

Padcontrole is geen sandbox: roots en symlinkdoelen worden per uitvoering opnieuw opgelost; wijzigingen tussen controle en gebruik blijven mogelijk. Beheerder beheert ook de roots en hun bovenliggende directories. Vrije strings vragen beoordeling per tool; prefixfilters blokkeren niet iedere mogelijke toolspecifieke instructie. De meegeleverde F04-tools gebruiken daarom uitsluitend vaste keuzes. Git-voorbeelden gebruiken de beheerde repository en omgeving van het serverproces.

Volgende stap op 2026-11-05: eigenaar herstart de mislukte macOS-job via Re-run failed jobs en deelt het resultaat. Bij groen kan de PR-review/merge naar develop worden afgerond. De eigenaar heeft eerder gemeld dat de voorbeeldtools werken; er is geen volledige handmatige checklist aangeleverd. Pas na geslaagde platformverificatie en merge F04 als Klaar markeren. Geen nieuwe code of documentatie wijzigen zonder aanleiding na groene eindcontroles.

## Correctie voorbeeldconfiguratie

De drie dynamische voorbeelden staan nu in config/tools.json. Na verwijdering van het aparte bestand zijn beide compile-time testverwijzingen en alle documentatielinks/startinstructies aangepast. Windows: formatter en Clippy geslaagd, 138 tests geslaagd en 5 proceshelpers genegeerd (2,25 s). De getoonde CI-fout was een ontbrekend include_str!-bestand; Ubuntu en Windows zijn daarna via Actions geslaagd; alleen macOS blijft open door ontbrekende runnercapaciteit.

## Overdracht voor 2026-11-05

Branch gecontroleerd: feature/F04-parameters. Laatste codecommit: 2d02834 (fix: laat tests de standaard toolconfiguratie gebruiken). Werkmap was schoon vóór deze documentatie-update. Geen broncodewijzigingen en geen herhaling van lokale tests in deze overdrachtssessie: de code is niet gewijzigd sinds de groene controles.

CI-bewijs uit screenshots: Ubuntu Success (41 s), Windows Success (1 m 30 s); macOS na 15 m 2 s mislukt zonder toegewezen runner. Meldingen: Internal server error en The job was not acquired by Runner of type hosted even after multiple attempts; notices noemen capaciteit voor macOS arm64. Dit is geen aangetoonde code- of testfout. Geen runnerlabel/timeout wijzigen op basis van deze infrastructuurmelding.

De workflow voert formatter, Clippy, tests en build uit op alle drie platforms. Alleen macOS-platformverificatie ontbreekt. Exact runnummer/URL en CI-testtotalen zijn niet beschikbaar. Na een groene herstart: bewijs in voortgang, F04-notitie en featureplan bijwerken; dan PR-review/merge volgens de projectwerkwijze. Status pas Klaar na bevestigde merge. Hervatprompt: [F04-hervatprompt.md](F04-hervatprompt.md).
