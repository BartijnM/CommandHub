---
id: F00
prioriteit: Must
status: Klaar
milestone: "Ontwerp bevestigd"
---

# F00 — MCP-proef en protocolkeuzes

## Doel

MCP-proef en protocolkeuzes. Dit is een werkpakket; gebruik waar nodig meerdere kleine issues/PR's.

## Afhankelijkheden

Geen.

## Requirements

R11, R16.

## Scope

Zie de taken en acceptatiecriteria hieronder. Dit featureplan beschrijft de eigen implementatiestap. Het actuele productcontract staat in [API en configuratie](../api-en-configuratie.md) en de opleverstand in [voortgang](../voortgang.md).

## Todo

- [x] Officiële SDK-documentatie onderzoeken en een minimale HTTP-MCP-server apart proberen.
- [x] Protocolversie, SDK-versie/features en testclientversie vastleggen voor de herhaalbare controles.
- [x] Toolontdekking en toolaanroep met één vaste testtool onderzoeken.
- [x] Transportbeveiliging en REST/MCP-verschil voor de lokale proef documenteren; waargenomen foutgedrag voor een onbekende tool en een extra argument vastleggen. Dit is geen volledige foutmatrix of remote-security-audit.

## Acceptatiecriteria

- [x] Echte MCP-client kan één testtool ontdekken en aanroepen.
- [x] Geverifieerde keuzes en beperkingen zijn vastgelegd in een beslisdocument; definitieve integratiekeuzes zijn vervolgens in F05 getoetst.
- [x] De proef is duidelijk gescheiden van definitieve functionaliteit.

## Bewijs en koppelingen

- GitHub issue: nog aan te maken.
- Branch: feature/F00-mcp-proef.
- Pull request gemerged, door mij bevestigd op 2026-11-11.
- Tests/handmatige verificatie: vanuit de repositoryroot en de proefmap zijn `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings` en `cargo test --locked` geslaagd op Windows. Beide testbuilds vonden 0 tests. MCP Inspector CLI 2.11.0 heeft `tools/list`, onbekende-toolaanroep en `hello` met extra argument getest; `initialize` en succesvolle `tools/call` zijn ook handmatig geverifieerd. Zie de F00-notitie.
- Beslissingen: [ADR-002](../beslissingen/ADR-002-mcp-proef.md).
- [Featurenotitie](../../notities/F00-mcp-proef.md).

## Afronden

- [x] Relevante formatterings-, Clippy- en testbuildcontroles geslaagd; er zijn nog geen automatische tests.
- [x] Documentatie en notitie bijgewerkt.
- [x] Voortgang bijgewerkt.
- [x] PR gemerged, door mij bevestigd op 2026-11-11.

## Oplevercontext op 2026-11-11

Dit plan bewaart de scope en het bewijs van de geïsoleerde proef. De productie-MCP-integratie is aanwezig en F05 is afgerond. Ik heb op 2026-11-11 de historische merge bevestigd van F00. De actuele demo en clientcontroles staan bij [F06](F06-oplevering.md).
