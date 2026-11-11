---
feature: F03
tags:
  - commandhub
  - feature-notitie
---

# F03 — Geconfigureerde tool met vaste argumenten uitvoeren

[Featureplan](../docs/features/F03-procesuitvoering.md)

## Leeswijzer bij de oplevering

Deze notitie bewaart de ontwikkeling en resultaten per werksessie. Beschrijvingen van oude ingangen, configuraties en open punten horen bij hun gedateerde stand. Het actuele gedrag staat in het [API-contract](../docs/api-en-configuratie.md), de [architectuur](../docs/architectuur.md) en de [voortgang](../docs/voortgang.md).

## Wat bouw ik en waarom?

F03 bevat een gedeelde, begrensde uitvoeringskern en parameterloze POST-route. Na de reviewcorrecties van 2026-11-03 slagen op Windows 64 tests; 4 genegeerde functies zijn expliciete proceshelpers. Formatter en Clippy slagen eveneens. CI-run #22 bevestigde eerder Ubuntu en Windows na de ESRCH-correctie. De huidige reviewwijzigingen zijn via CI op Ubuntu en Windows gecontroleerd voor commit 105e69b; deze resultaten horen bij de historische reviewstand. De merge is in de featurechecklist bevestigd.

## Hoe werkte de code in F03?

`src/execution.rs` bevat `ExecutionResult`, `ExecutionError` en `ExecutionService`. Toolselectie weigert onbekende tools en tooldefinities met parameters/placeholders. ToolRegistry::find zoekt eenmaal op naam; logging en uitvoering gebruiken dezelfde geleende definitie. De referentie kopieert de tool niet en mag niet langer leven dan de registry.

`Arc<Semaphore>` deelt vier capaciteits-permits. `try_acquire_owned` wijst volle capaciteit direct af. Een `OwnedSemaphorePermit` geeft de plek terug bij drop. De command-opbouw houdt executable en argumenten gescheiden, sluit stdin en pipet stdout/stderr. Omgeving en werkdirectory worden van de server geërfd.

`read_capped` leest blokken van 8192 bytes. Een gedeeld bytebudget begrenst stdout en stderr samen op 1 MiB. De outputstructuur bewaart de bijbehorende permits zolang de bytes aanwezig zijn. `try_join!` leest beide streams tegelijk. De uitvoeringsfuture wacht daarna op het proces; `timeout` bewaakt de geconfigureerde duur.

`process-wrap` 10.0.1 met `tokio1` gebruikt Unix `ProcessGroup`, Windows `JobObject` en `KillOnDrop`. `Box<dyn ChildWrapper>` biedt een gemeenschappelijke interface voor pipes, wachten en beëindigen. `stop_child` verschilt per platform: Windows wacht kill alleen af wanneer het directe child nog loopt; het Job Object beheert de subprocessen. Unix stuurt ook na afloop van de leider een groepssignaal en wacht daarna de child af. ESRCH betekent dat de groep al verdwenen is. Een opruimfout kan via `?` de oorspronkelijke uitvoeringsfout vervangen.

## Annulering, shutdown en eigendom

`execute_parameterless` delegeert naar de annuleerbare methode. Die ontvangt een `oneshot::Receiver<()>`: één expliciet signaal zonder aanvullende gegevens. `tokio::select!` wacht op uitvoering met timeout, `Ok(())` uit dit kanaal of shutdown. Een gesloten zender zonder bericht past niet op `Ok(())` en annuleert niet.

Shutdown gebruikt `watch::Sender<bool>`: `false` is actief, `true` betekent afsluiten. Elke uitvoering maakt vóór processtart een ontvanger. `try_acquire` houdt de leesbewaker vast tijdens controle en reservering, zodat shutdown niet daartussen kan worden ingesteld. `send_replace(true)` bewaart de status ook zonder ontvangers. Nieuwe uitvoeringen krijgen `Cancelled`; actieve uitvoeringen gaan door hun opruimroute.

`shutdown().await` vraagt alle vier permits tegelijk aan en wacht zo op afronding van bestaande uitvoeringsfutures. Hij verzamelt geen opruimfouten en bewijst zonder PID-controle niet dat elk proces weg is. `main.rs` geeft de Ctrl+C-future aan `server::serve`. Dat signaleert shutdown, laat Axum graceful afsluiten en wacht op de uitvoeringsservice, ook vóór het doorgeven van een serverfout. TCP-tests injecteren een kanaalsignaal in hetzelfde serverpad.

