# Architectuur

Bijgewerkt op 2026-11-11 voor F06. REST en MCP gebruiken dezelfde configuratie, registry, schemaopbouw, validatie en uitvoeringsservice. Ik heb bevestigd dat de demo met Claude Code werkt. Het testbewijs staat in het [testplan](testplan.md).

## Opstarten

main initialiseert tracing met RUST_LOG, laadt het JSON-bestand via config::load en valideert het vóór het openen van de listener. COMMANDHUB_CONFIG kiest het bestand. De standaard is config/tools.json, relatief aan de serverwerkmap. De gevalideerde tools gaan naar ToolRegistry. Daarna bevat AppState gedeelde Arc-verwijzingen naar de registry en één ExecutionService.

De listener bindt aan 127.0.0.1:3000. server::serve bouwt de router en koppelt Ctrl+C aan het gedeelde afsluitpad. Een configuratiewijziging wordt actief na herstart van dezelfde executable.

## Modules

| Bestand | Verantwoordelijkheid |
| --- | --- |
| [main.rs](../src/main.rs) | Logging, configuratie laden, state bouwen, listener en Ctrl+C |
| [config.rs](../src/config.rs) | Serde-types, JSON laden en startupvalidatie |
| [registry.rs](../src/registry.rs) | Tooldefinities opzoeken en gedeeld inputSchema opbouwen |
| [app_state.rs](../src/app_state.rs) | Arc-verwijzingen naar registry en uitvoeringsservice |
| [validation.rs](../src/validation.rs) | Parameters controleren, paden canonicaliseren en argumenten opbouwen |
| [execution.rs](../src/execution.rs) | Processtart, capaciteit, outputbudget, timeout, annulering, opruimen en uitvoeringslogging |
| [routes.rs](../src/routes.rs) | REST-routes, POST-bodylimiet en MCP-service aansluiten |
| [controllers](../src/controllers) en [models](../src/models) | HTTP-extractie, statuscodes en serialiseerbare responses |
| [mcp.rs](../src/mcp.rs) | rmcp-handler, transportinstellingen en MCP-resultaatmapping |
| [server.rs](../src/server.rs) | Axum-server en gecoördineerde shutdown |
| [personal_tools.py](../tools/personal_tools.py) | Geconfigureerde bestands-, notitie-, media- en diagnosetools |

## Gegevensstroom

Een REST- of MCP-call gaat via de adapter naar execute_with_parameters_with_cancellation. De service zoekt de tool eenmaal op en gebruikt die definitie voor logging en uitvoering. build_arguments valideert de invoer en vervangt volledige placeholders vóór processtart. Het resultaat gaat via de adapter terug naar de client.

De registry en executor zijn gedeeld tussen requests en MCP-sessies. Arc::clone maakt een extra gedeelde verwijzing. De registry wordt na startup gelezen. input_schema levert voor beide adapters properties, required, enum en additionalProperties met waarde false.

De kern gebruikt serde_json als invoermodel en eigen ExecutionResult/ExecutionError-types. Axum-statuscodes en MCP-responsemodellen worden door de adapters bepaald. De HTTP-ontdekcontroller ontvangt via FromRef uitsluitend de registry.

## Parameters en paden

Iedere placeholder beslaat één argument. De kern bouwt een Vec<String> en geeft een geleende argumentenslice aan Command::args. De executable komt uit de registry. Spaties, quotes en shelltekens binnen een waarde blijven deel van hetzelfde argument.

Alleen stringparameters zijn ondersteund. allowed_values vergelijkt exact. Vrije strings hebben prefixbeperkingen. allowed_roots vereist een bestaand absoluut clientpad en bestaande absolute directories als roots. Pad en roots worden gecanonicaliseerd en per component vergeleken. Het canonieke pad wordt het procesargument. De configuratie staat één optionele placeholder als laatste argument toe, met beperkingen op vaste opties. Het volledige contract staat in [API en configuratie](api-en-configuratie.md).

## Procesuitvoering

ExecutionService deelt vier uitvoeringsplekken met een semaphore. try_acquire_owned wijst volle capaciteit direct af. De permit blijft bij de uitvoeringsfuture en komt bij drop vrij. Bij startfout komt de gereserveerde capaciteit eveneens vrij.

Command::new en args houden programma en argumenten afzonderlijk. Stdin is gesloten. Stdout en stderr worden met try_join! gelijktijdig gelezen. Beide readers reserveren bytes uit één budget van 1 MiB per uitvoering. CapturedOutput bewaart de budgetpermits zolang de verzamelde bytes aanwezig zijn. Overschrijding wordt een fout en activeert procesopruiming. De response zet bytes met UTF-8-vervanging om naar tekst.

