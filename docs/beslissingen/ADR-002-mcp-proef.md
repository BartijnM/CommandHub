# ADR-002 — Bevindingen uit de F00-MCP-proef

Status van de oorspronkelijke proef op 2026-10-20. De productie-integratie is hieronder aangevuld met de bevestigde F05-keuzes.

## Context

F00 onderzoekt de officiële Rust-SDK en een HTTP-transport voordat MCP in de CommandHub-kern wordt geïntegreerd. De test is bewust geïsoleerd van de hoofdapplicatie.

## Bevindingen

- SDK: `rmcp` 3.6.0 met `server`, `schemars` en `transport-streamable-http-server`.
- Transport: Streamable HTTP op `127.0.0.1:3001/mcp`.
- MCP Inspector CLI 2.11.0 kon `hello` opnieuw ontdekken en rapporteerde voor een onbekende tool de foutcode `tool_not_found`.
- Een `hello`-aanroep met het extra argument `extra=value` werd succesvol uitgevoerd. Omdat de handler geen parameters accepteert, toont dit dat dit extra veld in deze proef niet werd afgewezen; het is geen bewijs van algemene SDK-schema-validatie.
- Eerder zijn `initialize` en de succesvolle aanroep van `hello` handmatig geverifieerd. De oorspronkelijke Inspector-aanroepen waren niet versiegepind.
- De proef onderhandelde protocolversie `2025-11-25`.

## Beveiliging en grenzen

De proef bindt aan loopback (`127.0.0.1`) en is bedoeld voor lokaal testen. De standaard Host-controle van de gebruikte SDK beperkt toegestane hosts tot loopback. De lege Origin-allowlist schakelt Origin-validatie standaard niet in; beschouw dit dus niet als een aanvullende browserbescherming.

Deze instellingen en waarnemingen zijn geen remote-deploymentadvies. Origin-validatie staat standaard uit en is niet met een kwaadaardige Origin-header getest. Voor F05 moeten de actuele MCP-transportvereisten, toepasselijke Host/Origin-controles en het beoogde lokale gebruik opnieuw worden beoordeeld. Publieke blootstelling vraagt een apart ontwerp voor authenticatie, transportbeveiliging en autorisatie.

## MCP tegenover REST

REST-endpoints zijn applicatiespecifieke HTTP-routes met zelfgekozen URL's, methodes en JSON-contracten. MCP gebruikt een gestandaardiseerd protocol over HTTP, met protocolinitialisatie/onderhandeling, JSON-RPC-methodes zoals `tools/list` en `tools/call`, en een MCP-specifieke tool- en resultaatstructuur. De proef test MCP; zij bewijst niet dat de latere REST-routes bestaan of dat beide transports al een gedeelde CommandHub-kern gebruiken.

## Besluit en vervolg

Gebruik deze proef als bewijs dat `rmcp` 3.6.0 en Streamable HTTP technisch geschikt zijn voor verder onderzoek. Leg de SDK-versie, het transport, de protocolwaarneming en de expliciet gebruikte Inspector-versie vast; behandel ze niet als definitieve F05-keuzes totdat integratie en beveiligingseisen zijn getoetst. De parameterloze proefhandler wees het extra invoerveld niet af; vertrouw dus niet op deze proef als bewijs van inputvalidatie. De proef documenteert de foutwaarnemingen voor `tool_not_found` en het extra argument, niet een volledige foutmatrix. Implementeer daadwerkelijke parametercontrole in de gedeelde kern wanneer die feature aan bod komt.

## Productie-integratie bevestigd bij F05

Aanvulling op 2026-11-11. De hoofdapplicatie gebruikt rmcp 3.6.0 met Streamable HTTP op 127.0.0.1:3000/mcp. De gedeelde kern valideert parameters en paden. Host/Origin-controles, de bodylimiet van 16 KiB en legacy-sessiemodus zijn expliciet ingesteld. De SDK-ketentest rapporteert protocol 2025-11-25. F05-CI en de F06-Inspector-basistest zijn geslaagd op Windows, Ubuntu en macOS. Ik bevestig de Claude Code-demo.

Deze aanvulling legt de latere integratie vast. De oorspronkelijke proefwaarnemingen hierboven blijven behouden. Het [API-contract](../api-en-configuratie.md), de [architectuur](../architectuur.md) en [security](../security.md) beschrijven de actuele instellingen en grenzen.
