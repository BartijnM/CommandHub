---
id: F05
prioriteit: Must
status: Review
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
- [x] REST en MCP tonen consistent inhoudelijk gedrag.
- [x] Protocol- en transportkeuzes zijn gedocumenteerd en lokaal getoetst.

## Bewijs en koppelingen

- GitHub issue: nog aan te maken.
- Branch: feature/F05-mcp-integratie.
- Pull request: nog niet aangemaakt.
- Tussenstand 2026-11-09. Eigenaar bevestigt Windows-formatter en Clippy met -D warnings geslaagd, volledige suite 156 passed en 7 proceshelpers. SDK-client onderhandelt 2025-11-25. Annulering met PID-bewijs, productie-shutdown met MCP-procesboom, gemengde REST/MCP-capaciteit, dynamische stringconsistentie en negatieve transportrequests zijn getest.
- De uitgebreide SDK-ketentest vergelijkt ook padvalidatie met REST. Laatste lokale suite geeft 156 passed, 0 failed en 7 helpers in 9,77 s. F05-platform-CI is geslaagd in run #50. Open voor afronding is PR-review/merge.
- [Featurenotitie](../../notities/F05-mcp-integratie.md).

## Afronden

- [x] Relevante controles geslaagd, inclusief platform-CI op Ubuntu, macOS en Windows (run #50).
- [x] Documentatie en notitie bijgewerkt tot 2026-11-09.
- [x] Voortgang bijgewerkt tot 2026-11-09.
- [ ] PR gemerged; pas daarna status Klaar.


## Tussenstand op 2026-11-09

De implementatietaken en acceptatiecriteria zijn lokaal op Windows aangetoond, inclusief padvalidatie via een echte SDK-client en vergelijking met REST. Formatter, Clippy met -D warnings en de volledige suite zijn na de uitbreiding als geslaagd bevestigd. Status Review wacht op PR-review/merge. F05-platform-CI is geslaagd op alle drie platforms in run #50. De shutdownclient gebruikt expliciet NeverRetry. De algemene README bevat handmatige Inspector-controles waarvan nog geen uitvoer is aangeleverd.