`AppState` bevat gedeelde `Arc`-verwijzingen naar registry en executor. `Clone` kopieert deze verwijzingen, niet de service. `FromRef<AppState>` laat de ontdekcontroller alleen de registry ontvangen. `server::serve` kloont de serviceverwijzing voor zijn `async move`-shutdownblok. De Ctrl+C-future in main bevat zelf geen serviceverwijzing. `state.clone()` behoudt state voor de latere afgewachte shutdown.

**Stopbeleid:** expliciete annulering/shutdown is de normale route met afgewachte opruimpoging. Future-drop en `JoinHandle::abort` gebruiken `KillOnDrop` als vangnet, aangevuld met `ProcessGuard` dat op Unix de groepswrapper een stopsignaal laat sturen. Daarbij is geen normaal uitvoeringsresultaat of afgewachte opruimfout beschikbaar. Na `abort` wacht de test de taak af voordat capaciteit en PID worden gecontroleerd. Deze scenario's zijn op Windows lokaal en op Linux via CI gecontroleerd. Expliciet HTTP-clientdisconnectbeleid blijft buiten deze afgeronde F03-scenario's.

## HTTP-adapter

`src/controllers/execution_controller.rs` leest `State`, `Path` en als laatste de body via `Result<Json<Value>, JsonRejection>`. De handler vangt extractiefouten op en accepteert uitsluitend een leeg object. Pas daarna wordt de tool geselecteerd. `DefaultBodyLimit` op de POST-route begrenst JSON-uitlezen op 16 KiB vóór deserialisatie. Precies 16 KiB mag door.

`src/models/execution.rs` vertaalt met `From<ExecutionResult>` het kernresultaat naar `ExecutionResponse`. De strings worden verplaatst, niet gekloond. Serde `camelCase` maakt `exitCode`; `Option<i32>` wordt een getal of null. `Duration::as_millis()` levert `u128`, gebruikt voor het numerieke veld `duration`. De uitvoeringskern krijgt geen HTTP- of Serde-afhankelijkheid.

Foutresponses gebruiken vaste `code` en `message` met `&'static str`; interne OS-fouten worden niet teruggestuurd. Mapping: 400 ongeldige JSON/invoer of parameters, 404 onbekende tool, 413 body te groot, 500 interne procesbeheerfout, 502 outputlimiet, 503 volle capaciteit of annulering/shutdown, 504 timeout. Een niet-nul exitcode blijft HTTP 200 met `success: false`. Zie [API-contract](../docs/api-en-configuratie.md) voor details.

## Testindeling en procescontrole

De vijf oorspronkelijke testmodules staan onder `src/tests/`; daar zijn HTTP-integratietests en gedeelde procescontroles bij gekomen. De productiebestanden koppelen ze via `#[cfg(test)]`, `#[path]` en `mod tests;`. Voor modules in `controllers/` en `models/` begint het pad met `../tests/`. De logische modules blijven hetzelfde: `super`-imports en de helperselectie `execution::tests::subprocess_timeout_helper` wijzigen niet. Het is geen projectmap met zelfstandige integratiecrates.

De testbinary start genegeerde helpers als aparte processen. De subprocess-helper schrijft een PID-bestand en start `ping` op Windows of `sleep` op Unix. Markers via `--skip` kiezen grote output of een direct child dat eerder terugkeert; drop van een `std::process::Child` beëindigt het subprocess niet automatisch. Oude PID-bestanden worden vooraf verwijderd; scenario's hebben verschillende namen met het bovenliggende test-PID. `join!` laat uitvoering en signaalgever voortgang maken totdat een geldig PID beschikbaar is.

De zeven opruimscenario’s en de TCP-tests delen `process_checks::assert_stopped`. Windows gebruikt sinds de review OpenProcess en WaitForSingleObject met timeout 0; er wordt geen extern statusproces gestart. OwnedHandle sluit het handle ook bij fouten. Maximaal 5 s wordt elke 50 ms gecontroleerd op beëindiging. API-/toegangsfouten blijven fouten. De Windows-only dev-dependency windows-sys is al transitief aanwezig en wordt nu rechtstreeks voor tests gebruikt. Linux heeft nu een `/proc/<pid>/stat`-controle; ontbrekende processen en zombies gelden als beëindigd. Deze platformtak is lokaal niet uitgevoerd; de geslaagde Ubuntu-CI controleert hem daarna.

