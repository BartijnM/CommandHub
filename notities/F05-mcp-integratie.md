---
feature: F05
tags:
  - commandhub
  - feature-notitie
---

# F05 — MCP via HTTP aan de kern koppelen

[Featureplan](../docs/features/F05-mcp-integratie.md)

## Leeswijzer bij de oplevering

Deze notitie bewaart de ontwikkeling en resultaten per werksessie. Beschrijvingen van oude ingangen, configuraties en open punten horen bij hun gedateerde stand. Het actuele gedrag staat in het [API-contract](../docs/api-en-configuratie.md), de [architectuur](../docs/architectuur.md) en de [voortgang](../docs/voortgang.md).

## Wat bouw ik en waarom?

Ik voeg een MCP-adapter toe waarmee clients de geconfigureerde tools via Streamable HTTP kunnen ontdekken en uitvoeren. REST en MCP gebruiken dezelfde registry, inputschema's, parametervalidatie en uitvoeringsservice. F05 is inmiddels afgerond volgens de voortgang bij F06. De annuleringsketen, productie-shutdown en gemengde belasting zijn op Windows getest. Lokale formatter, Clippy en de volledige suite slagen. MCP-padvalidatie is ook via de clientketen met REST vergeleken. Platform-CI is geslaagd op Ubuntu, macOS en Windows in run #50. De latere afronding is in de F06-voortgang vastgelegd.

## Hoe werkt de code?

- src/registry.rs bevat input_schema(&ToolDefinition). REST en MCP gebruiken dezelfde schemaopbouw met properties, required, enum en additionalProperties: false.
- src/mcp.rs bevat McpServer met AppState. Clone deelt de bestaande Arc-verwijzingen; er ontstaat geen executor per sessie. get_info gebruikt ServerConfig en kondigt tools aan. list_tools levert de volledige registry zonder vervolgpagina.
- call_tool gebruikt execute_with_parameters_with_cancellation. Ontbrekende arguments worden een leeg object. De gedeelde kern controleert parameters en paden en bewaakt capaciteit, output, timeout en procesbeheer.
- context.ct is het SDK-annuleringssignaal. tokio::select! wacht op uitvoering of annulering. Bij annulering wordt een oneshot-bericht naar de kern gestuurd en de uitvoering afgewacht. De echte SDK-client stuurt een annuleringsbericht. Vier rondes controleren procesbeëindiging en succesvolle vervolgaanroepen.
- execution_result levert success, stdout, stderr, exitCode en duration in milliseconden als structuredContent en als JSON-tekst. Een niet-succesvolle procesexit geeft isError: true.
- execution_error vertaalt een onbekende tool naar METHOD_NOT_FOUND. Andere kernfouten worden veilige toolresultaten met isError: true; interne OS-details blijven buiten de response.
- routes.rs bevat router_with_shutdown en koppelt de service aan /mcp. De kleine router-wrapper staat in src/tests/routes_tests.rs en maakt een eigen testtoken. routes.rs herexporteert deze helper onder cfg(test).
- server.rs maakt het transporttoken en geeft een kloon aan de router. Het afsluitsignaal start eerst de shutdown van de uitvoeringsservice en annuleert daarna het MCP-token. Ook na terugkeer van Axum wordt het token geannuleerd en execution.shutdown afgewacht.

Rust: Arc-klonen delen gegevens; move bewaart state in de servicefactory. Result en ? geven protocolfouten door. into() vertaalt een toolresultaat naar CallToolResponse. tokio::pin! laat dezelfde uitvoeringsfuture door select! en de aansluitende afhandeling gebruiken. CancellationToken is een gedeeld stopsignaal; oneshot blijft het transportonafhankelijke annuleringsmechanisme in de kern.

## Keuzes en grenzen

