---
feature: F05
tags:
  - commandhub
  - feature-notitie
---

# F05 — MCP via HTTP aan de kern koppelen

[Featureplan](../docs/features/F05-mcp-integratie.md)

## Wat bouw ik en waarom?

Ik voeg een MCP-adapter toe waarmee clients de geconfigureerde tools via Streamable HTTP kunnen ontdekken en uitvoeren. REST en MCP gebruiken dezelfde registry, inputschema's, parametervalidatie en uitvoeringsservice. F05 is in uitvoering; de annuleringsketen en eindcontroles zijn nog niet afgerond.

## Hoe werkt de code?

- src/registry.rs bevat input_schema(&ToolDefinition). REST en MCP gebruiken dezelfde schemaopbouw met properties, required, enum en additionalProperties: false.
- src/mcp.rs bevat McpServer met AppState. Clone deelt de bestaande Arc-verwijzingen; er ontstaat geen executor per sessie. get_info gebruikt ServerConfig en kondigt tools aan. list_tools levert de volledige registry zonder vervolgpagina.
- call_tool gebruikt execute_with_parameters_with_cancellation. Ontbrekende arguments worden een leeg object. De gedeelde kern controleert parameters en paden en bewaakt capaciteit, output, timeout en procesbeheer.
- context.ct is het SDK-annuleringssignaal. tokio::select! wacht op uitvoering of annulering. Bij annulering wordt een oneshot-bericht naar de kern gestuurd en de uitvoering afgewacht. De code compileert; het gedrag bij een echte protocolannulering is nog niet getest.
- execution_result levert success, stdout, stderr, exitCode en duration in milliseconden als structuredContent en als JSON-tekst. Een niet-succesvolle procesexit geeft isError: true.
- execution_error vertaalt een onbekende tool naar METHOD_NOT_FOUND. Andere kernfouten worden veilige toolresultaten met isError: true; interne OS-details blijven buiten de response.
- routes.rs bevat router_with_shutdown en koppelt de service aan /mcp. De bestaande router maakt voor routetests een eigen token.
- server.rs maakt het transporttoken en geeft een kloon aan de router. Het afsluitsignaal start eerst de shutdown van de uitvoeringsservice en annuleert daarna het MCP-token. Ook na terugkeer van Axum wordt het token geannuleerd en execution.shutdown afgewacht.

Rust: Arc-klonen delen gegevens; move bewaart state in de servicefactory. Result en ? geven protocolfouten door. into() vertaalt een toolresultaat naar CallToolResponse. tokio::pin! laat dezelfde uitvoeringsfuture door select! en de aansluitende afhandeling gebruiken. CancellationToken is een gedeeld stopsignaal; oneshot blijft het transportonafhankelijke annuleringsmechanisme in de kern.

## Keuzes en grenzen

- rmcp staat exact op 3.6.0. De normale dependencies activeren server en transport-streamable-http-server. De dev-dependency voegt client en transport-streamable-http-client-reqwest toe. tokio-util 0.7 met rt levert CancellationToken.
- De productielistener blijft 127.0.0.1:3000. Host is beperkt tot 127.0.0.1:3000 en localhost:3000. Aanwezige origins moeten overeenkomen met http://127.0.0.1:3000 of http://localhost:3000. Requests zonder Origin zijn toegestaan. Geen remote deployment of authenticatie toegevoegd.
- De MCP-bodylimiet is 16 KiB en wordt door de SDK tijdens lezen afgedwongen.
- legacy_session_mode is true. Routertests initialiseren expliciet met protocol 2025-11-25 en controleren de sessieheader. De SDK behandelt 2026-07-28 zonder protocolsessies. De TCP-ketentest gebruikt standaard SDK-onderhandeling; de werkelijk gekozen protocolversie is niet afzonderlijk vastgelegd en wordt niet afgeleid uit de oudere tests.
- De TCP-ketentest bindt aan een vrije loopbackpoort en past alleen in de test de Host-allowlist aan. Hij gebruikt een eigen Axum-testserver, niet server::serve. Dit is dus geen bewijs voor het volledige productieafsluitpad met een actieve MCP-call.
- Clientdisconnectbeleid, volledige protocolfoutmatrix, gedeelde REST/MCP-belasting en dynamische MCP-parameters vragen nog verificatie. De bestaande grenzen rond padcanonicalisatie en het ontbreken van een sandbox blijven gelden.

## Problemen en oplossingen