## Historische verificatie — stand 2026-11-02

| Controle | Resultaat |
| --- | --- |
| `cargo fmt --check` | Geslaagd op Windows |
| `cargo clippy --all-targets --locked -- -D warnings` | Geslaagd op Windows |
| `cargo test --locked` na ESRCH-correctie | 62 passed, 0 failed, 4 ignored, 0 filtered out; 5,78 s op Windows |
| `cargo build --locked` | Geslaagd op Windows |
| Echte TCP-concurrency | Vier bevestigde actieve procesbomen, vijfde request 503 binnen testbewaking; na vrijgave vier HTTP 200-responses en succesvolle nieuwe uitvoering |
| Actieve HTTP-shutdown | Hetzelfde afsluitpad als Ctrl+C, geïnjecteerd signaal, client 503, server afgerond, child/subprocess gestopt en geen nieuwe TCP-verbinding |
| Logging | Succes/niet-nul exit en onbekende tool gecontroleerd; argumentwaarden, clientwaarde en procesoutput ontbreken in testlogs |
| Ubuntu/Windows CI | #20 beide groen; #21 Ubuntu-fout door ESRCH; na correctie #22 beide platformjobs Success. Exacte testtotalen van #22 niet zichtbaar |

De twee nieuwe HTTP-helpers verklaren de stijging van 2 naar 4 genegeerde functies. Ze worden expliciet als child gestart en zijn geen overgeslagen acceptatietests. De TCP-testclient gebruikt HTTP/1.0 met connection-close; alle requests lopen over echte sockets. PID-bestanden bevestigen dat processen gestart zijn vóór capaciteit/shutdown wordt beoordeeld. De tests controleren zowel direct child als subprocess.

De OS-signaalbron zelf is niet tijdens actieve uitvoering geautomatiseerd aangeroepen: de integratietest gebruikt een kanaal om hetzelfde `server::serve`-afsluitpad te activeren. Eerder is echte Ctrl+C zonder actieve tools handmatig gecontroleerd. Op 2026-10-30 is tevens een echte POST naar `git_version` vastgelegd (succes, exitcode 0, 61 ms).

De bestaande CI-matrix bevat Ubuntu en Windows en voert formatter, Clippy, tests en build uit. Er is een jobtimeout van 15 minuten toegevoegd; de vastgelegde push-run #20 is op beide platforms geslaagd. Er zijn bij deze documentatie-update geen Git-commando's uitgevoerd. Review/merge zijn nog niet bevestigd.

## Problemen en oplossingen

- De eerste subprocess-timeouttest controleerde alleen een foutuitkomst. PID-bestanden en processtatuscontrole maken opruiming nu controleerbaar.
- Een overgebleven ongebruikt `oneshot::channel()` in de shutdowntest gaf E0282: zonder gebruik kon Rust het berichttype niet afleiden. Het overbodige kanaal is verwijderd.
- State zonder `.clone()` aan de router doorgeven gaf E0382 bij later shutdowngebruik. Gedeelde verwijzingen klonen behoudt beide eigenaars.
- Blokkerend `tasklist` en één momentopname zijn vervangen door async begrensd pollen. `from_utf8_lossy` voorkomt fouten door procesnamen met niet-UTF-8-bytes.
- Testcode maakte productiebestanden lang. Verplaatsing naar `src/tests/` houdt code gescheiden met behoud van private toegang en helperpaden.
- Windows-formatter-/lintmeldingen zijn opgelost: overbodige haakjes, ongebruikte lint-expectation en samen te voegen voorwaarden. De zombie-lint bij de helper met vroegtijdige parent-exit heeft een expliciete `expect` met reden: het bewust achterlaten is precies het scenario dat de test moet opvangen. De geconfigureerde Linux-opruimscenario's zijn daarna via Ubuntu-CI geslaagd.

## Werksessies

### 2026-10-26 — F03 gestart

