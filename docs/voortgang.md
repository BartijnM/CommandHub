# Huidige voortgang

Bijgewerkt op 2026-11-10.

## Actief

[F06 — Platformcontrole en demonstratie](features/F06-oplevering.md) staat op **In uitvoering**. F05 is afgerond. De platformprofielen en de schone-checkouttest van de MCP-basisketen zijn gecontroleerd.

## Platformcontrole op 2026-11-10

De laatste CI-run is groen op alle drie de runners. De getoonde tijden zijn de looptijden van de volledige jobs.

| Runner | Profiel | CI | Jobduur |
| --- | --- | --- | --- |
| Windows (`windows-latest`) | `config/tools.json` | Geslaagd | 4 min 47 s |
| Ubuntu (`ubuntu-latest`) | `config/tools.linux.json` | Geslaagd | 2 min 5 s |
| macOS (`macos-latest`) | `config/tools.macos.json` | Geslaagd | 3 min 14 s |

Elke job begint met een schone checkout en draait formatter, Clippy, Rust-tests, build en de stap **MCP-basistest**. De MCP-stap start de gebouwde server met het platformprofiel, wacht op `/status`, vergelijkt de namen uit `tools/list` met de configuratie en controleert `git_version` via `tools/call`. Ook controleert de stap dat `git_diff_view` met `view=--patch` de toolfout `invalid_parameters` geeft. Inspector 2.11.0 gebruikt Streamable HTTP in legacy-modus. De stap ruimt het serverproces op bij afsluiten.

Dit bewijs geldt voor de basisketen op gehoste runners. De profielen zijn alle drie geladen en ontdekt. De afzonderlijke aanroepen voor Python, `rg`, `pdfinfo` en `ffprobe` vallen buiten deze CI-basistest. Een onafhankelijke README-doorloop en de volledige opleveringsdemo staan nog open.

De laatste getoonde CI-run waarschuwt over de Node.js-runtime van `actions/checkout@v4`. De drie jobs zijn geslaagd. Zie de [F06-notitie](../notities/F06-oplevering.md) voor het bewijs en het exacte stoppunt.

## Volgende stap

Maak een reproduceerbaar demo-draaiboek voor configuratie, ontdekking, uitvoering, validatiefouten, logging en timeout. Toon daarin ook hoe een nieuwe tool zichtbaar wordt na een JSON-wijziging en herstart van dezelfde executable. Daarna volgen de definitieve documentatie, requirementdekking, beperkingen en opleverings-PR.

## Overzicht uit F05

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

## Aandachtspunten

De shutdowntestclient gebruikt NeverRetry en schakelt sessieherinitialisatie uit. Met standaard SSE-herverbinden bleef de clientcall na serverafsluiting wachten. Deze testinstelling moet bij interpretatie van het shutdownbewijs worden meegenomen.

rmcp 3.6.0 geeft 415 bij een niet-deserialiseerbare JSON-RPC-body. Een onbekende protocolversieheader geeft 400, onvolledige Accept 406 en een onbekende sessie 404. Dit SDK-gedrag staat in het API-contract.

De F05-platform-CI uit run #50 is historisch bewijs voor F05. Expliciet beleid voor onverwacht clientdisconnect blijft open. Padcanonicalisatie en beheerdersconfiguratie hebben de eerder vastgelegde grenzen.

## Platformverificatie op 2026-11-09

Een screenshot van CI-run #50 legt de platformresultaten vast. De jobs Checks (ubuntu-latest), Checks (macos-latest) en Checks (windows-latest) zijn geslaagd. Op alle drie zijn cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings, cargo test --locked en cargo build --locked groen. Exacte CI-testtotalen, commit-SHA en run-URL zijn niet zichtbaar in de screenshot. De lokale testtotalen hierboven blijven afzonderlijk vastgelegd.

## Toolset op 2026-11-10

Elk platformprofiel bevat achttien tools. Naast de bestaande testtools zijn projectinspectie, bestands- en notitiebeheer, lokale diagnose en media-informatie beschikbaar. De persoonlijke tools gebruiken het vaste Python-script `tools/personal_tools.py`.

Bestandstools gebruiken `CommandHub-data` in de thuismap van het serveraccount of een bestaande absolute map uit `COMMANDHUB_DATA_DIR`. De client geeft een relatief pad op. Voor notities staat de submap `notes` vast. De gezondheidscheck gebruikt uitsluitend de twee geconfigureerde loopbackadressen.

De bestands-, notitie-, CSV-, hash-, schijf- en PDF-functies zijn met tijdelijke gegevens en de gebundelde Python-runtime gecontroleerd. De Windows-alias `python` startte in de testomgeving niet. Voor de mediatools is de foutmelding bij ontbrekende `ffprobe` gecontroleerd. De gezondheidscheck is getest met gesimuleerde HTTP-antwoorden doordat lokale socketverbindingen in de testomgeving geblokkeerd waren. Afzonderlijke Inspector-aanroepen voor de persoonlijke tools staan nog open.

De [README](../README.md#mcp-handmatig-testen) bevat Inspector-voorbeelden. De [configuratiehandleiding](../config/README.md) beschrijft de parameters, afhankelijkheden en uitvoergrenzen.
