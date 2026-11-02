# Testplan

Status bijgewerkt: 2026-11-02. Formatter, Clippy, build en de volledige suite slagen op Windows: **59 passed, 0 failed, 4 ignored**. Logging, echte TCP-concurrency en actieve server-shutdown zijn gecontroleerd. Aanvullend toont de aangeleverde screenshot van CI-run #20 succes op Ubuntu en Windows. Daarmee is de platformacceptatie via CI aangetoond; review en merge blijven open.

| Eisen | Aanwezig bewijs en resterende controles |
| --- | --- |
| R01–R03 | Configuratievalidatie en laadfouten; ontdekking via HTTP; eerdere handmatige herstartcontrole zonder rebuild |
| R04–R06 | Toolselectie, succes, niet-nul exit, startfout, output, timeout, annulering en shutdown. Windows-PID-controles na stoproutes, future-drop en taakafbreking. Linux-CI geslaagd |
| R07 | Gedeeld outputbudget, child-output boven 1 MiB, capaciteitsgrens/vrijgave. HTTP-body exact 16 KiB toegestaan en groter afgewezen. HTTP 503 bij vier vastgehouden permits én bij vier echte actieve TCP-uitvoeringen; capaciteitsherstel na hun succesvolle afloop getest |
| R08–R10 | F04: dynamische parameters, typen, onbekende velden, optie-injectie en padbeleid |
| R11/R16 | F00: MCP Inspector CLI; protocol-/Origin-/Host-controles bij F05 verder verifiëren |
| R12 | Windows-routetests voor succes, niet-nul exit als 200, 400/404/413/500/502/503/504 en shutdownafwijzing; echte POST naar geconfigureerde tool geslaagd |
| R13 | Vaste foutcodes en geen OS-details in JSON getest. Loggingwrapper en geïsoleerde loggingtests controleren geselecteerde metadata en het ontbreken van client-/argumentwaarden en procesoutput |
| R14/R15 | Kern is transportonafhankelijk; HTTP-adapter gebruikt gedeelde state. Dezelfde kernscenario's en PID-opruiming worden op Linux door de geslaagde Ubuntu-CI gecontroleerd |
| R17 | README bevat start- en POST-voorbeeld; complete demo en schone-checkoutcontrole volgen bij oplevering |

## Testindeling

De oorspronkelijke vijf testbestanden staan in `src/tests/`: `config_tests.rs`, `execution_tests.rs`, `routes_tests.rs`, `execution_controller_tests.rs` en `execution_model_tests.rs`. De productiemodules koppelen ze met `#[cfg(test)]`, `#[path]` en `mod tests;`. De logische paden en toegang tot private code blijven gelijk. Deze indeling is geen zelfstandige integratiecrate onder `tests/` in de projecthoofdmap.

Daarnaast zijn `http_execution_tests.rs` voor echte loopback/TCP-tests en `process_checks.rs` voor gedeelde Windows/Linux-statuscontrole toegevoegd.

De vier genegeerde helpers worden bewust via de testbinary als child gestart met `--exact`, `--ignored` en `--nocapture`. De helper voor subprocessen start `ping` op Windows of `sleep` op Unix en schrijft een PID-bestand. Elke test gebruikt een eigen naam en verwijdert oude bestanden vooraf. Markers via `--skip` kiezen grote output of vroegtijdig afgerond child. Annulering en shutdown wachten met `join!` op een leesbaar PID.

## Procesopruiming

De gedeelde helper `process_checks::assert_stopped` gebruikt op Windows async `tasklist` met 3 s timeout en `kill_on_drop(true)`. Hij controleert herhaald met 50 ms tussenpozen en maximaal 5 s totale wachttijd. Zo wordt vertraagde beëindiging opgevangen. Dit is geen harde tijdsgarantie voor eventuele blokkerende OS-processtart.

Tests bewaken ook capaciteitsvrijgave. Shutdown weigert nieuwe reserveringen en wacht op alle permits. Een afzonderlijke test houdt één permit vast om te bewijzen dat shutdown wacht. Future-drop en Tokio-`abort` zijn getest; na `abort` wordt de taak afgewacht voordat capaciteit en PID worden beoordeeld. Normaal stoppen gebruikt expliciete annulering/shutdown; drop/abort is een vangnet zonder teruggegeven uitvoerings-/opruimresultaat.

