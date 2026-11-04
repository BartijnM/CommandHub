# Testplan

Historisch bewijs vóór F04: 2026-11-03. De laatste geverifieerde Windows-suite vóór F07 bevatte 64 geslaagde tests en 4 genegeerde proceshelpers. F07 voegt macOS-tests toe; de aangeleverde screenshot toont de nieuwe jobs op macOS, Ubuntu en Windows geslaagd. Exacte testtotalen zijn niet zichtbaar. De Windows-statuscontrole is vervangen; de aangeleverde CI-screenshot bevestigt de reviewstand op Ubuntu en Windows (commit 105e69b).

| Eisen | Aanwezig bewijs en resterende controles |
| --- | --- |
| R01–R03 | Configuratie-, laadfout- en ontdekkingstests; lege namen/commands worden afgewezen. Herstart zonder rebuild is eerder handmatig gecontroleerd |
| R04–R06 | Succes, niet-nul exit, startfout, output, timeout, annulering en shutdown; PID-controles bij stoproutes, future-drop en taakafbreking |
| R07 | Gedeeld outputbudget, capaciteitsgrens/vrijgave, body exact 16 KiB en overschrijding; vier echte actieve TCP-uitvoeringen en vijfde 503, daarna capaciteitsherstel |
| R08–R10 | F04: lokale configuratie-, validatie-, argument- en servicetests als geslaagd gemeld; HTTP-padtests en voorbeeldtoolreview geslaagd; Unix-symlink-CI nog open |
| R11/R16 | Geïsoleerde F00-proef; protocol- en Host/Origin-controles bij F05 |
| R12 | Routertests voor resultaat en fouten; echte TCP-tests voor capaciteit en actieve shutdown |
| R13 | Vaste foutcodes en geen OS-details in JSON; geïsoleerde loggingtests bewaken het ontbreken van argumentwaarden en procesoutput |
| R14/R15 | Kern zonder HTTP-afhankelijkheid; Windows-suite slaagt. Historische CI-run #22 controleerde Ubuntu/Windows na ESRCH-correctie. De reviewstand van commit 105e69b is eveneens op Ubuntu en Windows via CI geslaagd |
| R17 | README-start-/POST-voorbeeld; volledige demo en schone-checkoutcontrole bij F06 |

## Aanvullende macOS-verificatie (F07)

Dezelfde volledige suite draait nu ook op macos-latest in GitHub Actions. De bestaande procesboom- en TCP-scenario’s gebruiken daar de nieuwe proc_pidinfo-controle. Toegevoegd zijn drie pure classificatietests (actief/zombie, verdwenen PID en echte fouten), een macOS-only levenscyclustest, validatie van tools.macos.json op alle platforms en een macOS-only uitvoering van echo_ready uit dat bestand. De overige voorbeeldtools zijn niet afzonderlijk getest.

De eigenaar laat Clippy, tests en build via Actions uitvoeren. Deze wijziging heeft lokaal alleen formatterings- en manifestcontrole gehad. De aangeleverde screenshot bevestigt nu alle drie jobs: macOS 56 s, Ubuntu 44 s en Windows 1 m 54 s. Daarmee zijn de F07-platformcriteria aangetoond; review/merge blijft open. Het eerdere bewijs voor 105e69b geldt uitsluitend voor de refactor vóór F07.

## Testindeling

Tests staan in src/tests/, maar zijn via cfg(test) en path submodules van hun oorspronkelijke module, met behoud van private toegang. Er zijn zeven bestanden: config_tests.rs, execution_tests.rs, routes_tests.rs, execution_controller_tests.rs, execution_model_tests.rs, http_execution_tests.rs en process_checks.rs.

Alle functies hebben Engelse namen. Exacte helperselecties zijn mee hernoemd. De vier genegeerde helpers worden met --exact en --ignored als child gestart. De subprocessscenario’s delen helpers voor toolopbouw, PID-bestanden, startbevestiging en gelijktijdig reserveren van alle permits. Routertests delen POST-opbouw en JSON-uitlezen; scenariospecifieke assertions blijven zichtbaar.

## Procesopruiming

Windows gebruikt OpenProcess met alleen PROCESS_SYNCHRONIZE en WaitForSingleObject met timeout 0. OwnedHandle sluit het handle automatisch; een gesignaleerd proceshandle of ontbrekend positief PID geldt als gestopt. Andere API-/toegangsfouten blijven fouten. De regressietest controleert de eigen actieve PID, een ontbrekende PID en een echt gestart en beëindigd childproces. Er wordt geen tasklist gestart.

Linux leest /proc/<pid>/stat. NotFound/ENOENT en ESRCH betekenen verdwenen; Z/X betekenen beëindigd, niet noodzakelijk gereaped. Echte lees-/formaatfouten blijven fouten. Drie pure regressietests draaien ook op Windows. assert_stopped controleert maximaal 5 s met 50 ms tussenpozen. Dit is geen sandbox of harde garantie voor blokkerende OS-processtart.

## HTTP en shutdown

Routertests gebruiken oneshot; TCP-tests starten echte loopbackservers op vrije poorten. PID-bestanden bevestigen processtart voordat capaciteit of shutdown wordt beoordeeld. De TCP-client gebruikt HTTP/1.0 met connection-close.

De actieve shutdowntest injecteert een kanaalsignaal in hetzelfde serverpad als Ctrl+C. Hij controleert 503, afgesloten listener en gestopte processen. Een echt OS-Ctrl+C-signaal tijdens actieve uitvoering is niet opnieuw getest. Historisch is Ctrl+C zonder actieve tools handmatig gecontroleerd, evenals een echte POST naar git_version.