- Doel: F03 voorbereiden en uitvoeringsafspraken vastleggen.
- Gewijzigd: gedeelde uitvoeringsresultaten en fouttypen, parameterloze toolselectie, command-opbouw, concurrency- en outputbudgethelpers, en vijf unit tests. Documentatie bijgewerkt; branch `feature/F03-procesuitvoering` actief.
- Geleerd: procesboomopruiming moet expliciet per platform worden gerealiseerd en getest; een timeouterror alleen bewijst niet dat subprocessen zijn beëindigd.
- Gecontroleerd: vijf gerichte unit tests zijn als geslaagd gerapporteerd. De complete procesuitvoering is nog niet aanwezig.
- Open vragen: issue-referentie vastleggen; procescoördinatie, timeout, outputstream-concurrency, procesboomimplementatie en tests op Windows/Linux uitvoeren.

### 2026-10-27 — Eerste procesbouwstenen en iteratieve leerwerkwijze

- Doel: de processtart en het gezamenlijk begrensd lezen van stdout en stderr stapsgewijs opbouwen en testen.
- Gewijzigd: `start_process` reserveert capaciteit en start de geconfigureerde tool; `create_command` gebruikt `kill_on_drop(true)`; `read_process_output` leest beide pipes tegelijk met het gedeelde budget. Tests controleren de capaciteitsgrens, vrijgave bij startfout, een succesvolle `git --version`-run en het gedeelde outputbudget.
- Geleerd: `Option::take()` haalt een pipe uit `Child`; als de test die pipe al heeft weggehaald, kan de helper hem niet opnieuw ophalen. De test hoort de pipes daarom aan de helper over te laten. Ook is een geslaagde read-helper nog geen volledige procescoördinatie.
- Gecontroleerd: de bovenstaande gerichte tests zijn als geslaagd gerapporteerd. Geen volledige suite of Clippy/formattercontrole gemeld.
- Werkwijze: eerst een kleine werkende verticale slice maken, die gericht verifiëren en daarna uitbreiden tot de acceptatiecriteria. Code in korte fragmenten aanbieden met bestand/plek en syntax-uitleg; na het invoeren volgt een gerichte controle voordat de volgende stap begint. Documentatie alleen aanpassen op verzoek.
- Open: resultaatcoördinatie, timeout, echte outputoverschrijding met opruiming, annulering, procesboomgedrag en HTTP-route. Zie ook de [projectbrede leerwerkwijze](../AGENTS.md#leerbegeleiding).

### 2026-10-27 — Resultaat, timeout en outputlimiet

- Doel: de verticale slice uitbreiden met een bruikbaar `ExecutionResult` en voorspelbare foutuitkomsten.
- Gewijzigd: `execute_parameterless` coördineert toolselectie, start, gelijktijdig begrensd lezen, wachten en resultaatvorming. Timeout wordt uit de toolconfiguratie gelezen met 10 seconden als standaard. `stop_child` probeert bij timeout en leesfout het directe proces te beëindigen. Een test gebruikt de Rust-testbinary als child voor output boven de limiet.
- Geleerd: een niet-nul exitcode is een normaal `ExecutionResult` met `success: false`, niet een processtartfout. Een gerichte test van outputoverschrijding moet aantonen dat de bedoelde childtest daadwerkelijk draait; een verkeerde `--exact`-filter resulteerde eerst in nul childtests en alleen output van het testharnas.
- Gecontroleerd: de tests voor succesvolle uitvoering, niet-nul exitcode, timeout (inclusief vrijgekomen capaciteit) en outputoverschrijding door een childproces (inclusief vrijgekomen capaciteit) slagen. Geen volledige Cargo-controles gemeld.
- Procesbeheer: onderzocht is [`process-wrap`](https://docs.rs/process-wrap/), opvolger van `command-group`, met Tokio-ondersteuning, Unix `ProcessGroup` en Windows `JobObject`. De geraadpleegde documentatie noemt Rust 1.87 als MSRV en MIT/Apache-2.0 als licenties. De dependency is nog niet toegevoegd of geïntegreerd; MSRV en gedrag bij een subprocess moeten eerst worden gecontroleerd. De huidige code stopt alleen het directe childproces.
- Open: bevestig cratekeuze na controle van API/features en MSRV; integreer procesboomopruiming en test met child én subprocess op Windows/Linux; werk annulering/shutdown af; daarna HTTP-route en integratietests.

### 2026-10-29 — Platformwrapper aangesloten

- Doel: de bestaande executor via `process-wrap` laten starten en beheren.
- Gewijzigd: dependency `process-wrap` 10.0.1 met feature `tokio1`; Unix gebruikt `ProcessGroup::leader()`, Windows `JobObject`, en `KillOnDrop` is toegevoegd. `start_process`, outputlezen, wachten en `stop_child` gebruiken `ChildWrapper`.
- Gecontroleerd: voor `cargo test --locked execution::tests` is het volgende resultaat gerapporteerd: 13 passed, 1 ignored en 14 filtered. De wrapper-smoketest start `git --version` en wacht op succes.
- Beperking: de smoke-test bewijst geen beëindiging van subprocessen; de test is niet op beide platforms aangetoond. Volledige formatter-, Clippy- en testsuitecontroles zijn niet gemeld.
- Open: controleren van de `try_wait()`-afhandeling in `stop_child`; een proces met eigen subprocess testen bij timeout/outputoverschrijding op Windows en Linux; daarna annulering/shutdown en HTTP-route.

### 2026-10-29 — Codecontrole en documentatie bijgewerkt

- Op verzoek uitsluitend documentatie aangepast; de bestaande wijzigingen in `src/execution.rs` en `src/execution/tests.rs` zijn ongemoeid gelaten.
- Vastgesteld: `stop_child` gebruikt nu `try_wait().is_none()`; tests voor een al afgerond child en timeout met een subprocess zijn toegevoegd en slagen op Windows.
- Geleerd: `Option<ExitStatus>` beschrijft de status van het child; een afgerond child bewijst niet dat alle subprocessen weg zijn. Ook bewijst een geslaagde timeouttest zonder statuscontrole achteraf geen volledige opruiming.
- Gecontroleerd: volledige testsuite 29 passed, 2 ignored; formatter en Clippy uitgevoerd maar afgekeurd, zoals hierboven vastgelegd. F03 blijft In uitvoering.

### 2026-10-29 — Procesboombewijs en expliciete annulering

- Werkwijze: kleine codefragmenten met bestandslocatie en Rust-uitleg, gevolgd door invoeren en een gerichte controle per stap. Bestanden lezen is toegestaan; wijzigingen en andere uitvoering alleen op expliciet verzoek. Geen Git-commando's.
- Toegevoegd: PID-bestanden, gedeelde Windows-proceslijsthelper, subprocessscenario's voor timeout/outputlimiet/eerder afgerond child, capaciteitscontroles, expliciete `oneshot`-annulering met `select!`, en drie annuleringstests.
- Geleerd: lenen met `&`, automatisch vrijgeven van permits bij drop, `Option`/`Result`-patronen, een kanaal sluiten tegenover een bericht versturen, en gelijktijdige voortgang via `select!` en `join!`. Een gestarte async future is geen afzonderlijk gespawnde taak; `join!` pollt beide futures in dezelfde taak.
- Verificatie: 20 geslaagde execution-tests gerapporteerd, naast 2 genegeerde helpers. Code gelezen; Linux en volledige Cargo-controles niet opnieuw uitgevoerd.
- Overdracht: volgende werksessie serverstate en Ctrl+C-handler lezen, daarna serverbrede annulering en afgewachte opruiming ontwerpen. F03 blijft In uitvoering. Op expliciet verzoek alleen documentatie bijgewerkt; geen commit uitgevoerd.

### 2026-10-30 — Shutdown, HTTP-route en teststructuur

- Toegevoegd: shutdown met `watch`, toelatingscontrole, wachten op capaciteitsvrijgave, gedeelde `AppState` en Ctrl+C-koppeling.
- Getest: shutdownafwijzing, wachten op een vastgehouden permit, subprocessopruiming bij shutdown, future-drop en taakafbreking op Windows.
- Verbeterd: Windows-PID-controles async en begrensd, met herhaalde controle in plaats van één momentopname.
- Toegevoegd: parameterloze POST-route, resultaat-/foutmodellen, uitsluitend leeg JSON-object, bodylimiet en foutmapping met routertests.
- Gecontroleerd: echte HTTP-aanroep geeft Git-versie, succes, exitcode 0 en 61 ms; volledige suite 55 geslaagd en 2 genegeerde helpers.
- Geordend: alle testmodules naar `src/tests/` verplaatst; volledige suite opnieuw geslaagd.
- Overdracht: eerst logging en echte HTTP-concurrency, daarna Windows-eindcontroles en documentatie, als laatste Linux. F03 blijft In uitvoering; geen commit/merge uitgevoerd.

### 2026-11-02 — Resterende F03-code en Windows-eindcontrole

- Expliciet toegestaan: resterende F03-codewijzigingen, controles en algemene documentatie zelfstandig uitvoeren. Geen Git-commando's.
- Loggingwrapper en foutcategorieën nagelezen. De isolatiehelper is aan het begin van beide loggingtests aangesloten en een dubbele aanroep is verwijderd. De gewone parallelle suite isoleert de tijdelijke subscribers nu per childproces.
- `server::serve` toegevoegd: de signaalbron wordt geïnjecteerd, terwijl de afsluitvolgorde gelijk blijft aan de productiecode met Ctrl+C. Rusts generieke `impl Future<Output = ()>` laat een kanaal in tests toe zonder HTTP in de uitvoeringskern te brengen.
- Echte TCP-concurrency met vier actieve child/subprocess-paren toegevoegd, vijfde request 503 en succesvolle uitvoering na vrijgave. Nieuwe subprocess-helpers hebben startbevestiging en een vrijgavesignaal; geen willekeurige wachttijd om capaciteit te veronderstellen.
- Actieve HTTP-shutdown getest: 503-response, afgewachte server, beide PID's gestopt en geen geaccepteerde nieuwe verbinding. Op Windows kan een verbinding naar een gesloten poort eerst timen-out; de test faalt alleen bij een werkelijk geslaagde verbinding.
- Gedeelde proceschecks ingevoerd. Lokale broncontrole van `process-wrap` wees uit dat Tokio kill-on-drop op Unix alleen de directe child stopt; `ProcessGuard` vraagt daar bij drop ook groepsbeëindiging. `stop_child` probeert op Unix de groep ook na parent-exit te stoppen. Windows behoudt JobObject-gedrag.
- Linux-`/proc`-controle en een Linux-specifieke controle van de eigen PID toegevoegd. Zombies zijn niet meer actief, maar kunnen nog op reaping wachten. Linux is niet uitgevoerd; `wsl --list --quiet` meldt niet geïnstalleerd en de toolchain heeft alleen de Windows-target.
- Formatter toegepast; Clippy met warnings als errors, volledige tests en build geslaagd op Windows. Laatste resultaat: 59 geslaagd, 4 genegeerde helpers. Ubuntu/Windows-workflow nagelezen en jobtimeout toegevoegd.
- Overdracht: voer Ubuntu-CI uit zodra repositorywijzigingen via de normale werkwijze beschikbaar zijn. Platformfouten oplossen en resultaten vastleggen voordat F03 naar Review/Klaar kan.

### 2026-11-02 — CI-platformbewijs en overdracht naar review

- Bewijs: GitHub Actions-screenshot van run #20, getriggerd door een push op `feature/F03-procesuitvoering`. Runstatus Success; `Checks (ubuntu-latest)` en `Checks (windows-latest)` zijn groen.
- Zichtbare duur: Ubuntu 51 s, Windows 1 m 45 s, totale run 1 m 48 s. Geen exacte testtotalen uit de screenshot afgeleid.
- De lokale workflow voert formatter, Clippy met warnings als errors, tests en build uit. Door de geslaagde Ubuntu-job zijn de Linux-specifieke status-/procesboomtests en compilatie via CI aangetoond.
- Niet-blokkerende annotaties over Node.js 20 bij actions zijn zichtbaar. De volledige annotatietekst is niet vastgelegd; action-onderhoud staat los van een mislukte test, want beide jobs zijn geslaagd.
- Linux-todo en platformacceptatie afgevinkt. Status naar Review; geen bewijs van review of merge, dus nog niet Klaar.
- Alleen algemene documentatie aangepast. Geen Rust-/workflowwijzigingen, tests of Git-commando's uitgevoerd.

### 2026-11-02 — Racecorrectie na CI-run #21

- Bewijs: Ubuntu faalt in `stopt_subproces_bij_annulering` bij lezen van `/proc/<pid>/stat`: ESRCH (os error 3). Het proces is al verdwenen; de helper herkende alleen NotFound als afwezig. Dit is geen bewijs van een nog draaiend subprocess.
- Oorzaak: het proces kan tussen openen en uitlezen verdwijnen. De eerdere groene #20-run kon deze timinggevoelige fout missen.
- Correctie: pure afhandeling van `io::Result<String>` afgesplitst van het echte /proc-lezen. Zowel NotFound/ENOENT als ESRCH betekenen beëindigd; andere leesfouten en ongeldige data blijven fouten. Geen extra wachttijd en geen verwijderde procesasserties.
- Drie regressietests toegevoegd voor verdwijnen tijdens lezen, actieve/zombie/exitstatus (ook met haakjes in de procesnaam) en behoud van toegangs-/formaatfouten.
- Verificatie op Windows: 3 gerichte regressietests, formatter, Clippy, build en volledige suite geslaagd; totaal 62 passed, 4 ignored, 5,78 s. De Linux-specifieke leesroute kan lokaal niet worden uitgevoerd.
- Status terug naar In uitvoering; Linux-acceptatie weer open totdat een nieuwe Ubuntu-run de correctie bevestigt. Historische run #20 blijft als eerder bewijs bewaard, maar is geen actuele stabiele eindacceptatie.
- Code en documentatie aangepast; geen Git-commando's uitgevoerd.

### 2026-11-02 — ESRCH-correctie bevestigd in CI-run #22

- Bewijs: screenshot van run #22 op `feature/F03-procesuitvoering`. Ubuntu en Windows hebben status Success.
- Zichtbare duur: Ubuntu 45 s, Windows 2 m 51 s, totaal 2 m 55 s. Geen exacte testtotalen uit de screenshot afgeleid.
- De nieuwe run bevestigt de eerder open Linux-hercontrole na de gerichte ESRCH-correctie en drie regressietests. Status terug naar Review; de eerdere fout uit #21 blijft als historie bewaard.
- De screenshot toont Node.js-verouderingswaarschuwingen en een aankondiging over migratie van het ubuntu-latest-label. Die annotaties verhinderen geen geslaagde run; onderhoud kan afzonderlijk worden beoordeeld.
- Alleen algemene documentatie aangepast. Geen code, tests of Git-commando's uitgevoerd. Review en merge zijn niet aangetoond.

## Vervolg

Commit en push de documentatie-update, controleer de laatste CI en bevestig daarna review en merge naar develop; de afrondingsmelding voor F03 was op dat moment nog niet gekoppeld aan vastgelegd mergebewijs. Daarna volgt F04. Git-acties voer ik afzonderlijk zelf uit.

### 2026-11-03 — Review en opruiming na F03

- Opdracht: alle reviewpunten verwerken; geen Git-commando’s.
- Geconstateerd: eerste Windows-run 62 passed en 4 ignored; later parallel 10 timingfouten, serieel 2 TCP-fouten op tasklist. De exacte oorzaak van de wisselende externe statusaanroepen is niet vastgesteld.
- Gewijzigd: Windows-processtatus rechtstreeks via OpenProcess/WaitForSingleObject met timeout 0. Geen externe statusprocessen en geen ruimere testtimeout. API-/toegangsfouten blijven fouten; procesbeëindiging wordt maximaal 5 s gecontroleerd.
- Rust: de kleine unsafe-blokken bevatten alleen Windows-FFI-aanroepen en het overnemen van een geldig handle. OwnedHandle sluit het handle automatisch, ook bij vroeg terugkeren. windows-sys 0.61.2 is uitsluitend een Windows-dev-dependency; de lockfile is offline bijgewerkt.
- Rust: ToolRegistry::find geeft een geleende tool terug; logging en uitvoering gebruiken dezelfde lookup. cancellation is daadwerkelijk gebruikt en heeft daarom geen underscoreprefix. Een async-blok maakt een future; select! pollt die in dezelfde taak.
- Refactor: zeven subprocessdefinities en herhaalde PID-/start-/capaciteitscontroles delen helpers. Alle permits blijven tijdens de capaciteitsassertie tegelijk gereserveerd. Routertests delen POST-opbouw en JSON-uitlezen; hun inhoudelijke assertions blijven behouden.
- Naamgeving: 51 test-/helpernamen zijn eerder in deze review naar Engels omgezet, inclusief de exacte child- en loggingselecties.
- Configuratie: lege namen en commands (ook alleen witruimte) worden afgewezen. Geen automatische normalisatie of uitbreiding naar F04.
- Documentatie: actuele beschrijvingen, F02-placeholdercriterium en dubbele checkbox, plan van aanpak, testplan, voortgang en helpervoorbeelden gecorrigeerd. Historische testresultaten blijven herkenbaar als historie.
- Gerichte tests: 14 configuratietests en 4 processtatustests geslaagd op Windows. Eerste volledige suite na refactor: 64 passed, 0 failed, 4 ignored, 3,24 s. Formatter/Clippy geslaagd.
- Bronnen voor de Windows-API: [OpenProcess](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-openprocess) en [WaitForSingleObject](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-waitforsingleobject).
- Beperking: Linux en nieuwe CI zijn niet uitgevoerd. Het oude run-#22-bewijs geldt voor de eerdere stand. Actuele externe links/GitHub-mergestatus zijn niet opnieuw gecontroleerd. Geen Git-commando’s uitgevoerd.

Eindcontrole (2026-11-03): cargo fmt --check en cargo clippy --all-targets --locked -- -D warnings slagen. cargo test --locked: 64 passed, 0 failed, 4 ignored, 3,00 s; ook de eerdere parallelle run na correctie slaagde (3,24 s). De gewone cargo build --locked kon target/debug/commandhub.exe niet vervangen (toegang geweigerd); proces 15952 draaide vanaf dat pad. De bestaande server is niet gestopt. De alternatieve build cargo build --locked --offline --target-dir target/review-build is daarna geslaagd (1 m 57 s). De nieuwe executable staat in target/review-build/debug/commandhub.exe; de draaiende server gebruikt nog de eerdere binary.

## CI-bevestiging refactor — 2026-11-03

De screenshot toont een geslaagde push-run voor commit `105e69b` op `refactor/na-feature-3`:

- Runstatus: Success; totale duur 1 m 50 s.
- Checks (ubuntu-latest): Success, 42 s.
- Checks (windows-latest): Success, 1 m 47 s.
- De lokale workflow voert formatter, Clippy met warnings als errors, tests en build uit. Exacte CI-testtotalen zijn niet zichtbaar en worden niet uit de screenshot afgeleid.
- Twee Node.js 20-verouderingswaarschuwingen bij actions/checkout@v4 en een Ubuntu-labelmelding zijn zichtbaar; de jobs zijn geslaagd. Workflowonderhoud blijft een afzonderlijk punt.
- Hiermee is de eerder open Linux-/CI-controle van de refactor bevestigd. Review/merge is niet zichtbaar en blijft te bevestigen. Deze documentatie-update volgt na de getoonde codecommit; controleer na push de CI van de laatste branchstand.
- Alleen documentatie aangepast op basis van het vastgelegde bewijs; geen tests opnieuw uitgevoerd en geen Git-commando’s uitgevoerd.

### 2026-11-03 — Overdracht naar F07

Ik ben na de refactor terug op develop gegaan en heb daarna feature/macos-ondersteuning aangemaakt. De volgende actieve uitbreiding is [F07](F07-macos-ondersteuning.md), vóór F04. Die gebruikt dezelfde Unix-executor; macOS-verificatie moet nog via Actions gebeuren. De eerdere CI-resultaten in deze notitie blijven historie van F03/de refactor.


## Opleverreview op 2026-11-11

De bestaande featurechecklist bevestigt de merge van F03. De actuele productie-ingang is execute_with_parameters_with_cancellation met gedeelde parameteropbouw. Parameterloze methoden uit deze notitie zijn historische F03-beschrijvingen. MCP en macOS zijn vervolgens toegevoegd.

De actuele requirementdekking staat in het [testplan](../docs/testplan.md). De [F06-notitie](F06-oplevering.md) bevat de resterende opleveringsstappen. De oorspronkelijke resultaten hierboven behouden hun datum en scope.