De timeout is standaard 10.000 ms en configureerbaar van 1 tot 60.000 ms. tokio::select! wacht op uitvoering, een expliciet oneshot-bericht of een wijziging van de shutdownstatus. Een gesloten oneshot-zender zonder bericht laat de uitvoering doorlopen.

process-wrap 10.0.1 gebruikt JobObject op Windows en ProcessGroup op Unix. KillOnDrop en de Unix ProcessGuard vragen ook bij future-drop of taakafbreking beëindiging aan. Bij gecontroleerde stop probeert stop_child de procesboom op te ruimen. Unix signaleert de groep ook na afloop van de leider. ESRCH geldt daar als een reeds verdwenen groep. Een opruimfout kan de oorspronkelijke timeout- of annuleringsfout vervangen.

## Shutdown

Elke uitvoering abonneert zich vóór processtart op de watch-status. De toelatingscontrole houdt een leesbewaker vast tijdens capaciteitsreservering. begin_shutdown gebruikt send_replace(true), waardoor de status ook zonder actieve ontvanger behouden blijft.

server::serve signaleert eerst uitvoeringsshutdown en annuleert daarna het MCP-transporttoken. Axum sluit graceful af. Vervolgens wacht execution.shutdown op alle vier permits. Dat toont dat uitvoeringsfutures zijn afgerond. Processtatuscontroles in de tests leveren het aanvullende PID-bewijs.

MCP-contextannulering wordt door de adapter naar een oneshot-bericht vertaald. Een onverwacht clientdisconnect heeft nog geen afzonderlijk vastgelegd annuleringsbeleid. De timeout en server-shutdown blijven beschikbaar als begrenzing.

## Transporten

REST biedt GET /status, GET /tools en POST /tools/{toolname}. De POST-extractor begrenst de body op 16 KiB, vereist een JSON-object en vertaalt kernfouten naar vaste HTTP-statuscodes en foutcodes.

MCP gebruikt rmcp 3.6.0 met Streamable HTTP op /mcp en een gedeelde LocalSessionManager. legacy_session_mode is ingeschakeld. De getoetste protocolonderhandeling levert 2025-11-25. De MCP-service heeft eigen Host/Origin-controles en een bodylimiet van 16 KiB. Resultaten staan in structuredContent en als JSON-tekst in content. Details staan in het [API-contract](api-en-configuratie.md).

## Geconfigureerde Python-tools

De platformprofielen starten tools/personal_tools.py via een vaste interpreter en vaste actie. Deze executable gebruikt dezelfde uitvoeringsgrenzen als andere tools. Eigen Python-validatie beperkt relatieve bestandsnamen tot CommandHub-data of COMMANDHUB_DATA_DIR. Dit beleid staat naast het absolute allowed_roots-beleid van de Rust-kern.

Git- en projecttools gebruiken de serverwerkmap. Persoonlijke bestandstools gebruiken de gegevensmap. De server moet daarom vanuit de projecthoofdmap worden gestart. Afhankelijkheden en uitvoergrenzen staan in de [configuratiehandleiding](../config/README.md).

## Tests en platformcontrole

Testcode staat in src/tests en is via cfg(test) en path gekoppeld als submodule van de oorspronkelijke module. De router-wrapper voor tests staat in routes_tests.rs en wordt onder cfg(test) geherexporteerd. Productie gebruikt router_with_shutdown.

Processtatuschecks gebruiken Windows-handles, Linux /proc/<pid>/stat of macOS proc_pidinfo. De helper controleert maximaal vijf seconden met intervallen van 50 ms. Verdwenen processen en zombies tellen als beëindigd. API-, toegangs- en formaatfouten blijven testfouten.

Routertests gebruiken Axum oneshot. TCP-tests starten echte loopbackservers, bevestigen processtart met PID-bestanden en controleren capaciteit, shutdown en herstel. De MCP-shutdownclient gebruikt NeverRetry en schakelt sessieherinitialisatie uit om streamsluiting begrensd waar te nemen. De CI-matrix bouwt en test op Windows, Ubuntu en macOS en voert een Inspector-basistest uit vanaf schone checkout.

Historische ontwerpstappen en meetresultaten staan in de [featurenotities](../notities/README.md). Resterende grenzen staan in [security](security.md).