## Controles en historie

Voer cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings en cargo test --locked uit. De CI-matrix voert ook cargo build --locked uit op Ubuntu, Windows en macOS.

- Voor de review: 62 geslaagde Windows-tests, 4 helpers; latere runs faalden wisselend op externe tasklist-aanroepen en HTTP-tijdslimieten.
- Na de correcties: gerichte configuratiesuite 14 geslaagd; processtatussuite 4 geslaagd; volledige parallelle suite 64 geslaagd, 4 helpers.
- Run #21: historische Ubuntu-fout door ESRCH tijdens /proc-lezen; drie regressietests toegevoegd.
- Run #22: eerder aangeleverde screenshot met Ubuntu en Windows Success (45 s en 2 m 51 s). Exacte CI-testtotalen zijn onbekend.
- De huidige refactor is lokaal op Windows en via CI op Ubuntu/Windows gecontroleerd voor commit 105e69b. Actuele externe links en mergestatus zijn niet geverifieerd.

De [F03-notitie](../notities/F03-procesuitvoering.md) bevat de actuele eindcontrole en eerdere werksessies.

## Demo bij oplevering

Toon configuratie laden, ontdekking, parameterloze uitvoering, niet-nul exit, invoerfouten, outputlimiet, timeout, capaciteit, logging en shutdown. Voeg bij F04/F05 dynamische parameters en MCP toe; rond bij F06 de volledige demo op alle drie platforms af.

CI-bewijs refactor: aangeleverde screenshot voor 105e69b op refactor/na-feature-3 toont Ubuntu Success (42 s), Windows Success (1 m 47 s) en run Success (1 m 50 s). Exacte CI-testtotalen zijn onbekend. De documentatie-update zelf is nog niet gepusht; review/merge blijft open.

## Historische F04-tussenstand — 2026-11-04

De eigenaar meldt lokale controles op Windows geslaagd: config::tests::, validation::tests::, execution::tests::, controllers::execution_controller::tests:: en routes::tests::. De volledige suite is tijdens de implementatie herhaald geslaagd gemeld. Eén tussenstand telde 107 tests; geen actueel eindtotaal aangeleverd. Na de laatste drie dynamische routetests is alleen de gerichte routes-suite bevestigd. De assistent heeft geen controles uitgevoerd.

Nieuw bewijs omvat vereiste/optionele velden, typen, NUL/Unicode, toegestane waarden, prefixes, optionele argumentregels, canonicalisatie/roots/traversal, een echte rustc-call via service en router en bodies van exact 16 KiB en 16 KiB + 1. De routertests gebruiken geen TCP. De Unix-only symlinktest is op Windows niet uitgevoerd.

Resterend: HTTP-padtests, dynamische logging zonder invoerwaarden/output, ontdekschema, laatste veiligheidsreview en aanvullende padgrenzen. Daarna cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings en cargo test --locked en CI op drie platforms. Oudere CI-resultaten gelden niet voor F04.

## Actuele F04-eindcontrole op Windows

De assistent heeft na expliciete toestemming formatter, Clippy met -D warnings en de volledige suite uitgevoerd: 138 passed, 0 failed, 5 ignored; 2,83 s. De extra ignored helper controleert in een childproces dat quotes, Unicode, spaties en shelltekens één argument blijven. De oude test die parameters geheel verbood is verwijderd, omdat dat gedrag niet meer bij F04 hoort.

Ook aanwezig en geslaagd: HTTP-padscenario’s, enum/additionalProperties in toolontdekking, geïsoleerde dynamische logging, verplichte placeholderkoppeling en ontbrekende root. Alle drie F04-voorbeeldtools zijn werkelijk uitgevoerd. Oude procestests gebruiken via testadapters het actuele generieke productiepad.

Nog open: Actions op Windows/Linux/macOS, met Unix-symlinktest. Windows-symlinks/junctions hebben geen afzonderlijke test. Handmatige voorbeelden zijn beschreven, niet namens de eigenaar uitgevoerd. Historische open punten hierboven zijn hiermee vervangen door deze actuele stand.

## Laatste F04-CI-bewijs

Screenshots voor commit 2d02834: Ubuntu Success (41 s), Windows Success (1 m 30 s). De workflow omvat formatter, Clippy, tests en build, inclusief de Unix-symlinktest op Ubuntu. macOS kreeg na meerdere pogingen geen hosted runner en eindigde na 15 m 2 s; macOS-tests zijn niet uitgevoerd. Alleen macOS-platformverificatie ontbreekt. Geen exacte run-URL of CI-testtotalen aangeleverd. Volgende stap: Re-run failed jobs, daarna bewijs vastleggen en PR-review/merge.

## CI-afronding — 2026-11-04

De aangeleverde screenshot van run #39 voor commit 1fb8d71 op feature/F04-parameters toont Success: macOS 57 s, Ubuntu 40 s en Windows 1 m 50 s; totale duur 1 m 54 s. De workflow voert formatter, Clippy, tests en build uit. Hiermee is ook de macOS-platformverificatie bevestigd; de eerdere runnercapaciteitsmelding is geen open blokkade meer. Exacte CI-testtotalen zijn niet zichtbaar. De eigenaar meldt dat alles werkt. Alleen PR-review/merge en branchafsluiting resten; status pas Klaar na bevestigde merge. Bij deze documentatie-update zijn geen broncode gewijzigd, tests herhaald of Git-mutaties uitgevoerd.