- rmcp staat exact op 3.6.0. De normale dependencies activeren server en transport-streamable-http-server. De dev-dependency voegt client en transport-streamable-http-client-reqwest toe. tokio-util 0.7 met rt levert CancellationToken.
- De productielistener blijft 127.0.0.1:3000. Host is beperkt tot 127.0.0.1:3000 en localhost:3000. Aanwezige origins moeten overeenkomen met http://127.0.0.1:3000 of http://localhost:3000. Requests zonder Origin zijn toegestaan. Geen remote deployment of authenticatie toegevoegd.
- De MCP-bodylimiet is 16 KiB en wordt door de SDK tijdens lezen afgedwongen.
- legacy_session_mode is true. Routertests initialiseren expliciet met protocol 2025-11-25 en controleren de sessieheader. De SDK behandelt 2026-07-28 zonder protocolsessies. De TCP-ketentest leest peer_info na SDK-onderhandeling en rapporteert op 2026-11-09 expliciet 2025-11-25.
- De TCP-ketentest bindt aan een vrije loopbackpoort en past alleen in de test de Host-allowlist aan. Hij gebruikt een eigen Axum-testserver, niet server::serve. Een afzonderlijke test in http_execution_tests.rs controleert inmiddels het volledige productieafsluitpad met een actieve MCP-procesboom.
- Expliciet beleid voor onverwacht clientdisconnect blijft open. Platform-CI is geslaagd in run #50. MCP-padvalidatie via de clientketen is inmiddels met REST vergeleken. Dynamische stringparameters en gemengde REST/MCP-belasting zijn getest. De vastgelegde grenzen rond padcanonicalisatie blijven gelden.

## Problemen en oplossingen

Een structliteral met ServerInfo werd afgewezen vanwege non_exhaustive. De instantie wordt nu via default gemaakt en daarna ingesteld. ServerInfo bleek ook een verouderde alias; ik gebruik ServerConfig.

## Historische verificatie op Windows, 2026-11-05

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

## Werksessie op 2026-11-09

### Protocolannulering en herstel

wait_for_mcp_pid wacht maximaal vijf seconden op een geldig PID. Een nog leeg of gedeeltelijk geschreven bestand wordt opnieuw gelezen. De test protocol_cancellation_stops_process_and_preserves_capacity gebruikt send_cancellable_request en RequestHandle::cancel om een expliciet MCP-annuleringsbericht te sturen. Daarna controleert process_checks::assert_stopped de processtatus. Vier rondes met telkens een succesvolle echo-aanroep maken een achtergebleven capaciteitspermit zichtbaar.

De request wordt gemaakt met CallToolRequest::new. Een structliteral gaf E0639 door non_exhaustive. De constructor vult method en extensions in en blijft bruikbaar wanneer de SDK velden toevoegt.

### Dynamische invoer en REST/MCP-consistentie

De SDK-ketentest bevat nu test_version met detail als verplichte string en allowed_values voor --version en -vV. De test vergelijkt MCP en de REST-router met dezelfde AppState. Succesvelden success, stdout, stderr en exitCode zijn gelijk. Duration wordt afzonderlijk op numeriek type gecontroleerd. Ontbrekende parameters, verkeerde typen, verboden waarden en extra velden leveren dezelfde inhoudelijke code/message op.

De uitgebreide clienttest ontdekt drie testtools en kan na afgewezen calls verder werken. peer_info rapporteert de daadwerkelijk onderhandelde protocolversie 2025-11-25.

### Productie-shutdown en gedeelde capaciteit

server_shutdown_stops_active_mcp_process_tree gebruikt server::serve via de bestaande TestServer en Fixture. De tool start een subprocess en schrijft beide PID’s. De test wacht op die bevestiging, stuurt het afsluitsignaal en controleert serverafronding, processtatussen, capaciteitsvrijgave en een gesloten luisterpoort.

De testclient gebruikt een vrije TCP-poort met Host 127.0.0.1:3000 om de productie-allowlist te respecteren. reinit_on_expired_session is false en retry_config gebruikt NeverRetry. De eerste versie liep na tien seconden vast op de wachtende clientcall. In de lokale SDK-bron bleek SSE-herverbinden apart van sessieherinitialisatie te werken. Na NeverRetry slaagde de test in 8,17 s. Deze instelling hoort bij de testclient en moet bij de interpretatie van dit bewijs worden meegenomen.

