# Huidige voortgang

Bijgewerkt op 2026-11-09.

## Actief

[F05 — MCP-integratie](features/F05-mcp-integratie.md) staat op Review. De eigenaar voert de code in en draait de controles. De implementatie en Windows-tests zijn uitgebreid sinds de sessie van 2026-11-05.

## Aanwezig

/mcp biedt Streamable HTTP via rmcp 3.6.0 naast REST. Beide adapters delen AppState, registry, input_schema, validatie en ExecutionService. MCP levert procesresultaten en veilige fouten als structuredContent en JSON-tekst. De echte SDK-client onderhandelt 2025-11-25. Legacy-sessiemodus, Host/Origin-controles en de bodylimiet van 16 KiB zijn ingesteld.

Expliciete MCP-requestannulering bereikt via context.ct en oneshot de uitvoeringskern. Vier annuleringsrondes controleren procesbeëindiging en succesvolle vervolgaanroepen. Het productieafsluitpad server::serve is getest met een actieve MCP-procesboom, beide PID’s, capaciteitsvrijgave en een gesloten luisterpoort.

Een gemengde belastingstest start twee REST- en twee MCP-procesbomen. Extra calls via beide protocollen worden afgewezen, waarna uitvoering via beide protocollen herstelt. De ketentest vergelijkt dynamische stringinvoer, keuzelijsten, typefouten, verplichte/extra velden en resultaatvelden tussen REST en MCP.

Transporttests controleren Host/Origin, sessieinitialisatie, onbekende sessies, protocolversieheaders, contentonderhandeling, ongeldige JSON-RPC-bodies en bodygrenzen. De kleine router-testwrapper staat in src/tests/routes_tests.rs en wordt uitsluitend voor tests vanuit routes.rs geherexporteerd.

## Verificatie op Windows

De eigenaar deelde de volgende geslaagde resultaten op 2026-11-09.

- cargo fmt --check gaf geen verschillen.
- cargo clippy --all-targets --locked -- -D warnings slaagde in 0,25 s.
- cargo test --locked gaf 156 passed, 0 failed, 7 ignored, 0 filtered out, in 9,77 s.
- De tien transporttests slaagden in 0,07 s.
- De SDK-ketentest rapporteerde protocolversie 2025-11-25 en slaagde na de paduitbreiding in 0,61 s.

De zeven ignored functies zijn expliciete proceshelpers, waaronder mcp_echo_helper en mcp_wait_helper. De assistent heeft de actuele code gelezen. De resultaten hierboven komen uit de gedeelde uitvoer van de eigenaar. Bij deze documentatie-update zijn uitsluitend Markdown-bestanden gewijzigd.

## Open en volgende stap

De uitgebreide SDK-ketentest vergelijkt geldige absolute paden, canonicalisatie met .. binnen de root en afwijzing buiten allowed_roots, relatieve paden en onoplosbare paden met REST. De eigenaar bevestigt de lokale eindcontroles na deze uitbreiding.

Volgende stap is committen door de eigenaar en F05 via GitHub Actions op Windows, Linux en macOS verifiëren. Daarna CI-bewijs vastleggen en PR-review/merge afronden. De [algemene README](../README.md#mcp-handmatig-testen) bevat Inspector-commando’s voor ontdekking, uitvoering en parameterafwijzing. Een zelfstandige Inspector-controle is nog niet bevestigd.

## Aandachtspunten

De shutdowntestclient gebruikt NeverRetry en schakelt sessieherinitialisatie uit. Met standaard SSE-herverbinden bleef de clientcall na serverafsluiting wachten. Deze testinstelling moet bij interpretatie van het shutdownbewijs worden meegenomen.

rmcp 3.6.0 geeft 415 bij een niet-deserialiseerbare JSON-RPC-body. Een onbekende protocolversieheader geeft 400, onvolledige Accept 406 en een onbekende sessie 404. Dit SDK-gedrag staat in het API-contract.

F05-platform-CI, definitieve review en merge zijn nog open. Oud F04-CI-bewijs geldt voor F04. Expliciet beleid voor onverwacht clientdisconnect blijft open. Padcanonicalisatie en beheerdersconfiguratie hebben de eerder vastgelegde grenzen. F05 staat op Review.
