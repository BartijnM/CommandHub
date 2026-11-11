---
id: F05
prioriteit: Must
status: Klaar
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

Zie de taken en acceptatiecriteria hieronder. Dit featureplan beschrijft de eigen implementatiestap. Het actuele productcontract staat in [API en configuratie](../api-en-configuratie.md) en de opleverstand in [voortgang](../voortgang.md).

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
- Afronding is vastgelegd in docs/voortgang.md bij de start van F06. Het exacte PR-nummer is niet vastgelegd.
- Tussenstand 2026-11-09. Op Windows slagen formatter en Clippy met -D warnings. De volledige suite geeft 156 passed en 7 proceshelpers. SDK-client onderhandelt 2025-11-25. Annulering met PID-bewijs, productie-shutdown met MCP-procesboom, gemengde REST/MCP-capaciteit, dynamische stringconsistentie en negatieve transportrequests zijn getest.
- De uitgebreide SDK-ketentest vergelijkt ook padvalidatie met REST. Laatste lokale suite geeft 156 passed, 0 failed en 7 helpers in 9,77 s. F05-platform-CI is geslaagd in run #50. F05 is afgerond bij de overgang naar F06.
- [Featurenotitie](../../notities/F05-mcp-integratie.md).

## Afronden

- [x] Relevante controles geslaagd, inclusief platform-CI op Ubuntu, macOS en Windows (run #50).
- [x] Documentatie en notitie bijgewerkt tot 2026-11-09.
- [x] Voortgang bijgewerkt tot 2026-11-09.
- [x] F05 afgerond en overgedragen aan F06 volgens de actuele voortgang.


## Tussenstand op 2026-11-09

De implementatietaken en acceptatiecriteria zijn lokaal op Windows aangetoond, inclusief padvalidatie via een echte SDK-client en vergelijking met REST. Formatter, Clippy met -D warnings en de volledige suite zijn na de uitbreiding als geslaagd bevestigd. De Review-vermelding hoorde bij deze tussenstand. F05 is inmiddels afgerond. F05-platform-CI is geslaagd op alle drie platforms in run #50. De shutdownclient gebruikt expliciet NeverRetry. De F06-CI voert inmiddels Inspector-controles op alle drie platforms uit.