De requestfuture staat in een eigen blok. Zij leent client tot haar destructor is uitgevoerd. Door pids en response uit het blok terug te geven eindigt de lening vóór client.cancel de client consumeert. Dit loste E0505 op. Een achtergebleven kopie van het oude requestblok veroorzaakte daarna E0425 en is verwijderd.

rest_and_mcp_share_execution_capacity start twee REST- en twee MCP-procesbomen. PID-bestanden bevestigen alle vier starts. Een extra REST-call geeft 503 en een extra MCP-call geeft een toolfout, beide met capacity_exceeded. Na het vrijgavesignaal slagen alle actieve calls en nieuwe calls via beide protocollen. Beide processen per boom worden gecontroleerd.

### Negatieve transportrequests

De vier nieuwe tests gebruiken geldige basisheaders en veranderen per scenario de relevante body of header. Alle tien transporttests slagen. De lokale rmcp 3.6.0-parsehelper expect_json geeft 415 bij ongeldige JSON of een ongeldige JSON-RPC-vorm. De aanvankelijke verwachting van 400 is gecorrigeerd naar het gecontroleerde SDK-gedrag. Content-Type text/plain geeft eveneens 415, Accept uitsluitend application/json geeft 406, een onbekende protocolversieheader geeft 400 en een onbekende sessie geeft 404.

### Scheiding tussen tests en productie

Clippy meldde de kleine router-wrapper als ongebruikt in de productiebuild. Deze helper is verplaatst naar src/tests/routes_tests.rs. Alleen de herexport en modulekoppeling in routes.rs dragen cfg(test). Productie gebruikt router_with_shutdown. Daarmee blijft testondersteuning fysiek in src/tests/.

### Laatste lokale controles

De volgende resultaten zijn tijdens de lokale controles vastgelegd.

| Controle | Gedeeld resultaat |
| --- | --- |
| cargo test --locked mcp::tests:: vóór de vier nieuwe transporttests | 12 passed, 2 ignored, 144 filtered, 0,88 s |
| cargo test --locked http_execution_tests:: vóór de gemengde capaciteitstest | 3 passed, 2 ignored, 153 filtered, 8,22 s |
| Gemengde REST/MCP-capaciteitstest | Geslaagd gemeld |
| cargo test --locked mcp::tests::transport_ | 10 passed, 0 failed, 153 filtered, 0,07 s |
| SDK-ketentest met --nocapture | 1 passed, 162 filtered, 0,28 s, protocol 2025-11-25 |
| cargo fmt --check | Geslaagd zonder verschillen |
| cargo clippy --all-targets --locked -- -D warnings | Geslaagd in 0,25 s |
| cargo test --locked | 156 passed, 0 failed, 7 ignored, 0 filtered, 9,83 s |

De zeven ignored functies zijn expliciete proceshelpers. Ten tijde van deze lokale tussencontrole waren F05-platform-CI en een zelfstandige Inspector-controle nog niet uitgevoerd. Oudere F04-CI bewijst deze F05-wijzigingen niet.

## Padvalidatie en lokale eindcontrole op 2026-11-09

De bestaande mcp_client_discovers_and_executes_configured_tool-test is uitgebreid met test_path. De beheerdersconfiguratie start Git met gescheiden argumenten -C, het pad, rev-parse en --show-toplevel. De toegestane root is de src-map van het project. Er is geen nieuwe uitvoerings- of validatielogica in de MCP-adapter toegevoegd.

Een absoluut pad naar src en src/tests/.. worden geaccepteerd. De projectroot en src/.. vallen buiten allowed_roots. Het relatieve pad src en een onoplosbaar pad onder Cargo.toml worden afgewezen. Iedere invoer gaat via een echte MCP-client over TCP en via REST met dezelfde AppState. De test vergelijkt resultaatvelden en veilige foutcodes/berichten. Bij succes wordt het canonieke repositorypad gecontroleerd. duration wordt afzonderlijk op een getal gecontroleerd omdat twee uitvoeringen elk een eigen duur hebben.

