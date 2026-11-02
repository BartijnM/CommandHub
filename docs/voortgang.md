# Huidige voortgang

Bijgewerkt: 2026-11-02.

## Actief

F00 staat klaar voor review. De aparte proefserver werkt via Streamable HTTP; MCP Inspector CLI heeft de vaste tool `hello` ontdekt en aangeroepen. F01 is lokaal gecontroleerd op Windows: formatter, Clippy, tests (0 tests), build en `GET /status` slagen. Een GitHub Actions-workflow voor Ubuntu en Windows is toegevoegd, maar nog niet op GitHub uitgevoerd.

F02 is gemerged. De hoofdapp leest en valideert de JSON-configuratie bij startup, bouwt een immutable registry en serveert `GET /tools` met toolmetadata en inputschema's. De route laat command en vaste argumenten weg. Configuratie-laadfouten zijn gericht getest; handmatig is bevestigd dat ongeldige configuratie startup stopt met een bruikbare diagnose en dat een aangepaste tool na herstart zonder rebuild zichtbaar wordt. Formatter, Clippy en 14 tests slaagden op Windows. De standaardtimeout is 10 seconden (maximaal 60 seconden); standaardbestand is `config/tools.json`, overschrijfbaar met `COMMANDHUB_CONFIG`. Zie [F02-featureplan](features/F02-toolconfiguratie.md) en [F02-notitie](../notities/F02-toolconfiguratie.md).

F03 is functioneel en automatisch gecontroleerd op Windows; status blijft In uitvoering vanwege Linux-verificatie en review/merge. De kern ondersteunt begrensde uitvoering, logging, expliciete annulering en shutdown. `server::serve` wordt zowel vanuit `main` met Ctrl+C als in TCP-tests met een gecontroleerd signaal gebruikt. Op 2026-11-02 slagen `cargo fmt --check`, Clippy met `-D warnings`, `cargo test --locked` (59 passed, 4 ignored) en build op Windows. Echte HTTP-concurrency, capaciteitsherstel en actieve procesboomopruiming bij shutdown zijn geverifieerd. Linux is voorbereid maar niet uitgevoerd. Zie [F03-featureplan](features/F03-procesuitvoering.md) en [F03-notitie](../notities/F03-procesuitvoering.md).

## Aanwezig

Documentatieopzet, featureplannen, notities per feature, Axum-statusroute, configuratielader, registry, ontdekking en uitvoering via HTTP, gedeelde `AppState`, uitvoeringskern met `process-wrap`, shutdown en automatische tests onder `src/tests/`. Verder de CI-workflow en de geïsoleerde F00-proef onder `prototypes/f00-mcp-probe`. MCP-integratie ontbreekt nog in de hoofdapplicatie.

## Nog open

- F00: review en PR-merge. De proef onderzoekt `tool_not_found` en een extra argument; bredere fout- en transportsecuritytests horen bij F05 waar relevant.
- F01: `Cargo.lock` committen en repository-/develop-/GitHub-Project-inrichting afronden. CI op GitHub laten draaien; Linux is lokaal niet gecontroleerd. Status blijft open tot review en merge.
- F03: Linux-CI uitvoeren en eventuele platformfouten oplossen. `/proc`-controles, Unix-groepssignalen en een dropbewaker zijn toegevoegd, maar niet op Linux gecompileerd of uitgevoerd. WSL is niet geïnstalleerd; alleen `x86_64-pc-windows-msvc` is beschikbaar. De bestaande CI-matrix omvat Ubuntu en Windows en heeft nu 15 minuten jobtimeout. Review/merge blijven apart open; geen Git-commando's uitgevoerd.

## Volgende stap

Laat de bestaande GitHub Actions-matrix op Ubuntu en Windows draaien zodra de wijzigingen via de normale repositorywerkwijze beschikbaar zijn. Controleer vooral procesopruiming na timeout, eerder afgerond child, future-drop en abort op Linux. Los eventuele Linux-fouten op voordat F03 naar Review gaat. Er is geen Linux-installatie uitgevoerd en geen CI-run gestart. Bestanden lezen mag; de expliciete toestemming voor zelfstandige codewijzigingen gold voor deze F03-afrondingssessie. Volg bij verdere leerstappen de afgesproken uitleg en controles. Documentatie algemeen formuleren; geen Git-commando's uitvoeren.

## Verificatie

Op Windows zijn voor F01 `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`, `cargo build --locked` en een HTTP-request naar `/status` uitgevoerd; alles slaagde. De testbuild bevat 0 tests. De server stopte na Ctrl+C; de terminal gaf exitcode 1 voor de interrupt. De workflow `.github/workflows/ci.yml` is toegevoegd, maar nog niet door GitHub uitgevoerd. Voor F00 zijn de Cargo-controles ook op de hoofdcrate en proefcrate geslaagd; beide test-runs vonden 0 tests. Linux is nog niet geverifieerd. MCP Inspector CLI 2.11.0 meldde `tool_not_found` voor een onbekende tool; een extra argument aan parameterloze `hello` werd niet afgewezen. Zie [F00-notitie](../notities/F00-mcp-proef.md) en [ADR-002](beslissingen/ADR-002-mcp-proef.md). Dit verifieert de basis op Windows, maar niet de platformondersteuning op Linux.

Actuele F03-verificatie (2026-11-02, Windows): `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked` en `cargo build --locked` zijn uitgevoerd en geslaagd. De laatste volledige test-run: **59 passed, 0 failed, 4 ignored, 0 filtered out**, 5,72 s. De vier genegeerde functies zijn twee bestaande helpers en twee nieuwe HTTP-proceshelpers; ze worden expliciet als child gestart.

De TCP-tests starten een echte Axum-server op een vrije loopbackpoort. Vier requests starten elk een child en subprocess; PID-bestanden bevestigen dat alle vier nog actief zijn voordat het vijfde verzoek 503 krijgt. Na vrijgave slagen de vier requests en werkt een nieuwe uitvoering. Een tweede test signaleert shutdown tijdens actieve HTTP-uitvoering: de client krijgt 503, de server sluit, beide processen stoppen en een nieuwe verbinding wordt niet geaccepteerd. De test injecteert het shutdownsignaal; hij verstuurt geen echte OS-Ctrl+C naar de terminal. `main` gebruikt hetzelfde serverpad met `ctrl_c()`.

Logging van toolnaam uit de registry, uitkomst/foutcategorie en duur is handmatig en automatisch gecontroleerd. Loggingtests gebruiken aparte childprocessen om tijdelijke tracing-subscribers te isoleren. Argumentwaarden en stdout/stderr ontbreken in de gecontroleerde logs.

Historisch: op 2026-10-30 slaagden 55 tests en een echte POST naar `git_version` (succes, exitcode 0, 61 ms); Ctrl+C zonder actieve tools gaf de terminal terug. Linux is op 2026-11-02 niet uitgevoerd: geen WSL of Linux-target aanwezig. De Unix-wijzigingen moeten nog door Ubuntu-CI worden geverifieerd. Er zijn geen Git-commando's, commits, pushes of merges uitgevoerd.
