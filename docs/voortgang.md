# Huidige voortgang

Bijgewerkt: 2026-11-05.

## Actief

[F05 — MCP-integratie](features/F05-mcp-integratie.md) is in uitvoering op opdracht van de eigenaar. De eigenaar voert code zelf in en voert de controles uit; de assistent geeft kleine fragmenten met uitleg. Alleen expliciet gevraagde wijzigingen worden door de assistent gemaakt. Geen Git-commando's uitvoeren.

De vorige overdracht vermeldde F04 op Review met geslaagde lokale controles en drieplatform-CI; PR-review/merge is in deze sessie niet bevestigd. De start van F05 is geen mergebewijs voor F04.

## Aanwezig

De bestaande dynamische REST-uitvoering deelt registry en ExecutionService met de nieuwe MCP-adapter. Schemaopbouw staat in registry::input_schema en wordt door REST en MCP gebruikt. McpServer biedt get_info, list_tools en call_tool; toolcalls gebruiken execute_with_parameters. Procesresultaten en veilige kernfouten worden naar MCP-responses vertaald.

Streamable HTTP is aangesloten op /mcp naast de bestaande REST-routes, met rmcp exact 3.6.0. Host- en Origin-controles zijn expliciet ingesteld voor localhost/127.0.0.1 op poort 3000; requests zonder Origin zijn toegestaan. MCP-bodies zijn begrensd op 16 KiB. Legacy-sessiemodus is ingeschakeld; initialisatie met 2025-11-25 is gericht getest. tokio-util is toegevoegd ter voorbereiding van transportshutdown.

## Verificatie

Door de eigenaar op Windows teruggekoppeld: 18 REST-routetests geslaagd na schema-extractie; vervolgens 9 MCP-tests geslaagd (0 failed, 143 filtered out, 0,01 s). De MCP-tests controleren resultaat-/foutmapping en transportcontroles, inclusief geldige initialisatie met sessieheader en exacte bodygrens. cargo check is tijdens de stappen geslaagd gemeld, laatst na toevoegen tokio-util.

Deze tests bewijzen nog geen complete MCP-toolketen. Geen volledige F05-suite, formatter, Clippy, echte client of F05-CI op drie platforms bevestigd. De assistent heeft tijdens deze werksessie geen tests uitgevoerd. Zie de [F05-notitie](../notities/F05-mcp-integratie.md) voor details.

Historisch F04-bewijs: lokaal 138 tests geslaagd met 5 proceshelpers; CI-run #39 voor 1fb8d71 geslaagd op Windows, Ubuntu en macOS. Dit geldt niet als verificatie van F05.

## Grenzen

MCP-transportshutdown en requestannulering zijn nog niet gekoppeld. Nieuwe protocolversies zijn niet end-to-end getest. Alleen lokale binding; geen remote deployment of authenticatie toegevoegd. De bestaande grenzen rond padcanonicalisatie, toolspecifieke argumentbeoordeling en ontbreken van een sandbox blijven gelden.

## Volgende stap

CancellationToken aansluiten op MCP-service en bestaand serverafsluitpad; gericht controleren dat sessies/streams stoppen. Daarna MCP-annulering, echte toolketen, REST/MCP-consistentie en protocolfouten testen. Vervolgens echte MCP-client, volledige controles en platform-CI. F05 blijft in uitvoering. Deze tussentijdse update wijzigde alleen documentatie; geen Git-acties uitgevoerd.
