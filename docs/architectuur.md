# Architectuur

Status: status, toolontdekking en parameterloze uitvoering via HTTP zijn aangesloten. `AppState` deelt registry en uitvoeringsservice. De kern ondersteunt timeout, outputbudget, capaciteit, expliciete annulering en shutdown. De review van 2026-11-03 is lokaal op Windows en via CI op Ubuntu en Windows gecontroleerd (commit 105e69b op refactor/na-feature-3). Windows-tests controleren subprocessopruiming na deze stoproutes, future-drop en Tokio-taakafbreking. Linux-run #20 slaagde; de statusleesrace uit run #21 is daarna gecorrigeerd en in run #22 op Ubuntu/Windows geverifieerd. MCP-integratie volgt in F05. De macOS-uitbreiding F07 is voorbereid; macOS-CI moet de werking nog bevestigen.

## Beoogde gegevensstroom

HTTP-handler of MCP-adapter → gedeelde toolservice → registry → parameter- en beleidsvalidatie → process executor → uitvoeringsresultaat → transportresponse.

Bij het starten: configuratie laden → configuratie valideren → registry bouwen → gedeelde state samenstellen → server starten.

## Onderdelen

| Module | Verantwoordelijkheid |
| --- | --- |
| HTTP/routes en controllers | Requests uitlezen en HTTP-responses maken |
| MCP-adapter | Protocolafhandeling via SDK en vertaling naar gedeelde kern |
| Toolservice | Opzoeken, valideren en uitvoering coördineren |
| Registry | Gevalideerde tooldefinities op naam beschikbaar maken |
| Config loader | JSON lezen en controleren; ongeldige configuratie stopt startup |
| Validation | Parameters en uitvoeringsbeleid controleren |
| Process executor | Proces, uitvoer, tijdslimiet, capaciteit, expliciete annulering en opruimen beheren; subprocessopruiming en shutdown zijn in Windows-tests gecontroleerd; Linux-hercontrole na ESRCH-correctie geslaagd in #22 |
| Logging | Context, uitkomst en duur vastleggen zonder gevoelige inhoud |

## Structuur groeit met de features

De basis bevat src/main.rs, routes.rs, controllers/ en models/. Voeg services/, config/, registry/, validation/ en mcp/ pas toe wanneer daar werkende code voor komt. Vermijd lege lagen zonder functie.

Models voor HTTP/MCP zijn transportmodellen. Gedeelde kernresultaten en fouttypen mogen niet afhankelijk zijn van Axum of HTTP-statuscodes.

## State en concurrency

De server maakt één `AppState` met `Arc<ToolRegistry>` en `Arc<ExecutionService>`. Klonen van die state kopieert gedeelde verwijzingen, geen registries of executors. `FromRef<AppState> for Arc<ToolRegistry>` laat de ontdekcontroller alleen de registry opvragen. De uitvoercontroller gebruikt de volledige state. ToolRegistry::find leent de tooldefinitie; dezelfde lookup levert zowel de veilige loggingnaam als de uitvoeringsdefinitie. Eén gedeelde executor bewaakt serverbreed vier uitvoeringsplekken en wijst volle capaciteit direct af.

`execute_parameterless` delegeert naar de annuleerbare methode. Die wacht met `tokio::select!` op uitvoering met timeout, een expliciet `oneshot`-bericht of een wijziging van het shutdown-`watch`-kanaal. Alleen een expliciet bericht annuleert via `oneshot`; sluiten zonder bericht niet. De shutdownontvanger wordt vóór processtart aangemaakt om een wijziging tijdens toelating niet te missen.

`try_acquire` controleert de shutdownstatus en houdt de leesbewaker vast tijdens capaciteitsreservering. `begin_shutdown` bewaart `true` via `send_replace`, ook zonder ontvangers. Nieuwe uitvoeringen krijgen daarna `Cancelled`. `shutdown().await` signaleert shutdown en wacht op alle vier permits. Dat bewijst afronding van de uitvoeringsfutures en hun opruimpogingen, maar verzamelt geen opruimfouten en is op zichzelf geen processtatuscontrole.

`main.rs` geeft de Ctrl+C-future aan `server::serve`. Die signaleert eerst `begin_shutdown`, laat Axum graceful afsluiten en wacht daarna op `shutdown().await` voordat een serverfout wordt doorgegeven. De signaalbron kan in TCP-tests door een kanaal worden vervangen; productie en test gebruiken hetzelfde serverafsluitpad. Normaal stoppen verloopt via expliciete annulering of shutdown. Future-drop en `JoinHandle::abort` gebruiken `KillOnDrop` als vangnet. Omdat die Tokio-vlag op Unix alleen de directe child stopt, bewaart `ProcessGuard` de wrapper en vraagt bij drop een groepssignaal via `start_kill` aan. Op Unix probeert `stop_child` de groep ook te beëindigen nadat de leider is afgerond; ESRCH betekent dat de groep al verdwenen is. Bij drop kan geen normaal uitvoeringsresultaat of afgewachte opruimfout worden teruggegeven. Dit vangnet wordt door de tests op beide CI-platforms gecontroleerd; run #20 toont geslaagde Ubuntu- en Windows-jobs.

## HTTP-adapter en tests

