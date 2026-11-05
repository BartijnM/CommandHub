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

- [ ] SDK en gekozen transport integreren.
- [ ] Registry vertalen naar MCP-tools en inputschema's.
- [ ] Toolcalls via dezelfde toolservice als REST uitvoeren.
- [x] Kernfouten en procesresultaten naar MCP-resultaten vertalen.
- [ ] Toepasselijke transportbeveiliging en sessiegedrag implementeren.
- [ ] Met echte MCP-client en ongeldige protocolrequests testen.

## Acceptatiecriteria

- [ ] MCP-client ontdekt geconfigureerde tools en voert ze uit.
- [ ] Geen aparte executor of validatieregels in MCP-adapter.
- [ ] REST en MCP tonen consistent inhoudelijk gedrag.
- [ ] Protocol- en transportkeuzes zijn gedocumenteerd en getoetst.

## Bewijs en koppelingen

- GitHub issue: nog aan te maken.
- Branch: feature/F05-mcp-integratie.
- Pull request: nog niet aangemaakt.
- Tussenstand 2026-11-05: eigenaar rapporteert 18 REST-routetests en 9 MCP-tests geslaagd op Windows; compilatie geslaagd na toevoegen tokio-util. Mapping en transportgrenzen getest; echte toolketen, annulering, transportshutdown en eindcontroles staan open. Zie de featurenotitie.
- [Featurenotitie](../../notities/F05-mcp-integratie.md).

## Afronden

- [ ] Relevante controles geslaagd.
- [ ] Documentatie en notitie bijgewerkt.
- [ ] Voortgang bijgewerkt.
- [ ] PR gemerged; pas daarna status Klaar.


## Tussenstand — 2026-11-05

SDK, /mcp-route, dynamische toolbeschrijvingen en gedeelde uitvoering zijn in code aanwezig. De mappingtaak is met gerichte tests gecontroleerd. Overige taken blijven open totdat ook de toolketen, sessie-/shutdown- en annuleringsgedrag en de benodigde eindcontroles zijn aangetoond. Documentatie is tussentijds bijgewerkt; definitieve afronding en acceptatiecriteria blijven open.

