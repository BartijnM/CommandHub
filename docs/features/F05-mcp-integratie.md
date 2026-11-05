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
- [ ] Toepasselijke transportbeveiliging en sessiegedrag implementeren.
- [ ] Met echte MCP-client en ongeldige protocolrequests testen.

## Acceptatiecriteria

- [x] MCP-client ontdekt geconfigureerde tools en voert ze uit.
- [x] Geen aparte executor of validatieregels in MCP-adapter.
- [ ] REST en MCP tonen consistent inhoudelijk gedrag.
- [ ] Protocol- en transportkeuzes zijn gedocumenteerd en getoetst.

## Bewijs en koppelingen

- GitHub issue: nog aan te maken.
- Branch: feature/F05-mcp-integratie.
- Pull request: nog niet aangemaakt.
- Tussenstand 2026-11-05: gerichte Windows-tests voor mapping, Host/Origin, bodygrens, streamannulering en SDK-clientontdekking/uitvoering slagen. De clientketen controleert ook extra parameters, onbekende tool en een geldige vervolgaanroep. Beide HTTP-integratiescenario's slagen na de shutdownwijziging. mcp_wait_helper compileert; de protocolannuleringstest staat open. Geen volledige F05-eindcontroles of CI bevestigd. Zie de featurenotitie.
- [Featurenotitie](../../notities/F05-mcp-integratie.md).

## Afronden

- [ ] Relevante controles geslaagd.
- [ ] Documentatie en notitie bijgewerkt.
- [ ] Voortgang bijgewerkt.
- [ ] PR gemerged; pas daarna status Klaar.


## Tussenstand — 2026-11-05

SDK, toolbeschrijvingen en gedeelde uitvoering zijn via een echte SDK-client over TCP gecontroleerd voor een parameterloze testtool. Mapping, Host/Origin en bodylimiet zijn getest; een actieve MCP-stream sluit na tokenannulering. Requestannulering is aangesloten maar nog niet via protocol met PID-bewijs getest. Daarom blijven de transport-/sessietaak en de brede client-/protocoltesttaak open. Eerstvolgende stap is de annuleringstest met mcp_wait_helper. Dynamische REST/MCP-consistentie, productie-shutdown met MCP en eindcontroles blijven open.

