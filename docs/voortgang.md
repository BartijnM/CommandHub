# Huidige voortgang

Bijgewerkt op 2026-11-10.

## Actief

[F05 — MCP-integratie](features/F05-mcp-integratie.md) staat op Review. De Windows-tests en platform-CI voor de F05-integratie zijn vastgelegd. Ik heb de toollijst daarna uitgebreid.

## Aanwezig

/mcp biedt Streamable HTTP via rmcp 3.6.0 naast REST. Beide adapters delen AppState, registry, input_schema, validatie en ExecutionService. MCP levert procesresultaten en veilige fouten als structuredContent en JSON-tekst. De echte SDK-client onderhandelt 2025-11-25. Legacy-sessiemodus, Host/Origin-controles en de bodylimiet van 16 KiB zijn ingesteld.

Expliciete MCP-requestannulering bereikt via context.ct en oneshot de uitvoeringskern. Vier annuleringsrondes controleren procesbeëindiging en succesvolle vervolgaanroepen. Het productieafsluitpad server::serve is getest met een actieve MCP-procesboom, beide PID’s, capaciteitsvrijgave en een gesloten luisterpoort.

Een gemengde belastingstest start twee REST- en twee MCP-procesbomen. Extra calls via beide protocollen worden afgewezen, waarna uitvoering via beide protocollen herstelt. De ketentest vergelijkt dynamische stringinvoer, keuzelijsten, typefouten, verplichte/extra velden en resultaatvelden tussen REST en MCP.

Transporttests controleren Host/Origin, sessieinitialisatie, onbekende sessies, protocolversieheaders, contentonderhandeling, ongeldige JSON-RPC-bodies en bodygrenzen. De kleine router-testwrapper staat in src/tests/routes_tests.rs en wordt uitsluitend voor tests vanuit routes.rs geherexporteerd.

## Verificatie op Windows

Op 2026-11-09 zijn de volgende resultaten vastgelegd.

- cargo fmt --check gaf geen verschillen.
- cargo clippy --all-targets --locked -- -D warnings slaagde in 0,25 s.
- cargo test --locked gaf 156 passed, 0 failed, 7 ignored, 0 filtered out, in 9,77 s.
- De tien transporttests slaagden in 0,07 s.
- De SDK-ketentest rapporteerde protocolversie 2025-11-25 en slaagde na de paduitbreiding in 0,61 s.

De zeven ignored functies zijn expliciete proceshelpers, waaronder mcp_echo_helper en mcp_wait_helper.

## Open en volgende stap

De uitgebreide SDK-ketentest vergelijkt geldige absolute paden, canonicalisatie met .. binnen de root en afwijzing buiten allowed_roots, relatieve paden en onoplosbare paden met REST. De lokale eindcontroles na deze uitbreiding zijn geslaagd.

CI-run #50 is geslaagd op Ubuntu, macOS en Windows. Voor F05 staat PR-review/merge open. Voor de uitgebreide toolset volgt een Inspector-controle met de benodigde executables op het server-PATH. De [algemene README](../README.md#mcp-handmatig-testen) bevat Inspector-commando’s voor ontdekking, uitvoering en parameterafwijzing. Een zelfstandige Inspector-controle is nog niet bevestigd.

## Aandachtspunten

De shutdowntestclient gebruikt NeverRetry en schakelt sessieherinitialisatie uit. Met standaard SSE-herverbinden bleef de clientcall na serverafsluiting wachten. Deze testinstelling moet bij interpretatie van het shutdownbewijs worden meegenomen.

rmcp 3.6.0 geeft 415 bij een niet-deserialiseerbare JSON-RPC-body. Een onbekende protocolversieheader geeft 400, onvolledige Accept 406 en een onbekende sessie 404. Dit SDK-gedrag staat in het API-contract.

F05-platform-CI is geslaagd in run #50. Definitieve review en merge zijn nog open. Oud F04-CI-bewijs geldt voor F04. Expliciet beleid voor onverwacht clientdisconnect blijft open. Padcanonicalisatie en beheerdersconfiguratie hebben de eerder vastgelegde grenzen. F05 staat op Review.

## Platformverificatie op 2026-11-09

Een screenshot van CI-run #50 legt de platformresultaten vast. De jobs Checks (ubuntu-latest), Checks (macos-latest) en Checks (windows-latest) zijn geslaagd. Op alle drie zijn cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings, cargo test --locked en cargo build --locked groen. Exacte CI-testtotalen, commit-SHA en run-URL zijn niet zichtbaar in de screenshot. De lokale testtotalen hierboven blijven afzonderlijk vastgelegd. PR-review/merge is nog open.

## Toolset op 2026-11-10

Beide toolconfiguraties bevatten achttien tools. Naast de bestaande testtools zijn projectinspectie, bestands- en notitiebeheer, lokale diagnose en media-informatie beschikbaar. De persoonlijke tools gebruiken het vaste Python-script `tools/personal_tools.py`.

Bestandstools gebruiken `CommandHub-data` in de thuismap van het serveraccount of een bestaande absolute map uit `COMMANDHUB_DATA_DIR`. De client geeft een relatief pad op. Voor notities staat de submap `notes` vast. De gezondheidscheck gebruikt uitsluitend de twee geconfigureerde loopbackadressen.

De bestands-, notitie-, CSV-, hash-, schijf- en PDF-functies zijn met tijdelijke gegevens en de gebundelde Python-runtime gecontroleerd. De Windows-alias `python` startte in de testomgeving niet. Voor de mediatools is de foutmelding bij ontbrekende `ffprobe` gecontroleerd. De gezondheidscheck is getest met gesimuleerde HTTP-antwoorden doordat lokale socketverbindingen in de testomgeving geblokkeerd waren. Een volledige Inspector-run voor deze toolset staat nog open.

De [README](../README.md#mcp-handmatig-testen) bevat Inspector-voorbeelden. De [configuratiehandleiding](../config/README.md) beschrijft de parameters, afhankelijkheden en uitvoergrenzen.