De uitvoercontroller vangt JSON-extractiefouten op, accepteert uitsluitend een leeg object en kiest daarna de tool uit de registry. `DefaultBodyLimit` begrenst deze POST-route op 16 KiB vóór JSON-deserialisatie. `ExecutionResponse::from` vertaalt het kernresultaat naar JSON met `exitCode` en milliseconden. De kern kent geen HTTP-statuscodes; de controller bepaalt foutmapping en vaste teksten zonder OS-details.

Tests staan fysiek in `src/tests/`, maar blijven via `#[cfg(test)]` en `#[path = "..."]` submodules van hun oorspronkelijke module. Alle test- en helperfuncties hebben Engelse namen. De subprocessscenario’s delen helpers voor configuratie, PID-bestanden, startbevestiging en capaciteitscontrole; de routetests delen POST-opbouw en JSON-uitlezen. Daardoor blijven private toegang, `super`-imports en helperselecties zoals `execution::tests::subprocess_timeout_helper` geldig. Dit zijn geen zelfstandige integratiecrates onder een projectmap `tests/`.

## Fouten

Maak onderscheid tussen ongeldige invoer, onbekende tool, capaciteitsgebrek, startfout, niet-succesvolle exitcode, timeout, outputlimiet, annulering en opruimfout. HTTP vertaalt dezelfde kernfout naar vaste statuscodes en JSON-foutcodes. Annulering/shutdown wordt HTTP 503; interne procesbeheerfouten worden 500. MCP-mapping volgt in F05. Clientdisconnects zijn nog niet als expliciet annuleringsbeleid gekoppeld.

## Platformbeleid

Cross-platform code betekent niet dat elke executable op ieder OS bestaat. Lever platformgeschikte configuratievoorbeelden. [`process-wrap`](https://docs.rs/process-wrap/) 10.0.1 gebruikt Tokio `ProcessGroup` op Unix en `JobObject` op Windows. De testhelper start `ping` op Windows of `sleep` op Unix. Windows controleert het stoppen rechtstreeks via OpenProcess en WaitForSingleObject met timeout 0. De Windows-only dev-dependency windows-sys levert de API; OwnedHandle sluit het handle automatisch. Er wordt geen tasklist-proces gestart. Ontbreken of een gesignaleerd proceshandle betekent gestopt; overige API-fouten blijven fouten. Linux heeft nu een `/proc/<pid>/stat`-controle; ontbreken en zombie/exitstatus betekenen beëindigd, niet noodzakelijk al gereaped. Windows is lokaal uitgevoerd; de Unix-groepsopruiming en dropbewaker zijn daarna via de geslaagde Ubuntu-CI gecontroleerd. De bron voor het CI-resultaat is de aangeleverde screenshot van run #20.

### macOS-uitbreiding (F07)

macOS gebruikt hetzelfde cfg(unix)-pad met ProcessGroup en de bestaande groepsbewaker. Alleen de teststatuscontrole is OS-specifiek: proc_pidinfo (PROC_PIDTBSDINFO) levert de processtatus rechtstreeks aan. De buffer wordt pas na een volledige API-return uitgelezen. ESRCH en zombie-status betekenen gestopt; API-/toegangsfouten en onbekende statussen blijven fouten. libc is hiervoor uitsluitend een macOS-dev-dependency. Er is geen /proc-bestand of extern statusprogramma nodig.

De CI-matrix omvat nu ubuntu-latest, windows-latest en macos-latest met dezelfde controles. Die macOS-job is nog niet uitgevoerd. Het voorbeeld config/tools.macos.json wordt expliciet gekozen via COMMANDHUB_CONFIG. Zie [F07](features/F07-macos-ondersteuning.md).

## Aanvullende verificatie en logging

`src/tests/http_execution_tests.rs` start echte loopbackservers en TCP-clients. Gecontroleerde tools starten een eigen subprocess en schrijven beide PID's weg; een bestandssignaal geeft ze vrij. Daardoor bewijzen de tests vier gelijktijdige uitvoeringen, een vijfde afwijzing, capaciteitsherstel en shutdown tijdens een actieve procesboom. De minimale testclient gebruikt HTTP/1.0 met connection-close zodat geen chunked decoder of extra dependency nodig is.

`src/tests/process_checks.rs` deelt platformafhankelijke processtatuscontroles. Logging blijft in de kernwrapper: alleen registry-toolnaam (anders `<unknown>`), resultaat, exitcode/foutcategorie en duur. Er worden geen argumenten, executablepaden of stdout/stderr opgenomen. De tijdelijke tracing-subscribers van loggingtests zijn in childprocessen geïsoleerd om parallelle testinteractie te vermijden.
## Linux-statusleesrace

Een proces kan verdwijnen nadat de /proc-statfile is geopend maar voordat lezen klaar is. Naast NotFound/ENOENT kan dan ESRCH optreden. De gedeelde testhelper scheidt het lezen van de classificatie van `io::Result<String>`, zodat die race deterministisch getest kan worden. Afwezigheid betekent beëindigd; toegangs- en formaatfouten blijven fouten. De correctie is op Windows getest met gesimuleerde Linux-leesresultaten; CI-run #22 heeft de echte platformroute daarna op Ubuntu succesvol gecontroleerd.