Linux heeft nu een `/proc`-statuscontrole, inclusief onderscheid van zombieprocessen. Unix-groepssignalen en future-dropbewaking zijn voorbereid op basis van de lokale wrapperbron. Deze code is lokaal niet op Linux uitgevoerd: WSL ontbreekt en alleen de Windows-target is geïnstalleerd. De aangeleverde screenshot bevestigt daarna een geslaagde Ubuntu-CI-job. De bestaande Ubuntu/Windows-CI-matrix voert alle Cargo-controles uit en heeft een jobtimeout van 15 minuten.

## HTTP-controles

De tests lopen rechtstreeks door de router via `oneshot`, zonder TCP-poort. Ze controleren succes, resultaat-JSON, niet-nul exit, ongeldige JSON/contenttype/invoer, onbekende tool, parameters afwijzen, exacte bodygrens en overschrijding, startfout, timeout, outputlimiet, capaciteit en shutdownafwijzing. De eerdere capaciteitstest in `execution_tests.rs` reserveert private permits. Aanvullend start `http_execution_tests.rs` een echte server met vier actieve toolrequests; PID-bestanden bewijzen dat alle procesbomen zijn gestart voordat het vijfde request 503 krijgt. Na vrijgave slagen de oorspronkelijke requests en een nieuwe uitvoering.

Handmatig is een echte POST uitgevoerd naar `/tools/git_version` met JSON `{}`. Resultaat: succes, Git 2.55.0.windows.3 op stdout, lege stderr, exitcode 0, duration 61 ms. Statusroute en Ctrl+C zonder actieve tools zijn ook gecontroleerd. Het actieve afsluitpad wordt nu via echte TCP-integratie met een geïnjecteerd shutdownsignaal getest, inclusief clientresponse, gesloten server en beide PID's. De OS-Ctrl+C-signaalbron is in die actieve test niet zelf geactiveerd.

## Controles

```powershell
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

## Meetresultaten

| Datum/platform | Controle | Resultaat |
| --- | --- | --- |
| 2026-10-30, Windows | `cargo test --locked` na testverplaatsing | Gerapporteerd: 55 passed, 0 failed, 2 ignored, 0 filtered out, 2,89 s |
| 2026-10-30, Windows | Echte HTTP POST | `git_version`: succes, exitcode 0, duration 61 ms |
| 2026-10-30, Windows | Status en Ctrl+C | Status werkt; terminalprompt keert terug, zonder actieve tools |
| 2026-11-02, Windows | Formatter, Clippy (`-D warnings`), build | Alle geslaagd |
| 2026-11-02, Windows | `cargo test --locked` | 59 passed, 0 failed, 4 ignored, 5,72 s |
| 2026-11-02, Windows | Echte TCP-concurrency en actieve shutdown | Geslaagd, inclusief processtatuscontroles |
| 2026-11-02, Ubuntu/Windows CI | GitHub Actions-run #20 op featurebranch | Beide platformjobs Success; screenshot als bewijs, exacte CI-testtotalen niet zichtbaar |

De eindcontroles van 2026-11-02 zijn daadwerkelijk uitgevoerd op Windows. Linux is daarna via CI gecontroleerd; de screenshot van run #20 is vastgelegd. Bij deze documentatie-update zijn geen tests of Git-commando's uitgevoerd.

## Demo

1. Configuratie laden en tools ontdekken.
2. Parameterloze tool uitvoeren via HTTP.
3. Resultaat en niet-nul exit uitleggen.
4. Invoerfouten, timeout, outputlimiet en volle capaciteit tonen.
5. Logging, concurrency en het gecontroleerde shutdownpad tijdens uitvoering tonen.
6. MCP-demo bij F05; complete demo en herstart zonder rebuild bij oplevering.
De CI-screenshot toont niet-blokkerende Node.js 20-verouderingsannotaties. De volledige annotatietekst en de afzonderlijke testlogs zijn niet aangeleverd; daaruit worden geen aanvullende aantallen of details afgeleid.