Een structliteral met ServerInfo werd afgewezen vanwege non_exhaustive. De instantie wordt nu via default gemaakt en daarna ingesteld. ServerInfo bleek ook een verouderde alias; ik gebruik ServerConfig.

## Verificatie — Windows, 2026-11-05

Ik voer na kleine implementatiestappen gerichte controles uit. Onderstaande resultaten zijn tijdens de werksessie vastgelegd; er is bij deze documentatie-update geen nieuwe controle uitgevoerd.

| Controle | Vastgelegd resultaat |
| --- | --- |
| cargo test --locked routes::tests:: na schema-extractie | 18 passed, 0 failed, 125 filtered out; 0,79 s |
| cargo check en cargo check --locked tijdens de implementatie | Geslaagd; na dependencywijzigingen zonder --locked om de lockfile bij te werken |
| cargo test --locked mcp::tests:: vóór uitbreiding van de ketentests | 9 passed, 0 failed, 143 filtered out; 0,01 s |
| cargo test --locked http_execution_tests:: na shutdownkoppeling | Beide scenario's geslaagd: concurrentielimiet/capaciteitsherstel en shutdown van een actieve HTTP-procesboom; twee functies zijn expliciete proceshelpers |
| cargo test --locked mcp::tests::transport_cancellation_closes_active_mcp_stream | 1 passed, 0 failed, 152 filtered out; 0,06 s |
| cargo test --locked mcp::tests::mcp_client_discovers_and_executes_configured_tool | Eerste run: 1 passed, 0 failed, 154 filtered out; 0,15 s. Na uitbreiding met extra parameters, onbekende tool en geldige vervolgaanroep opnieuw geslaagd |
| cargo check --tests na clientdependency | Geslaagd |
| cargo check --tests --locked na mcp_wait_helper | Geslaagd; dit compileert de helper maar voert hem niet uit |

De mappingtests controleren resultaatvelden, tekst/structuredContent, niet-nul exit, ontbrekende exitcode en veilige foutmapping. Routertests controleren afgewezen/toegestane hosts en origins, initialisatie met sessieheader en de grens 16 KiB versus 16 KiB + 1. De streamtest bewijst dat een open legacy-MCP-stream sluit na tokenannulering.

De echte SDK-client ontdekt via TCP test_echo, vergelijkt metadata en inputschema, voert de testbinary als childproces uit, ontvangt een toolfout bij extra parameters en een fout bij een onbekende tool en kan daarna opnieuw succesvol aanroepen. Hierbij wordt geen Git-proces gestart.

Nog niet uitgevoerd voor de huidige F05-stand: volledige suite, cargo fmt --check, Clippy met -D warnings, platform-CI, zelfstandige Inspector-controle, protocolannulering met PID-bewijs en een MCP-procesboom tijdens het productieafsluitpad. Er is geen nieuw volledig testtotaal vastgesteld.

## Werksessie — 2026-11-05

Ik werk in kleine stappen: één codefragment met uitleg, daarna de gerichte controle. Samenhangende testcode kan in één keer worden toegevoegd. De volgende stap begint pas na een geslaagde controle. Documentatie werk ik op afzonderlijke momenten bij; de centrale checklist staat in het featureplan. Git-acties voer ik zelf uit; deze update bevat geen nieuwe commit- of CI-verificatie.

## Stoppunt en vervolg

De functie mcp_wait_helper is toegevoegd aan src/tests/mcp_tests.rs. Deze ignored proceshelper leest een commandhub-mcp-pid-file= argument, schrijft zijn eigen PID en wacht maximaal 30 seconden. Alleen de compilatie is gecontroleerd; de helper is nog niet in een test gestart.

Eerstvolgende stap: een SDK-client een langlopende toolcall laten starten, wachten op een geldig PID-bestand, expliciet het MCP-annuleringsbericht sturen en met de bestaande process_checks::assert_stopped controleren dat het proces stopt. Controleer ook herstel/bruikbaarheid na annulering. Inspecteer vooraf hoe de SDK bij annulering de handlerfuture behandelt: de gecompileerde koppeling alleen bewijst nog geen afgewachte opruiming. Gebruik unieke tijdelijke bestanden en begrensde wachttijden.

Daarna de resterende transport-/protocol-, dynamische parameter-, REST/MCP-consistentie-, capaciteits- en shutdowntests uitwerken, gevolgd door eindcontroles en platform-CI. F05 blijft in uitvoering.
