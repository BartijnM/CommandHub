---
id: F05
prioriteit: Must
status: In uitvoering
milestone: "MCP-keten"
---

# F05 — MCP via HTTP aan de kern koppelen

## Doel

MCP via HTTP aan de kern koppelen. Dit is een werkpakket; gebruik waar nodig meerdere kleine issues/PR's.

## Afhankelijkheden

F00, F04.

## Requirements

R11, R13, R15, R16.

## Scope

Zie onderstaande taken. Nieuwe uitbreidingen krijgen een apart issue; vergroot de scope niet stilzwijgend. Bij F01 is de minimale codebasis al aanwezig, maar lokale controle blijft nodig.

## Todo

- [x] SDK en gekozen transport integreren.
- [x] Registry vertalen naar MCP-tools en inputschema's.
- [x] Toolcalls via dezelfde toolservice als REST uitvoeren.
- [x] Kernfouten en procesresultaten naar MCP-resultaten vertalen.
- [x] Toepasselijke transportbeveiliging en sessiegedrag implementeren.
- [x] Met echte MCP-client en ongeldige protocolrequests testen.

## Acceptatiecriteria

- [x] MCP-client ontdekt geconfigureerde tools en voert ze uit.
- [x] Geen aparte executor of validatieregels in MCP-adapter.
- [ ] REST en MCP tonen consistent inhoudelijk gedrag.
- [ ] Protocol- en transportkeuzes zijn gedocumenteerd en getoetst.

## Bewijs en koppelingen

- GitHub issue: nog aan te maken.
- Branch: feature/F05-mcp-integratie.
- Pull request: nog niet aangemaakt.
- Tussenstand 2026-11-09. Eigenaar bevestigt Windows-formatter en Clippy met -D warnings geslaagd, volledige suite 156 passed en 7 proceshelpers. SDK-client onderhandelt 2025-11-25. Annulering met PID-bewijs, productie-shutdown met MCP-procesboom, gemengde REST/MCP-capaciteit, dynamische stringconsistentie en negatieve transportrequests zijn getest.
- Open voor afronding zijn MCP-padvalidatie via de clientketen, definitieve beoordeling en F05-platform-CI.
- [Featurenotitie](../../notities/F05-mcp-integratie.md).

## Afronden

- [ ] Relevante controles geslaagd op de definitieve stand, inclusief platform-CI (lokale tussenstand groen).
- [x] Documentatie en notitie bijgewerkt tot 2026-11-09.
- [x] Voortgang bijgewerkt tot 2026-11-09.
- [ ] PR gemerged; pas daarna status Klaar.


## Tussenstand op 2026-11-09

De implementatietaken zijn op Windows getest. De acceptatiecriteria voor volledige REST/MCP-consistentie en de definitieve protocol-/transportbeoordeling blijven open tot de MCP-padtest en eindbeoordeling zijn afgerond. De shutdownclient gebruikt expliciet NeverRetry. Platform-CI en merge volgen daarna.