De uitgebreide ketentest slaagt met 1 passed, 162 filtered out in 0,61 s. Daarna zijn formatter, Clippy met -D warnings en de volledige suite als geslaagd bevestigd. De laatste volledige uitvoer geeft 156 passed, 0 failed, 7 ignored, 0 filtered out in 9,77 s. Het aantal tests blijft gelijk doordat de bestaande ketentest is uitgebreid. De eerdere tabel legt de tussencontrole vóór deze uitbreiding vast.

De algemene README bevat nu vastgezette Inspector CLI-commando’s voor tools/list, een call naar git_version en toegestane/afgewezen detail-invoer voor rust_details uit de standaardconfiguratie. Er is nog geen handmatig Inspector-resultaat aangeleverd. Bij deze documentatie-update zijn uitsluitend Markdown-bestanden aangepast en zijn de Cargo-controles niet opnieuw uitgevoerd.

## Platformverificatie op 2026-11-09

Een screenshot van CI-run #50 legt de platformresultaten vast. De jobs Checks (ubuntu-latest), Checks (macos-latest) en Checks (windows-latest) zijn geslaagd. Op alle drie zijn cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings, cargo test --locked en cargo build --locked groen. Exacte CI-testtotalen, commit-SHA en run-URL zijn niet zichtbaar in de screenshot. De lokale testtotalen hierboven blijven afzonderlijk vastgelegd. PR-review/merge is nog open.

## Historisch stoppunt vóór F06

F05 staat op Review. De lokale implementatie en acceptatiecriteria zijn aangetoond met de gedeelde Windows-resultaten. CI-run #50 is geslaagd op Ubuntu, macOS en Windows. Ik rond PR-review/merge af. Na bevestigde merge kan de status op Klaar.

## Uitbreiding van de toollijst op 2026-11-10

Beide toolconfiguraties bevatten achttien tools. Naast de bestaande testtools zijn projectinspectie, bestands- en notitiebeheer, lokale diagnose en media-informatie beschikbaar. De persoonlijke tools gebruiken het vaste Python-script `tools/personal_tools.py`.

Bestandstools gebruiken `CommandHub-data` in de thuismap van het serveraccount of een bestaande absolute map uit `COMMANDHUB_DATA_DIR`. De client geeft een relatief pad op. Voor notities staat de submap `notes` vast. De gezondheidscheck gebruikt uitsluitend de twee geconfigureerde loopbackadressen.

De bestands-, notitie-, CSV-, hash-, schijf- en PDF-functies zijn met tijdelijke gegevens en de gebundelde Python-runtime gecontroleerd. De Windows-alias `python` startte in de testomgeving niet. Voor de mediatools is de foutmelding bij ontbrekende `ffprobe` gecontroleerd. De gezondheidscheck is getest met gesimuleerde HTTP-antwoorden doordat lokale socketverbindingen in de testomgeving geblokkeerd waren. Een volledige Inspector-run voor deze toolset staat nog open.

Ik kan de nieuwe set via `tools/list` ontdekken en via `tools/call` aanroepen zodra de vereiste executables beschikbaar zijn. De voorbeelden in de [README](../README.md#mcp-handmatig-testen) gebruiken Inspector 2.11.0. Deze uitbreiding gebruikt de bestaande MCP- en REST-uitvoeringsroute. De latere F06-voortgang legt F05 als afgerond vast.

## Opleverreview op 2026-11-11

F05 is volgens de actuele voortgang afgerond bij de overgang naar F06. De MCP-keten wordt op alle drie platforms door Inspector gecontroleerd. Ik bevestig de volledige Claude Code-demo op 2026-11-11.

De actuele requirementdekking staat in het [testplan](../docs/testplan.md). De [F06-notitie](F06-oplevering.md) bevat de resterende opleveringsstappen. De oorspronkelijke resultaten hierboven behouden hun datum en scope.
