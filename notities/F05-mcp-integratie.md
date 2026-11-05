---
feature: F05
tags:
  - commandhub
  - feature-notitie
---

# F05 — MCP via HTTP aan de kern koppelen

[Featureplan](../docs/features/F05-mcp-integratie.md)

## Wat bouw ik en waarom?

F05 voegt een MCP-adapter toe aan de bestaande CommandHub-kern. De implementatie is in uitvoering: /mcp is aangesloten, maar volledige toolinteractie met een echte MCP-client is nog niet gecontroleerd. REST en MCP delen registry, schemaopbouw en uitvoeringsservice.

## Hoe werkt de code?

- src/registry.rs bevat input_schema(&ToolDefinition), gebruikt door zowel de REST-ontdekcontroller als MCP. Het schema bevat properties, required, enum waar ingesteld en additionalProperties: false.
- src/mcp.rs bevat McpServer met gedeelde AppState. Clone kopieert de Arc-verwijzingen; er ontstaat geen afzonderlijke executor per MCP-handler of sessie.
- get_info gebruikt ServerConfig en kondigt toolondersteuning aan. list_tools vertaalt de volledige registry naar SDK-tools zonder paginering.
- call_tool behandelt ontbrekende arguments als een leeg object en roept execute_with_parameters aan. De bestaande kern blijft verantwoordelijk voor parameter- en padvalidatie, argumentopbouw, capaciteit en procesbeheer.
- execution_result levert stdout, stderr, exitCode, success en duration in milliseconden als structuredContent en als JSON-tekst. Niet-succesvolle procesexit geeft isError: true.
- execution_error vertaalt UnknownTool naar een protocolfout met METHOD_NOT_FOUND. Overige kernfouten worden toolresultaten met isError: true en vaste foutcodes/teksten; interne OS-details worden niet gepubliceerd.
- routes.rs koppelt de StreamableHttpService aan /mcp, naast de bestaande REST-routes. De listener blijft 127.0.0.1:3000.
- src/tests/mcp_tests.rs is via cfg(test), path en mod tests gekoppeld aan mcp.rs, zonder extra modulewrapper.

Rust: Result en ? geven protocolfouten door; match onderscheidt kernresultaten en foutvarianten. into() vertaalt CallToolResult naar CallToolResponse. De servicefactory gebruikt move om AppState te bewaren en clone om gedeelde state aan handlers te geven.

## Keuzes en grenzen

- rmcp is exact vastgezet op 3.6.0 met server en transport-streamable-http-server. Dit is het uitgangspunt uit F00; het definitieve protocol-/clientcontract is nog niet afgerond.
- Native clients zonder Origin zijn toegestaan. Aanwezige origins zijn beperkt tot http://127.0.0.1:3000 en http://localhost:3000. Hosts zijn beperkt tot 127.0.0.1:3000 en localhost:3000. Browserclients op andere poorten zijn daarmee niet toegestaan.
- De SDK begrenst de MCP-requestbody op 16 KiB tijdens het lezen.
- legacy_session_mode is true. Initialisatie met protocol 2025-11-25 levert in de gerapporteerde tests een sessieheader op. Volgens de geïnspecteerde SDK-bron gebruikt protocol 2026-07-28 geen protocolsessies; deze nieuwere keten is nog niet getest.
- tokio-util 0.7 met rt is toegevoegd voor CancellationToken. Het token is nog niet gekoppeld aan de service en het serverafsluitpad.
- MCP-requestannulering is nog niet aan de bestaande annuleerbare uitvoeringsmethode gekoppeld. Clientdisconnectbeleid is nog niet vastgesteld.

## Problemen en oplossingen

ServerInfo bleek een non_exhaustive structalias: een structliteral werd afgewezen. Een instantie wordt nu via default gemaakt en daarna ingesteld. Vervolgens meldde de editor dat ServerInfo verouderd is; de code gebruikt nu ServerConfig.

## Verificatie — Windows, door eigenaar teruggekoppeld

De assistent heeft bronbestanden gelezen, maar in deze werksessie geen builds of tests uitgevoerd. Onderstaande resultaten komen uit de terugkoppeling van de eigenaar.

| Controle | Resultaat en grens |
| --- | --- |
| cargo test --locked routes::tests:: na gedeelde schemaopbouw | 18 passed, 0 failed, 125 filtered out; 0,79 s |
| cargo check / cargo check --locked tijdens stappen | Geslaagd gemeld, laatst ook na toevoegen tokio-util |
| cargo test --locked mcp::tests:: | Laatste run: 9 passed, 0 failed, 0 ignored, 143 filtered out; 0,01 s |

De negen MCP-tests controleren resultaatvelden en tekst/structuredContent, niet-nul exit, ontbrekende exitcode, onbekende tool, alle kernfoutvarianten met veilige OS-foutafhandeling, afgewezen hosts/origins, toegestane hosts/origins, sessieheader na initialisatie en de bodygrens van precies 16 KiB tegenover 16 KiB + 1. De transporttests lopen door de router met oneshot; ze openen geen TCP-poort en starten geen toolprocessen.

Niet geverifieerd voor F05: volledige testsuite, formatter, Clippy, build-/testmatrix op Linux/macOS, echte MCP-client, tools/list en tools/call end-to-end, gedeelde capaciteit onder REST/MCP-belasting, MCP-annulering en transportshutdown. Eerdere F04-CI is geen bewijs voor F05.

## Werksessie — 2026-11-05

De eigenaar voerde code zelf in op basis van kleine fragmenten en voerde gerichte controles uit. Tests werden per samenhangende stap gebundeld. Alleen op expliciet verzoek is deze tussentijdse documentatie door de assistent bijgewerkt. Geen broncode aangepast en geen Git-commando's uitgevoerd bij deze documentatie-update. Branchnaam is de geplande naam; actuele Git-status en merge zijn niet gecontroleerd.

## Vervolg

Eerst CancellationToken aan MCP-service en serverafsluitpad koppelen en gericht testen. Daarna MCP-annulering, toolontdekking/uitvoering, REST/MCP-consistentie en protocolnegatieve tests uitwerken, gevolgd door een echte MCP-client en eindcontroles. De centrale checklist staat uitsluitend in het featureplan.
