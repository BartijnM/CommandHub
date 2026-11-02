# Huidige voortgang

Bijgewerkt: 2026-11-02.

## Actief

F00 staat klaar voor review. De aparte proefserver werkt via Streamable HTTP; MCP Inspector CLI heeft de vaste tool `hello` ontdekt en aangeroepen. F01 is lokaal gecontroleerd op Windows: formatter, Clippy, tests (0 tests), build en `GET /status` slagen. De GitHub Actions-workflow is inmiddels op Ubuntu en Windows geslaagd; zie het actuele F03-CI-bewijs hieronder.

F02 is gemerged. De hoofdapp leest en valideert de JSON-configuratie bij startup, bouwt een immutable registry en serveert `GET /tools` met toolmetadata en inputschema's. De route laat command en vaste argumenten weg. Configuratie-laadfouten zijn gericht getest; handmatig is bevestigd dat ongeldige configuratie startup stopt met een bruikbare diagnose en dat een aangepaste tool na herstart zonder rebuild zichtbaar wordt. Formatter, Clippy en 14 tests slaagden op Windows. De standaardtimeout is 10 seconden (maximaal 60 seconden); standaardbestand is `config/tools.json`, overschrijfbaar met `COMMANDHUB_CONFIG`. Zie [F02-featureplan](features/F02-toolconfiguratie.md) en [F02-notitie](../notities/F02-toolconfiguratie.md).

F03 staat op Review. De uitvoeringskern, logging, HTTP-concurrency, capaciteitsherstel en shutdown met actieve procesbomen zijn gecontroleerd. Lokale Windows-eindcontroles slagen met 59 tests en 4 proceshelpers. De aangeleverde screenshot van GitHub Actions-run #20 toont `Success` voor `Checks (ubuntu-latest)` en `Checks (windows-latest)` op `feature/F03-procesuitvoering`. Daarmee is ook Linux-verificatie via CI aangetoond. Exacte CI-testtotalen zijn niet zichtbaar; review en merge zijn nog te bevestigen. Zie [F03-featureplan](features/F03-procesuitvoering.md) en [F03-notitie](../notities/F03-procesuitvoering.md).

## Aanwezig

Documentatieopzet, featureplannen, notities per feature, Axum-statusroute, configuratielader, registry, ontdekking en uitvoering via HTTP, gedeelde `AppState`, uitvoeringskern met `process-wrap`, shutdown en automatische tests onder `src/tests/`. Verder de CI-workflow en de geïsoleerde F00-proef onder `prototypes/f00-mcp-probe`. MCP-integratie ontbreekt nog in de hoofdapplicatie.

## Nog open

- F00: review en PR-merge. De proef onderzoekt `tool_not_found` en een extra argument; bredere fout- en transportsecuritytests horen bij F05 waar relevant.
- F01: `Cargo.lock` committen en repository-/develop-/GitHub-Project-inrichting afronden. CI is inmiddels geslaagd op GitHub; Linux is lokaal niet uitgevoerd. Status blijft open tot review en merge.
- F03: review en merge naar develop nog bevestigen. De CI-screenshot bevat niet-blokkerende Node.js 20-annotaties bij de actions; de volledige waarschuwingstekst is niet aangeleverd. Onderhoud van action-versies kan afzonderlijk worden beoordeeld. Geen Git-commando's uitgevoerd.

## Volgende stap

Neem deze documentatie-update mee op de featurebranch en rond review en merge naar develop af volgens de projectwerkwijze. Wacht op de CI van de laatste branchstand voordat de branch wordt afgesloten. Markeer F03 pas als Klaar zodra de merge is bevestigd. Daarna kan F04 (dynamische parameters en padbeleid) worden opgepakt. Geen Git-commando's uitvoeren; documentatie algemeen formuleren.

## Verificatie

Op Windows zijn voor F01 `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`, `cargo build --locked` en een HTTP-request naar `/status` uitgevoerd; alles slaagde. De testbuild bevat 0 tests. De server stopte na Ctrl+C; de terminal gaf exitcode 1 voor de interrupt. De workflow `.github/workflows/ci.yml` is later in de F03-run op beide platforms geslaagd. Voor F00 zijn de Cargo-controles ook op de hoofdcrate en proefcrate geslaagd; beide test-runs vonden 0 tests. Linux is later via de geslaagde F03-CI geverifieerd. MCP Inspector CLI 2.11.0 meldde `tool_not_found` voor een onbekende tool; een extra argument aan parameterloze `hello` werd niet afgewezen. Zie [F00-notitie](../notities/F00-mcp-proef.md) en [ADR-002](beslissingen/ADR-002-mcp-proef.md). Deze historische F01/F00-controles betroffen Windows; het latere F03-platformbewijs staat hieronder.

Actuele F03-verificatie (2026-11-02, Windows): `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked` en `cargo build --locked` zijn uitgevoerd en geslaagd. De laatste volledige test-run: **59 passed, 0 failed, 4 ignored, 0 filtered out**, 5,72 s. De vier genegeerde functies zijn twee bestaande helpers en twee nieuwe HTTP-proceshelpers; ze worden expliciet als child gestart.

De TCP-tests starten een echte Axum-server op een vrije loopbackpoort. Vier requests starten elk een child en subprocess; PID-bestanden bevestigen dat alle vier nog actief zijn voordat het vijfde verzoek 503 krijgt. Na vrijgave slagen de vier requests en werkt een nieuwe uitvoering. Een tweede test signaleert shutdown tijdens actieve HTTP-uitvoering: de client krijgt 503, de server sluit, beide processen stoppen en een nieuwe verbinding wordt niet geaccepteerd. De test injecteert het shutdownsignaal; hij verstuurt geen echte OS-Ctrl+C naar de terminal. `main` gebruikt hetzelfde serverpad met `ctrl_c()`.

Logging van toolnaam uit de registry, uitkomst/foutcategorie en duur is handmatig en automatisch gecontroleerd. Loggingtests gebruiken aparte childprocessen om tijdelijke tracing-subscribers te isoleren. Argumentwaarden en stdout/stderr ontbreken in de gecontroleerde logs.

Historisch: op 2026-10-30 slaagden 55 tests en een echte POST naar `git_version` (succes, exitcode 0, 61 ms); Ctrl+C zonder actieve tools gaf de terminal terug. Linux is lokaal niet uitgevoerd wegens ontbrekende WSL/Linux-target. Aanvullend toont de aangeleverde screenshot van CI-run #20 twee geslaagde platformjobs (Ubuntu 51 s, Windows 1 m 45 s; totale run 1 m 48 s). De lokale workflow bevat formatter, Clippy, tests en build. Testtotalen per CI-job zijn niet zichtbaar; het lokale totaal van 59 wordt niet als Linux-totaal gepresenteerd. Er zijn geen Git-commando's, commits, pushes of merges uitgevoerd.
De screenshot toont ook Node.js 20-verouderingsannotaties bij de actions. Beide jobs en de run zijn groen; de annotaties zijn geen bewijs van mislukte tests. Review/merge zijn niet zichtbaar en blijven open. Alleen documentatie is op basis van dit bewijs aangepast.
